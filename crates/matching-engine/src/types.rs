//! 撮合引擎类型定义

use common::*;
use serde::{Deserialize, Serialize};

/// 订单簿订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookOrder {
    /// 订单ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 订单方向
    pub side: Side,
    /// 价格
    pub price: Decimal,
    /// 数量
    pub quantity: Decimal,
    /// 已成交数量
    pub filled_quantity: Decimal,
    /// 订单类型
    pub order_type: OrderType,
    /// 创建时间（用于时间优先）
    pub timestamp: i64,
}

impl BookOrder {
    pub fn new(
        id: String,
        user_id: String,
        side: Side,
        price: Decimal,
        quantity: Decimal,
        order_type: OrderType,
    ) -> Self {
        Self {
            id,
            user_id,
            side,
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            order_type,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    pub fn is_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }
}

/// 成交结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// 成交记录
    pub trades: Vec<Trade>,
    /// 订单更新
    pub order_updates: Vec<OrderUpdate>,
    /// 是否完全成交
    pub fully_filled: bool,
    /// 剩余数量
    pub remaining_quantity: Decimal,
}

/// 订单更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdate {
    /// 订单ID
    pub order_id: String,
    /// 已成交数量
    pub filled_quantity: Decimal,
    /// 订单状态
    pub status: OrderStatus,
}

/// 撮合统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    /// 交易对
    pub symbol: String,
    /// 总成交笔数
    pub total_trades: u64,
    /// 总成交量
    pub total_volume: Decimal,
    /// 总成交额
    pub total_quote_volume: Decimal,
    /// 最后一笔成交价
    pub last_price: Option<Decimal>,
    /// 最后更新时间
    pub updated_at: i64,
}

impl Default for EngineStats {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            total_trades: 0,
            total_volume: Decimal::ZERO,
            total_quote_volume: Decimal::ZERO,
            last_price: None,
            updated_at: chrono::Utc::now().timestamp_millis(),
        }
    }
}

/// 订单输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderInput {
    /// 订单ID
    pub order_id: String,
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
}

/// 撤单输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelInput {
    /// 订单ID
    pub order_id: String,
    /// 用户ID
    pub user_id: String,
    /// 交易对
    pub symbol: String,
}