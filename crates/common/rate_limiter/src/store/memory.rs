//! Memory Store (for testing)

use std::collections::HashMap;
use async_trait::async_trait;
use tokio::sync::RwLock as TokioRwLock;

use crate::traits::{RateLimiter, RateLimitKey, RateLimitResult, RateLimitError, Result};
use crate::algorithm::{TokenBucket, SlidingWindow, FixedWindow};
use crate::algorithm::token_bucket::TokenBucketState;
use crate::algorithm::sliding_window::SlidingWindowState;
use crate::algorithm::fixed_window::FixedWindowState;

/// Algorithm type
#[derive(Debug, Clone)]
pub enum Algorithm {
    TokenBucket { capacity: f64, refill_rate: f64 },
    SlidingWindow { window_ms: i64, max_requests: u64 },
    FixedWindow { window_ms: i64, max_requests: u64 },
}

/// Memory store state
pub struct MemoryStoreInner {
    token_bucket: HashMap<String, TokenBucketState>,
    sliding_window: HashMap<String, SlidingWindowState>,
    fixed_window: HashMap<String, FixedWindowState>,
}

/// Memory store implementation
pub struct MemoryStore {
    algorithm: Algorithm,
    inner: TokioRwLock<MemoryStoreInner>,
}

impl MemoryStore {
    pub fn new(algorithm: Algorithm) -> Self {
        Self {
            algorithm,
            inner: TokioRwLock::new(MemoryStoreInner {
                token_bucket: HashMap::new(),
                sliding_window: HashMap::new(),
                fixed_window: HashMap::new(),
            }),
        }
    }

    async fn check_token_bucket(&self, key: &RateLimitKey, cost: u64, now_ms: i64) -> Result<RateLimitResult> {
        let mut inner = self.inner.write().await;
        let k = key.key();

        if let Algorithm::TokenBucket { capacity, refill_rate } = self.algorithm {
            let state = inner.token_bucket.entry(k).or_insert_with(|| TokenBucketState {
                tokens: capacity,
                last_update: now_ms,
            });

            let allowed = TokenBucket::check_and_consume(
                state,
                capacity,
                refill_rate,
                cost as f64,
                now_ms,
            );

            let remaining = TokenBucket::get_remaining(state, capacity, refill_rate, now_ms);
            let reset_ms = now_ms + ((capacity - remaining) / refill_rate * 1000.0) as i64;

            Ok(RateLimitResult {
                allowed,
                remaining: remaining as u64,
                reset_at_ms: reset_ms,
            })
        } else {
            Err(RateLimitError::LimitExceeded)
        }
    }

    async fn check_sliding_window(&self, key: &RateLimitKey, cost: u64, now_ms: i64) -> Result<RateLimitResult> {
        let mut inner = self.inner.write().await;
        let k = key.key();

        if let Algorithm::SlidingWindow { window_ms, max_requests } = self.algorithm {
            let state = inner.sliding_window.entry(k).or_insert_with(|| SlidingWindowState {
                requests: std::collections::VecDeque::new(),
            });

            let allowed = SlidingWindow::check_and_consume(state, window_ms, max_requests, cost, now_ms);
            let remaining = SlidingWindow::get_remaining(state, window_ms, max_requests, now_ms);

            Ok(RateLimitResult {
                allowed,
                remaining,
                reset_at_ms: now_ms + window_ms,
            })
        } else {
            Err(RateLimitError::LimitExceeded)
        }
    }

    async fn check_fixed_window(&self, key: &RateLimitKey, cost: u64, now_ms: i64) -> Result<RateLimitResult> {
        let mut inner = self.inner.write().await;
        let k = key.key();

        if let Algorithm::FixedWindow { window_ms, max_requests } = self.algorithm {
            let state = inner.fixed_window.entry(k).or_insert_with(|| FixedWindowState {
                count: 0,
                window_start: 0,
            });

            let allowed = FixedWindow::check_and_consume(state, window_ms, max_requests, cost, now_ms);
            let remaining = FixedWindow::get_remaining(state, window_ms, max_requests, now_ms);

            Ok(RateLimitResult {
                allowed,
                remaining,
                reset_at_ms: state.window_start + window_ms,
            })
        } else {
            Err(RateLimitError::LimitExceeded)
        }
    }
}

#[async_trait]
impl RateLimiter for MemoryStore {
    async fn check_and_consume(&self, key: &RateLimitKey, cost: u64) -> Result<RateLimitResult> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        match self.algorithm {
            Algorithm::TokenBucket { .. } => self.check_token_bucket(key, cost, now_ms).await,
            Algorithm::SlidingWindow { .. } => self.check_sliding_window(key, cost, now_ms).await,
            Algorithm::FixedWindow { .. } => self.check_fixed_window(key, cost, now_ms).await,
        }
    }

    async fn get_limit(&self, key: &RateLimitKey) -> Result<RateLimitResult> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        match self.algorithm {
            Algorithm::TokenBucket { capacity, refill_rate } => {
                let inner = self.inner.read().await;
                let k = key.key();
                let state = inner.token_bucket.get(&k).cloned().unwrap_or(TokenBucketState {
                    tokens: capacity,
                    last_update: now_ms,
                });
                let remaining = TokenBucket::get_remaining(&state, capacity, refill_rate, now_ms);
                Ok(RateLimitResult {
                    allowed: true,
                    remaining: remaining as u64,
                    reset_at_ms: now_ms + ((capacity - remaining) / refill_rate * 1000.0) as i64,
                })
            },
            Algorithm::SlidingWindow { window_ms, max_requests } => {
                let inner = self.inner.read().await;
                let k = key.key();
                let state = inner.sliding_window.get(&k).cloned().unwrap_or(SlidingWindowState {
                    requests: std::collections::VecDeque::new(),
                });
                let remaining = SlidingWindow::get_remaining(&state, window_ms, max_requests, now_ms);
                Ok(RateLimitResult {
                    allowed: true,
                    remaining,
                    reset_at_ms: now_ms + window_ms,
                })
            },
            Algorithm::FixedWindow { window_ms, max_requests } => {
                let inner = self.inner.read().await;
                let k = key.key();
                let state = inner.fixed_window.get(&k).cloned().unwrap_or(FixedWindowState {
                    count: 0,
                    window_start: 0,
                });
                let remaining = FixedWindow::get_remaining(&state, window_ms, max_requests, now_ms);
                Ok(RateLimitResult {
                    allowed: true,
                    remaining,
                    reset_at_ms: state.window_start + window_ms,
                })
            },
        }
    }

    async fn reset(&self, key: &RateLimitKey) -> Result<()> {
        let mut inner = self.inner.write().await;
        let k = key.key();
        inner.token_bucket.remove(&k);
        inner.sliding_window.remove(&k);
        inner.fixed_window.remove(&k);
        Ok(())
    }
}
