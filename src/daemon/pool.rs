//! Browser pool management
//!
//! Maintains a pool of warm Chromium instances for fast JS preflight.

use chromiumoxide::browser::Browser;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};

use super::browser_state::{BrowserState, BrowserStateTracker};
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
        self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0)
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
    state_tracker: BrowserStateTracker,
}

/// Browser pool for JS preflight operations
pub struct BrowserPool {
    config: PoolConfig,
    browsers: Mutex<Vec<PooledBrowser>>,
    stats: Arc<PoolStats>,
    /// Cookie cache per domain
    cookie_cache: RwLock<HashMap<String, HashMap<String, String>>>,
    /// Semaphore to limit concurrent browser creation
    creation_semaphore: Semaphore,
}

impl BrowserPool {
    /// Create a new browser pool
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            browsers: Mutex::new(Vec::new()),
            stats: Arc::new(PoolStats::new()),
            cookie_cache: RwLock::new(HashMap::new()),
            creation_semaphore: Semaphore::new(2),
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
        for _ in 0..self.config.min_size {
            match self.create_browser().await {
                Ok(browser) => {
                    let mut browsers = self.browsers.lock().await;
                    browsers.push(PooledBrowser {
                        browser,
                        state_tracker: BrowserStateTracker::new(),
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

        let result = self
            .do_preflight(url, timeout_ms, wait_selector, return_html)
            .await;

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
        let pooled = match self.acquire_browser().await {
            Ok(b) => b,
            Err(e) => {
                return DaemonResponse::PreflightError {
                    error: format!("Failed to acquire browser: {}", e),
                };
            }
        };

        let timeout = Duration::from_millis(timeout_ms.unwrap_or(30000));

        // Execute preflight
        let result = self
            .run_preflight(&pooled.browser, url, timeout, wait_selector, return_html)
            .await;

        // Health check: can the browser still create pages?
        let healthy = matches!(
            tokio::time::timeout(
                Duration::from_secs(5),
                pooled.browser.new_page("about:blank")
            )
            .await,
            Ok(Ok(_))
        );

        // Return browser to pool or destroy if unhealthy
        self.release_browser(pooled, healthy).await;

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
        let page =
            match tokio::time::timeout(Duration::from_secs(10), browser.new_page("about:blank"))
                .await
            {
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

        // Inject stealth patches before navigation
        let stealth_js = crate::stealth::get_all_patches();
        if let Err(e) = page.evaluate(stealth_js).await {
            return DaemonResponse::PreflightError {
                error: format!("Failed to inject stealth: {}", e),
            };
        }

        // Navigate to target URL
        if let Err(e) = page.goto(url).await {
            return DaemonResponse::PreflightError {
                error: format!("Failed to navigate: {}", e),
            };
        }

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

            // Check for common challenge indicators and wait longer if needed
            if let Ok(html) = page.content().await {
                if html.contains("Just a moment")
                    || html.contains("cf-browser-verification")
                    || html.contains("challenge-platform")
                {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
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
            page.content().await.ok()
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

    async fn acquire_browser(&self) -> Result<PooledBrowser, String> {
        let mut browsers = self.browsers.lock().await;

        // Try to get an existing ready browser
        if let Some(idx) = browsers
            .iter()
            .position(|p| p.state_tracker.state() == BrowserState::Ready)
        {
            let mut pooled = browsers.remove(idx);
            pooled.state_tracker.mark_in_use();
            return Ok(pooled);
        }

        // No ready browsers available; drop lock before creating
        drop(browsers);
        self.create_browser_with_tracker().await
    }

    async fn release_browser(&self, mut pooled: PooledBrowser, healthy: bool) {
        if !healthy {
            pooled.state_tracker.mark_unhealthy();
        }

        if pooled.state_tracker.state().should_destroy() {
            // Drop the browser without returning it to the pool
            return;
        }

        let mut browsers = self.browsers.lock().await;

        // Check for expired browsers while we have the lock
        browsers.retain(|b| !b.state_tracker.is_idle_expired(self.config.idle_timeout));

        // Only keep if under max size
        if browsers.len() < self.config.max_size {
            pooled.state_tracker.mark_ready_after_use();
            browsers.push(pooled);
        }
        // Otherwise, browser is dropped
    }

    async fn create_browser_with_tracker(&self) -> Result<PooledBrowser, String> {
        let browser = self.create_browser().await?;
        let mut tracker = BrowserStateTracker::new();
        tracker.mark_ready();
        Ok(PooledBrowser {
            browser,
            state_tracker: tracker,
        })
    }

    async fn create_browser(&self) -> Result<Browser, String> {
        let _permit = self
            .creation_semaphore
            .acquire()
            .await
            .map_err(|_| "Failed to acquire creation semaphore")?;

        let config = crate::browser_config::build_pool_browser_config()
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

        // Mark expired browsers
        for b in browsers.iter_mut() {
            if b.state_tracker.is_idle_expired(self.config.idle_timeout) {
                b.state_tracker.mark_expired();
            }
        }

        // Remove expired/unhealthy browsers
        browsers.retain(|b| !b.state_tracker.state().should_destroy());

        // Ensure minimum pool size
        let current = browsers.len();
        drop(browsers);

        if current < self.config.min_size {
            for _ in current..self.config.min_size {
                if let Ok(browser) = self.create_browser().await {
                    let mut browsers = self.browsers.lock().await;
                    browsers.push(PooledBrowser {
                        browser,
                        state_tracker: BrowserStateTracker::new(),
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
