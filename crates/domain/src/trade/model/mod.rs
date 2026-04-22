//! Trade Model - 成交数据模型

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 成交方向 (Taker 方向)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TradeSide {
    Buy,
    Sell,
}

impl std::fmt::Display for TradeSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeSide::Buy => write!(f, "buy"),
            TradeSide::Sell => write!(f, "sell"),
        }
    }
}

/// 成交记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,               // 成交ID
    pub trade_id: String,         // 成交ID (同 id)
    pub order_id: String,         // Taker 订单ID
    pub counter_order_id: String, // Maker 订单ID
    pub market_id: i64,
    pub outcome_id: i64,
    pub maker_user_id: i64,
    pub taker_user_id: i64,
    pub side: TradeSide,
    pub price: Decimal,
    pub quantity: Decimal,
    pub amount: Decimal,
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
    pub fee_token: String,
    pub created_at: i64,
}

impl Trade {
    pub fn new(
        order_id: String,
        counter_order_id: String,
        market_id: i64,
        outcome_id: i64,
        maker_user_id: i64,
        taker_user_id: i64,
        side: TradeSide,
        price: Decimal,
        quantity: Decimal,
    ) -> Self {
        let trade_id = utils::id::generate_trade_id(market_id);
        let amount = price * quantity;
        let taker_fee = amount * Decimal::new(20, 4); // 0.2%
        let maker_fee = amount * Decimal::new(10, 4); // 0.1%

        Self {
            id: trade_id.clone(),
            trade_id,
            order_id,
            counter_order_id,
            market_id,
            outcome_id,
            maker_user_id,
            taker_user_id,
            side,
            price,
            quantity,
            amount,
            maker_fee,
            taker_fee,
            fee_token: "USDT".to_string(),
            created_at: chrono::Utc::now().timestamp_millis(),
        }
    }
}

/// 成交查询
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradeQuery {
    pub user_id: Option<i64>,
    pub market_id: Option<i64>,
    pub outcome_id: Option<i64>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub page: i32,
    pub page_size: i32,
}