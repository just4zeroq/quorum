//! 订单相关类型

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    /// 买单
    Buy,
    /// 卖单
    Sell,
}

impl Side {
    pub fn is_buy(&self) -> bool {
        matches!(self, Side::Buy)
    }

    pub fn is_sell(&self) -> bool {
        matches!(self, Side::Sell)
    }
}

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// 限价单
    Limit,
    /// 市价单
    Market,
    /// 立即成交或取消
    IOC,
    /// 全部成交或取消
    FOK,
    /// 只做 Maker
    PostOnly,
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// 待提交
    Pending,
    /// 已提交（待撮合）
    Submitted,
    /// 部分成交
    PartiallyFilled,
    /// 完全成交
    Filled,
    /// 已取消
    Cancelled,
    /// 已拒绝
    Rejected,
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// 订单ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 交易对
    pub symbol: String,
    /// 订单方向
    pub side: Side,
    /// 订单类型
    pub order_type: OrderType,
    /// 价格
    pub price: Decimal,
    /// 数量
    pub quantity: Decimal,
    /// 已成交数量
    pub filled_quantity: Decimal,
    /// 订单状态
    pub status: OrderStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(
        user_id: String,
        symbol: String,
        side: Side,
        order_type: OrderType,
        price: Decimal,
        quantity: Decimal,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            symbol,
            side,
            order_type,
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected
        )
    }
}
