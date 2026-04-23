//! Consumer Manager with callback support for matching-engine

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::config::MergedConfig;
use crate::consumer::{ConsumerError, ConsumeMessage, MessageConsumer};

/// 消息处理器回调
pub type MessageHandler = Arc<dyn Fn(ConsumeMessage) -> Result<(), String> + Send + Sync>;

/// 最大重试次数
const MAX_HANDLER_RETRIES: u32 = 3;

/// 消费者管理器，支持回调处理和死信处理
pub struct ConsumerManagerWithHandler {
    consumer: Arc<RwLock<Option<Box<dyn MessageConsumer + Send + Sync>>>>,
    config: MergedConfig,
    topics: Vec<String>,
    dead_letter_topic: Option<String>,
}

impl ConsumerManagerWithHandler {
    pub fn new(config: MergedConfig, topics: Vec<String>) -> Self {
        Self {
            consumer: Arc::new(RwLock::new(None)),
            config,
            topics,
            dead_letter_topic: None,
        }
    }

    /// 设置死信主题
    pub fn with_dead_letter_topic(mut self, topic: String) -> Self {
        self.dead_letter_topic = Some(topic);
        self
    }

    /// 初始化消费者
    pub async fn init(&self) -> Result<(), ConsumerError> {
        use crate::config::Backend;

        let consumer: Box<dyn MessageConsumer + Send + Sync> = match self.config.backend {
            Backend::Redis => {
                let redis_consumer = crate::consumer::RedisConsumer::new(&self.config, &self.topics[0]).await?;
                redis_consumer.ensure_group().await?;
                Box::new(redis_consumer)
            }
            Backend::Kafka => {
                let kafka_consumer = crate::consumer::KafkaConsumer::new(&self.config, &self.topics.iter().map(|s| s.as_str()).collect::<Vec<_>>()).await?;
                Box::new(kafka_consumer)
            }
        };

        let mut guard = self.consumer.write().await;
        *guard = Some(consumer);
        Ok(())
    }

    /// 启动消费循环，传入回调处理器
    pub async fn start(&self, handler: MessageHandler) -> Result<(), ConsumerError> {
        loop {
            let msg = {
                let guard = self.consumer.read().await;
                if let Some(ref c) = *guard {
                    match c.consume().await {
                        Ok(Some(msg)) => msg,
                        Ok(None) => {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            continue;
                        }
                        Err(ConsumerError::Timeout) => {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            continue;
                        }
                        Err(e) => {
                            tracing::error!("Consumer error: {}", e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    }
                } else {
                    return Err(ConsumerError::NotInitialized);
                }
            };

            // 执行回调处理，带重试
            let result = process_with_retry(&handler, &msg, MAX_HANDLER_RETRIES);

            if let Err(e) = result {
                tracing::error!(
                    "Message failed after {} retries, sending to dead letter queue: {}",
                    MAX_HANDLER_RETRIES, e
                );
                // TODO: 发送到死信队列
                // self.send_to_dead_letter(&msg, &e).await;
            }
        }
    }
}

/// 带重试的消息处理
fn process_with_retry(
    handler: &MessageHandler,
    msg: &ConsumeMessage,
    max_retries: u32,
) -> Result<(), String> {
    let mut last_error = String::new();
    for attempt in 0..max_retries {
        match handler(msg.clone()) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_error = e.clone();
                if attempt < max_retries - 1 {
                    tracing::warn!(
                        "Handler attempt {} failed: {}, retrying...",
                        attempt + 1,
                        e
                    );
                    // 指数退避: 100ms, 200ms, 400ms...
                    std::thread::sleep(Duration::from_millis(100 * 2u64.pow(attempt)));
                }
            }
        }
    }
    Err(last_error)
}
