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

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Spot => "spot",
            AccountType::Futures => "futures",
        }
    }
}

impl std::str::FromStr for AccountType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "spot" => Ok(AccountType::Spot),
            "futures" => Ok(AccountType::Futures),
            _ => Err(format!("Unknown account type: {}", s)),
        }
    }
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
    pub version: i64,
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

impl PositionSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            PositionSide::Long => "long",
            PositionSide::Short => "short",
        }
    }
}

impl std::str::FromStr for PositionSide {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "long" => Ok(PositionSide::Long),
            "short" => Ok(PositionSide::Short),
            _ => Err(format!("Unknown position side: {}", s)),
        }
    }
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
    pub version: i64,
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

impl SettlementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SettlementStatus::Pending => "pending",
            SettlementStatus::Completed => "completed",
            SettlementStatus::Failed => "failed",
        }
    }
}

impl std::str::FromStr for SettlementStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(SettlementStatus::Pending),
            "completed" => Ok(SettlementStatus::Completed),
            "failed" => Ok(SettlementStatus::Failed),
            _ => Err(format!("Unknown settlement status: {}", s)),
        }
    }
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

impl LedgerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LedgerType::Deposit => "deposit",
            LedgerType::Withdraw => "withdraw",
            LedgerType::Freeze => "freeze",
            LedgerType::Unfreeze => "unfreeze",
            LedgerType::Trade => "trade",
            LedgerType::Settle => "settle",
            LedgerType::TransferIn => "transfer_in",
            LedgerType::TransferOut => "transfer_out",
        }
    }
}

impl std::str::FromStr for LedgerType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deposit" => Ok(LedgerType::Deposit),
            "withdraw" => Ok(LedgerType::Withdraw),
            "freeze" => Ok(LedgerType::Freeze),
            "unfreeze" => Ok(LedgerType::Unfreeze),
            "trade" => Ok(LedgerType::Trade),
            "settle" => Ok(LedgerType::Settle),
            "transfer_in" => Ok(LedgerType::TransferIn),
            "transfer_out" => Ok(LedgerType::TransferOut),
            _ => Err(format!("Unknown ledger type: {}", s)),
        }
    }
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
