//! 消息队列消费者 - 支持 Redis 和 Kafka 双后端

use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::config::{Backend, MergedConfig};

#[derive(Error, Debug)]
pub enum ConsumerError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ConsumerError>;

/// 消费消息
#[derive(Debug, Clone)]
pub struct ConsumeMessage {
    pub key: Option<String>,
    pub value: String,
    pub topic: String,
}

/// 消息消费者 trait
#[async_trait::async_trait]
pub trait MessageConsumer: Send + Sync {
    /// 消费单条消息
    async fn consume(&self, topic: &str) -> Result<ConsumeMessage>;
}

/// Redis 消费者 (使用 Redis Streams)
pub struct RedisConsumer {
    client: redis::aio::ConnectionManager,
}

impl RedisConsumer {
    pub async fn new(config: &MergedConfig) -> Result<Self> {
        let client = redis::Client::open(config.redis_url().as_str())
            .map_err(|e| ConsumerError::Config(e.to_string()))?
            .get_connection_manager()
            .await
            .map_err(|e| ConsumerError::Redis(e))?;

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl MessageConsumer for RedisConsumer {
    async fn consume(&self, topic: &str) -> Result<ConsumeMessage> {
        let mut conn = self.client.clone();

        // 使用 XREAD 消费消息
        let result: Vec<(String, Vec<(String, Vec<(String, String)>)>)> = redis::cmd("XREAD")
            .arg("COUNT")
            .arg(1)
            .arg("BLOCK")
            .arg(5000)
            .arg("STREAMS")
            .arg(topic)
            .arg("0")
            .query_async(&mut conn)
            .await
            .map_err(|e| ConsumerError::Redis(e))?;

        if result.is_empty() {
            return Err(ConsumerError::Config("No message".to_string()));
        }

        let (_stream, messages) = &result[0];
        if messages.is_empty() {
            return Err(ConsumerError::Config("No message".to_string()));
        }

        let (_msg_id, fields) = &messages[0];
        let mut key = None;
        let mut value = String::new();

        for (field, val) in fields {
            if field == "key" {
                key = Some(val.clone());
            } else if field == "value" {
                value = val.clone();
            }
        }

        Ok(ConsumeMessage {
            key,
            value,
            topic: topic.to_string(),
        })
    }
}

/// Kafka 消费者 (简化版)
pub struct KafkaConsumer {
    _config: MergedConfig,
}

impl KafkaConsumer {
    pub async fn new(config: &MergedConfig) -> Result<Self> {
        // TODO: 实现完整的 Kafka 消费者
        Ok(Self { _config: config.clone() })
    }
}

#[async_trait::async_trait]
impl MessageConsumer for KafkaConsumer {
    async fn consume(&self, topic: &str) -> Result<ConsumeMessage> {
        // TODO: 实现 Kafka 消费
        tracing::debug!("Kafka consume from {}", topic);
        Err(ConsumerError::Config("Not implemented".to_string()))
    }
}

/// 统一的消费者管理器
pub struct ConsumerManager {
    consumer: Arc<RwLock<Option<Box<dyn MessageConsumer>>>>,
    config: MergedConfig,
}

impl ConsumerManager {
    pub fn new(config: MergedConfig) -> Self {
        Self {
            consumer: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// 初始化消费者 (根据配置自动选择后端)
    pub async fn init(&self) -> Result<()> {
        let consumer: Box<dyn MessageConsumer> = match self.config.backend {
            Backend::Redis => {
                Box::new(RedisConsumer::new(&self.config).await?)
            }
            Backend::Kafka => {
                Box::new(KafkaConsumer::new(&self.config).await?)
            }
        };

        let mut guard = self.consumer.write().await;
        *guard = Some(consumer);
        Ok(())
    }

    /// 消费消息
    pub async fn consume(&self, topic: &str) -> Result<ConsumeMessage> {
        let guard = self.consumer.read().await;
        if let Some(ref consumer) = *guard {
            consumer.consume(topic).await
        } else {
            Err(ConsumerError::Config("Consumer not initialized".to_string()))
        }
    }

    /// 关闭消费者
    pub async fn close(&self) {
        let mut guard = self.consumer.write().await;
        guard.take();
    }
}