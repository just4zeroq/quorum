//! 系统常量定义

use once_cell::sync::Lazy;
use rust_decimal::Decimal;

/// 默认手续费率 (0.1%)
pub static DEFAULT_FEE_RATE: Lazy<Decimal> = Lazy::new(|| Decimal::try_from(0.001).unwrap());

/// 最小交易数量
pub static MIN_QUANTITY: Lazy<Decimal> = Lazy::new(|| Decimal::try_from(0.00000001).unwrap());

/// 最小交易金额
pub static MIN_NOTIONAL: Lazy<Decimal> = Lazy::new(|| Decimal::try_from(0.01).unwrap());

/// 市价单保护价偏差比例 (5%)
pub static MARKET_ORDER_PROTECTION_RATE: Lazy<Decimal> = Lazy::new(|| Decimal::try_from(0.05).unwrap());

/// 强平保证金率阈值
pub static LIQUIDATION_MARGIN_RATE: Lazy<Decimal> = Lazy::new(|| Decimal::try_from(0.01).unwrap());

/// 维持保证金率（默认）
pub static DEFAULT_MAINTENANCE_MARGIN_RATE: Lazy<Decimal> = Lazy::new(|| Decimal::try_from(0.005).unwrap());

/// BTC 确认数
pub const BTC_CONFIRMATIONS: u32 = 1;
pub const BTC_UNLOCK_CONFIRMATIONS: u32 = 2;

/// ETH 确认数
pub const ETH_CONFIRMATIONS: u32 = 6;
pub const ETH_UNLOCK_CONFIRMATIONS: u32 = 64;

/// USDT 确认数
pub const USDT_CONFIRMATIONS: u32 = 1;

/// Funding 结算间隔（秒）
pub const FUNDING_INTERVAL_SECS: i64 = 8 * 60 * 60; // 8小时

/// 订单簿最大深度
pub const ORDERBOOK_MAX_DEPTH: usize = 100;

/// 价格精度
pub const PRICE_PRECISION: u32 = 8;

/// 数量精度
pub const QUANTITY_PRECISION: u32 = 8;

/// 默认杠杆
pub const DEFAULT_LEVERAGE: u32 = 10;

/// 最大杠杆
pub const MAX_LEVERAGE: u32 = 125;

/// 最小杠杆
pub const MIN_LEVERAGE: u32 = 1;

/// 风险监控更新间隔（毫秒）
pub const RISK_UPDATE_INTERVAL_MS: u64 = 100;

/// 行情推送间隔（毫秒）
pub const MARKET_DATA_INTERVAL_MS: u64 = 100;

/// 服务默认端口
pub mod ports {
    pub const API_GATEWAY: u16 = 8080;
    pub const USER_SERVICE: u16 = 8081;
    pub const ORDER_SERVICE: u16 = 8082;
    pub const ACCOUNT_SERVICE: u16 = 8083;
    pub const RISK_SERVICE: u16 = 8084;
    pub const MATCHING_ENGINE: u16 = 8085;
    pub const POSITION_SERVICE: u16 = 8086;
    pub const CLEARING_SERVICE: u16 = 8087;
    pub const MARKET_DATA_SERVICE: u16 = 8088;
    pub const LEDGER_SERVICE: u16 = 8089;
    pub const WALLET_SERVICE: u16 = 8090;
    pub const FUNDING_SERVICE: u16 = 8091;
    pub const MARK_PRICE_SERVICE: u16 = 8092;
}

/// MQ 主题
pub mod topics {
    pub const ORDERS_PREFIX: &str = "orders.";
    pub const TRADES: &str = "trades";
    pub const ORDERBOOK_CHANGES: &str = "orderbook_changes";
    pub const MARK_PRICE_UPDATES: &str = "mark_price_updates";
    pub const RISK_SIGNALS: &str = "risk_signals";
    pub const LIQUIDATION_ORDERS: &str = "liquidation_orders";
    pub const CLEARING_EVENTS: &str = "clearing_events";
    pub const POSITION_UPDATES: &str = "position_updates";
}