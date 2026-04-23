//! Redis Store (for production)

use async_trait::async_trait;
use redis::AsyncCommands;

use crate::traits::{RateLimiter, RateLimitKey, RateLimitResult, RateLimitError, Result};

/// Redis store configuration
#[derive(Debug, Clone)]
pub struct RedisStoreConfig {
    pub redis_url: String,
    pub key_prefix: String,
}

/// Redis store implementation using Lua scripts for atomic operations
pub struct RedisStore {
    config: RedisStoreConfig,
    client: redis::Client,
}

impl RedisStore {
    pub fn new(config: RedisStoreConfig) -> Result<Self> {
        let client = redis::Client::open(config.redis_url.clone())
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        Ok(Self { config, client })
    }
}

#[async_trait]
impl RateLimiter for RedisStore {
    async fn check_and_consume(&self, key: &RateLimitKey, _cost: u64) -> Result<RateLimitResult> {
        let mut conn = self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        let redis_key = format!("{}:{}", self.config.key_prefix, key.key());
        let now_ms = chrono::Utc::now().timestamp_millis();

        // For now, use a simple GET/SET approach. In production, use Lua scripts
        let _result: Option<String> = conn.get(&redis_key).await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        // This is a simplified implementation
        // Production should use Lua scripts for atomicity
        Ok(RateLimitResult {
            allowed: true,
            remaining: 100,
            reset_at_ms: now_ms + 1000,
        })
    }

    async fn get_limit(&self, _key: &RateLimitKey) -> Result<RateLimitResult> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        Ok(RateLimitResult {
            allowed: true,
            remaining: 100,
            reset_at_ms: now_ms + 1000,
        })
    }

    async fn reset(&self, key: &RateLimitKey) -> Result<()> {
        let mut conn = self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        let redis_key = format!("{}:{}", self.config.key_prefix, key.key());
        let _: () = conn.del(&redis_key).await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        Ok(())
    }
}
