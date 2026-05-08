//! Preflight process state machine
//!
//! Replaces the sequential async function chain in run_preflight_inner
//! with explicit states, making timeouts and retries easier to model.

use std::fmt;
use std::time::Duration;
use tokio::time::Instant;

/// States in a JS preflight operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightState {
    /// Preflight is initializing (browser launch, setup)
    Initializing,
    /// Navigating to about:blank for stealth injection
    InjectingStealth,
    /// Navigating to the target URL
    Navigating,
    /// Waiting for page load / challenge resolution
    WaitingForChallenge,
    /// Waiting for a specific CSS selector
    WaitingForSelector,
    /// Extracting cookies and final URL
    Extracting,
    /// Preflight completed successfully
    Complete,
    /// Preflight failed (timeout, error, etc.)
    Failed,
}

impl fmt::Display for PreflightState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreflightState::Initializing => write!(f, "initializing"),
            PreflightState::InjectingStealth => write!(f, "injecting_stealth"),
            PreflightState::Navigating => write!(f, "navigating"),
            PreflightState::WaitingForChallenge => write!(f, "waiting_for_challenge"),
            PreflightState::WaitingForSelector => write!(f, "waiting_for_selector"),
            PreflightState::Extracting => write!(f, "extracting"),
            PreflightState::Complete => write!(f, "complete"),
            PreflightState::Failed => write!(f, "failed"),
        }
    }
}

/// Tracks a preflight operation's state and timing
#[derive(Debug)]
pub struct PreflightStateMachine {
    state: PreflightState,
    started_at: Instant,
    current_step_started_at: Instant,
    url: String,
    step_durations: Vec<(PreflightState, Duration)>,
    error: Option<String>,
}

impl PreflightStateMachine {
    /// Create a new state machine for the given URL
    pub fn new(url: impl Into<String>) -> Self {
        let now = Instant::now();
        Self {
            state: PreflightState::Initializing,
            started_at: now,
            current_step_started_at: now,
            url: url.into(),
            step_durations: Vec::new(),
            error: None,
        }
    }

    /// Transition to a new state, recording the previous step's duration
    pub fn transition(&mut self, new_state: PreflightState) {
        let elapsed = self.current_step_started_at.elapsed();
        self.step_durations.push((self.state, elapsed));
        self.state = new_state;
        self.current_step_started_at = Instant::now();
    }

    /// Mark the preflight as complete
    pub fn mark_complete(&mut self) {
        self.transition(PreflightState::Complete);
    }

    /// Mark the preflight as failed with an error message
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
        self.transition(PreflightState::Failed);
    }

    /// Get the current state
    pub fn state(&self) -> PreflightState {
        self.state
    }

    /// Get the URL being prefetched
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the total elapsed time since the preflight started
    pub fn total_elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Get the elapsed time for the current step
    pub fn current_step_elapsed(&self) -> Duration {
        self.current_step_started_at.elapsed()
    }

    /// Get the error message if the preflight failed
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Check if the total elapsed time exceeds the given timeout
    pub fn is_timeout(&self, timeout: Duration) -> bool {
        self.total_elapsed() > timeout
    }

    /// Get a summary of all step durations for debugging
    pub fn step_summary(&self) -> String {
        let parts: Vec<String> = self
            .step_durations
            .iter()
            .map(|(state, duration)| format!("{}: {:.1}s", state, duration.as_secs_f64()))
            .collect();
        parts.join(", ")
    }
}

/// Result of a preflight step
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightStepResult {
    /// Step succeeded, continue to next state
    Continue,
    /// Step failed with an error
    Failed(String),
    /// Step timed out
    Timeout,
    /// Step succeeded and preflight is complete
    Complete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let mut sm = PreflightStateMachine::new("https://example.com");
        assert_eq!(sm.state(), PreflightState::Initializing);
        assert_eq!(sm.url(), "https://example.com");

        sm.transition(PreflightState::Navigating);
        assert_eq!(sm.state(), PreflightState::Navigating);

        sm.transition(PreflightState::WaitingForChallenge);
        assert_eq!(sm.state(), PreflightState::WaitingForChallenge);

        sm.mark_complete();
        assert_eq!(sm.state(), PreflightState::Complete);
        assert!(sm.error().is_none());
    }

    #[test]
    fn test_failed_state() {
        let mut sm = PreflightStateMachine::new("https://example.com");
        sm.mark_failed("navigation timeout");
        assert_eq!(sm.state(), PreflightState::Failed);
        assert_eq!(sm.error(), Some("navigation timeout"));
    }

    #[test]
    fn test_timeout_check() {
        let sm = PreflightStateMachine::new("https://example.com");
        assert!(!sm.is_timeout(Duration::from_secs(60)));
        // Zero timeout should immediately be considered timed out
        assert!(sm.is_timeout(Duration::from_millis(0)));
    }

    #[test]
    fn test_step_summary() {
        let mut sm = PreflightStateMachine::new("https://example.com");
        sm.transition(PreflightState::Navigating);
        sm.transition(PreflightState::WaitingForChallenge);
        sm.mark_complete();

        let summary = sm.step_summary();
        assert!(summary.contains("initializing"));
        assert!(summary.contains("navigating"));
        assert!(summary.contains("waiting_for_challenge"));
    }
}
