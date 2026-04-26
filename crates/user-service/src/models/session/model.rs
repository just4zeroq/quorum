//! 会话模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 用户会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: i64,
    pub user_id: i64,
    pub session_token: String,
    pub refresh_token: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
    pub login_method: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

/// 登录日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub login_method: String,
    pub wallet_address: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub success: bool,
    pub fail_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Token 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

/// 刷新 Token 请求
#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 登出请求
#[derive(Debug, Clone, Deserialize)]
pub struct LogoutRequest {
    pub token: Option<String>,
}

/// 会话列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: i64,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
    pub login_method: String,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl From<UserSession> for SessionInfo {
    fn from(s: UserSession) -> Self {
        Self {
            id: s.id,
            ip_address: s.ip_address,
            user_agent: s.user_agent,
            device_id: s.device_id,
            login_method: s.login_method,
            created_at: s.created_at,
            last_active_at: s.last_active_at,
            expires_at: s.expires_at,
        }
    }
}