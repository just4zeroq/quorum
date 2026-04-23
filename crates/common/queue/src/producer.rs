//! 消息队列生产者 - 支持 Redis 和 Kafka 双后端

use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::config::{Backend, MergedConfig};

#[derive(Error, Debug)]
pub enum ProducerError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Kafka error: {0}")]
    Kafka(#[from] crate::kafka_producer::KafkaProducerError),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Unsupported backend: {0}")]
    UnsupportedBackend(String),
}

pub type Result<T> = std::result::Result<T, ProducerError>;

/// 消息
#[derive(Debug, Clone)]
pub struct Message {
    pub key: Option<String>,
    pub value: String,
}

/// 消息生产者 trait
#[async_trait::async_trait]
pub trait MessageProducer: Send + Sync {
    /// 发送消息
    async fn send(&self, topic: &str, message: Message) -> Result<()>;
}

/// Redis 生产者 (使用 Redis Streams)
pub struct RedisProducer {
    client: redis::aio::ConnectionManager,
}

impl RedisProducer {
    pub async fn new(config: &MergedConfig) -> Result<Self> {
        let client = redis::Client::open(config.redis_url().as_str())
            .map_err(|e| ProducerError::Config(e.to_string()))?
            .get_connection_manager()
            .await
            .map_err(|e| ProducerError::Redis(e))?;

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl MessageProducer for RedisProducer {
    async fn send(&self, topic: &str, message: Message) -> Result<()> {
        let mut conn = self.client.clone();
        let _: () = redis::cmd("XADD")
            .arg(topic)
            .arg("*")
            .arg("key")
            .arg(message.key.unwrap_or_default())
            .arg("value")
            .arg(message.value)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

/// Kafka 生产者
pub struct KafkaProducer {
    producer: crate::kafka_producer::KafkaQueueProducer,
}

impl KafkaProducer {
    pub async fn new(config: &MergedConfig) -> Result<Self> {
        let producer = crate::kafka_producer::KafkaQueueProducer::new(config).await?;
        Ok(Self { producer })
    }
}

#[async_trait::async_trait]
impl MessageProducer for KafkaProducer {
    async fn send(&self, topic: &str, message: Message) -> Result<()> {
        self.producer.send(topic, message).await?;
        Ok(())
    }
}

/// 统一的生产者管理器
pub struct ProducerManager {
    producer: Arc<RwLock<Option<Box<dyn MessageProducer>>>>,
    config: MergedConfig,
}

impl ProducerManager {
    pub fn new(config: MergedConfig) -> Self {
        Self {
            producer: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// 初始化生产者 (根据配置自动选择后端)
    pub async fn init(&self) -> Result<()> {
        let producer: Box<dyn MessageProducer> = match self.config.backend {
            Backend::Redis => {
                Box::new(RedisProducer::new(&self.config).await?)
            }
            Backend::Kafka => {
                Box::new(KafkaProducer::new(&self.config).await?)
            }
        };

        let mut guard = self.producer.write().await;
        *guard = Some(producer);
        Ok(())
    }

    /// 发送消息
    pub async fn send(&self, topic: &str, message: Message) -> Result<()> {
        let guard = self.producer.read().await;
        if let Some(ref producer) = *guard {
            producer.send(topic, message).await
        } else {
            Err(ProducerError::Config("Producer not initialized".to_string()))
        }
    }

    /// 发送字符串消息
    pub async fn send_string(&self, topic: &str, key: Option<&str>, value: &str) -> Result<()> {
        self.send(topic, Message {
            key: key.map(String::from),
            value: value.to_string(),
        }).await
    }

    /// 发送 JSON 消息
    pub async fn send_json<T: serde::Serialize>(&self, topic: &str, key: Option<&str>, value: &T) -> Result<()> {
        let json = serde_json::to_string(value)?;
        self.send_string(topic, key, &json).await
    }

    /// 关闭生产者
    pub async fn close(&self) {
        let mut guard = self.producer.write().await;
        guard.take();
    }
}
