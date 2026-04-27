//! Wallet Service Models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 充值地址
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepositAddress {
    pub id: i64,
    pub user_id: i64,
    pub chain: String,
    pub address: String,
    pub created_at: i64,
}

/// 充值记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepositRecord {
    pub id: i64,
    pub user_id: i64,
    pub tx_id: String,
    pub chain: String,
    pub amount: String,
    pub address: String,
    pub created_at: i64,
}

/// 提现状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WithdrawStatus {
    Pending,
    Confirmed,
    Completed,
    Cancelled,
    Failed,
}

impl ToString for WithdrawStatus {
    fn to_string(&self) -> String {
        match self {
            WithdrawStatus::Pending => "pending".to_string(),
            WithdrawStatus::Confirmed => "confirmed".to_string(),
            WithdrawStatus::Completed => "completed".to_string(),
            WithdrawStatus::Cancelled => "cancelled".to_string(),
            WithdrawStatus::Failed => "failed".to_string(),
        }
    }
}

impl WithdrawStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(WithdrawStatus::Pending),
            "confirmed" => Some(WithdrawStatus::Confirmed),
            "completed" => Some(WithdrawStatus::Completed),
            "cancelled" => Some(WithdrawStatus::Cancelled),
            "failed" => Some(WithdrawStatus::Failed),
            _ => None,
        }
    }
}

/// 提现记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WithdrawRecord {
    pub id: i64,
    pub user_id: i64,
    pub asset: String,
    pub amount: String,
    pub fee: String,
    pub to_address: String,
    pub chain: String,
    pub status: String,
    pub tx_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 地址白名单
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WhitelistAddress {
    pub id: i64,
    pub user_id: i64,
    pub chain: String,
    pub address: String,
    pub label: Option<String>,
    pub created_at: i64,
}

/// 支付密码
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentPassword {
    pub id: i64,
    pub user_id: i64,
    pub password_hash: String,
    pub created_at: i64,
    pub updated_at: i64,
}
