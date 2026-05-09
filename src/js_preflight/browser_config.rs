//! Shared Chromium browser configuration builder
//!
//! Provides a unified `build_stealth_browser_config()` used by both
//! direct JS preflights (`js_preflight/browser.rs`) and the daemon pool
//! (`daemon/pool.rs`) so that stealth settings never diverge.

use chromiumoxide::browser::BrowserConfig as ChromeConfig;
use std::path::PathBuf;

/// Build a stealth-oriented Chromium `BrowserConfig`.
///
/// The returned config disables automation indicators, sets a realistic
/// viewport and user-agent, and applies the same flags used by
/// `puppeteer-extra-plugin-stealth`.
///
/// # Arguments
/// * `chrome_path` – Optional explicit path to the Chromium executable.
/// * `debug`       – When `true` a future caller may log the chosen path.
pub fn build_stealth_browser_config(
    chrome_path: Option<PathBuf>,
) -> Result<ChromeConfig, Box<dyn std::error::Error + Send + Sync>> {
    let mut builder = ChromeConfig::builder();

    if let Some(path) = chrome_path {
        builder = builder.chrome_executable(path);
    }

    builder = builder
        .disable_default_args()
        // Headless mode (new headless is less detectable)
        .arg("--headless=new")
        // Basic required args
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-gpu")
        // Window / viewport
        .arg("--window-size=1920,1080")
        .arg("--start-maximized")
        // Stealth: disable automation indicators
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--disable-features=TranslateUI")
        .arg("--disable-infobars")
        .arg("--disable-background-networking")
        .arg("--disable-backgrounding-occluded-windows")
        .arg("--disable-breakpad")
        .arg("--disable-component-update")
        .arg("--disable-default-apps")
        .arg("--disable-domain-reliability")
        .arg("--disable-extensions")
        .arg("--disable-hang-monitor")
        .arg("--disable-ipc-flooding-protection")
        .arg("--disable-popup-blocking")
        .arg("--disable-prompt-on-repost")
        .arg("--disable-renderer-backgrounding")
        .arg("--disable-sync")
        .arg("--enable-features=NetworkService,NetworkServiceInProcess")
        .arg("--force-color-profile=srgb")
        .arg("--metrics-recording-only")
        .arg("--no-first-run")
        .arg("--password-store=basic")
        .arg("--use-mock-keychain")
        // Stealth: language and locale
        .arg("--lang=en-US,en")
        // Stealth: realistic user agent (Chrome 120 on Windows)
        .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");

    builder
        .build()
        .map_err(|e| format!("Failed to build browser config: {}", e).into())
}

/// Minimal browser config for the daemon pool (no auto-detect path).
///
/// This is identical to `build_stealth_browser_config(None)` but lives here
/// so the daemon pool does not depend on the auto-download logic in
/// `js_preflight::chromium`.
pub fn build_pool_browser_config() -> Result<ChromeConfig, Box<dyn std::error::Error + Send + Sync>>
{
    build_stealth_browser_config(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth_config_builds() {
        let config = build_stealth_browser_config(None);
        assert!(config.is_ok(), "Browser config should build without error");
    }
}
