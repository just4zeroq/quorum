//! Sliding Window Algorithm

use std::collections::VecDeque;

/// Sliding window counter entry
#[derive(Debug, Clone)]
pub struct SlidingWindowEntry {
    pub timestamp_ms: i64,
    pub count: u64,
}

/// Sliding window algorithm state
#[derive(Debug, Clone)]
pub struct SlidingWindowState {
    pub requests: VecDeque<SlidingWindowEntry>,
}

/// Sliding window rate limiter
#[derive(Debug, Clone)]
pub struct SlidingWindow {
    pub window_size_ms: i64,
    pub max_requests: u64,
}

impl SlidingWindow {
    pub fn new(window_size_ms: i64, max_requests: u64) -> Self {
        Self {
            window_size_ms,
            max_requests,
        }
    }

    /// Check if request is allowed
    pub fn check_and_consume(
        state: &mut SlidingWindowState,
        window_size_ms: i64,
        max_requests: u64,
        cost: u64,
        now_ms: i64,
    ) -> bool {
        // Remove expired entries
        let window_start = now_ms - window_size_ms;
        while let Some(entry) = state.requests.front() {
            if entry.timestamp_ms < window_start {
                state.requests.pop_front();
            } else {
                break;
            }
        }

        // Count current requests
        let current_count: u64 = state.requests.iter().map(|e| e.count).sum();

        if current_count + cost <= max_requests {
            // Add new entry
            state.requests.push_back(SlidingWindowEntry {
                timestamp_ms: now_ms,
                count: cost,
            });
            true
        } else {
            false
        }
    }

    /// Get remaining requests
    pub fn get_remaining(
        state: &SlidingWindowState,
        window_size_ms: i64,
        max_requests: u64,
        now_ms: i64,
    ) -> u64 {
        let window_start = now_ms - window_size_ms;
        let current_count: u64 = state
            .requests
            .iter()
            .filter(|e| e.timestamp_ms >= window_start)
            .map(|e| e.count)
            .sum();

        max_requests.saturating_sub(current_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window() {
        let mut state = SlidingWindowState {
            requests: VecDeque::new(),
        };
        let window_size_ms = 1000;
        let max_requests = 10;

        // First request
        let allowed = SlidingWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 100);
        assert!(allowed);

        // Within limit
        for _ in 0..8 {
            let allowed = SlidingWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 100);
            assert!(allowed);
        }

        // At limit
        let allowed = SlidingWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 100);
        assert!(!allowed);
    }

    #[test]
    fn test_sliding_window_expiry() {
        let mut state = SlidingWindowState {
            requests: VecDeque::new(),
        };
        let window_size_ms = 1000;
        let max_requests = 5;

        // Use up all requests
        for _ in 0..5 {
            SlidingWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 100);
        }

        // At limit
        let allowed = SlidingWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 100);
        assert!(!allowed);

        // After window expires
        let allowed = SlidingWindow::check_and_consume(&mut state, window_size_ms, max_requests, 1, 1100);
        assert!(allowed);
    }
}
