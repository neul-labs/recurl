//! Daemon client for rcurl
//!
//! Connects to rcurld for fast JS preflight operations.

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::UnixStream;

use serde::{Deserialize, Serialize};

/// Request to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonRequest {
    JsPreflight {
        url: String,
        timeout_ms: Option<u64>,
        wait_selector: Option<String>,
        return_html: bool,
    },
    Status,
    Ping,
}

/// Response from daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonResponse {
    PreflightSuccess {
        cookies: HashMap<String, String>,
        final_url: String,
        html: Option<String>,
    },
    PreflightError {
        error: String,
    },
    Status {
        version: String,
        uptime_secs: u64,
        pool_size: usize,
        requests_served: u64,
        active_requests: usize,
    },
    Pong,
    Error {
        error: String,
    },
}

/// Get the socket path
fn get_socket_path() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(format!(r"\\.\pipe\rcurl-{}", whoami()))
    } else {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/rcurl.{}.sock", uid))
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
        eprintln!("[rcurl] Starting daemon...");
    }

    // Find rcurld binary (same directory as rcurl)
    let rcurld_path = std::env::current_exe()?
        .parent()
        .map(|p| p.join(if cfg!(windows) { "rcurld.exe" } else { "rcurld" }))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cannot find rcurld"))?;

    if !rcurld_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("rcurld not found at {:?}", rcurld_path),
        ));
    }

    // Start daemon in background
    Command::new(&rcurld_path)
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
                eprintln!("[rcurl] Daemon started");
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

/// Execute JS preflight via daemon (sync wrapper)
pub fn daemon_preflight_sync(
    url: &str,
    timeout_ms: Option<u64>,
    wait_selector: Option<String>,
    return_html: bool,
) -> io::Result<DaemonResponse> {
    #[cfg(unix)]
    {
        tokio::runtime::Runtime::new()?.block_on(daemon_preflight(
            url,
            timeout_ms,
            wait_selector,
            return_html,
        ))
    }

    #[cfg(windows)]
    {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Daemon not supported on Windows yet",
        ))
    }
}

/// Convert daemon response cookies to curl -b format
pub fn cookies_to_curl_args(cookies: &HashMap<String, String>) -> Vec<String> {
    if cookies.is_empty() {
        return vec![];
    }

    let cookie_str = cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ");

    vec!["-b".to_string(), cookie_str]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        let path = get_socket_path();
        assert!(!path.to_string_lossy().is_empty());
    }

    #[test]
    fn test_cookies_to_curl_args() {
        let mut cookies = HashMap::new();
        cookies.insert("a".to_string(), "1".to_string());
        cookies.insert("b".to_string(), "2".to_string());

        let args = cookies_to_curl_args(&cookies);
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "-b");
        assert!(args[1].contains("a=1"));
        assert!(args[1].contains("b=2"));
    }

    #[test]
    fn test_cookies_to_curl_args_empty() {
        let cookies = HashMap::new();
        let args = cookies_to_curl_args(&cookies);
        assert!(args.is_empty());
    }
}
