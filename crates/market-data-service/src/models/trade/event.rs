//! Trade Event - 成交事件

use serde::{Deserialize, Serialize};
use super::model::Trade;

/// 成交事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TradeEvent {
    /// 成交创建
    Created {
        trade_id: String,
        order_id: String,
        market_id: i64,
        taker_user_id: i64,
        maker_user_id: i64,
    },
    /// 成交结算 (市场结算后)
    Settled {
        trade_id: String,
        market_id: i64,
        outcome_id: i64,
    },
}

impl TradeEvent {
    pub fn trade_id(&self) -> &str {
        match self {
            TradeEvent::Created { trade_id, .. } => trade_id,
            TradeEvent::Settled { trade_id, .. } => trade_id,
        }
    }
}