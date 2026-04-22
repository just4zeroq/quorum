//! 清算服务

use common::*;

/// 清算服务
pub struct ClearingService;

impl ClearingService {
    pub fn new() -> Self {
        Self
    }

    /// 执行成交结算
    pub fn settle_trade(&self, trade: &Trade) -> Result<SettlementResult, Error> {
        tracing::info!("Settling trade: {}", trade.id);

        // 1. 买方扣款 (USDT)
        let buyer_amount = trade.amount;
        // 2. 卖方入账 (USDT)
        let seller_amount = trade.amount - trade.seller_fee;
        // 3. 买方入账 (BTC)
        let buyer_btc = trade.quantity;
        // 4. 卖方扣款 (BTC)
        let seller_btc = trade.quantity;

        Ok(SettlementResult {
            trade_id: trade.id.clone(),
            buyer_id: trade.buyer_id.clone(),
            seller_id: trade.seller_id.clone(),
            buyer_debit: vec![
                LedgerItem { account: trade.buyer_id.clone(), asset: "USDT".to_string(), direction: Direction::Debit, amount: buyer_amount },
            ],
            buyer_credit: vec![
                LedgerItem { account: trade.buyer_id.clone(), asset: "BTC".to_string(), direction: Direction::Credit, amount: buyer_btc },
            ],
            seller_debit: vec![
                LedgerItem { account: trade.seller_id.clone(), asset: "BTC".to_string(), direction: Direction::Debit, amount: seller_btc },
            ],
            seller_credit: vec![
                LedgerItem { account: trade.seller_id.clone(), asset: "USDT".to_string(), direction: Direction::Credit, amount: seller_amount },
            ],
            fees: vec![
                LedgerItem { account: "platform".to_string(), asset: "USDT".to_string(), direction: Direction::Credit, amount: trade.buyer_fee + trade.seller_fee },
            ],
        })
    }
}

impl Default for ClearingService {
    fn default() -> Self {
        Self::new()
    }
}

/// 结算结果
#[derive(Debug, Clone)]
pub struct SettlementResult {
    pub trade_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub buyer_debit: Vec<LedgerItem>,
    pub buyer_credit: Vec<LedgerItem>,
    pub seller_debit: Vec<LedgerItem>,
    pub seller_credit: Vec<LedgerItem>,
    pub fees: Vec<LedgerItem>,
}