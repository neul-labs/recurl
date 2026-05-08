//! Browser instance state machine
//!
//! Each pooled browser tracks its state explicitly so that unhealthy
//! browsers are destroyed instead of returned to the pool.

use std::fmt;
use std::time::{Duration, Instant};

/// Lifecycle states for a pooled browser instance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserState {
    /// Browser process is being created
    Creating,
    /// Browser is ready and available in the pool
    Ready,
    /// Browser is currently in use for a preflight
    InUse,
    /// Browser failed a health check or preflight error
    Unhealthy,
    /// Browser has been idle too long and should be recycled
    Expired,
}

impl fmt::Display for BrowserState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowserState::Creating => write!(f, "creating"),
            BrowserState::Ready => write!(f, "ready"),
            BrowserState::InUse => write!(f, "in_use"),
            BrowserState::Unhealthy => write!(f, "unhealthy"),
            BrowserState::Expired => write!(f, "expired"),
        }
    }
}

impl BrowserState {
    /// Returns true if the browser can be reused (returned to the pool)
    pub fn is_reusable(self) -> bool {
        matches!(self, BrowserState::Ready)
    }

    /// Returns true if the browser should be destroyed
    pub fn should_destroy(self) -> bool {
        matches!(self, BrowserState::Unhealthy | BrowserState::Expired)
    }
}

/// Tracks the state of a single browser instance over time
#[derive(Debug)]
pub struct BrowserStateTracker {
    state: BrowserState,
    created_at: Instant,
    last_used: Instant,
    use_count: u64,
}

impl BrowserStateTracker {
    /// Create a new tracker in the Creating state
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            state: BrowserState::Creating,
            created_at: now,
            last_used: now,
            use_count: 0,
        }
    }

    /// Mark the browser as ready (after successful creation)
    pub fn mark_ready(&mut self) {
        self.state = BrowserState::Ready;
        self.created_at = Instant::now();
        self.last_used = Instant::now();
    }

    /// Mark the browser as in use
    pub fn mark_in_use(&mut self) {
        self.state = BrowserState::InUse;
        self.use_count += 1;
    }

    /// Mark the browser as ready after use (before returning to pool)
    pub fn mark_ready_after_use(&mut self) {
        self.state = BrowserState::Ready;
        self.last_used = Instant::now();
    }

    /// Mark the browser as unhealthy
    pub fn mark_unhealthy(&mut self) {
        self.state = BrowserState::Unhealthy;
    }

    /// Mark the browser as expired (idle timeout exceeded)
    pub fn mark_expired(&mut self) {
        self.state = BrowserState::Expired;
    }

    /// Check if the browser has exceeded the idle timeout
    pub fn is_idle_expired(&self, idle_timeout: Duration) -> bool {
        self.last_used.elapsed() > idle_timeout
    }

    /// Get the current state
    pub fn state(&self) -> BrowserState {
        self.state
    }

    /// Get the number of times this browser has been used
    pub fn use_count(&self) -> u64 {
        self.use_count
    }

    /// Get the age of this browser
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get time since last use
    pub fn idle_time(&self) -> Duration {
        self.last_used.elapsed()
    }
}

impl Default for BrowserStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let mut tracker = BrowserStateTracker::new();
        assert_eq!(tracker.state(), BrowserState::Creating);

        tracker.mark_ready();
        assert_eq!(tracker.state(), BrowserState::Ready);
        assert!(tracker.state().is_reusable());
        assert!(!tracker.state().should_destroy());

        tracker.mark_in_use();
        assert_eq!(tracker.state(), BrowserState::InUse);
        assert_eq!(tracker.use_count(), 1);
        assert!(!tracker.state().is_reusable());

        tracker.mark_ready_after_use();
        assert_eq!(tracker.state(), BrowserState::Ready);

        tracker.mark_unhealthy();
        assert_eq!(tracker.state(), BrowserState::Unhealthy);
        assert!(!tracker.state().is_reusable());
        assert!(tracker.state().should_destroy());
    }

    #[test]
    fn test_idle_expiry() {
        let mut tracker = BrowserStateTracker::new();
        tracker.mark_ready();

        // Should not be expired immediately
        assert!(!tracker.is_idle_expired(Duration::from_secs(60)));

        // Manually mark expired
        tracker.mark_expired();
        assert!(tracker.state().should_destroy());
    }
}
