//! 风控规则 - 预测市场专用
//!
//! 适用于预测市场 (Prediction Market) 的现货风控规则
//! 不涉及合约/杠杆/强平等期货概念

use rust_decimal::Decimal;

use crate::errors::RiskError;

/// 风险检查上下文
#[derive(Debug, Clone)]
pub struct RiskCheckContext {
    pub user_id: String,
    pub market_id: u64,
    pub outcome_id: u64,
    pub side: String,
    pub order_type: String,
    pub price: Decimal,
    pub quantity: Decimal,
}

/// 风控规则检查结果
#[derive(Debug, Clone)]
pub struct RiskCheckResult {
    pub accepted: bool,
    pub reason: String,
}

/// 执行所有风控规则检查
pub fn evaluate(ctx: &RiskCheckContext) -> RiskCheckResult {
    // 1. 基础参数校验
    if let Err(e) = validate_basic(ctx) {
        return RiskCheckResult {
            accepted: false,
            reason: e.to_string(),
        };
    }

    // 2. 预测市场价格范围检查
    if let Err(e) = check_prediction_market_price(ctx) {
        return RiskCheckResult {
            accepted: false,
            reason: e.to_string(),
        };
    }

    // 3. 订单数量检查
    if let Err(e) = check_order_quantity(ctx) {
        return RiskCheckResult {
            accepted: false,
            reason: e.to_string(),
        };
    }

    RiskCheckResult {
        accepted: true,
        reason: String::new(),
    }
}

/// 基础参数校验
fn validate_basic(ctx: &RiskCheckContext) -> Result<(), RiskError> {
    if ctx.user_id.is_empty() {
        return Err(RiskError::InvalidParam("user_id is required".into()));
    }
    if ctx.side.is_empty() {
        return Err(RiskError::InvalidParam("side is required".into()));
    }
    let side = ctx.side.to_lowercase();
    if side != "yes" && side != "no" {
        return Err(RiskError::InvalidParam(format!(
            "Invalid side: {}, must be YES or NO",
            ctx.side
        )));
    }
    let order_type = ctx.order_type.to_lowercase();
    if order_type != "limit" && order_type != "market" {
        return Err(RiskError::InvalidParam(format!(
            "Invalid order type: {}, must be limit or market",
            ctx.order_type
        )));
    }
    Ok(())
}

/// 预测市场价格范围检查
///
/// 预测市场中，YES/NO 价格必须在 (0, 1) 范围内
/// (实际使用 SCALE 精度，比如最小报价单位 0.01)
fn check_prediction_market_price(ctx: &RiskCheckContext) -> Result<(), RiskError> {
    if ctx.price <= Decimal::ZERO {
        return Err(RiskError::Rejected(format!(
            "Price must be positive, got {}",
            ctx.price
        )));
    }
    if ctx.price >= Decimal::ONE {
        return Err(RiskError::Rejected(format!(
            "Price must be less than 1.0 for prediction market, got {}",
            ctx.price
        )));
    }
    Ok(())
}

/// 订单数量检查
fn check_order_quantity(ctx: &RiskCheckContext) -> Result<(), RiskError> {
    if ctx.quantity <= Decimal::ZERO {
        return Err(RiskError::Rejected(format!(
            "Quantity must be positive, got {}",
            ctx.quantity
        )));
    }
    Ok(())
}
