//! Portfolio 数据模型

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 账户类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Spot,
    Futures,
}

/// 账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub user_id: String,
    pub asset: String,
    pub account_type: AccountType,
    pub available: Decimal,
    pub frozen: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn total(&self) -> Decimal {
        self.available + self.frozen
    }
}

/// 持仓方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionSide {
    Long,   // 买 YES
    Short,  // 买 NO
}

/// 持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub user_id: String,
    pub market_id: u64,
    pub outcome_id: u64,
    pub side: PositionSide,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 结算状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementStatus {
    Pending,
    Completed,
    Failed,
}

/// 结算记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub id: String,
    pub trade_id: String,
    pub market_id: u64,
    pub user_id: String,
    pub outcome_id: u64,
    pub side: PositionSide,
    pub amount: Decimal,
    pub fee: Decimal,
    pub payout: Decimal,
    pub status: SettlementStatus,
    pub created_at: DateTime<Utc>,
}

/// 账本类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LedgerType {
    Deposit,     // 充值
    Withdraw,    // 提现
    Freeze,      // 冻结
    Unfreeze,    // 解冻
    Trade,       // 交易
    Settle,      // 结算
    TransferIn,  // 转入
    TransferOut, // 转出
}

/// 账本流水
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub user_id: String,
    pub account_id: String,
    pub ledger_type: LedgerType,
    pub asset: String,
    pub amount: Decimal,
    pub balance_after: Decimal,
    pub reference_id: String,
    pub reference_type: String,
    pub created_at: DateTime<Utc>,
}

/// 冻结请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreezeRequest {
    pub user_id: String,
    pub asset: String,
    pub amount: Decimal,
    pub order_id: String,
}

/// 解冻请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnfreezeRequest {
    pub user_id: String,
    pub asset: String,
    pub amount: Decimal,
    pub order_id: String,
}

/// 划转请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub from_user_id: String,
    pub to_user_id: String,
    pub asset: String,
    pub amount: Decimal,
}
