//! Market Data Service Models

pub mod market_data;
pub mod prediction_market;
pub mod trade;

pub use market_data::model::{Market, Outcome, OrderBook, Kline, KlineInterval, OrderBookLevel};
pub use prediction_market::model::{PredictionMarket, MarketOutcome, MarketStatus};
pub use trade::model::{Trade, TradeSide};

// Service-specific models
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 24h 统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market24hStats {
    pub market_id: i64,
    pub volume_24h: Decimal,
    pub amount_24h: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub price_change: Decimal,
    pub price_change_percent: Decimal,
    pub trade_count_24h: i64,
    pub timestamp: i64,
}
