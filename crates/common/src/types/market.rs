//! 行情相关类型

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 价格档位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// 价格
    pub price: Decimal,
    /// 数量
    pub quantity: Decimal,
}

/// 深度数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    /// 交易对
    pub symbol: String,
    /// 卖盘（价格从低到高）
    pub asks: Vec<PriceLevel>,
    /// 买盘（价格从高到低）
    pub bids: Vec<PriceLevel>,
    /// 更新时间
    pub timestamp: DateTime<Utc>,
}

impl Orderbook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            asks: Vec::new(),
            bids: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    /// 获取最佳卖价
    pub fn best_ask(&self) -> Option<&PriceLevel> {
        self.asks.first()
    }

    /// 获取最佳买价
    pub fn best_bid(&self) -> Option<&PriceLevel> {
        self.bids.first()
    }

    /// 获取价差
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask.price - bid.price),
            _ => None,
        }
    }
}

/// K线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    /// 交易对
    pub symbol: String,
    /// 周期
    pub interval: String,
    /// 开盘价
    pub open: Decimal,
    /// 最高价
    pub high: Decimal,
    /// 最低价
    pub low: Decimal,
    /// 收盘价
    pub close: Decimal,
    /// 成交量
    pub volume: Decimal,
    /// 成交额
    pub quote_volume: Decimal,
    /// 时间
    pub timestamp: DateTime<Utc>,
}

/// 行情数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    /// 交易对
    pub symbol: String,
    /// 最新价
    pub last_price: Decimal,
    /// 24h 涨跌
    pub price_change: Decimal,
    /// 24h 涨跌幅
    pub price_change_percent: Decimal,
    /// 24h 最高价
    pub high_price: Decimal,
    /// 24h 最低价
    pub low_price: Decimal,
    /// 24h 成交量
    pub volume: Decimal,
    /// 24h 成交额
    pub quote_volume: Decimal,
    /// 更新时间
    pub timestamp: DateTime<Utc>,
}
