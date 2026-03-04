//! Signal handling for graceful shutdown on SIGINT (Ctrl+C)
//!
//! This module provides a shared cancellation state that allows the signal handler
//! to communicate with the execution pipeline. The atomic state ensures thread-safe
//! cancellation signaling.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

/// Cancellation state shared across signal handler and execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CancellationState {
    /// Normal execution, no cancellation requested
    Running = 0,
    /// First SIGINT received - graceful shutdown with cleanup
    FirstSignal = 1,
    /// Second SIGINT received - force immediate exit
    SecondSignal = 2,
}

/// Thread-safe cancellation state for signal handling
///
/// This struct is cloned across the signal handler task and the execution pipeline.
/// The signal handler increments the state on each SIGINT, while the execution
/// pipeline checks the state to determine if it should cancel.
#[derive(Debug, Clone)]
pub struct Cancellation {
    state: Arc<AtomicU8>,
}

impl Cancellation {
    /// Create a new cancellation state in the Running state
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(CancellationState::Running as u8)),
        }
    }

    /// Check if cancellation has been requested (first or second signal)
    pub fn is_cancelled(&self) -> bool {
        self.state.load(Ordering::Relaxed) >= CancellationState::FirstSignal as u8
    }

    /// Check if force quit has been requested (second signal)
    pub fn should_force_quit(&self) -> bool {
        self.state.load(Ordering::Relaxed) >= CancellationState::SecondSignal as u8
    }

    /// Request cancellation by incrementing the state
    ///
    /// First call: Running → FirstSignal (graceful)
    /// Second call: FirstSignal → SecondSignal (force quit)
    pub fn request_cancel(&self) {
        self.state.fetch_add(1, Ordering::SeqCst);
    }
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_states() {
        let cancel = Cancellation::new();

        assert!(!cancel.is_cancelled());
        assert!(!cancel.should_force_quit());

        cancel.request_cancel();
        assert!(cancel.is_cancelled());
        assert!(!cancel.should_force_quit());

        cancel.request_cancel();
        assert!(cancel.is_cancelled());
        assert!(cancel.should_force_quit());
    }

    #[test]
    fn test_cancellation_clone() {
        let cancel1 = Cancellation::new();
        let cancel2 = cancel1.clone();

        cancel1.request_cancel();
        assert!(cancel2.is_cancelled());
    }
}
