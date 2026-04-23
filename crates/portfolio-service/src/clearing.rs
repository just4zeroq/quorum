//! Clearing 模块 - 结算清算
//!
//! 职责:
//! - 成交结算
//! - Taker/Maker 手续费计算
//! - 资金划转

use rust_decimal::Decimal;
use crate::errors::PortfolioError;
use crate::models::{Settlement, SettlementStatus, PositionSide};

/// 成交信息
#[derive(Debug, Clone)]
pub struct TradeInfo {
    pub trade_id: String,
    pub market_id: u64,
    pub outcome_id: u64,
    pub buyer_id: String,
    pub seller_id: String,
    pub side: PositionSide,
    pub price: Decimal,
    pub size: Decimal,
    pub taker_fee: Decimal,
    pub maker_fee: Decimal,
}

/// 清算是服务
pub struct ClearingService;

impl ClearingService {
    /// 结算一笔成交
    ///
    /// 流程:
    /// 1. 计算手续费
    /// 2. 买家扣款 (price * size + taker_fee)
    /// 3. 卖家收款 (price * size - maker_fee)
    /// 4. 手续费归平台
    pub async fn settle_trade(&self, trade: TradeInfo) -> Result<Settlement, PortfolioError> {
        let amount = trade.price * trade.size;

        tracing::info!(
            "Settling trade {}: amount={}, taker_fee={}, maker_fee={}",
            trade.trade_id, amount, trade.taker_fee, trade.maker_fee
        );

        // TODO: 原子操作
        // 1. 扣买家款
        // 2. 付卖家款
        // 3. 收手续费
        // 4. 记录 Settlement

        Ok(Settlement {
            id: uuid::Uuid::new_v4().to_string(),
            trade_id: trade.trade_id,
            market_id: trade.market_id,
            user_id: trade.buyer_id.clone(),
            outcome_id: trade.outcome_id,
            side: trade.side,
            amount,
            fee: trade.taker_fee,
            payout: Decimal::ZERO,
            status: SettlementStatus::Completed,
            created_at: chrono::Utc::now(),
        })
    }

    /// 市场结算（预测市场结果确定后）
    ///
    /// 胜出方获得:
    /// - 买 YES @ 0.6 数量 100，成本 60
    /// - 胜出后获得 100 (1:1 赔付)
    /// - 利润 = 100 - 60 = 40
    pub async fn settle_market(
        &self,
        market_id: u64,
        winning_outcome_id: u64,
    ) -> Result<(), PortfolioError> {
        tracing::info!(
            "Settling market {} with outcome {}",
            market_id, winning_outcome_id
        );

        // TODO:
        // 1. 找出所有持有 winning_outcome_id 持仓的用户
        // 2. 计算应得赔付 (成本 / 胜出价格 * 总赔付池)
        // 3. 执行赔付

        Ok(())
    }

    /// 计算 Taker 手续费
    pub fn calculate_taker_fee(&self, amount: Decimal) -> Decimal {
        // 默认 1%
        amount * Decimal::new(1, 2)
    }

    /// 计算 Maker 返佣
    pub fn calculate_maker_rebate(&self, amount: Decimal) -> Decimal {
        // 默认 0.2% 返佣
        amount * Decimal::new(2, 3)
    }
}
