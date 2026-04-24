//! Queue Consumer for Prediction Market Events

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

use queue::{ConsumerManager, MergedConfig, ConsumeMessage};
use crate::session::{Channel, SessionManager};

/// 市场事件处理器
pub struct MarketEventHandler {
    session_manager: Arc<SessionManager>,
    consumer: Arc<RwLock<Option<ConsumerManager>>>,
    topics: Vec<String>,
}

impl MarketEventHandler {
    pub fn new(session_manager: Arc<SessionManager>, topics: Vec<String>) -> Self {
        Self {
            session_manager,
            consumer: Arc::new(RwLock::new(None)),
            topics,
        }
    }

    pub async fn init(&self, config: MergedConfig) -> Result<(), queue::ConsumerError> {
        let manager = ConsumerManager::new(config, self.topics.clone());
        manager.init().await?;
        let mut guard = self.consumer.write().await;
        *guard = Some(manager);
        Ok(())
    }

    pub async fn start(&self) {
        loop {
            let msg = {
                let guard = self.consumer.read().await;
                if let Some(ref consumer) = *guard {
                    match consumer.recv().await {
                        Ok(Some(msg)) => msg,
                        Ok(None) => {
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
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            };

            if let Err(e) = self.process_message(&msg).await {
                tracing::warn!("Failed to process market event: {}", e);
            }
        }
    }

    /// 处理市场事件并广播
    async fn process_message(&self, msg: &ConsumeMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json: serde_json::Value = serde_json::from_str(&msg.value)?;

        // 根据事件类型决定 channel 和 market_id
        let event_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let market_id = json.get("market_id").and_then(|v| v.as_i64()).unwrap_or(0);

        let channel = match event_type {
            "market_status" | "market_resolved" | "market_closed" => Channel::MarketStatus,
            "settlement" | "market_settled" => Channel::Settlement,
            _ => {
                tracing::debug!("Unknown market event type: {}", event_type);
                return Ok(());
            }
        };

        let message_str = serde_json::to_string(&json)?;

        // 广播到订阅该 market 的会话
        if market_id > 0 {
            self.session_manager.broadcast_to_market(&channel, market_id, &message_str).await;
        } else {
            // 对没有market_id的消息，广播到所有订阅该频道的会话
            let sessions = self.session_manager.get_all().await;
            for session in sessions {
                let session = session.read().await;
                let is_subscribed = session.subscriptions.contains_key(&channel);
                if is_subscribed {
                    if let Err(e) = session.send(&message_str).await {
                        tracing::warn!("Failed to send message: {}", e);
                    }
                }
            }
        }

        tracing::debug!("Broadcast {} event for market {}", event_type, market_id);
        Ok(())
    }
}
