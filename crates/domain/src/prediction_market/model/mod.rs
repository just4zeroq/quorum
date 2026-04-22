//! Prediction Market Model - 预测市场数据模型

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 市场状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MarketStatus {
    Open,
    Resolved,
    Cancelled,
}

impl Default for MarketStatus {
    fn default() -> Self {
        Self::Open
    }
}

/// 预测市场
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionMarket {
    pub id: i64,
    pub question: String,
    pub description: Option<String>,
    pub category: String,
    pub image_url: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub status: MarketStatus,
    pub resolved_outcome_id: Option<i64>,
    pub resolved_at: Option<i64>,
    pub total_volume: Decimal,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 市场选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOutcome {
    pub id: i64,
    pub market_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub price: Decimal,
    pub volume: Decimal,
    pub probability: Decimal,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 结算记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub id: i64,
    pub market_id: i64,
    pub outcome_id: i64,
    pub total_payout: Decimal,
    pub winning_quantity: Decimal,
    pub payout_ratio: Decimal,
    pub resolved_at: i64,
}