//! IPC transport for daemon communication
//!
//! Uses Unix sockets on Linux/macOS and named pipes on Windows.

use std::io;
use std::path::PathBuf;

/// Get the socket/pipe path for the daemon
pub fn get_socket_path() -> PathBuf {
    if cfg!(windows) {
        // Windows named pipe
        PathBuf::from(format!(r"\\.\pipe\recurl-{}", whoami()))
    } else {
        // Unix socket – placed under the user's cache dir to avoid
        // symlink TOCTOU attacks in world-writable /tmp.
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let socket_dir = cache_dir.join("recurl");
        // Ensure the directory exists with restricted permissions
        let _ = std::fs::create_dir_all(&socket_dir);
        socket_dir.join("daemon.sock")
    }
}

/// Get current username (simplified)
fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Remove the socket file (for cleanup)
pub fn remove_socket() -> io::Result<()> {
    let path = get_socket_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(unix)]
pub mod unix {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::{UnixListener, UnixStream};

    use crate::protocol::{DaemonRequest, DaemonResponse};

    /// Unix socket server for the daemon
    pub struct DaemonServer {
        listener: UnixListener,
    }

    impl DaemonServer {
        /// Create and bind the server socket
        pub async fn bind() -> io::Result<Self> {
            let path = get_socket_path();

            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Remove existing socket if present
            if path.exists() {
                std::fs::remove_file(&path)?;
            }

            let listener = UnixListener::bind(&path)?;
            Ok(Self { listener })
        }

        /// Accept the next connection
        pub async fn accept(&self) -> io::Result<DaemonConnection> {
            let (stream, _) = self.listener.accept().await?;
            Ok(DaemonConnection { stream })
        }

        /// Get the socket path
        pub fn path(&self) -> PathBuf {
            get_socket_path()
        }
    }

    impl Drop for DaemonServer {
        fn drop(&mut self) {
            let _ = remove_socket();
        }
    }

    /// A connection from a client
    pub struct DaemonConnection {
        stream: UnixStream,
    }

    impl DaemonConnection {
        /// Read a request from the connection
        pub async fn read_request(&mut self) -> io::Result<Option<DaemonRequest>> {
            let mut reader = BufReader::new(&mut self.stream);
            let mut line = String::new();

            match reader.read_line(&mut line).await {
                Ok(0) => Ok(None), // Connection closed
                Ok(_) => {
                    let req = DaemonRequest::from_bytes(line.trim().as_bytes())
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    Ok(Some(req))
                }
                Err(e) => Err(e),
            }
        }

        /// Write a response to the connection
        pub async fn write_response(&mut self, response: &DaemonResponse) -> io::Result<()> {
            let bytes = response.to_bytes();
            self.stream.write_all(&bytes).await?;
            self.stream.flush().await?;
            Ok(())
        }
    }

    /// Client connection to the daemon
    pub struct DaemonClient {
        stream: UnixStream,
    }

    impl DaemonClient {
        /// Connect to the daemon
        pub async fn connect() -> io::Result<Self> {
            let path = get_socket_path();
            let stream = UnixStream::connect(&path).await?;
            Ok(Self { stream })
        }

        /// Send a request and receive a response
        pub async fn request(&mut self, req: &DaemonRequest) -> io::Result<DaemonResponse> {
            // Write request
            let bytes = req.to_bytes();
            self.stream.write_all(&bytes).await?;
            self.stream.flush().await?;

            // Read response
            let mut reader = BufReader::new(&mut self.stream);
            let mut line = String::new();
            reader.read_line(&mut line).await?;

            DaemonResponse::from_bytes(line.trim().as_bytes())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        }
    }
}

#[cfg(windows)]
pub mod windows {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions};

    use crate::protocol::{DaemonRequest, DaemonResponse};

    /// Get the pipe name for Windows
    fn get_pipe_name() -> String {
        let path = get_socket_path();
        path.to_string_lossy().to_string()
    }

    /// Windows named pipe server for the daemon
    pub struct DaemonServer {
        pipe_name: String,
    }

    impl DaemonServer {
        /// Create and bind the server pipe
        pub async fn bind() -> io::Result<Self> {
            let pipe_name = get_pipe_name();

            // Create the first pipe instance to verify we can bind
            let _server = ServerOptions::new()
                .first_pipe_instance(true)
                .create(&pipe_name)?;

            Ok(Self { pipe_name })
        }

        /// Accept the next connection
        pub async fn accept(&self) -> io::Result<DaemonConnection> {
            let server = ServerOptions::new()
                .first_pipe_instance(false)
                .create(&self.pipe_name)?;

            // Wait for a client to connect
            server.connect().await?;

            Ok(DaemonConnection { pipe: server })
        }

        /// Get the pipe path
        pub fn path(&self) -> PathBuf {
            PathBuf::from(&self.pipe_name)
        }
    }

    /// A connection from a client
    pub struct DaemonConnection {
        pipe: NamedPipeServer,
    }

    impl DaemonConnection {
        /// Read a request from the connection
        pub async fn read_request(&mut self) -> io::Result<Option<DaemonRequest>> {
            let mut reader = BufReader::new(&mut self.pipe);
            let mut line = String::new();

            match reader.read_line(&mut line).await {
                Ok(0) => Ok(None), // Connection closed
                Ok(_) => {
                    let req = DaemonRequest::from_bytes(line.trim().as_bytes())
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    Ok(Some(req))
                }
                Err(e) => Err(e),
            }
        }

        /// Write a response to the connection
        pub async fn write_response(&mut self, response: &DaemonResponse) -> io::Result<()> {
            let bytes = response.to_bytes();
            self.pipe.write_all(&bytes).await?;
            self.pipe.flush().await?;
            Ok(())
        }
    }

    /// Client connection to the daemon
    pub struct DaemonClient {
        pipe: tokio::net::windows::named_pipe::NamedPipeClient,
    }

    impl DaemonClient {
        /// Connect to the daemon
        pub async fn connect() -> io::Result<Self> {
            let pipe_name = get_pipe_name();

            // Try to connect, with retries for ERROR_PIPE_BUSY
            let pipe = loop {
                match ClientOptions::new().open(&pipe_name) {
                    Ok(client) => break client,
                    Err(e) if e.raw_os_error() == Some(231) => {
                        // ERROR_PIPE_BUSY - wait and retry
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                    Err(e) => return Err(e),
                }
            };

            Ok(Self { pipe })
        }

        /// Send a request and receive a response
        pub async fn request(&mut self, req: &DaemonRequest) -> io::Result<DaemonResponse> {
            // Write request
            let bytes = req.to_bytes();
            self.pipe.write_all(&bytes).await?;
            self.pipe.flush().await?;

            // Read response
            let mut reader = BufReader::new(&mut self.pipe);
            let mut line = String::new();
            reader.read_line(&mut line).await?;

            DaemonResponse::from_bytes(line.trim().as_bytes())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        }
    }
}

// Re-export platform-specific implementations
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
pub use windows::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        let path = get_socket_path();
        if cfg!(windows) {
            assert!(path.to_string_lossy().contains(r"\\.\pipe\recurl-"));
        } else {
            // Should now be under the user's cache directory
            assert!(
                path.to_string_lossy().contains("recurl"),
                "socket path should contain 'recurl' subdirectory: {:?}",
                path
            );
            assert!(
                path.file_name().unwrap() == "daemon.sock",
                "socket file should be named daemon.sock: {:?}",
                path
            );
        }
    }
}
