//! 钱包相关类型（充值/提现）

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
