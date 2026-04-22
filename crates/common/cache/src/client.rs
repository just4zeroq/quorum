use std::sync::Arc;
use redis::{aio::ConnectionManager, Client, RedisError, AsyncCommands, aio};
use tokio::sync::RwLock;
use thiserror::Error;

use crate::config::MergedConfig;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CacheError>;

/// Redis 客户端包装
#[derive(Clone)]
pub struct RedisClient {
    conn_manager: aio::ConnectionManager,
}

impl RedisClient {
    /// 创建 Redis 客户端
    pub async fn new(config: &MergedConfig) -> Result<Self> {
        let client = Client::open(config.url().as_str())
            .map_err(|e| CacheError::Config(e.to_string()))?;

        let conn_manager = ConnectionManager::new(client)
            .await
            .map_err(|e| CacheError::Config(e.to_string()))?;

        Ok(Self { conn_manager })
    }

    /// 获取字符串值
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.get(key).await?)
    }

    /// 设置字符串值
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.conn_manager.clone();
        let _: () = conn.set(key, value).await?;
        Ok(())
    }

    /// 设置字符串值 (带过期时间)
    pub async fn set_ex(&self, key: &str, value: &str, seconds: u64) -> Result<()> {
        let mut conn = self.conn_manager.clone();
        let _: () = conn.set_ex(key, value, seconds).await?;
        Ok(())
    }

    /// 设置值 (带过期时间，毫秒)
    pub async fn set_px(&self, key: &str, value: &str, milliseconds: i64) -> Result<()> {
        let mut conn = self.conn_manager.clone();
        let _: () = conn.set(key, value).await?;
        let _: () = conn.pexpire(key, milliseconds).await?;
        Ok(())
    }

    /// 删除键
    pub async fn del(&self, keys: &[&str]) -> Result<u64> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.del(keys).await?)
    }

    /// 检查键是否存在
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.exists(key).await?)
    }

    /// 设置过期时间 (秒)
    pub async fn expire(&self, key: &str, seconds: i64) -> Result<bool> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.expire(key, seconds).await?)
    }

    /// 设置过期时间 (毫秒)
    pub async fn pexpire(&self, key: &str, milliseconds: i64) -> Result<bool> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.pexpire(key, milliseconds).await?)
    }

    /// 获取哈希值
    pub async fn hget(&self, key: &str, field: &str) -> Result<Option<String>> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.hget(key, field).await?)
    }

    /// 设置哈希值
    pub async fn hset(&self, key: &str, field: &str, value: &str) -> Result<()> {
        let mut conn = self.conn_manager.clone();
        let _: () = conn.hset(key, field, value).await?;
        Ok(())
    }

    /// 获取所有哈希值
    pub async fn hgetall(&self, key: &str) -> Result<std::collections::HashMap<String, String>> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.hgetall(key).await?)
    }

    /// 自增
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.incr(key, 1).await?)
    }

    /// 自增指定值
    pub async fn incr_by(&self, key: &str, increment: i64) -> Result<i64> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.incr(key, increment).await?)
    }

    /// 自增并设置过期时间 (用于限流)
    pub async fn incr_with_expire(&self, key: &str, seconds: u64) -> Result<i64> {
        let mut conn = self.conn_manager.clone();

        // 使用 Lua 脚本原子操作
        let script = redis::Script::new(
            r#"
            local current = redis.call('INCR', KEYS[1])
            if current == 1 then
                redis.call('EXPIRE', KEYS[1], ARGV[1])
            end
            return current
            "#
        );

        let result: i64 = script
            .key(key)
            .arg(seconds)
            .invoke_async(&mut conn)
            .await?;

        Ok(result)
    }

    /// 获取 List 长度
    pub async fn llen(&self, key: &str) -> Result<u64> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.llen(key).await?)
    }

    /// 获取 List 元素
    pub async fn lrange(&self, key: &str, start: isize, stop: isize) -> Result<Vec<String>> {
        let mut conn = self.conn_manager.clone();
        Ok(conn.lrange(key, start, stop).await?)
    }

    /// 分布式锁 - 获取锁
    pub async fn lock(&self, key: &str, value: &str, seconds: u64) -> Result<bool> {
        let mut conn = self.conn_manager.clone();

        let script = redis::Script::new(
            r#"
            if redis.call('SET', KEYS[1], ARGV[1], 'NX', 'EX', ARGV[2]) then
                return 1
            else
                return 0
            end
            "#
        );

        let result: i64 = script
            .key(key)
            .arg(value)
            .arg(seconds)
            .invoke_async(&mut conn)
            .await?;

        Ok(result == 1)
    }

    /// 分布式锁 - 释放锁
    pub async fn unlock(&self, key: &str, value: &str) -> Result<bool> {
        let mut conn = self.conn_manager.clone();

        let script = redis::Script::new(
            r#"
            if redis.call('GET', KEYS[1]) == ARGV[1] then
                return redis.call('DEL', KEYS[1])
            else
                return 0
            end
            "#
        );

        let result: i64 = script
            .key(key)
            .arg(value)
            .invoke_async(&mut conn)
            .await?;

        Ok(result == 1)
    }

    /// 获取连接池引用
    pub fn connection_manager(&self) -> &ConnectionManager {
        &self.conn_manager
    }
}

/// Cache 管理器
pub struct CacheManager {
    client: Arc<RwLock<Option<RedisClient>>>,
    config: MergedConfig,
}

impl CacheManager {
    pub fn new(config: MergedConfig) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// 初始化客户端
    pub async fn init(&self) -> Result<()> {
        let client = RedisClient::new(&self.config).await?;
        let mut guard = self.client.write().await;
        *guard = Some(client);
        Ok(())
    }

    /// 获取客户端
    pub async fn get_client(&self) -> Option<RedisClient> {
        let guard = self.client.read().await;
        guard.clone()
    }

    /// 关闭客户端
    pub async fn close(&self) {
        let mut guard = self.client.write().await;
        if let Some(_client) = guard.take() {
            // ConnectionManager 在 drop 时会自动关闭连接
        }
    }
}