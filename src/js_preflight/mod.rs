//! JavaScript preflight module
//!
//! Solves JavaScript challenges using headless Chromium, then extracts
//! cookies and headers for curl replay.
//!
//! Chromium is automatically downloaded on first use and cached locally.

mod browser;
mod chromium;
mod cookies;
mod stealth;

pub use browser::BrowserConfig;
pub use chromium::{ensure_chromium, is_chromium_available, get_chromium_cache_dir};
pub use cookies::ExtractedCookies;
pub use stealth::get_all_patches;

use std::time::Duration;

/// Result of a JS preflight operation
#[derive(Debug, Clone)]
pub struct PreflightResult {
    /// Cookies extracted from the browser session
    pub cookies: ExtractedCookies,
    /// Final URL after redirects/challenge resolution
    pub final_url: String,
    /// Whether the preflight was successful
    pub success: bool,
    /// Error message if preflight failed
    pub error: Option<String>,
    /// HTML content if js_rendered mode is enabled
    pub rendered_html: Option<String>,
}

impl PreflightResult {
    /// Create a successful result
    pub fn success(cookies: ExtractedCookies, final_url: String) -> Self {
        Self {
            cookies,
            final_url,
            success: true,
            error: None,
            rendered_html: None,
        }
    }

    /// Create a successful result with rendered HTML
    pub fn success_with_html(cookies: ExtractedCookies, final_url: String, html: String) -> Self {
        Self {
            cookies,
            final_url,
            success: true,
            error: None,
            rendered_html: Some(html),
        }
    }

    /// Create a failed result
    pub fn failed(error: String) -> Self {
        Self {
            cookies: ExtractedCookies::empty(),
            final_url: String::new(),
            success: false,
            error: Some(error),
            rendered_html: None,
        }
    }
}

/// Options for JS preflight
#[derive(Debug, Clone)]
pub struct PreflightOptions {
    /// Timeout for the entire preflight operation
    pub timeout: Duration,
    /// Optional CSS selector to wait for
    pub wait_selector: Option<String>,
    /// Whether to return rendered HTML instead of just cookies
    pub return_html: bool,
    /// Enable debug output
    pub debug: bool,
}

impl Default for PreflightOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            wait_selector: None,
            return_html: false,
            debug: false,
        }
    }
}

impl PreflightOptions {
    /// Create options from RcurlConfig values
    pub fn from_config(
        timeout_ms: Option<u64>,
        wait_selector: Option<String>,
        return_html: bool,
        debug: bool,
    ) -> Self {
        Self {
            timeout: Duration::from_millis(timeout_ms.unwrap_or(30000)),
            wait_selector,
            return_html,
            debug,
        }
    }
}

/// Execute a JS preflight for the given URL
///
/// This launches a headless browser, navigates to the URL,
/// waits for any JavaScript challenges to resolve, and extracts
/// cookies for replay with curl.
pub async fn execute_preflight(
    url: &str,
    options: &PreflightOptions,
) -> PreflightResult {
    if options.debug {
        eprintln!("[recurl] JS preflight: starting for {}", url);
    }

    // Launch browser and execute preflight
    match browser::run_preflight(url, options).await {
        Ok(result) => {
            if options.debug {
                eprintln!("[recurl] JS preflight: success");
                eprintln!("[recurl] JS preflight: extracted {} cookies", result.cookies.count());
                eprintln!("[recurl] JS preflight: final URL = {}", result.final_url);
            }
            result
        }
        Err(e) => {
            if options.debug {
                eprintln!("[recurl] JS preflight: failed - {}", e);
            }
            PreflightResult::failed(e.to_string())
        }
    }
}

/// Execute preflight synchronously (for use in main.rs)
pub fn execute_preflight_sync(
    url: &str,
    options: &PreflightOptions,
) -> PreflightResult {
    // Create a new tokio runtime for the async operation
    match tokio::runtime::Runtime::new() {
        Ok(rt) => rt.block_on(execute_preflight(url, options)),
        Err(e) => PreflightResult::failed(format!("Failed to create async runtime: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preflight_options_default() {
        let opts = PreflightOptions::default();
        assert_eq!(opts.timeout, Duration::from_secs(30));
        assert!(opts.wait_selector.is_none());
        assert!(!opts.return_html);
    }

    #[test]
    fn test_preflight_options_from_config() {
        let opts = PreflightOptions::from_config(
            Some(5000),
            Some(".content".to_string()),
            true,
            false,
        );
        assert_eq!(opts.timeout, Duration::from_millis(5000));
        assert_eq!(opts.wait_selector, Some(".content".to_string()));
        assert!(opts.return_html);
    }

    #[test]
    fn test_preflight_result_success() {
        let cookies = ExtractedCookies::empty();
        let result = PreflightResult::success(cookies, "https://example.com".to_string());
        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_preflight_result_failed() {
        let result = PreflightResult::failed("Browser crashed".to_string());
        assert!(!result.success);
        assert_eq!(result.error, Some("Browser crashed".to_string()));
    }
}
