//! Daemon protocol definitions
//!
//! Defines the request/response messages exchanged between recurl and recurld.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request message from recurl to recurld
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonRequest {
    /// Execute JS preflight for a URL
    JsPreflight {
        /// URL to preflight
        url: String,
        /// Timeout in milliseconds
        timeout_ms: Option<u64>,
        /// CSS selector to wait for
        wait_selector: Option<String>,
        /// Whether to return rendered HTML
        return_html: bool,
    },

    /// Get daemon status
    Status,

    /// Shutdown the daemon
    Shutdown,

    /// Ping to check if daemon is alive
    Ping,
}

/// Response message from recurld to recurl
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonResponse {
    /// Preflight completed successfully
    PreflightSuccess {
        /// Extracted cookies (name -> value)
        cookies: HashMap<String, String>,
        /// Final URL after redirects
        final_url: String,
        /// Rendered HTML (if requested)
        html: Option<String>,
    },

    /// Preflight failed
    PreflightError {
        /// Error message
        error: String,
    },

    /// Daemon status
    Status {
        /// Daemon version
        version: String,
        /// Uptime in seconds
        uptime_secs: u64,
        /// Number of browser instances in pool
        pool_size: usize,
        /// Number of requests served
        requests_served: u64,
        /// Number of active requests
        active_requests: usize,
    },

    /// Shutdown acknowledged
    ShutdownAck,

    /// Pong response
    Pong,

    /// Generic error
    Error {
        /// Error message
        error: String,
    },
}

impl DaemonRequest {
    /// Serialize request to JSON bytes with newline delimiter
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(self).unwrap_or_default();
        bytes.push(b'\n');
        bytes
    }

    /// Deserialize request from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl DaemonResponse {
    /// Serialize response to JSON bytes with newline delimiter
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(self).unwrap_or_default();
        bytes.push(b'\n');
        bytes
    }

    /// Deserialize response from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = DaemonRequest::JsPreflight {
            url: "https://example.com".to_string(),
            timeout_ms: Some(30000),
            wait_selector: None,
            return_html: false,
        };

        let bytes = req.to_bytes();
        let parsed = DaemonRequest::from_bytes(&bytes[..bytes.len() - 1]).unwrap();

        match parsed {
            DaemonRequest::JsPreflight { url, .. } => {
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let mut cookies = HashMap::new();
        cookies.insert("session".to_string(), "abc123".to_string());

        let resp = DaemonResponse::PreflightSuccess {
            cookies,
            final_url: "https://example.com".to_string(),
            html: None,
        };

        let bytes = resp.to_bytes();
        let parsed = DaemonResponse::from_bytes(&bytes[..bytes.len() - 1]).unwrap();

        match parsed {
            DaemonResponse::PreflightSuccess { cookies, .. } => {
                assert_eq!(cookies.get("session"), Some(&"abc123".to_string()));
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_ping_pong() {
        let req = DaemonRequest::Ping;
        let bytes = req.to_bytes();
        let parsed = DaemonRequest::from_bytes(&bytes[..bytes.len() - 1]).unwrap();
        assert!(matches!(parsed, DaemonRequest::Ping));

        let resp = DaemonResponse::Pong;
        let bytes = resp.to_bytes();
        let parsed = DaemonResponse::from_bytes(&bytes[..bytes.len() - 1]).unwrap();
        assert!(matches!(parsed, DaemonResponse::Pong));
    }
}
