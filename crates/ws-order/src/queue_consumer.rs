//! Queue Consumer for Order Events

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

use queue::{ConsumerManager, MergedConfig, ConsumeMessage};
use crate::session::SessionManager;

/// 订单事件处理器
pub struct OrderEventHandler {
    session_manager: Arc<SessionManager>,
    consumer: Arc<RwLock<Option<ConsumerManager>>>,
    topics: Vec<String>,
}

impl OrderEventHandler {
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
                tracing::warn!("Failed to process order event: {}", e);
            }
        }
    }

    /// 处理订单事件并推送到对应用户
    async fn process_message(&self, msg: &ConsumeMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json: serde_json::Value = serde_json::from_str(&msg.value)?;

        // 从消息中提取 user_id
        let user_id = json.get("user_id").and_then(|v| v.as_i64());
        let user_id = match user_id {
            Some(id) => id,
            None => {
                tracing::warn!("Order event missing user_id, skipping");
                return Ok(());
            }
        };

        // 构造发送消息（直接在原始消息基础上确保有 type 字段）
        let payload = if json.get("type").is_none() {
            let mut with_type = json.clone();
            if with_type.get("type").is_none() {
                with_type.as_object_mut()
                    .map(|obj| { obj.insert("type".to_string(), serde_json::Value::String("order_update".to_string())); });
            }
            serde_json::to_string(&with_type)?
        } else {
            msg.value.clone()
        };

        // 推送给该用户的所有 WebSocket 会话
        self.session_manager.send_to_user(user_id, &payload).await;
        tracing::debug!("Pushed order event to user {}: order_id={:?}", user_id, json.pointer("/data/order_id"));

        Ok(())
    }
}
