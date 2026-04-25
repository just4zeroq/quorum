//! Shared constants for the exchange system
//!
//! These constants are shared across services (order-service, matching-engine, etc.)
//! to ensure consistency in price scaling and symbol encoding.

/// 价格缩放系数
///
/// 所有价格在传输和撮合引擎中使用 i64 定点数表示，实际价格 = scaled_price / PRICE_SCALE
/// 与 matching-engine 的 CoreSymbolSpecification.quote_scale_k 对齐
pub const PRICE_SCALE: i64 = 100000;

/// 预测市场 symbol 编码乘数
///
/// symbol = market_id * OUTCOME_MULTIPLIER + outcome_id
/// 用于在单个 i32/i64 字段中编码市场和结果信息
pub const OUTCOME_MULTIPLIER: i64 = 1000;
