//! Market Data Event - 行情事件

use serde::{Deserialize, Serialize};

/// 行情事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MarketDataEvent {
    /// 价格更新
    PriceUpdated {
        market_id: i64,
        outcome_id: i64,
        price: String,
    },
    /// 订单簿更新
    OrderBookUpdated {
        market_id: i64,
    },
    /// 新成交
    TradeExecuted {
        market_id: i64,
        price: String,
        quantity: String,
    },
    /// K线更新
    KlineUpdated {
        market_id: i64,
        interval: String,
    },
}