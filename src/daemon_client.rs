//! Daemon client for recurl
//!
//! Connects to recurld for fast JS preflight operations.

use crate::protocol::{DaemonRequest, DaemonResponse};
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::UnixStream;

/// Get the socket path
fn get_socket_path() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(format!(r"\\.\pipe\recurl-{}", whoami()))
    } else {
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        cache_dir.join("recurl").join("daemon.sock")
    }
}

fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Check if daemon is running
pub fn is_daemon_running() -> bool {
    get_socket_path().exists()
}

/// Start the daemon if not running
pub fn start_daemon_if_needed(debug: bool) -> io::Result<()> {
    if is_daemon_running() {
        return Ok(());
    }

    if debug {
        eprintln!("[recurl] Starting daemon...");
    }

    // Find recurld binary (same directory as recurl)
    let recurld_path = std::env::current_exe()?
        .parent()
        .map(|p| {
            p.join(if cfg!(windows) {
                "recurld.exe"
            } else {
                "recurld"
            })
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cannot find recurld"))?;

    if !recurld_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("recurld not found at {:?}", recurld_path),
        ));
    }

    // Start daemon in background
    Command::new(&recurld_path)
        .arg("start")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Wait for daemon to start (up to 5 seconds)
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if is_daemon_running() {
            if debug {
                eprintln!("[recurl] Daemon started");
            }
            return Ok(());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "Daemon failed to start",
    ))
}

/// Execute JS preflight via daemon
#[cfg(unix)]
pub async fn daemon_preflight(
    url: &str,
    timeout_ms: Option<u64>,
    wait_selector: Option<String>,
    return_html: bool,
) -> io::Result<DaemonResponse> {
    let path = get_socket_path();
    let mut stream = UnixStream::connect(&path).await?;

    // Send request
    let request = DaemonRequest::JsPreflight {
        url: url.to_string(),
        timeout_ms,
        wait_selector,
        return_html,
    };

    let mut bytes = serde_json::to_vec(&request)?;
    bytes.push(b'\n');
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    // Read response
    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    serde_json::from_str(&line).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Execute JS preflight via daemon (sync wrapper with timeout)
pub fn daemon_preflight_sync(
    url: &str,
    timeout_ms: Option<u64>,
    wait_selector: Option<String>,
    return_html: bool,
) -> io::Result<DaemonResponse> {
    #[cfg(unix)]
    {
        let rt = tokio::runtime::Runtime::new()?;
        let deadline = std::time::Duration::from_millis(timeout_ms.unwrap_or(30000));
        rt.block_on(async {
            match tokio::time::timeout(
                deadline,
                daemon_preflight(url, timeout_ms, wait_selector, return_html),
            )
            .await
            {
                Ok(result) => result,
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Daemon preflight timed out",
                )),
            }
        })
    }

    #[cfg(windows)]
    {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Daemon not supported on Windows yet",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        let path = get_socket_path();
        assert!(!path.to_string_lossy().is_empty());
    }
}
