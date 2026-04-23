//! Kafka Consumer - 消费撮合事件

use common::queue::{MessageConsumer, ConsumerManager};
use crate::repository::OrderRepository;
use crate::models::OrderStatus;
use tracing::{info, error};

/// 撮合事件消费者
pub struct MatchEventConsumer {
    consumer: ConsumerManager,
    order_repo: OrderRepository,
}

impl MatchEventConsumer {
    pub fn new(consumer: ConsumerManager, order_repo: OrderRepository) -> Self {
        Self {
            consumer,
            order_repo,
        }
    }

    /// 启动消费
    pub async fn start(&self) {
        info!("MatchEventConsumer starting...");

        loop {
            match self.consumer.recv().await {
                Ok(msg) => {
                    if let Err(e) = self.process_message(&msg.value).await {
                        error!("Failed to process message: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to receive message: {}", e);
                }
            }
        }
    }

    /// 处理消息
    async fn process_message(&self, data: &str) -> Result<(), String> {
        #[derive(serde::Deserialize)]
        struct TradeEvent {
            event_type: String,
            size: i64,
            price: i64,
            matched_order_id: u64,
            matched_order_uid: u64,
        }

        let event: TradeEvent = serde_json::from_str(data)
            .map_err(|e| format!("Failed to parse event: {}", e))?;

        match event.event_type.as_str() {
            "Trade" => {
                info!("Processing trade event for order {}", event.matched_order_id);
                // 更新订单成交信息
                // 注意：这里需要从事件中获取 filled_quantity 和 filled_amount
                // 简化处理：直接标记为完全成交
                self.order_repo.update_status(
                    &event.matched_order_id.to_string(),
                    &OrderStatus::Filled.to_string(),
                    &event.size.to_string(),
                    &(event.size * event.price).to_string(),
                ).await.map_err(|e| e.to_string())?;
            }
            "Reject" => {
                info!("Processing reject event for order {}", event.matched_order_id);
                self.order_repo.update_status(
                    &event.matched_order_id.to_string(),
                    &OrderStatus::Rejected.to_string(),
                    &"0".to_string(),
                    &"0".to_string(),
                ).await.map_err(|e| e.to_string())?;
            }
            "Reduce" => {
                info!("Processing reduce event for order {}", event.matched_order_id);
                self.order_repo.update_status(
                    &event.matched_order_id.to_string(),
                    &OrderStatus::PartiallyFilled.to_string(),
                    &event.size.to_string(),
                    &(event.size * event.price).to_string(),
                ).await.map_err(|e| e.to_string())?;
            }
            _ => {
                info!("Unknown event type: {}", event.event_type);
            }
        }

        Ok(())
    }
}
