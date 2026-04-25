//! Queue Producer - 发布订单命令
//!
//! 发送匹配引擎兼容格式的订单命令（与 matching-engine/src/server.rs 的 parse_and_validate_order 对齐）

use queue::{ProducerManager, ProducerError, Message};
use crate::models::Order;
use rust_decimal::prelude::ToPrimitive;
use tracing::info;

/// 编码 symbol: market_id * OUTCOME_MULTIPLIER + outcome_id

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
        // order_id: String → u64, 提取数字部分
        let order_id = extract_numeric_id(&order.id).unwrap_or(0);

        // uid: i64 → u64
        let uid = order.user_id.max(0) as u64;

        // symbol: 编码 market_id + outcome_id
        let symbol = order.market_id as i32 * (utils::constants::OUTCOME_MULTIPLIER as i32) + order.outcome_id as i32;

        // price: Decimal → i64 (缩放)
        let price = (order.price * rust_decimal::Decimal::from(utils::constants::PRICE_SCALE))
            .to_u64()
            .unwrap_or(0) as i64;

        // size: Decimal → i64 (预测市场 base_scale_k = 1)
        let size = order.quantity.to_u64().unwrap_or(0) as i64;

        // side: buy → bid, sell → ask
        let action = match order.side {
            crate::models::OrderSide::Buy => "bid",
            crate::models::OrderSide::Sell => "ask",
        };

        // order_type: limit → gtc, market → ioc, others
        let order_type = match order.order_type {
            crate::models::OrderType::Limit => "gtc",
            crate::models::OrderType::Market => "ioc",
            crate::models::OrderType::IOC => "ioc",
            crate::models::OrderType::FOK => "fok",
            crate::models::OrderType::PostOnly => "post_only",
        };

        #[derive(serde::Serialize)]
        struct PlaceOrderCmd {
            command: String,
            order_id: u64,
            uid: u64,
            symbol: i32,
            price: i64,
            size: i64,
            action: String,
            order_type: String,
        }

        let cmd = PlaceOrderCmd {
            command: "place".to_string(),
            order_id,
            uid,
            symbol,
            price,
            size,
            action: action.to_string(),
            order_type: order_type.to_string(),
        };

        let json = serde_json::to_string(&cmd)
            .map_err(|e| ProducerError::Serialization(e))?;

        let msg = Message {
            key: Some(order.id.clone()),
            value: json,
        };

        self.producer.send("order.commands", msg).await?;
        info!("Sent PlaceOrder command: order_id={}, uid={}, symbol={}, price={}, size={}",
            order_id, uid, symbol, price, size);
        Ok(())
    }

    /// 发送取消订单命令
    pub async fn send_cancel_order(&self, order_id: &str, user_id: i64) -> Result<(), ProducerError> {
        let numeric_id = extract_numeric_id(order_id).unwrap_or(0);
        let uid = user_id.max(0) as u64;

        #[derive(serde::Serialize)]
        struct CancelOrderCmd {
            command: String,
            order_id: u64,
            uid: u64,
        }

        let cmd = CancelOrderCmd {
            command: "cancel".to_string(),
            order_id: numeric_id,
            uid,
        };

        let json = serde_json::to_string(&cmd)
            .map_err(|e| ProducerError::Serialization(e))?;

        let msg = Message {
            key: Some(order_id.to_string()),
            value: json,
        };

        self.producer.send("order.commands", msg).await?;
        info!("Sent CancelOrder command: order_id={}, uid={}", numeric_id, uid);
        Ok(())
    }
}

/// 从订单ID字符串中提取数字部分
/// 例如 "ord_abc123" → 取最后一段数字 → 123
fn extract_numeric_id(id: &str) -> Option<u64> {
    // 尝试直接解析
    if let Ok(n) = id.parse::<u64>() {
        return Some(n);
    }
    // 提取末尾或下划线后的数字
    let digits: String = id.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        // 最后兜底：用哈希
        Some(id.as_bytes().iter().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(*b as u64)))
    } else {
        // 取最后最多10位数字
        let trimmed = digits.chars().rev().take(10).collect::<String>().chars().rev().collect::<String>();
        trimmed.parse::<u64>().ok()
    }
}
