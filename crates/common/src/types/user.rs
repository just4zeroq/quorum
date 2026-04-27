//! 用户相关类型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 用户ID
    pub id: String,
    /// 用户名
    pub username: String,
    /// 邮箱
    pub email: Option<String>,
    /// 手机号
    pub phone: Option<String>,
    /// KYC 状态
    pub kyc_status: KycStatus,
    /// 2FA 状态
    pub two_factor_enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// KYC 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KycStatus {
    /// 未认证
    None,
    /// 待审核
    Pending,
    /// 已认证
    Verified,
    /// 拒绝
    Rejected,
}
