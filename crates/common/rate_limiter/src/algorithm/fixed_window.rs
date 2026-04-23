//! Fixed Window Algorithm

/// Fixed window counter state
#[derive(Debug, Clone)]
pub struct FixedWindowState {
    pub count: u64,
    pub window_start: i64,
}

/// Fixed window rate limiter
#[derive(Debug, Clone)]
pub struct FixedWindow {
    pub window_size_ms: i64,
    pub max_requests: u64,
}

impl FixedWindow {
    pub fn new(window_size_ms: i64, max_requests: u64) -> Self {
        Self {
            window_size_ms,
            max_requests,
        }
    }

    /// Check if request is allowed
    pub fn check_and_consume(
        state: &mut FixedWindowState,
        window_size_ms: i64,
        max_requests: u64,
        cost: u64,
        now_ms: i64,
    ) -> bool {
        // Calculate current window
        let current_window = now_ms / window_size_ms;
        let state_window = state.window_start / window_size_ms;

        if current_window != state_window {
            // New window, reset counter
            state.count = 0;
            state.window_start = current_window * window_size_ms;
        }

        if state.count + cost <= max_requests {
            state.count += cost;
            true
        } else {
            false
        }
    }

    /// Get remaining requests
    pub fn get_remaining(
        state: &FixedWindowState,
        window_size_ms: i64,
        max_requests: u64,
        now_ms: i64,
    ) -> u64 {
        let current_window = now_ms / window_size_ms;
        let state_window = state.window_start / window_size_ms;

        if current_window != state_window {
            max_requests
        } else {
            max_requests.saturating_sub(state.count)
        }
    }

    /// Reset at window boundary
    pub fn should_reset(state: &FixedWindowState, window_size_ms: i64, now_ms: i64) -> bool {
        let current_window = now_ms / window_size_ms;
        let state_window = state.window_start / window_size_ms;
        current_window != state_window
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_window() {
        let mut state = FixedWindowState {
            count: 0,
            window_start: 0,
        };
        let window_size_ms = 1000;
        let max_requests = 10;

        // First request at t=100
        let allowed = FixedWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 100);
        assert!(allowed);
        assert_eq!(state.count, 1);

        // Within limit
        for i in 2..=10 {
            let allowed = FixedWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 100);
            assert!(allowed, "Request {} should be allowed", i);
        }

        // At limit
        let allowed = FixedWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 100);
        assert!(!allowed);
    }

    #[test]
    fn test_fixed_window_reset() {
        let mut state = FixedWindowState {
            count: 10,
            window_start: 0,
        };
        let window_size_ms = 1000;
        let max_requests = 10;

        // At limit in window 0
        let allowed = FixedWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 100);
        assert!(!allowed);

        // New window at t=1000
        let allowed = FixedWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 1000);
        assert!(allowed);
        assert_eq!(state.count, 1);
        assert_eq!(state.window_start, 1000);
    }
}
