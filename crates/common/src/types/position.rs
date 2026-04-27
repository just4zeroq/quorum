//! 持仓相关类型

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 持仓方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PositionSide {
    /// 多仓
    Long,
    /// 空仓
    Short,
    /// 双向持仓
    Both,
}

/// 持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// 持仓ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 交易对
    pub symbol: String,
    /// 持仓方向
    pub side: PositionSide,
    /// 持仓数量
    pub quantity: Decimal,
    /// 开仓均价
    pub entry_price: Decimal,
    /// 保证金
    pub margin: Decimal,
    /// 维持保证金
    pub maintenance_margin: Decimal,
    /// 未实现盈亏
    pub unrealized_pnl: Decimal,
    /// 杠杆倍数
    pub leverage: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Position {
    pub fn new(
        user_id: String,
        symbol: String,
        side: PositionSide,
        quantity: Decimal,
        entry_price: Decimal,
        margin: Decimal,
        leverage: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            symbol,
            side,
            quantity,
            entry_price,
            margin,
            maintenance_margin: Decimal::ZERO,
            unrealized_pnl: Decimal::ZERO,
            leverage,
            created_at: now,
            updated_at: now,
        }
    }

    /// 计算强平价格（逐仓）
    pub fn liquidation_price(&self, maint_margin_rate: Decimal) -> Option<Decimal> {
        match self.side {
            PositionSide::Long => {
                Some(self.entry_price - (self.margin - maint_margin_rate * self.quantity) / self.quantity)
            }
            PositionSide::Short => {
                Some(self.entry_price + (self.margin - maint_margin_rate * self.quantity) / self.quantity)
            }
            PositionSide::Both => None, // 全仓需要综合计算
        }
    }
}
