//! Kafka Producer 实现 (Stub)

use thiserror::Error;
use crate::config::MergedConfig;
use crate::producer::Message;

#[derive(Error, Debug)]
pub enum KafkaProducerError {
    #[error("Kafka not implemented: {0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, KafkaProducerError>;

/// Kafka 生产者 (占位实现)
pub struct KafkaQueueProducer {
    _config: MergedConfig,
}

impl KafkaQueueProducer {
    /// 创建 Kafka 生产者
    pub async fn new(config: &MergedConfig) -> Result<Self> {
        Ok(Self { _config: config.clone() })
    }

    /// 发送消息 (占位)
    pub async fn send(&self, topic: &str, message: Message) -> Result<()> {
        tracing::debug!("[KAFKA STUB] Would send to {}: key={:?}", topic, message.key);
        Ok(())
    }

    /// 关闭生产者
    pub async fn close(&self) {}
}
