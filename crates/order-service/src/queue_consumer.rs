//! Queue Consumer - 消费撮合事件

use queue::{MessageConsumer, ConsumerManager, ProducerManager, Message};
use crate::repository::OrderRepository;
use crate::models::OrderStatus;
use tracing::{info, error};

/// 撮合事件消费者
pub struct MatchEventConsumer {
    consumer: ConsumerManager,
    order_repo: OrderRepository,
    event_producer: Option<ProducerManager>,
}

impl MatchEventConsumer {
    pub fn new(consumer: ConsumerManager, order_repo: OrderRepository) -> Self {
        Self {
            consumer,
            order_repo,
            event_producer: None,
        }
    }

    /// 设置事件生产者
    pub fn with_event_producer(mut self, producer: ProducerManager) -> Self {
        self.event_producer = Some(producer);
        self
    }

    /// 启动消费
    pub async fn start(&self) {
        info!("MatchEventConsumer starting...");

        loop {
            match self.consumer.recv().await {
                Ok(Some(msg)) => {
                    if let Err(e) = self.process_message(&msg.value).await {
                        error!("Failed to process message: {}", e);
                    }
                }
                Ok(None) => {
                    // No message available, continue polling
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
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
                let filled_quantity = event.size.to_string();
                let filled_amount = (event.size * event.price).to_string();
                self.order_repo.update_status(
                    &event.matched_order_id.to_string(),
                    &OrderStatus::Filled.to_string(),
                    &filled_quantity,
                    &filled_amount,
                ).await.map_err(|e| e.to_string())?;

                // 发布 order_events
                self.publish_order_event(
                    &event.matched_order_id.to_string(),
                    event.matched_order_uid as i64,
                    "filled",
                    &filled_quantity,
                    &filled_amount,
                    &event.price.to_string(),
                ).await;
            }
            "Reject" => {
                info!("Processing reject event for order {}", event.matched_order_id);
                self.order_repo.update_status(
                    &event.matched_order_id.to_string(),
                    &OrderStatus::Rejected.to_string(),
                    &"0".to_string(),
                    &"0".to_string(),
                ).await.map_err(|e| e.to_string())?;

                // 发布 order_events
                self.publish_order_event(
                    &event.matched_order_id.to_string(),
                    event.matched_order_uid as i64,
                    "rejected",
                    "0",
                    "0",
                    &event.price.to_string(),
                ).await;
            }
            "Reduce" => {
                info!("Processing reduce event for order {}", event.matched_order_id);
                let filled_quantity = event.size.to_string();
                let filled_amount = (event.size * event.price).to_string();
                self.order_repo.update_status(
                    &event.matched_order_id.to_string(),
                    &OrderStatus::PartiallyFilled.to_string(),
                    &filled_quantity,
                    &filled_amount,
                ).await.map_err(|e| e.to_string())?;

                // 发布 order_events
                self.publish_order_event(
                    &event.matched_order_id.to_string(),
                    event.matched_order_uid as i64,
                    "partially_filled",
                    &filled_quantity,
                    &filled_amount,
                    &event.price.to_string(),
                ).await;
            }
            _ => {
                info!("Unknown event type: {}", event.event_type);
            }
        }

        Ok(())
    }

    /// 发布订单事件到 order_events 主题
    async fn publish_order_event(
        &self,
        order_id: &str,
        user_id: i64,
        status: &str,
        filled_quantity: &str,
        filled_amount: &str,
        price: &str,
    ) {
        if let Some(ref producer) = self.event_producer {
            let now = chrono::Utc::now().timestamp_millis();
            let payload = serde_json::json!({
                "type": "order_update",
                "user_id": user_id,
                "data": {
                    "order_id": order_id,
                    "status": status,
                    "filled_quantity": filled_quantity,
                    "filled_amount": filled_amount,
                    "price": price,
                    "updated_at": now
                }
            });

            let json_str = serde_json::to_string(&payload).unwrap_or_default();
            let msg = Message {
                key: Some(order_id.to_string()),
                value: json_str,
            };

            if let Err(e) = producer.send("order_events", msg).await {
                error!("Failed to publish order event: {}", e);
            } else {
                info!("Published order_event for order {}", order_id);
            }
        }
    }
}
