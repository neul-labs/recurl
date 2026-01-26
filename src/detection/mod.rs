//! Failure detection module
//!
//! Detects when a response indicates bot blocking (403, captcha, etc.)
//! so recurl can decide whether to escalate to impersonation or JS preflight.

mod patterns;
mod status;

pub use patterns::{detect_antibot_patterns, AntibotService};
pub use status::is_blocking_status;

/// Result of analyzing a curl response for blocking signals
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// HTTP status code (if detected)
    pub status_code: Option<u16>,

    /// Whether the status code indicates blocking
    pub status_blocked: bool,

    /// Detected anti-bot service (if any)
    pub antibot_service: Option<AntibotService>,

    /// Whether we should escalate to next layer
    pub should_escalate: bool,

    /// Human-readable summary
    pub summary: String,
}

impl DetectionResult {
    /// Create a result indicating no blocking detected
    pub fn ok() -> Self {
        Self {
            status_code: None,
            status_blocked: false,
            antibot_service: None,
            should_escalate: false,
            summary: "No blocking detected".to_string(),
        }
    }

    /// Create a result from status code and body analysis
    pub fn analyze(status_code: Option<u16>, body: &[u8]) -> Self {
        let status_blocked = status_code.map(is_blocking_status).unwrap_or(false);
        let antibot_service = detect_antibot_patterns(body);

        let should_escalate = status_blocked || antibot_service.is_some();

        let summary = if let Some(ref service) = antibot_service {
            format!(
                "Blocked: {} (status: {})",
                service,
                status_code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
            )
        } else if status_blocked {
            format!(
                "Blocked: HTTP {}",
                status_code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
            )
        } else {
            "No blocking detected".to_string()
        };

        Self {
            status_code,
            status_blocked,
            antibot_service,
            should_escalate,
            summary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_result_ok() {
        let result = DetectionResult::ok();
        assert!(!result.should_escalate);
        assert!(result.antibot_service.is_none());
    }

    #[test]
    fn test_detection_result_403() {
        let result = DetectionResult::analyze(Some(403), b"Access Denied");
        assert!(result.status_blocked);
        assert!(result.should_escalate);
    }

    #[test]
    fn test_detection_result_cloudflare() {
        let body = b"<title>Just a moment...</title>";
        let result = DetectionResult::analyze(Some(403), body);
        assert!(result.should_escalate);
        assert!(matches!(result.antibot_service, Some(AntibotService::Cloudflare)));
    }

    #[test]
    fn test_detection_result_success() {
        let result = DetectionResult::analyze(Some(200), b"OK");
        assert!(!result.should_escalate);
        assert!(result.antibot_service.is_none());
    }
}
