//! Token Bucket Algorithm

use std::time::Duration;

/// Token bucket state
#[derive(Debug, Clone)]
pub struct TokenBucketState {
    pub tokens: f64,
    pub last_update: i64,
}

/// Token bucket algorithm
#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: f64,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    pub fn new(capacity: u64, refill_rate: f64) -> Self {
        Self {
            capacity: capacity as f64,
            refill_rate,
        }
    }

    /// Calculate if request is allowed and update state
    pub fn check_and_consume(
        state: &mut TokenBucketState,
        capacity: f64,
        refill_rate: f64,
        cost: f64,
        now_ms: i64,
    ) -> bool {
        // Refill tokens
        let elapsed = (now_ms - state.last_update) as f64 / 1000.0;
        state.tokens = (state.tokens + elapsed * refill_rate).min(capacity);
        state.last_update = now_ms;

        if state.tokens >= cost {
            state.tokens -= cost;
            true
        } else {
            false
        }
    }

    /// Get remaining tokens
    pub fn get_remaining(state: &TokenBucketState, capacity: f64, refill_rate: f64, now_ms: i64) -> f64 {
        let elapsed = (now_ms - state.last_update) as f64 / 1000.0;
        (state.tokens + elapsed * refill_rate).min(capacity)
    }

    /// Time until full refill
    pub fn time_until_full(state: &TokenBucketState, capacity: f64, refill_rate: f64) -> Duration {
        if state.tokens >= capacity {
            Duration::ZERO
        } else {
            let needed = capacity - state.tokens;
            Duration::from_secs_f64(needed / refill_rate)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket() {
        let mut state = TokenBucketState {
            tokens: 10.0,
            last_update: 0,
        };
        let capacity = 10.0;
        let refill_rate = 1.0; // 1 token per second

        // Consume 5 tokens
        let allowed = TokenBucket::check_and_consume(&mut state, capacity, refill_rate, 5.0, 1000);
        assert!(allowed);
        assert_eq!(state.tokens, 5.0);

        // Consume remaining 5
        let allowed = TokenBucket::check_and_consume(&mut state, capacity, refill_rate, 5.0, 1000);
        assert!(allowed);
        assert_eq!(state.tokens, 0.0);

        // Should fail - no tokens
        let allowed = TokenBucket::check_and_consume(&mut state, capacity, refill_rate, 1.0, 1000);
        assert!(!allowed);
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut state = TokenBucketState {
            tokens: 5.0,
            last_update: 0,
        };
        let capacity = 10.0;
        let refill_rate = 10.0; // 10 tokens per second

        // Wait 500ms, should refill 5 tokens
        let remaining = TokenBucket::get_remaining(&state, capacity, refill_rate, 500);
        assert_eq!(remaining, 10.0); // capped at capacity

        // Wait 1 second from start
        let remaining = TokenBucket::get_remaining(&state, capacity, refill_rate, 1000);
        assert_eq!(remaining, 10.0);
    }
}
