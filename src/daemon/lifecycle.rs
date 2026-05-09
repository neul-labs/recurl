//! Daemon lifecycle state machine
//!
//! Replaces ad-hoc `AtomicBool` shutdown flags and `Mutex<Instant>` idle tracking
//! with an explicit state machine that supports graceful shutdown.

use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

/// Lifecycle states for the recurld daemon
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonState {
    /// Daemon is starting up, warming the browser pool
    Starting,
    /// Daemon is running and accepting connections
    Running,
    /// Daemon is running but has no recent activity (can auto-shutdown)
    Idle,
    /// Daemon is shutting down, draining active requests
    ShuttingDown,
}

impl fmt::Display for DaemonState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaemonState::Starting => write!(f, "starting"),
            DaemonState::Running => write!(f, "running"),
            DaemonState::Idle => write!(f, "idle"),
            DaemonState::ShuttingDown => write!(f, "shutting_down"),
        }
    }
}

/// Tracks daemon lifecycle state and active requests
pub struct DaemonLifecycle {
    state: RwLock<DaemonState>,
    start_time: Instant,
    last_activity: RwLock<Instant>,
    idle_timeout: Duration,
    active_requests: tokio::sync::Semaphore,
    shutdown_timeout: Duration,
}

impl DaemonLifecycle {
    /// Create a new lifecycle tracker
    pub fn new(idle_timeout: Duration, max_concurrent_requests: usize) -> Self {
        let now = Instant::now();
        Self {
            state: RwLock::new(DaemonState::Starting),
            start_time: now,
            last_activity: RwLock::new(now),
            idle_timeout,
            active_requests: tokio::sync::Semaphore::new(max_concurrent_requests),
            shutdown_timeout: Duration::from_secs(30),
        }
    }

    /// Mark the daemon as fully started and ready to accept connections
    pub async fn mark_running(&self) {
        let mut state = self.state.write().await;
        *state = DaemonState::Running;
    }

    /// Record activity to prevent idle timeout
    pub async fn record_activity(&self) {
        let mut last = self.last_activity.write().await;
        *last = Instant::now();
    }

    /// Check if the daemon has exceeded its idle timeout
    pub async fn is_idle_timeout(&self) -> bool {
        let state = *self.state.read().await;
        if !matches!(state, DaemonState::Running | DaemonState::Idle) {
            return false;
        }

        let last = *self.last_activity.read().await;
        let elapsed = last.elapsed();

        // Transition Running -> Idle if timeout exceeded
        if elapsed > self.idle_timeout && state == DaemonState::Running {
            let mut state = self.state.write().await;
            if *state == DaemonState::Running {
                *state = DaemonState::Idle;
            }
        }

        elapsed > self.idle_timeout
    }

    /// Initiate graceful shutdown
    pub async fn initiate_shutdown(&self) {
        let mut state = self.state.write().await;
        *state = DaemonState::ShuttingDown;
    }

    /// Check if shutdown has been requested
    pub async fn is_shutting_down(&self) -> bool {
        *self.state.read().await == DaemonState::ShuttingDown
    }

    /// Wait for all active requests to complete (with timeout)
    pub async fn drain_active_requests(&self) {
        let start = Instant::now();
        while start.elapsed() < self.shutdown_timeout {
            match self.active_requests.try_acquire() {
                Ok(_) => {
                    // Acquired all permits => no active requests
                    return;
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Get the current state
    pub async fn current_state(&self) -> DaemonState {
        *self.state.read().await
    }

    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get a permit for handling a request (acquires a semaphore permit)
    pub async fn acquire_request_permit(
        &self,
    ) -> Result<tokio::sync::SemaphorePermit<'_>, tokio::sync::AcquireError> {
        self.active_requests.acquire().await
    }

    /// Get number of active requests (approximate, for stats)
    pub fn active_request_count(&self) -> usize {
        self.active_requests
            .available_permits()
            .saturating_sub(self.active_requests.available_permits())
    }
}

/// Shared handle to the daemon lifecycle
pub type LifecycleHandle = Arc<DaemonLifecycle>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lifecycle_transitions() {
        let lifecycle = DaemonLifecycle::new(Duration::from_secs(60), 10);

        assert_eq!(lifecycle.current_state().await, DaemonState::Starting);
        assert_eq!(lifecycle.uptime_secs(), 0);

        lifecycle.mark_running().await;
        assert_eq!(lifecycle.current_state().await, DaemonState::Running);

        lifecycle.initiate_shutdown().await;
        assert_eq!(lifecycle.current_state().await, DaemonState::ShuttingDown);
        assert!(lifecycle.is_shutting_down().await);
    }

    #[tokio::test]
    async fn test_idle_timeout() {
        let lifecycle = DaemonLifecycle::new(Duration::from_millis(50), 10);
        lifecycle.mark_running().await;

        assert!(!lifecycle.is_idle_timeout().await);

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(lifecycle.is_idle_timeout().await);
        assert_eq!(lifecycle.current_state().await, DaemonState::Idle);
    }

    #[tokio::test]
    async fn test_activity_resets_idle() {
        let lifecycle = DaemonLifecycle::new(Duration::from_millis(100), 10);
        lifecycle.mark_running().await;

        tokio::time::sleep(Duration::from_millis(50)).await;
        lifecycle.record_activity().await;

        tokio::time::sleep(Duration::from_millis(60)).await;
        // Total 110ms but last activity was 60ms ago
        assert!(!lifecycle.is_idle_timeout().await);
    }

    #[tokio::test]
    async fn test_request_permits() {
        let lifecycle = DaemonLifecycle::new(Duration::from_secs(60), 2);

        let permit1 = lifecycle.acquire_request_permit().await.unwrap();
        let permit2 = lifecycle.acquire_request_permit().await.unwrap();

        // Third should fail immediately
        let permit3 = lifecycle.active_requests.try_acquire();
        assert!(permit3.is_err());

        drop(permit1);
        drop(permit2);
    }
}
