//! Kafka Consumer 实现 (Stub)

use thiserror::Error;
use crate::config::MergedConfig;
use crate::consumer::ConsumeMessage;

#[derive(Error, Debug)]
pub enum KafkaConsumerError {
    #[error("Kafka not implemented: {0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, KafkaConsumerError>;

/// Kafka 消费者 (占位实现)
pub struct KafkaQueueConsumer {
    _config: MergedConfig,
}

impl KafkaQueueConsumer {
    /// 创建 Kafka 消费者
    pub async fn new(config: &MergedConfig) -> Result<Self> {
        Ok(Self { _config: config.clone() })
    }

    /// 订阅主题
    pub async fn subscribe(&self, _topics: &[&str]) -> Result<()> {
        Ok(())
    }

    /// 消费消息 (占位)
    pub async fn consume(&self) -> Result<Option<ConsumeMessage>> {
        // 睡眠一段时间，模拟没有消息
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        Ok(None)
    }

    /// 消费消息（带超时）
    pub async fn recv(&self) -> Result<Option<ConsumeMessage>> {
        self.consume().await
    }
}
