//! 消息队列消费者 - 支持 Redis 和 Kafka 双后端

use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use async_trait::async_trait;
use serde::Deserialize;

use crate::config::{Backend, MergedConfig};

#[derive(Error, Debug)]
pub enum ConsumerError {
    #[error("Redis error: {0}")]
    Redis(String),
    #[error("Kafka error: {0}")]
    Kafka(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Consumer not initialized")]
    NotInitialized,
    #[error("Timeout")]
    Timeout,
}

impl From<crate::kafka_consumer::KafkaConsumerError> for ConsumerError {
    fn from(e: crate::kafka_consumer::KafkaConsumerError) -> Self {
        ConsumerError::Kafka(e.to_string())
    }
}

impl From<redis::RedisError> for ConsumerError {
    fn from(e: redis::RedisError) -> Self {
        ConsumerError::Redis(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ConsumerError>;

/// 消费消息
#[derive(Debug, Clone, Deserialize)]
pub struct ConsumeMessage {
    pub key: Option<String>,
    pub value: String,
    pub topic: String,
}

/// 消息消费者 trait
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    /// 消费单条消息
    async fn consume(&self) -> Result<Option<ConsumeMessage>>;
}

/// Redis 消费者 (使用 Redis Streams)
pub struct RedisConsumer {
    client: redis::aio::ConnectionManager,
    stream: String,
    group: String,
    consumer: String,
}

impl RedisConsumer {
    pub async fn new(config: &MergedConfig, stream: &str) -> Result<Self> {
        let client = redis::Client::open(config.redis_url().as_str())
            .map_err(|e| ConsumerError::Config(e.to_string()))?
            .get_connection_manager()
            .await?;

        let group = config.kafka_group_id.clone()
            .unwrap_or_else(|| "quorum-consumer".to_string());
        let consumer = format!("consumer-{}", std::process::id());

        Ok(Self {
            client,
            stream: stream.to_string(),
            group,
            consumer,
        })
    }

    /// 确保消费者组存在
    pub async fn ensure_group(&self) -> Result<()> {
        let mut conn = self.client.clone();
        // 忽略错误，因为消费者组可能已存在
        let _: std::result::Result<String, redis::RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.stream)
            .arg(&self.group)
            .arg("0")
            .arg("MKSTREAM")
            .query_async(&mut conn)
            .await;
        Ok(())
    }
}

#[async_trait]
impl MessageConsumer for RedisConsumer {
    async fn consume(&self) -> Result<Option<ConsumeMessage>> {
        let mut conn = self.client.clone();

        // 读取消息 - 使用 XREADGROUP
        let result: Vec<String> = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(&self.group)
            .arg(&self.consumer)
            .arg("COUNT")
            .arg(1)
            .arg("STREAMS")
            .arg(&self.stream)
            .arg(">")
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        if result.is_empty() {
            return Ok(None);
        }

        // 解析结果 - 格式: [stream_name, [[id, [field, value, ...]]]]
        // 简化处理：直接返回原始 value
        if result.len() >= 2 {
            let value = result[1].clone();
            // 尝试提取 JSON 数据
            if let Ok(msg) = serde_json::from_str::<ConsumeMessage>(&value) {
                return Ok(Some(msg));
            }
            // 如果不是 JSON，直接返回 value
            return Ok(Some(ConsumeMessage {
                key: None,
                value,
                topic: self.stream.clone(),
            }));
        }

        Ok(None)
    }
}

/// Kafka 消费者
pub struct KafkaConsumer {
    consumer: crate::kafka_consumer::KafkaQueueConsumer,
}

impl KafkaConsumer {
    pub async fn new(config: &MergedConfig, topics: &[&str]) -> Result<Self> {
        let consumer = crate::kafka_consumer::KafkaQueueConsumer::new(config).await?;
        consumer.subscribe(topics).await?;
        Ok(Self { consumer })
    }
}

#[async_trait]
impl MessageConsumer for KafkaConsumer {
    async fn consume(&self) -> Result<Option<ConsumeMessage>> {
        self.consumer.recv().await.map_err(|e| e.into())
    }
}

/// 统一的消费者管理器
pub struct ConsumerManager {
    consumer: Arc<RwLock<Option<Box<dyn MessageConsumer>>>>,
    config: MergedConfig,
    topics: Vec<String>,
}

impl ConsumerManager {
    pub fn new(config: MergedConfig, topics: Vec<String>) -> Self {
        Self {
            consumer: Arc::new(RwLock::new(None)),
            config,
            topics,
        }
    }

    /// 初始化消费者 (根据配置自动选择后端)
    pub async fn init(&self) -> Result<()> {
        let consumer: Box<dyn MessageConsumer> = match self.config.backend {
            Backend::Redis => {
                let redis_consumer = RedisConsumer::new(&self.config, &self.topics[0]).await?;
                redis_consumer.ensure_group().await?;
                Box::new(redis_consumer)
            }
            Backend::Kafka => {
                let kafka_consumer = KafkaConsumer::new(&self.config, &self.topics.iter().map(|s| s.as_str()).collect::<Vec<_>>()).await?;
                Box::new(kafka_consumer)
            }
        };

        let mut guard = self.consumer.write().await;
        *guard = Some(consumer);
        Ok(())
    }

    /// 消费消息
    pub async fn recv(&self) -> Result<Option<ConsumeMessage>> {
        let guard = self.consumer.read().await;
        if let Some(ref consumer) = *guard {
            consumer.consume().await
        } else {
            Err(ConsumerError::NotInitialized)
        }
    }

    /// 阻塞等待消息
    pub async fn next(&self) -> Result<ConsumeMessage> {
        loop {
            match self.recv().await {
                Ok(Some(msg)) => return Ok(msg),
                Ok(None) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(ConsumerError::Timeout) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
