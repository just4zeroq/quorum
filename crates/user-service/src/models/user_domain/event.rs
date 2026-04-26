//! User Event - 用户事件

use serde::{Deserialize, Serialize};

/// 用户事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserEvent {
    Registered { user_id: i64, email: String },
    Login { user_id: i64, method: String },
    Logout { user_id: i64 },
    Frozen { user_id: i64, reason: Option<String> },
    Unfrozen { user_id: i64 },
}