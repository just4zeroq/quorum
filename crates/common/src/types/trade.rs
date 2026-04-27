//! 成交记录类型

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 成交记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// 成交ID
    pub id: String,
    /// 订单ID
    pub order_id: String,
    /// 对手方订单ID
    pub counter_order_id: String,
    /// 交易对
    pub symbol: String,
    /// 买方用户ID
    pub buyer_id: String,
    /// 卖方用户ID
    pub seller_id: String,
    /// 成交价格
    pub price: Decimal,
    /// 成交数量
    pub quantity: Decimal,
    /// 成交金额
    pub amount: Decimal,
    /// 买方手续费
    pub buyer_fee: Decimal,
    /// 卖方手续费
    pub seller_fee: Decimal,
    /// 成交时间
    pub timestamp: DateTime<Utc>,
}

impl Trade {
    pub fn new(
        order_id: String,
        counter_order_id: String,
        symbol: String,
        buyer_id: String,
        seller_id: String,
        price: Decimal,
        quantity: Decimal,
    ) -> Self {
        let amount = price * quantity;
        // 手续费率 0.1%
        let fee_rate = Decimal::new(1, 3);
        let buyer_fee = amount * fee_rate;
        let seller_fee = amount * fee_rate;

        Self {
            id: Uuid::new_v4().to_string(),
            order_id,
            counter_order_id,
            symbol,
            buyer_id,
            seller_id,
            price,
            quantity,
            amount,
            buyer_fee,
            seller_fee,
            timestamp: Utc::now(),
        }
    }
}
