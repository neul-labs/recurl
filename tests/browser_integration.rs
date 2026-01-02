//! Integration tests for headless Chrome
//!
//! These tests use rcurl's auto-download feature for Chromium.
//! On first run, Chromium will be downloaded to ~/.local/share/rcurl/chromium/
//!
//! Run with: cargo test --test browser_integration
//!
//! Tests verify:
//!   - Chromium auto-download
//!   - Browser launch and page creation
//!   - Navigation and content extraction
//!   - Cookie extraction for curl replay

use chromiumoxide_fetcher::{BrowserFetcher, BrowserFetcherOptions};
use std::path::PathBuf;
use std::time::Duration;

/// Get path to cached or auto-downloaded Chromium
async fn get_chromium_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rcurl")
        .join("chromium");

    // Check for existing chrome
    if cache_dir.exists() {
        for entry in std::fs::read_dir(&cache_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                let chrome = path.join("chrome-linux").join("chrome");
                if chrome.exists() {
                    return Ok(chrome);
                }
            }
        }
    }

    // Download if not present
    eprintln!("Downloading Chromium for tests...");
    std::fs::create_dir_all(&cache_dir)?;
    let options = BrowserFetcherOptions::builder()
        .with_path(&cache_dir)
        .build()?;
    let fetcher = BrowserFetcher::new(options);
    let info = fetcher.fetch().await?;
    Ok(info.executable_path)
}

#[test]
fn test_browser_launch() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        use chromiumoxide::browser::{Browser, BrowserConfig};
        use futures::StreamExt;

        let chrome_path = get_chromium_path().await.expect("Failed to get Chromium");

        let config = BrowserConfig::builder()
            .chrome_executable(chrome_path)
            .disable_default_args()
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .build()
            .expect("Failed to build config");

        let (browser, mut handler) = Browser::launch(config)
            .await
            .expect("Failed to launch browser");

        tokio::spawn(async move {
            while handler.next().await.is_some() {}
        });

        // Create a page
        let page = browser
            .new_page("about:blank")
            .await
            .expect("Failed to create page");

        // Verify we can get the URL
        let url = page.url().await.expect("Failed to get URL");
        assert!(url.is_some());

        drop(browser);
    });
}

#[test]
fn test_browser_navigation() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        use chromiumoxide::browser::{Browser, BrowserConfig};
        use futures::StreamExt;

        let chrome_path = get_chromium_path().await.expect("Failed to get Chromium");

        let config = BrowserConfig::builder()
            .chrome_executable(chrome_path)
            .disable_default_args()
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .build()
            .expect("Failed to build config");

        let (browser, mut handler) = Browser::launch(config)
            .await
            .expect("Failed to launch browser");

        tokio::spawn(async move {
            while handler.next().await.is_some() {}
        });

        // Navigate to httpbin
        let page = browser
            .new_page("https://httpbin.org/html")
            .await
            .expect("Failed to navigate");

        // Wait for content
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Get page content
        let content = page.content().await.expect("Failed to get content");
        assert!(
            content.contains("Herman Melville"),
            "Expected page to contain 'Herman Melville'"
        );

        drop(browser);
    });
}

#[test]
fn test_cookie_extraction() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        use chromiumoxide::browser::{Browser, BrowserConfig};
        use futures::StreamExt;

        let chrome_path = get_chromium_path().await.expect("Failed to get Chromium");

        let config = BrowserConfig::builder()
            .chrome_executable(chrome_path)
            .disable_default_args()
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .build()
            .expect("Failed to build config");

        let (browser, mut handler) = Browser::launch(config)
            .await
            .expect("Failed to launch browser");

        tokio::spawn(async move {
            while handler.next().await.is_some() {}
        });

        // Navigate to a page that sets cookies
        let page = browser
            .new_page("https://httpbin.org/cookies/set/testcookie/testvalue")
            .await
            .expect("Failed to navigate");

        // Wait for redirect
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Extract cookies
        let cookies = page.get_cookies().await.expect("Failed to get cookies");

        // Should have at least one cookie
        assert!(!cookies.is_empty(), "Expected at least one cookie");

        // Find our test cookie
        let test_cookie = cookies.iter().find(|c| c.name == "testcookie");
        assert!(test_cookie.is_some(), "Expected to find 'testcookie'");
        assert_eq!(test_cookie.unwrap().value, "testvalue");

        drop(browser);
    });
}
