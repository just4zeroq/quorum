//! Order Service Data Models

pub mod order;
pub mod trade;

pub use order::model::{Order, OrderStatus, OrderType, OrderSide, OrderQuery, OrderEventRecord};
pub use order::event::OrderEvent;
pub use trade::model::{Trade, TradeSide};

/// 创建订单请求
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: i64,
    pub market_id: i64,
    pub outcome_id: i64,
    pub side: String,
    pub order_type: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub client_order_id: Option<String>,
}
