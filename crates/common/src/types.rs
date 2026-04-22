//! 公共类型定义

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 资产类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Asset {
    USDT,
    BTC,
    ETH,
    BNB,
    SOL,
    /// 其他资产
    Other(String),
}

impl Asset {
    pub fn as_str(&self) -> &str {
        match self {
            Asset::USDT => "USDT",
            Asset::BTC => "BTC",
            Asset::ETH => "ETH",
            Asset::BNB => "BNB",
            Asset::SOL => "SOL",
            Asset::Other(s) => s,
        }
    }
}

impl From<&str> for Asset {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "USDT" => Asset::USDT,
            "BTC" => Asset::BTC,
            "ETH" => Asset::ETH,
            "BNB" => Asset::BNB,
            "SOL" => Asset::SOL,
            other => Asset::Other(other.to_string()),
        }
    }
}

/// 账户类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// 资金账户（法币出入金、内部转账中转）
    Funding,
    /// 现货账户
    Spot,
    /// 合约账户
    Futures,
}

/// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    /// 买单
    Buy,
    /// 卖单
    Sell,
}

impl Side {
    pub fn is_buy(&self) -> bool {
        matches!(self, Side::Buy)
    }

    pub fn is_sell(&self) -> bool {
        matches!(self, Side::Sell)
    }
}

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// 限价单
    Limit,
    /// 市价单
    Market,
    /// 立即成交或取消
    IOC,
    /// 全部成交或取消
    FOK,
    /// 只做 Maker
    PostOnly,
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// 待提交
    Pending,
    /// 已提交（待撮合）
    Submitted,
    /// 部分成交
    PartiallyFilled,
    /// 完全成交
    Filled,
    /// 已取消
    Cancelled,
    /// 已拒绝
    Rejected,
}

/// 持仓方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PositionSide {
    /// 多仓
    Long,
    /// 空仓
    Short,
    /// 双向持仓
    Both,
}

/// 账本业务类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BizType {
    /// 现货成交
    Trade,
    /// 手续费
    Fee,
    /// 充值
    Deposit,
    /// 提现
    Withdraw,
    /// Funding 费率
    Funding,
    /// 强平
    Liquidation,
    /// 内部划转
    Transfer,
}

/// 账本方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction {
    /// 借方（支出/减少）
    Debit,
    /// 贷方（收入/增加）
    Credit,
}

/// 账本条目状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EntryStatus {
    /// 已确认（不可回滚）
    Confirmed,
    /// 待确认
    Pending,
}

/// 账户余额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    /// 可用余额
    pub available: Decimal,
    /// 冻结余额
    pub frozen: Decimal,
    /// 权益（合约）
    pub equity: Decimal,
}

impl AccountBalance {
    pub fn new() -> Self {
        Self {
            available: Decimal::ZERO,
            frozen: Decimal::ZERO,
            equity: Decimal::ZERO,
        }
    }

    pub fn total(&self) -> Decimal {
        self.available + self.frozen
    }
}

impl Default for AccountBalance {
    fn default() -> Self {
        Self::new()
    }
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// 订单ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 交易对
    pub symbol: String,
    /// 订单方向
    pub side: Side,
    /// 订单类型
    pub order_type: OrderType,
    /// 价格
    pub price: Decimal,
    /// 数量
    pub quantity: Decimal,
    /// 已成交数量
    pub filled_quantity: Decimal,
    /// 订单状态
    pub status: OrderStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(
        user_id: String,
        symbol: String,
        side: Side,
        order_type: OrderType,
        price: Decimal,
        quantity: Decimal,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            symbol,
            side,
            order_type,
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected
        )
    }
}

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

/// 持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// 持仓ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 交易对
    pub symbol: String,
    /// 持仓方向
    pub side: PositionSide,
    /// 持仓数量
    pub quantity: Decimal,
    /// 开仓均价
    pub entry_price: Decimal,
    /// 保证金
    pub margin: Decimal,
    /// 维持保证金
    pub maintenance_margin: Decimal,
    /// 未实现盈亏
    pub unrealized_pnl: Decimal,
    /// 杠杆倍数
    pub leverage: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Position {
    pub fn new(
        user_id: String,
        symbol: String,
        side: PositionSide,
        quantity: Decimal,
        entry_price: Decimal,
        margin: Decimal,
        leverage: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            symbol,
            side,
            quantity,
            entry_price,
            margin,
            maintenance_margin: Decimal::ZERO,
            unrealized_pnl: Decimal::ZERO,
            leverage,
            created_at: now,
            updated_at: now,
        }
    }

    /// 计算强平价格（逐仓）
    pub fn liquidation_price(&self, maint_margin_rate: Decimal) -> Option<Decimal> {
        match self.side {
            PositionSide::Long => {
                Some(self.entry_price - (self.margin - maint_margin_rate * self.quantity) / self.quantity)
            }
            PositionSide::Short => {
                Some(self.entry_price + (self.margin - maint_margin_rate * self.quantity) / self.quantity)
            }
            PositionSide::Both => None, // 全仓需要综合计算
        }
    }
}

/// 账本条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// 流水号
    pub id: u64,
    /// 业务类型
    pub biz_type: BizType,
    /// 关联业务ID
    pub ref_id: String,
    /// 借贷明细
    pub entries: Vec<LedgerItem>,
    /// 创建时间
    pub timestamp: DateTime<Utc>,
    /// 状态
    pub status: EntryStatus,
}

/// 账本明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerItem {
    /// 账户
    pub account: String,
    /// 资产
    pub asset: String,
    /// 方向
    pub direction: Direction,
    /// 金额
    pub amount: Decimal,
}

/// 价格档位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// 价格
    pub price: Decimal,
    /// 数量
    pub quantity: Decimal,
}

/// 深度数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    /// 交易对
    pub symbol: String,
    /// 卖盘（价格从低到高）
    pub asks: Vec<PriceLevel>,
    /// 买盘（价格从高到低）
    pub bids: Vec<PriceLevel>,
    /// 更新时间
    pub timestamp: DateTime<Utc>,
}

impl Orderbook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            asks: Vec::new(),
            bids: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    /// 获取最佳卖价
    pub fn best_ask(&self) -> Option<&PriceLevel> {
        self.asks.first()
    }

    /// 获取最佳买价
    pub fn best_bid(&self) -> Option<&PriceLevel> {
        self.bids.first()
    }

    /// 获取价差
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask.price - bid.price),
            _ => None,
        }
    }
}

/// K线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    /// 交易对
    pub symbol: String,
    /// 周期
    pub interval: String,
    /// 开盘价
    pub open: Decimal,
    /// 最高价
    pub high: Decimal,
    /// 最低价
    pub low: Decimal,
    /// 收盘价
    pub close: Decimal,
    /// 成交量
    pub volume: Decimal,
    /// 成交额
    pub quote_volume: Decimal,
    /// 时间
    pub timestamp: DateTime<Utc>,
}

/// 行情数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    /// 交易对
    pub symbol: String,
    /// 最新价
    pub last_price: Decimal,
    /// 24h 涨跌
    pub price_change: Decimal,
    /// 24h 涨跌幅
    pub price_change_percent: Decimal,
    /// 24h 最高价
    pub high_price: Decimal,
    /// 24h 最低价
    pub low_price: Decimal,
    /// 24h 成交量
    pub volume: Decimal,
    /// 24h 成交额
    pub quote_volume: Decimal,
    /// 更新时间
    pub timestamp: DateTime<Utc>,
}

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 用户ID
    pub id: String,
    /// 用户名
    pub username: String,
    /// 邮箱
    pub email: Option<String>,
    /// 手机号
    pub phone: Option<String>,
    /// KYC 状态
    pub kyc_status: KycStatus,
    /// 2FA 状态
    pub two_factor_enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// KYC 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KycStatus {
    /// 未认证
    None,
    /// 待审核
    Pending,
    /// 已认证
    Verified,
    /// 拒绝
    Rejected,
}

/// 充值记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    /// 充值ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 资产
    pub asset: String,
    /// 金额
    pub amount: Decimal,
    /// 充值地址
    pub address: String,
    /// 交易哈希
    pub tx_hash: String,
    /// 区块确认数
    pub confirmations: u32,
    /// 状态
    pub status: DepositStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 充值状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositStatus {
    /// 待确认
    Pending,
    /// 已到账
    Credited,
    /// 已解锁
    Unlocked,
    /// 已拒绝
    Rejected,
}

/// 提现记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Withdrawal {
    /// 提现ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 资产
    pub asset: String,
    /// 金额
    pub amount: Decimal,
    /// 提现地址
    pub address: String,
    /// 状态
    pub status: WithdrawalStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 提现状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WithdrawalStatus {
    /// 待审核
    Pending,
    /// 审核通过
    Approved,
    /// 已提交
    Submitted,
    /// 已确认
    Confirmed,
    /// 已拒绝
    Rejected,
    /// 失败
    Failed,
}

/// 风控信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSignal {
    /// 用户ID
    pub user_id: String,
    /// 信号类型
    pub signal_type: RiskSignalType,
    /// 交易对
    pub symbol: Option<String>,
    /// 详情
    pub details: String,
    /// 时间
    pub timestamp: DateTime<Utc>,
}

/// 风控信号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskSignalType {
    /// 强平信号
    Liquidation,
    /// 保证金不足
    MarginInsufficient,
    /// 价格偏离过大
    PriceDeviation,
    /// 持仓超限
    PositionLimit,
    /// ADL 信号
    ADL,
}

/// 强平单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationOrder {
    /// 用户ID
    pub user_id: String,
    /// 交易对
    pub symbol: String,
    /// 持仓方向
    pub side: PositionSide,
    /// 数量
    pub quantity: Decimal,
    /// 价格（市价单为0）
    pub price: Decimal,
    /// 强平原因
    pub reason: String,
    /// 时间
    pub timestamp: DateTime<Utc>,
}