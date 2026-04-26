//! 标签模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 用户标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户标签关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTagMap {
    pub user_id: i64,
    pub tag_id: i64,
    pub assigned_by: Option<i64>,
    pub assigned_at: DateTime<Utc>,
    pub note: Option<String>,
}

/// 添加标签请求
#[derive(Debug, Clone, Deserialize)]
pub struct AddTagRequest {
    pub user_id: i64,
    pub tag_id: i64,
    pub note: Option<String>,
}

/// 移除标签请求
#[derive(Debug, Clone, Deserialize)]
pub struct RemoveTagRequest {
    pub user_id: i64,
    pub tag_id: i64,
}

/// 系统预设标签
pub mod system_tags {
    pub const VIP: &str = "VIP";
    pub const WHALE: &str = "Whale";
    pub const MARKET_MAKER: &str = "Market Maker";
    pub const RISK: &str = "Risk";
    pub const BLOCKED: &str = "Blocked";
    pub const TESTER: &str = "Tester";

    pub fn all() -> Vec<(&'static str, &'static str)> {
        vec![
            (VIP, "VIP用户"),
            (WHALE, "大户"),
            (MARKET_MAKER, "做市商"),
            (RISK, "风控关注"),
            (BLOCKED, "黑名单"),
            (TESTER, "测试用户"),
        ]
    }
}