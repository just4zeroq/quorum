//! Rate Limiter Trait

use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum RateLimitError {
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("Limit exceeded")]
    LimitExceeded,
}

pub type Result<T> = std::result::Result<T, RateLimitError>;

/// Rate limiter key type
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct RateLimitKey {
    pub user_id: Option<String>,
    pub ip: Option<String>,
    pub endpoint: Option<String>,
}

impl RateLimitKey {
    pub fn new() -> Self {
        Self {
            user_id: None,
            ip: None,
            endpoint: None,
        }
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = Some(ip.into());
        self
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn key(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref uid) = self.user_id {
            parts.push(format!("u:{}", uid));
        }
        if let Some(ref ip) = self.ip {
            parts.push(format!("ip:{}", ip));
        }
        if let Some(ref ep) = self.endpoint {
            parts.push(format!("ep:{}", ep));
        }
        parts.join(":")
    }
}

impl Default for RateLimitKey {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limit result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u64,
    pub reset_at_ms: i64,
}

/// Rate limiter trait
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if request is allowed and consume a token
    async fn check_and_consume(&self, key: &RateLimitKey, cost: u64) -> Result<RateLimitResult>;

    /// Get current limit status without consuming
    async fn get_limit(&self, key: &RateLimitKey) -> Result<RateLimitResult>;

    /// Reset limit for a key
    async fn reset(&self, key: &RateLimitKey) -> Result<()>;
}
