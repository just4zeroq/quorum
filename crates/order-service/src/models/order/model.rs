//! Order Model - 订单数据模型

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,        // 待提交
    Submitted,      // 已提交
    PartiallyFilled,// 部分成交
    Filled,         // 完全成交
    Cancelled,      // 已取消
    Rejected,       // 已拒绝
}

impl Default for OrderStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::Submitted => write!(f, "submitted"),
            OrderStatus::PartiallyFilled => write!(f, "partially_filled"),
            OrderStatus::Filled => write!(f, "filled"),
            OrderStatus::Cancelled => write!(f, "cancelled"),
            OrderStatus::Rejected => write!(f, "rejected"),
        }
    }
}

/// 订单方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "buy"),
            OrderSide::Sell => write!(f, "sell"),
        }
    }
}

/// 订单类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Limit,      // 限价单
    Market,     // 市价单
    IOC,        // 即时成交否则取消
    FOK,        // 全部成交否则取消
    PostOnly,   // 只挂单
}

impl Default for OrderType {
    fn default() -> Self {
        Self::Limit
    }
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Limit => write!(f, "limit"),
            OrderType::Market => write!(f, "market"),
            OrderType::IOC => write!(f, "ioc"),
            OrderType::FOK => write!(f, "fok"),
            OrderType::PostOnly => write!(f, "post_only"),
        }
    }
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: i64,
    pub market_id: i64,
    pub outcome_id: i64,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub filled_amount: Decimal,
    pub status: OrderStatus,
    pub client_order_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Order {
    pub fn new(
        user_id: i64,
        market_id: i64,
        outcome_id: i64,
        side: OrderSide,
        order_type: OrderType,
        price: Decimal,
        quantity: Decimal,
        client_order_id: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: utils::id::generate_order_id(market_id, user_id),
            user_id,
            market_id,
            outcome_id,
            side,
            order_type,
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            filled_amount: Decimal::ZERO,
            status: OrderStatus::Pending,
            client_order_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected
        )
    }

    pub fn can_cancel(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Pending | OrderStatus::Submitted | OrderStatus::PartiallyFilled
        )
    }
}

/// 订单查询条件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderQuery {
    pub user_id: Option<i64>,
    pub market_id: Option<i64>,
    pub outcome_id: Option<i64>,
    pub status: Option<OrderStatus>,
    pub side: Option<OrderSide>,
    pub page: i32,
    pub page_size: i32,
}

impl OrderQuery {
    pub fn new(page: i32, page_size: i32) -> Self {
        Self {
            page,
            page_size,
            ..Default::default()
        }
    }
}

/// 订单事件记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderEventRecord {
    pub id: i64,
    pub order_id: String,
    pub event_type: String,
    pub old_status: Option<OrderStatus>,
    pub new_status: OrderStatus,
    pub filled_quantity: Option<Decimal>,
    pub filled_amount: Option<Decimal>,
    pub price: Option<Decimal>,
    pub reason: Option<String>,
    pub created_at: i64,
}