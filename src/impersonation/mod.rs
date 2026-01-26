//! Impersonation layer using curl-impersonate
//!
//! Provides browser TLS fingerprint mimicking to bypass anti-bot detection.
//! Uses curl-impersonate binaries (curl_chrome, curl_ff, curl_safari).

use std::io;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use crate::engine::{find_engine, EngineType};

/// Available impersonation profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpersonationProfile {
    /// Chrome browser fingerprint (default)
    Chrome,
    /// Firefox browser fingerprint
    Firefox,
    /// Safari browser fingerprint
    Safari,
}

impl Default for ImpersonationProfile {
    fn default() -> Self {
        ImpersonationProfile::Chrome
    }
}

impl ImpersonationProfile {
    /// Get the engine type for this profile
    pub fn engine_type(&self) -> EngineType {
        match self {
            ImpersonationProfile::Chrome => EngineType::Chrome,
            ImpersonationProfile::Firefox => EngineType::Firefox,
            ImpersonationProfile::Safari => EngineType::Safari,
        }
    }

    /// Parse profile from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "chrome" | "chrome119" | "chrome120" => Some(ImpersonationProfile::Chrome),
            "firefox" | "ff" | "firefox121" => Some(ImpersonationProfile::Firefox),
            "safari" => Some(ImpersonationProfile::Safari),
            _ => None,
        }
    }

    /// Get profile name
    pub fn name(&self) -> &'static str {
        match self {
            ImpersonationProfile::Chrome => "chrome",
            ImpersonationProfile::Firefox => "firefox",
            ImpersonationProfile::Safari => "safari",
        }
    }

    /// List of profiles to try in escalation order
    pub fn escalation_order() -> &'static [ImpersonationProfile] {
        &[
            ImpersonationProfile::Chrome,
            ImpersonationProfile::Firefox,
            ImpersonationProfile::Safari,
        ]
    }
}

/// Result of an impersonation attempt
#[derive(Debug)]
pub struct ImpersonationResult {
    /// The profile that was used
    pub profile: ImpersonationProfile,
    /// Whether impersonation engine was available
    pub available: bool,
    /// The command output (if execution succeeded)
    pub output: Option<Output>,
    /// Error message (if execution failed)
    pub error: Option<String>,
}

impl ImpersonationResult {
    /// Create a result for unavailable engine
    pub fn unavailable(profile: ImpersonationProfile) -> Self {
        Self {
            profile,
            available: false,
            output: None,
            error: Some(format!(
                "{} impersonation engine not found",
                profile.name()
            )),
        }
    }

    /// Create a result for successful execution
    pub fn success(profile: ImpersonationProfile, output: Output) -> Self {
        Self {
            profile,
            available: true,
            output: Some(output),
            error: None,
        }
    }

    /// Create a result for execution error
    pub fn failed(profile: ImpersonationProfile, error: String) -> Self {
        Self {
            profile,
            available: true,
            output: None,
            error: Some(error),
        }
    }

    /// Check if the request succeeded (exit code 0)
    pub fn is_success(&self) -> bool {
        self.output
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get stdout bytes
    pub fn stdout(&self) -> &[u8] {
        self.output.as_ref().map(|o| o.stdout.as_slice()).unwrap_or(&[])
    }

    /// Get exit code
    pub fn exit_code(&self) -> u8 {
        self.output
            .as_ref()
            .and_then(|o| o.status.code())
            .unwrap_or(1) as u8
    }
}

/// Execute a request with impersonation
pub fn execute_impersonation(
    profile: ImpersonationProfile,
    args: &[String],
) -> ImpersonationResult {
    // Find the impersonation engine
    let engine_path = match find_engine(profile.engine_type()) {
        Ok(path) => path,
        Err(_) => return ImpersonationResult::unavailable(profile),
    };

    // Execute with captured output
    match execute_with_engine(&engine_path, args) {
        Ok(output) => ImpersonationResult::success(profile, output),
        Err(e) => ImpersonationResult::failed(profile, e.to_string()),
    }
}

/// Try impersonation with escalation through profiles
pub fn execute_with_escalation(
    args: &[String],
    preferred_profile: Option<ImpersonationProfile>,
    debug: bool,
) -> Option<ImpersonationResult> {
    // If a specific profile is requested, only try that one
    if let Some(profile) = preferred_profile {
        let result = execute_impersonation(profile, args);
        if debug {
            eprintln!(
                "[recurl] impersonation: {} -> {}",
                profile.name(),
                if result.available {
                    if result.is_success() { "success" } else { "failed" }
                } else {
                    "unavailable"
                }
            );
        }
        return Some(result);
    }

    // Otherwise, try profiles in escalation order
    for profile in ImpersonationProfile::escalation_order() {
        let result = execute_impersonation(*profile, args);

        if debug {
            eprintln!(
                "[recurl] impersonation: {} -> {}",
                profile.name(),
                if result.available {
                    if result.is_success() { "success" } else { "failed" }
                } else {
                    "unavailable"
                }
            );
        }

        // Return first available result (even if request failed)
        if result.available {
            return Some(result);
        }
    }

    None
}

/// Execute curl with a specific engine
fn execute_with_engine(engine: &PathBuf, args: &[String]) -> io::Result<Output> {
    Command::new(engine)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
}

/// Check if any impersonation engine is available
pub fn is_impersonation_available() -> bool {
    ImpersonationProfile::escalation_order()
        .iter()
        .any(|p| find_engine(p.engine_type()).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_default() {
        assert_eq!(ImpersonationProfile::default(), ImpersonationProfile::Chrome);
    }

    #[test]
    fn test_profile_from_str() {
        assert_eq!(
            ImpersonationProfile::from_str("chrome"),
            Some(ImpersonationProfile::Chrome)
        );
        assert_eq!(
            ImpersonationProfile::from_str("Chrome"),
            Some(ImpersonationProfile::Chrome)
        );
        assert_eq!(
            ImpersonationProfile::from_str("firefox"),
            Some(ImpersonationProfile::Firefox)
        );
        assert_eq!(
            ImpersonationProfile::from_str("ff"),
            Some(ImpersonationProfile::Firefox)
        );
        assert_eq!(
            ImpersonationProfile::from_str("safari"),
            Some(ImpersonationProfile::Safari)
        );
        assert_eq!(ImpersonationProfile::from_str("unknown"), None);
    }

    #[test]
    fn test_profile_name() {
        assert_eq!(ImpersonationProfile::Chrome.name(), "chrome");
        assert_eq!(ImpersonationProfile::Firefox.name(), "firefox");
        assert_eq!(ImpersonationProfile::Safari.name(), "safari");
    }

    #[test]
    fn test_profile_engine_type() {
        assert_eq!(
            ImpersonationProfile::Chrome.engine_type(),
            EngineType::Chrome
        );
        assert_eq!(
            ImpersonationProfile::Firefox.engine_type(),
            EngineType::Firefox
        );
        assert_eq!(
            ImpersonationProfile::Safari.engine_type(),
            EngineType::Safari
        );
    }

    #[test]
    fn test_escalation_order() {
        let order = ImpersonationProfile::escalation_order();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], ImpersonationProfile::Chrome);
    }

    #[test]
    fn test_impersonation_result_unavailable() {
        let result = ImpersonationResult::unavailable(ImpersonationProfile::Chrome);
        assert!(!result.available);
        assert!(result.error.is_some());
        assert!(!result.is_success());
    }
}
