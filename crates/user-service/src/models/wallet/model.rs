//! 钱包地址模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 钱包地址
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddress {
    pub id: i64,
    pub user_id: i64,
    pub wallet_address: String,
    pub wallet_type: WalletType,
    pub chain_type: ChainType,
    pub is_primary: bool,
    pub verified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// 钱包类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WalletType {
    MetaMask,
    Coinbase,
    Phantom,
    TrustWallet,
    Other,
}

impl Default for WalletType {
    fn default() -> Self {
        Self::MetaMask
    }
}

impl std::fmt::Display for WalletType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletType::MetaMask => write!(f, "metamask"),
            WalletType::Coinbase => write!(f, "coinbase"),
            WalletType::Phantom => write!(f, "phantom"),
            WalletType::TrustWallet => write!(f, "trustwallet"),
            WalletType::Other => write!(f, "other"),
        }
    }
}

/// 链类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    Evm,
    Solana,
    Aptos,
    Tron,
    Bitcoin,
    Other,
}

impl Default for ChainType {
    fn default() -> Self {
        Self::Evm
    }
}

impl std::fmt::Display for ChainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainType::Evm => write!(f, "evm"),
            ChainType::Solana => write!(f, "solana"),
            ChainType::Aptos => write!(f, "aptos"),
            ChainType::Tron => write!(f, "tron"),
            ChainType::Bitcoin => write!(f, "bitcoin"),
            ChainType::Other => write!(f, "other"),
        }
    }
}

/// 钱包信息 (返回给前端)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub wallet_address: String,
    pub wallet_type: String,
    pub chain_type: String,
    pub is_primary: bool,
    pub bound_at: DateTime<Utc>,
}

impl From<WalletAddress> for WalletInfo {
    fn from(w: WalletAddress) -> Self {
        Self {
            wallet_address: w.wallet_address,
            wallet_type: w.wallet_type.to_string(),
            chain_type: w.chain_type.to_string(),
            is_primary: w.is_primary,
            bound_at: w.verified_at,
        }
    }
}

/// 钱包登录请求
#[derive(Debug, Clone, Deserialize)]
pub struct WalletLoginRequest {
    pub wallet_address: String,
    pub wallet_type: String,
    pub signature: String,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
}

/// 钱包登录响应
#[derive(Debug, Clone, Serialize)]
pub struct WalletLoginResponse {
    pub user_id: i64,
    pub token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub is_new_user: bool,
    pub user: Option<crate::models::user_domain::model::User>,
    pub need_bind_email: bool,
}

/// 获取钱包 nonce 请求
#[derive(Debug, Clone, Deserialize)]
pub struct GetWalletNonceRequest {
    pub wallet_address: String,
    pub wallet_type: String,
}

/// 获取钱包 nonce 响应
#[derive(Debug, Clone, Serialize)]
pub struct GetWalletNonceResponse {
    pub nonce: String,
    pub expires_at: i64,
}

/// 绑定钱包请求
#[derive(Debug, Clone, Deserialize)]
pub struct WalletBindRequest {
    pub user_id: i64,
    pub wallet_address: String,
    pub signature: String,
    pub email: String,
    pub password: String,
}

/// 绑定钱包响应
#[derive(Debug, Clone, Serialize)]
pub struct WalletBindResponse {
    pub success: bool,
    pub message: String,
}