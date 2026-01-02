//! Browser automation for JS preflight

use chromiumoxide::browser::{Browser, BrowserConfig as ChromeConfig};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

use super::chromium::ensure_chromium;
use super::cookies::{Cookie, ExtractedCookies};
use super::stealth;
use super::{PreflightOptions, PreflightResult};

/// Browser configuration for preflight
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// Path to Chrome/Chromium executable (None = auto-detect)
    pub chrome_path: Option<String>,
    /// Whether to run in headless mode
    pub headless: bool,
    /// Viewport width
    pub width: u32,
    /// Viewport height
    pub height: u32,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            chrome_path: None,
            headless: true,
            width: 1920,
            height: 1080,
        }
    }
}

/// Run the preflight operation
pub async fn run_preflight(
    url: &str,
    options: &PreflightOptions,
) -> Result<PreflightResult, Box<dyn std::error::Error + Send + Sync>> {
    // Ensure Chromium is available (download if necessary)
    let chrome_path = ensure_chromium(options.debug).await?;

    if options.debug {
        eprintln!("[rcurl] Using Chromium at: {}", chrome_path.display());
    }

    // Build browser config with stealth settings
    // Based on puppeteer-extra-plugin-stealth techniques
    let browser_config = ChromeConfig::builder()
        .chrome_executable(chrome_path)
        .disable_default_args()
        // Headless mode (new headless is less detectable)
        .arg("--headless=new")
        // Basic required args
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-gpu")
        // Window/viewport
        .arg("--window-size=1920,1080")
        .arg("--start-maximized")
        // Stealth: Disable automation indicators
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
        // Stealth: Language and locale
        .arg("--lang=en-US,en")
        // Stealth: Realistic user agent (Chrome 120 on Windows)
        .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Failed to build browser config: {}", e))?;

    // Launch browser with timeout
    let launch_result = timeout(
        Duration::from_secs(30),
        Browser::launch(browser_config),
    )
    .await
    .map_err(|_| "Browser launch timeout")?
    .map_err(|e| format!("Failed to launch browser: {}", e))?;

    let (browser, mut handler) = launch_result;

    // Spawn handler task
    let handler_task = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(e) = event {
                eprintln!("[rcurl] Browser event error: {}", e);
            }
        }
    });

    // Run the actual preflight
    let result = run_preflight_inner(&browser, url, options).await;

    // Clean up
    drop(browser);
    handler_task.abort();

    result
}

async fn run_preflight_inner(
    browser: &Browser,
    url: &str,
    options: &PreflightOptions,
) -> Result<PreflightResult, Box<dyn std::error::Error + Send + Sync>> {
    // Create new page (navigate to about:blank first to inject stealth)
    let page = timeout(
        Duration::from_secs(10),
        browser.new_page("about:blank"),
    )
    .await
    .map_err(|_| "Page creation timeout")?
    .map_err(|e| format!("Failed to create page: {}", e))?;

    // Inject stealth patches before navigation
    if options.debug {
        eprintln!("[rcurl] JS preflight: injecting stealth patches");
    }
    let stealth_js = stealth::get_all_patches();
    page.evaluate(stealth_js)
        .await
        .map_err(|e| format!("Failed to inject stealth patches: {}", e))?;

    // Navigate to target URL
    page.goto(url)
        .await
        .map_err(|e| format!("Failed to navigate to {}: {}", url, e))?;

    // Wait for navigation and any challenges to resolve
    let wait_duration = options.timeout;

    // First, wait for the page to load
    timeout(wait_duration, page.wait_for_navigation())
        .await
        .map_err(|_| "Navigation timeout")?
        .map_err(|e| format!("Navigation failed: {}", e))?;

    // If a wait selector is specified, wait for it
    if let Some(ref selector) = options.wait_selector {
        if options.debug {
            eprintln!("[rcurl] JS preflight: waiting for selector '{}'", selector);
        }
        timeout(
            wait_duration,
            page.find_element(selector),
        )
        .await
        .map_err(|_| format!("Timeout waiting for selector: {}", selector))?
        .map_err(|e| format!("Failed to find selector '{}': {}", selector, e))?;
    } else {
        // Default: wait a bit for JS challenges to resolve
        // Cloudflare typically takes 2-5 seconds
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Check for common challenge indicators and wait longer if needed
        if let Ok(html) = page.content().await {
            if html.contains("Just a moment")
                || html.contains("cf-browser-verification")
                || html.contains("challenge-platform")
            {
                if options.debug {
                    eprintln!("[rcurl] JS preflight: detected challenge, waiting longer...");
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    // Get final URL after any redirects
    let final_url = page
        .url()
        .await
        .map_err(|e| format!("Failed to get URL: {}", e))?
        .map(|u| u.to_string())
        .unwrap_or_else(|| url.to_string());

    // Extract cookies
    let cookies = extract_cookies(&page).await?;

    // Optionally get rendered HTML
    let rendered_html = if options.return_html {
        Some(
            page.content()
                .await
                .map_err(|e| format!("Failed to get page content: {}", e))?,
        )
    } else {
        None
    };

    if let Some(ref html) = rendered_html {
        Ok(PreflightResult::success_with_html(cookies, final_url, html.clone()))
    } else {
        Ok(PreflightResult::success(cookies, final_url))
    }
}

async fn extract_cookies(
    page: &chromiumoxide::Page,
) -> Result<ExtractedCookies, Box<dyn std::error::Error + Send + Sync>> {
    let chrome_cookies = page
        .get_cookies()
        .await
        .map_err(|e| format!("Failed to get cookies: {}", e))?;

    let mut extracted = ExtractedCookies::empty();

    for c in chrome_cookies {
        let cookie = Cookie::full(
            c.name,
            c.value,
            Some(c.domain),
            Some(c.path),
            c.secure,
            c.http_only,
        );
        extracted.add(cookie);
    }

    Ok(extracted)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_config_default() {
        let config = BrowserConfig::default();
        assert!(config.headless);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }
}
