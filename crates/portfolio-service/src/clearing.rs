//! Clearing 模块 - 结算清算
//!
//! 职责:
//! - 成交结算
//! - Taker/Maker 手续费计算
//! - 资金划转

use rust_decimal::Decimal;

use crate::errors::PortfolioError;
use crate::models::{Settlement, SettlementStatus, PositionSide};
use crate::repository::PortfolioRepository;

/// 成交信息
#[derive(Debug, Clone)]
pub struct TradeInfo {
    pub trade_id: String,
    pub market_id: u64,
    pub outcome_id: u64,
    pub buyer_id: String,
    pub seller_id: String,
    pub price: Decimal,
    pub size: Decimal,
    pub taker_fee_rate: Decimal,
    pub maker_fee_rate: Decimal,
}

/// 清算是服务
pub struct ClearingService {
    repo: PortfolioRepository,
}

impl ClearingService {
    pub fn new(repo: PortfolioRepository) -> Self {
        Self { repo }
    }

    /// 计算 Taker 手续费
    pub fn calculate_taker_fee(&self, amount: Decimal, rate: Decimal) -> Decimal {
        amount * rate
    }

    /// 计算 Maker 手续费
    pub fn calculate_maker_fee(&self, amount: Decimal, rate: Decimal) -> Decimal {
        amount * rate
    }

    /// 结算一笔成交
    ///
    /// 流程:
    /// 1. 计算成交金额和手续费
    /// 2. 买家扣款 (price * size + taker_fee)
    /// 3. 卖家收款 (price * size - maker_fee)
    /// 4. 记录 Settlement
    pub async fn settle_trade(&self, trade: &TradeInfo) -> Result<Vec<Settlement>, PortfolioError> {
        let amount = trade.price * trade.size;
        let taker_fee = self.calculate_taker_fee(amount, trade.taker_fee_rate);
        let maker_fee = self.calculate_maker_fee(amount, trade.maker_fee_rate);

        tracing::info!(
            "Settling trade {}: amount={}, taker_fee={}, maker_fee={}",
            trade.trade_id, amount, taker_fee, maker_fee
        );

        // TODO: 需要数据库事务保证原子性
        // 当前实现为独立操作（后续需补充事务支持）

        let buyer_settlement = Settlement {
            id: uuid::Uuid::new_v4().to_string(),
            trade_id: trade.trade_id.clone(),
            market_id: trade.market_id,
            user_id: trade.buyer_id.clone(),
            outcome_id: trade.outcome_id,
            side: PositionSide::Long,
            amount,
            fee: taker_fee,
            payout: Decimal::ZERO,
            status: SettlementStatus::Completed,
            created_at: chrono::Utc::now(),
        };

        let seller_settlement = Settlement {
            id: uuid::Uuid::new_v4().to_string(),
            trade_id: trade.trade_id.clone(),
            market_id: trade.market_id,
            user_id: trade.seller_id.clone(),
            outcome_id: trade.outcome_id,
            side: PositionSide::Short,
            amount,
            fee: maker_fee,
            payout: Decimal::ZERO,
            status: SettlementStatus::Completed,
            created_at: chrono::Utc::now(),
        };

        self.repo.insert_settlement(&buyer_settlement).await?;
        self.repo.insert_settlement(&seller_settlement).await?;

        Ok(vec![buyer_settlement, seller_settlement])
    }
}
