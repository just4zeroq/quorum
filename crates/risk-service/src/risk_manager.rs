//! 风控管理器

use common::*;
use parking_lot::RwLock;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 用户风险状态
#[derive(Debug, Clone)]
pub struct UserRiskState {
    pub user_id: String,
    pub total_margin: Decimal,
    pub total_unrealized_pnl: Decimal,
    pub positions: Vec<Position>,
}

/// 风控管理器
pub struct RiskManager {
    /// 用户风险状态
    risk_states: RwLock<HashMap<String, UserRiskState>>,
}

impl RiskManager {
    pub fn new() -> Self {
        Self {
            risk_states: RwLock::new(HashMap::new()),
        }
    }

    /// Pre-trade 检查
    pub fn pre_trade_check(&self, order: &Order, available_balance: Decimal) -> Result<(), Error> {
        // 1. 检查保证金是否足够（开仓）
        if order.side.is_buy() || order.quantity > Decimal::ZERO {
            let required_margin = order.price * order.quantity / Decimal::new(10, 0); // 假设10x杠杆
            if required_margin > available_balance {
                return Err(Error::RiskRejected("Insufficient margin".to_string()));
            }
        }

        // 2. 检查价格偏离（简化版）
        let max_deviation = Decimal::new(5, 1); // 5%
        // 这里应该对比市场价，简化处理

        Ok(())
    }

    /// 更新风险状态
    pub fn update_risk_state(&self, state: UserRiskState) {
        let mut states = self.risk_states.write();
        states.insert(state.user_id.clone(), state);
    }

    /// 检查是否需要强平
    pub fn check_liquidation(&self, user_id: &str, mark_price: Decimal) -> Option<RiskSignal> {
        let states = self.risk_states.read();
        if let Some(state) = states.get(user_id) {
            let equity = state.total_margin + state.total_unrealized_pnl;
            // 简化：保证金率低于10%触发强平
            let margin_ratio = if state.total_margin > Decimal::ZERO {
                equity / state.total_margin
            } else {
                Decimal::ZERO
            };

            if margin_ratio < Decimal::new(1, 1) {
                // 10%
                return Some(RiskSignal {
                    user_id: user_id.to_string(),
                    signal_type: RiskSignalType::Liquidation,
                    symbol: None,
                    details: format!("Margin ratio {} below 10%", margin_ratio),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        None
    }
}

impl Default for RiskManager {
    fn default() -> Self {
        Self::new()
    }
}