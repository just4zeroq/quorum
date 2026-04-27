//! 风控相关类型

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::position::PositionSide;

/// 风控信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSignal {
    /// 用户ID
    pub user_id: String,
    /// 信号类型
    pub signal_type: RiskSignalType,
    /// 交易对
    pub symbol: Option<String>,
    /// 详情
    pub details: String,
    /// 时间
    pub timestamp: DateTime<Utc>,
}

/// 风控信号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskSignalType {
    /// 强平信号
    Liquidation,
    /// 保证金不足
    MarginInsufficient,
    /// 价格偏离过大
    PriceDeviation,
    /// 持仓超限
    PositionLimit,
    /// ADL 信号
    ADL,
}

/// 强平单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationOrder {
    /// 用户ID
    pub user_id: String,
    /// 交易对
    pub symbol: String,
    /// 持仓方向
    pub side: PositionSide,
    /// 数量
    pub quantity: Decimal,
    /// 价格（市价单为0）
    pub price: Decimal,
    /// 强平原因
    pub reason: String,
    /// 时间
    pub timestamp: DateTime<Utc>,
}
