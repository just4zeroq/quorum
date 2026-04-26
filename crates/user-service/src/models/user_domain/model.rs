//! User Model - 用户数据模型

use serde::{Deserialize, Serialize};

/// 用户状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
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

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub phone: Option<String>,
    pub status: UserStatus,
    pub kyc_status: String,
    pub two_factor_enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 用户会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
    pub created_at: i64,
}