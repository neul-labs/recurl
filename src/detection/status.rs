//! HTTP status code detection
//!
//! Identifies status codes that commonly indicate bot blocking.

/// Status codes that typically indicate bot blocking
const BLOCKING_STATUS_CODES: &[u16] = &[
    403, // Forbidden - most common bot block
    429, // Too Many Requests - rate limiting
    503, // Service Unavailable - often used by anti-bot services
];

/// Additional status codes that might indicate blocking in context
const SUSPICIOUS_STATUS_CODES: &[u16] = &[
    401, // Unauthorized - sometimes used for bot detection
    407, // Proxy Authentication Required
    451, // Unavailable For Legal Reasons
];

/// Check if a status code indicates definite blocking
pub fn is_blocking_status(code: u16) -> bool {
    BLOCKING_STATUS_CODES.contains(&code)
}

/// Check if a status code is suspicious (might be blocking)
pub fn is_suspicious_status(code: u16) -> bool {
    SUSPICIOUS_STATUS_CODES.contains(&code)
}

/// Check if a status code indicates success
pub fn is_success_status(code: u16) -> bool {
    (200..300).contains(&code)
}

/// Parse HTTP status code from curl output with -i flag
/// Returns the first status code found in the response headers
pub fn parse_status_from_headers(response: &[u8]) -> Option<u16> {
    let text = String::from_utf8_lossy(response);

    // Look for "HTTP/1.1 XXX" or "HTTP/2 XXX" pattern
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("HTTP/") {
            // Extract status code from "HTTP/1.1 200 OK" or "HTTP/2 200"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(code) = parts[1].parse::<u16>() {
                    return Some(code);
                }
            }
        }
    }

    None
}

/// Parse HTTP status code from curl's -w '%{http_code}' output
pub fn parse_status_from_write_out(output: &str) -> Option<u16> {
    output.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocking_status_codes() {
        assert!(is_blocking_status(403));
        assert!(is_blocking_status(429));
        assert!(is_blocking_status(503));
        assert!(!is_blocking_status(200));
        assert!(!is_blocking_status(404));
        assert!(!is_blocking_status(500));
    }

    #[test]
    fn test_suspicious_status_codes() {
        assert!(is_suspicious_status(401));
        assert!(is_suspicious_status(407));
        assert!(!is_suspicious_status(403)); // This is blocking, not suspicious
        assert!(!is_suspicious_status(200));
    }

    #[test]
    fn test_success_status() {
        assert!(is_success_status(200));
        assert!(is_success_status(201));
        assert!(is_success_status(204));
        assert!(!is_success_status(301));
        assert!(!is_success_status(403));
    }

    #[test]
    fn test_parse_status_from_headers_http11() {
        let response = b"HTTP/1.1 403 Forbidden\r\nContent-Type: text/html\r\n\r\nBlocked";
        assert_eq!(parse_status_from_headers(response), Some(403));
    }

    #[test]
    fn test_parse_status_from_headers_http2() {
        let response = b"HTTP/2 200 \r\ncontent-type: application/json\r\n\r\n{}";
        assert_eq!(parse_status_from_headers(response), Some(200));
    }

    #[test]
    fn test_parse_status_from_headers_no_status() {
        let response = b"Just some text without HTTP headers";
        assert_eq!(parse_status_from_headers(response), None);
    }

    #[test]
    fn test_parse_status_from_write_out() {
        assert_eq!(parse_status_from_write_out("200"), Some(200));
        assert_eq!(parse_status_from_write_out("403\n"), Some(403));
        assert_eq!(parse_status_from_write_out("invalid"), None);
    }
}
