//! 账户相关类型

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
