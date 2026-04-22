//! 预测市场数据模型
//!
//! Re-exports from domain crate

pub use domain::prediction_market::model::{
    PredictionMarket, MarketOutcome, MarketStatus, Resolution
};
pub use domain::trade::model::TradeSide;

// Service-specific models
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 用户持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPosition {
    pub id: i64,
    pub user_id: i64,
    pub market_id: i64,
    pub outcome_id: i64,
    pub quantity: Decimal,
    pub avg_price: Decimal,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTrade {
    pub id: i64,
    pub market_id: i64,
    pub outcome_id: i64,
    pub user_id: i64,
    pub side: TradeSide,
    pub price: Decimal,
    pub quantity: Decimal,
    pub amount: Decimal,
    pub fee: Decimal,
    pub created_at: i64,
}

/// 创建市场请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMarketRequest {
    pub question: String,
    pub description: Option<String>,
    pub category: String,
    pub image_url: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub outcomes: Vec<CreateOutcomeRequest>,
}

/// 创建选项请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOutcomeRequest {
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

/// 创建市场响应
#[derive(Debug, Clone, Serialize)]
pub struct CreateMarketResponse {
    pub market_id: i64,
    pub outcomes: Vec<MarketOutcome>,
}

/// 市场列表请求
#[derive(Debug, Clone, Deserialize)]
pub struct ListMarketsRequest {
    pub category: Option<String>,
    pub status: Option<String>,
    pub page: i32,
    pub page_size: i32,
}

impl Default for ListMarketsRequest {
    fn default() -> Self {
        Self {
            category: None,
            status: None,
            page: 1,
            page_size: 20,
        }
    }
}

/// 市场列表响应
#[derive(Debug, Clone, Serialize)]
pub struct ListMarketsResponse {
    pub markets: Vec<PredictionMarket>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 结算请求
#[derive(Debug, Clone, Deserialize)]
pub struct ResolveMarketRequest {
    pub market_id: i64,
    pub outcome_id: i64,
}

/// 结算响应
#[derive(Debug, Clone, Serialize)]
pub struct ResolveMarketResponse {
    pub success: bool,
    pub resolution: Option<Resolution>,
}

/// 用户持仓请求
#[derive(Debug, Clone, Deserialize)]
pub struct GetUserPositionsRequest {
    pub user_id: i64,
    pub market_id: Option<i64>,
}

/// 用户持仓响应
#[derive(Debug, Clone, Serialize)]
pub struct GetUserPositionsResponse {
    pub positions: Vec<UserPosition>,
}