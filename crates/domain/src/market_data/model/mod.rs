//! Market Data Model - 行情数据模型

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// K线周期
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KlineInterval {
    #[serde(rename = "1m")]
    Interval1m,
    #[serde(rename = "5m")]
    Interval5m,
    #[serde(rename = "15m")]
    Interval15m,
    #[serde(rename = "1h")]
    Interval1h,
    #[serde(rename = "4h")]
    Interval4h,
    #[serde(rename = "1d")]
    Interval1d,
}

/// 市场 (行情维度)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub status: String,
    pub total_volume: Decimal,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 选项 (行情维度)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub id: i64,
    pub market_id: i64,
    pub name: String,
    pub price: Decimal,
    pub volume: Decimal,
    pub probability: Decimal,
}

/// 订单簿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub market_id: i64,
    pub outcome_id: i64,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: i64,
}

/// 订单簿档位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
    pub orders: i32,
}

/// K线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub market_id: i64,
    pub outcome_id: i64,
    pub interval: KlineInterval,
    pub timestamp: i64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
}