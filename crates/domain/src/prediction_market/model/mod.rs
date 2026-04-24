//! Prediction Market Model - 预测市场数据模型

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 市场状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MarketStatus {
    Open,
    Closed,
    Resolved,
    Cancelled,
}

impl Default for MarketStatus {
    fn default() -> Self {
        Self::Open
    }
}

impl std::fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketStatus::Open => write!(f, "open"),
            MarketStatus::Closed => write!(f, "closed"),
            MarketStatus::Resolved => write!(f, "resolved"),
            MarketStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl MarketStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "open" => MarketStatus::Open,
            "closed" => MarketStatus::Closed,
            "resolved" => MarketStatus::Resolved,
            "cancelled" => MarketStatus::Cancelled,
            _ => MarketStatus::Open,
        }
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

impl PredictionMarket {
    pub fn new(
        question: String,
        description: Option<String>,
        category: String,
        image_url: Option<String>,
        start_time: i64,
        end_time: i64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: 0,
            question,
            description,
            category,
            image_url,
            start_time,
            end_time,
            status: MarketStatus::Open,
            resolved_outcome_id: None,
            resolved_at: None,
            total_volume: Decimal::ZERO,
            created_at: now,
            updated_at: now,
        }
    }
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

impl MarketOutcome {
    pub fn new(
        market_id: i64,
        name: String,
        description: Option<String>,
        image_url: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: 0,
            market_id,
            name,
            description,
            image_url,
            price: Decimal::ZERO,
            volume: Decimal::ZERO,
            probability: Decimal::ZERO,
            created_at: now,
            updated_at: now,
        }
    }
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