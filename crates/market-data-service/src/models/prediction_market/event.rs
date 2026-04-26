//! Prediction Market Event - 预测市场事件

use serde::{Deserialize, Serialize};

/// 预测市场事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PredictionMarketEvent {
    /// 市场创建
    MarketCreated {
        market_id: i64,
        question: String,
    },
    /// 市场关闭
    MarketClosed {
        market_id: i64,
    },
    /// 市场结算
    MarketResolved {
        market_id: i64,
        outcome_id: i64,
    },
    /// 添加选项
    OutcomeAdded {
        market_id: i64,
        outcome_id: i64,
        name: String,
    },
}