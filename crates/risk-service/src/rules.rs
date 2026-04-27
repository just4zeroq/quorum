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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn make_ctx(
        user_id: &str,
        side: &str,
        order_type: &str,
        price: &str,
        quantity: &str,
    ) -> RiskCheckContext {
        RiskCheckContext {
            user_id: user_id.to_string(),
            market_id: 1,
            outcome_id: 1,
            side: side.to_string(),
            order_type: order_type.to_string(),
            price: Decimal::from_str(price).unwrap(),
            quantity: Decimal::from_str(quantity).unwrap(),
        }
    }

    // ==================== evaluate() ====================

    #[test]
    fn test_evaluate_all_valid() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0.5", "100");
        let result = evaluate(&ctx);
        assert!(result.accepted);
        assert!(result.reason.is_empty());
    }

    #[test]
    fn test_evaluate_invalid_price() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0", "100");
        let result = evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("Price"));
    }

    #[test]
    fn test_evaluate_invalid_quantity() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0.5", "0");
        let result = evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("Quantity"));
    }

    #[test]
    fn test_evaluate_empty_user_id() {
        let ctx = make_ctx("", "YES", "limit", "0.5", "100");
        let result = evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("user_id"));
    }

    #[test]
    fn test_evaluate_invalid_side() {
        let ctx = make_ctx("usr_001", "BUY", "limit", "0.5", "100");
        let result = evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("side"));
    }

    // ==================== validate_basic() ====================

    #[test]
    fn test_validate_basic_empty_user_id() {
        let ctx = make_ctx("", "YES", "limit", "0.5", "100");
        let result = validate_basic(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_basic_empty_side() {
        let ctx = make_ctx("usr_001", "", "limit", "0.5", "100");
        let result = validate_basic(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_basic_side_yes_no() {
        for side in &["YES", "yes", "NO", "no"] {
            let ctx = make_ctx("usr_001", side, "limit", "0.5", "100");
            assert!(validate_basic(&ctx).is_ok(), "side={} should be valid", side);
        }
    }

    #[test]
    fn test_validate_basic_side_invalid() {
        for side in &["buy", "sell", "long", "short"] {
            let ctx = make_ctx("usr_001", side, "limit", "0.5", "100");
            assert!(validate_basic(&ctx).is_err(), "side={} should be invalid", side);
        }
    }

    #[test]
    fn test_validate_basic_order_type_valid() {
        for ot in &["limit", "LIMIT", "market", "MARKET"] {
            let ctx = make_ctx("usr_001", "YES", ot, "0.5", "100");
            assert!(validate_basic(&ctx).is_ok(), "order_type={} should be valid", ot);
        }
    }

    #[test]
    fn test_validate_basic_order_type_invalid() {
        for ot in &["stop", "stop_limit", ""] {
            let ctx = make_ctx("usr_001", "YES", ot, "0.5", "100");
            assert!(validate_basic(&ctx).is_err(), "order_type={} should be invalid", ot);
        }
    }

    // ==================== check_prediction_market_price() ====================

    #[test]
    fn test_price_zero() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0", "100");
        let result = check_prediction_market_price(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_price_negative() {
        let ctx = make_ctx("usr_001", "YES", "limit", "-0.5", "100");
        let result = check_prediction_market_price(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_price_equal_to_one() {
        let ctx = make_ctx("usr_001", "YES", "limit", "1", "100");
        let result = check_prediction_market_price(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_price_above_one() {
        let ctx = make_ctx("usr_001", "YES", "limit", "1.5", "100");
        let result = check_prediction_market_price(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_price_valid_range() {
        for p in &["0.01", "0.5", "0.99"] {
            let ctx = make_ctx("usr_001", "YES", "limit", p, "100");
            assert!(
                check_prediction_market_price(&ctx).is_ok(),
                "price={} should be valid",
                p
            );
        }
    }

    // ==================== check_order_quantity() ====================

    #[test]
    fn test_quantity_zero() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0.5", "0");
        let result = check_order_quantity(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_quantity_negative() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0.5", "-10");
        let result = check_order_quantity(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_quantity_positive() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0.5", "100");
        let result = check_order_quantity(&ctx);
        assert!(result.is_ok());
    }
}
