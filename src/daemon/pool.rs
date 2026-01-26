//! Browser pool management
//!
//! Maintains a pool of warm Chromium instances for fast JS preflight.

use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

use super::protocol::DaemonResponse;

/// Configuration for the browser pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of warm browsers
    pub min_size: usize,
    /// Maximum number of browsers
    pub max_size: usize,
    /// Browser idle timeout before recycling
    pub idle_timeout: Duration,
    /// Default preflight timeout
    pub default_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: 1,
            max_size: 3,
            idle_timeout: Duration::from_secs(300), // 5 minutes
            default_timeout: Duration::from_secs(30),
        }
    }
}

/// Statistics for the pool
#[derive(Debug, Default)]
pub struct PoolStats {
    /// Total requests served
    pub requests_served: AtomicU64,
    /// Currently active requests
    pub active_requests: AtomicUsize,
    /// Start time
    pub start_time: Option<Instant>,
}

impl PoolStats {
    pub fn new() -> Self {
        Self {
            requests_served: AtomicU64::new(0),
            active_requests: AtomicUsize::new(0),
            start_time: Some(Instant::now()),
        }
    }

    pub fn uptime_secs(&self) -> u64 {
        self.start_time
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
    }

    pub fn increment_served(&self) {
        self.requests_served.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_active(&self) {
        self.active_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_active(&self) {
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
    }
}

/// A browser instance in the pool
struct PooledBrowser {
    browser: Browser,
    #[allow(dead_code)]
    created_at: Instant,
    last_used: Instant,
}

/// Browser pool for JS preflight operations
pub struct BrowserPool {
    config: PoolConfig,
    browsers: Mutex<Vec<PooledBrowser>>,
    stats: Arc<PoolStats>,
    /// Cookie cache per domain
    cookie_cache: RwLock<HashMap<String, HashMap<String, String>>>,
}

impl BrowserPool {
    /// Create a new browser pool
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            browsers: Mutex::new(Vec::new()),
            stats: Arc::new(PoolStats::new()),
            cookie_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> Arc<PoolStats> {
        Arc::clone(&self.stats)
    }

    /// Get current pool size
    pub async fn size(&self) -> usize {
        self.browsers.lock().await.len()
    }

    /// Warm up the pool with initial browsers
    pub async fn warmup(&self) -> Result<(), String> {
        let mut browsers = self.browsers.lock().await;

        for _ in 0..self.config.min_size {
            match self.create_browser().await {
                Ok(browser) => {
                    browsers.push(PooledBrowser {
                        browser,
                        created_at: Instant::now(),
                        last_used: Instant::now(),
                    });
                }
                Err(e) => {
                    return Err(format!("Failed to warm up pool: {}", e));
                }
            }
        }

        Ok(())
    }

    /// Execute a JS preflight using a pooled browser
    pub async fn execute_preflight(
        &self,
        url: &str,
        timeout_ms: Option<u64>,
        wait_selector: Option<String>,
        return_html: bool,
    ) -> DaemonResponse {
        self.stats.increment_active();

        let result = self.do_preflight(url, timeout_ms, wait_selector, return_html).await;

        self.stats.decrement_active();
        self.stats.increment_served();

        result
    }

    async fn do_preflight(
        &self,
        url: &str,
        timeout_ms: Option<u64>,
        wait_selector: Option<String>,
        return_html: bool,
    ) -> DaemonResponse {
        // Try to get a browser from the pool
        let browser = match self.acquire_browser().await {
            Ok(b) => b,
            Err(e) => {
                return DaemonResponse::PreflightError {
                    error: format!("Failed to acquire browser: {}", e),
                };
            }
        };

        let timeout = Duration::from_millis(timeout_ms.unwrap_or(30000));

        // Execute preflight
        let result = self.run_preflight(&browser, url, timeout, wait_selector, return_html).await;

        // Return browser to pool
        self.release_browser(browser).await;

        result
    }

    async fn run_preflight(
        &self,
        browser: &Browser,
        url: &str,
        timeout: Duration,
        wait_selector: Option<String>,
        return_html: bool,
    ) -> DaemonResponse {
        // Create new page
        let page = match tokio::time::timeout(Duration::from_secs(10), browser.new_page(url)).await {
            Ok(Ok(page)) => page,
            Ok(Err(e)) => {
                return DaemonResponse::PreflightError {
                    error: format!("Failed to create page: {}", e),
                };
            }
            Err(_) => {
                return DaemonResponse::PreflightError {
                    error: "Page creation timeout".to_string(),
                };
            }
        };

        // Wait for navigation
        if let Err(e) = tokio::time::timeout(timeout, page.wait_for_navigation()).await {
            return DaemonResponse::PreflightError {
                error: format!("Navigation timeout: {}", e),
            };
        }

        // Wait for selector if specified
        if let Some(selector) = wait_selector {
            match tokio::time::timeout(timeout, page.find_element(&selector)).await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    return DaemonResponse::PreflightError {
                        error: format!("Failed to find selector '{}': {}", selector, e),
                    };
                }
                Err(_) => {
                    return DaemonResponse::PreflightError {
                        error: format!("Timeout waiting for selector: {}", selector),
                    };
                }
            }
        } else {
            // Default wait for JS challenges
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        // Get final URL
        let final_url = match page.url().await {
            Ok(Some(u)) => u.to_string(),
            _ => url.to_string(),
        };

        // Extract cookies
        let cookies: std::collections::HashMap<String, String> = match page.get_cookies().await {
            Ok(c) => c
                .into_iter()
                .map(|cookie| (cookie.name, cookie.value))
                .collect(),
            Err(e) => {
                return DaemonResponse::PreflightError {
                    error: format!("Failed to get cookies: {}", e),
                };
            }
        };

        // Get HTML if requested
        let html = if return_html {
            match page.content().await {
                Ok(h) => Some(h),
                Err(_) => None,
            }
        } else {
            None
        };

        // Cache cookies for this domain
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(domain) = parsed_url.domain() {
                let mut cache = self.cookie_cache.write().await;
                cache.insert(domain.to_string(), cookies.clone());
            }
        }

        DaemonResponse::PreflightSuccess {
            cookies,
            final_url,
            html,
        }
    }

    async fn acquire_browser(&self) -> Result<Browser, String> {
        let mut browsers = self.browsers.lock().await;

        // Try to get an existing browser
        if let Some(pooled) = browsers.pop() {
            return Ok(pooled.browser);
        }

        // Create a new one if pool is empty
        drop(browsers); // Release lock before async operation
        self.create_browser().await
    }

    async fn release_browser(&self, browser: Browser) {
        let mut browsers = self.browsers.lock().await;

        // Only keep if under max size
        if browsers.len() < self.config.max_size {
            browsers.push(PooledBrowser {
                browser,
                created_at: Instant::now(),
                last_used: Instant::now(),
            });
        }
        // Otherwise, browser is dropped
    }

    async fn create_browser(&self) -> Result<Browser, String> {
        let config = BrowserConfig::builder()
            .disable_default_args()
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--window-size=1920,1080")
            .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| format!("Failed to build browser config: {}", e))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| format!("Failed to launch browser: {}", e))?;

        // Spawn handler task
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    eprintln!("[recurld] Browser event error: {}", e);
                }
            }
        });

        Ok(browser)
    }

    /// Clean up idle browsers
    pub async fn cleanup_idle(&self) {
        let mut browsers = self.browsers.lock().await;
        let now = Instant::now();

        browsers.retain(|b| {
            let idle_time = now.duration_since(b.last_used);
            idle_time < self.config.idle_timeout
        });

        // Ensure minimum pool size
        let current = browsers.len();
        drop(browsers);

        if current < self.config.min_size {
            for _ in current..self.config.min_size {
                if let Ok(browser) = self.create_browser().await {
                    let mut browsers = self.browsers.lock().await;
                    browsers.push(PooledBrowser {
                        browser,
                        created_at: Instant::now(),
                        last_used: Instant::now(),
                    });
                }
            }
        }
    }

    /// Get cached cookies for a domain
    pub async fn get_cached_cookies(&self, domain: &str) -> Option<HashMap<String, String>> {
        let cache = self.cookie_cache.read().await;
        cache.get(domain).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.min_size, 1);
        assert_eq!(config.max_size, 3);
    }

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats::new();
        assert_eq!(stats.requests_served.load(Ordering::Relaxed), 0);

        stats.increment_served();
        assert_eq!(stats.requests_served.load(Ordering::Relaxed), 1);

        stats.increment_active();
        assert_eq!(stats.active_requests.load(Ordering::Relaxed), 1);

        stats.decrement_active();
        assert_eq!(stats.active_requests.load(Ordering::Relaxed), 0);
    }
}
