//! Queue Producer - 发布订单命令

use queue::{MessageProducer, ProducerManager, ProducerError, Message};
use crate::models::Order;
use tracing::info;

/// 订单命令生产者
pub struct OrderCommandProducer {
    producer: ProducerManager,
}

impl OrderCommandProducer {
    pub fn new(producer: ProducerManager) -> Self {
        Self { producer }
    }

    /// 发送下单命令
    pub async fn send_place_order(&self, order: &Order) -> Result<(), ProducerError> {
        #[derive(serde::Serialize)]
        struct PlaceOrderCmd {
            command: String,
            order_id: String,
            user_id: i64,
            market_id: i64,
            outcome_id: i64,
            side: String,
            price: String,
            quantity: String,
            order_type: String,
        }

        let cmd = PlaceOrderCmd {
            command: "PlaceOrder".to_string(),
            order_id: order.id.clone(),
            user_id: order.user_id,
            market_id: order.market_id,
            outcome_id: order.outcome_id,
            side: order.side.to_string(),
            price: order.price.to_string(),
            quantity: order.quantity.to_string(),
            order_type: order.order_type.to_string(),
        };

        let json = serde_json::to_string(&cmd)
            .map_err(|e| ProducerError::Serialization(e))?;

        let msg = Message {
            key: Some(order.id.clone()),
            value: json,
        };

        self.producer.send("order.commands", msg).await?;
        info!("Sent PlaceOrder command for order {}", order.id);
        Ok(())
    }

    /// 发送取消订单命令
    pub async fn send_cancel_order(&self, order_id: &str, user_id: i64) -> Result<(), ProducerError> {
        #[derive(serde::Serialize)]
        struct CancelOrderCmd {
            command: String,
            order_id: String,
            user_id: i64,
        }

        let cmd = CancelOrderCmd {
            command: "CancelOrder".to_string(),
            order_id: order_id.to_string(),
            user_id,
        };

        let json = serde_json::to_string(&cmd)
            .map_err(|e| ProducerError::Serialization(e))?;

        let msg = Message {
            key: Some(order_id.to_string()),
            value: json,
        };

        self.producer.send("order.commands", msg).await?;
        info!("Sent CancelOrder command for order {}", order_id);
        Ok(())
    }
}
