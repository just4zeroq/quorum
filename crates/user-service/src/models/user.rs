//! 用户模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::wallet::{WalletAddress, WalletInfo};

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password_hash: Option<String>,
    pub kyc_status: KycStatus,
    pub kyc_level: i32,
    pub kyc_submitted_at: Option<DateTime<Utc>>,
    pub kyc_verified_at: Option<DateTime<Utc>>,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
    pub status: UserStatus,
    pub status_reason: Option<String>,
    pub frozen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// KYC 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KycStatus {
    None,
    Submitting,
    Pending,
    Verified,
    Rejected,
}

impl Default for KycStatus {
    fn default() -> Self {
        Self::None
    }
}

impl std::fmt::Display for KycStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KycStatus::None => write!(f, "none"),
            KycStatus::Submitting => write!(f, "submitting"),
            KycStatus::Pending => write!(f, "pending"),
            KycStatus::Verified => write!(f, "verified"),
            KycStatus::Rejected => write!(f, "rejected"),
        }
    }
}

/// 用户状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Frozen,
    Closed,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Active => write!(f, "active"),
            UserStatus::Frozen => write!(f, "frozen"),
            UserStatus::Closed => write!(f, "closed"),
        }
    }
}

/// 登录方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoginMethod {
    Password,
    Wallet,
}

impl Default for LoginMethod {
    fn default() -> Self {
        Self::Password
    }
}

impl std::fmt::Display for LoginMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginMethod::Password => write!(f, "password"),
            LoginMethod::Wallet => write!(f, "wallet"),
        }
    }
}

/// 账户概要 (登录时返回)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub spot_enabled: bool,
    pub futures_enabled: bool,
    pub deposit_addresses: Vec<WalletAddress>,
}

impl Default for AccountSummary {
    fn default() -> Self {
        Self {
            spot_enabled: false,
            futures_enabled: false,
            deposit_addresses: vec![],
        }
    }
}

/// 用户注册请求
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub invite_code: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
}

/// 用户登录请求
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub code_2fa: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
}

/// 登录响应
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub user_id: i64,
    pub token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub need_2fa: bool,
    pub user: User,
    pub accounts: AccountSummary,
}

/// 用户信息响应
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub user: User,
    pub wallets: Vec<WalletInfo>,
    pub tags: Vec<String>,
}