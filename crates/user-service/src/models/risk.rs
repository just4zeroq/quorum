//! 风控模型

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 用户风控信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRisk {
    pub user_id: i64,
    pub risk_level: i32,           // 0=正常, 1=观察, 2=限制, 3=高风险, 4=冻结交易, 5=完全冻结
    pub kyc_level: i32,            // 0=未认证, 1=邮箱, 2=身份, 3=高级
    pub daily_withdraw_limit: Decimal,
    pub monthly_withdraw_limit: Decimal,
    pub single_withdraw_limit: Decimal,
    pub daily_trade_limit: Decimal,
    pub daily_withdraw_count: i32,
    pub daily_withdraw_reset: Option<DateTime<Utc>>,
    pub daily_login_count: i32,
    pub daily_login_reset: Option<DateTime<Utc>>,
    pub frozen: bool,
    pub frozen_reason: Option<String>,
    pub frozen_at: Option<DateTime<Utc>>,
    pub frozen_until: Option<DateTime<Utc>>,
    pub last_risk_review: Option<DateTime<Utc>>,
    pub risk_review_note: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Default for UserRisk {
    fn default() -> Self {
        Self {
            user_id: 0,
            risk_level: 0,
            kyc_level: 0,
            daily_withdraw_limit: Decimal::ZERO,
            monthly_withdraw_limit: Decimal::ZERO,
            single_withdraw_limit: Decimal::ZERO,
            daily_trade_limit: Decimal::ZERO,
            daily_withdraw_count: 0,
            daily_withdraw_reset: None,
            daily_login_count: 0,
            daily_login_reset: None,
            frozen: false,
            frozen_reason: None,
            frozen_at: None,
            frozen_until: None,
            last_risk_review: None,
            risk_review_note: None,
            updated_at: Utc::now(),
            created_at: Utc::now(),
        }
    }
}

/// 风险等级常量
pub mod risk_level {
    pub const NORMAL: i32 = 0;
    pub const WATCH: i32 = 1;
    pub const RESTRICTED: i32 = 2;
    pub const HIGH_RISK: i32 = 3;
    pub const FROZEN_TRADE: i32 = 4;
    pub const FROZEN_ALL: i32 = 5;
}

/// KYC 等级常量
pub mod kyc_level {
    pub const NONE: i32 = 0;
    pub const EMAIL: i32 = 1;
    pub const IDENTITY: i32 = 2;
    pub const ADVANCED: i32 = 3;
}

/// 修改密码请求
#[derive(Debug, Clone, Deserialize)]
pub struct ChangePasswordRequest {
    pub user_id: i64,
    pub old_password: String,
    pub new_password: String,
    pub ip_address: String,
}

/// 2FA 请求
#[derive(Debug, Clone, Deserialize)]
pub struct Enable2FARequest {
    pub user_id: i64,
    pub password: String,
}

/// 2FA 响应
#[derive(Debug, Clone, Serialize)]
pub struct Enable2FAResponse {
    pub success: bool,
    pub secret: String,
    pub qr_code: String,
    pub message: String,
}

/// 验证 2FA 请求
#[derive(Debug, Clone, Deserialize)]
pub struct Verify2FARequest {
    pub user_id: i64,
    pub code: String,
}

/// 验证 2FA 响应
#[derive(Debug, Clone, Serialize)]
pub struct Verify2FAResponse {
    pub success: bool,
    pub valid: bool,
}

/// 禁用 2FA 请求
#[derive(Debug, Clone, Deserialize)]
pub struct Disable2FARequest {
    pub user_id: i64,
    pub password: String,
    pub code: String,
}

/// 禁用 2FA 响应
#[derive(Debug, Clone, Serialize)]
pub struct Disable2FAResponse {
    pub success: bool,
    pub message: String,
}