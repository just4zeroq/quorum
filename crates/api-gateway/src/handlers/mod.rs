//! API 处理器
//!
//! 按领域拆分为多个子模块，每个子模块包含对应的请求/响应类型和处理函数。

use salvo::prelude::*;
use serde::de::DeserializeOwned;

pub mod health;
pub mod user;
pub mod order;
pub mod portfolio;
pub mod market;
pub mod wallet;
pub mod prediction_market;

pub use health::*;
pub use user::*;
pub use order::*;
pub use portfolio::*;
pub use market::*;
pub use wallet::*;
pub use prediction_market::*;

// ========== 公共辅助函数 ==========

/// 解析 JSON 请求体，失败自动返回 BAD_REQUEST
pub async fn parse_json<T>(req: &mut Request) -> Result<T, StatusCode>
where
    T: DeserializeOwned,
{
    req.parse_json::<T>().await.map_err(|e| {
        tracing::error!("Failed to parse request: {}", e);
        StatusCode::BAD_REQUEST
    })
}

/// 从 depot 中获取 user_id，失败返回 UNAUTHORIZED
pub fn get_user_id(depot: &Depot) -> Result<String, StatusCode> {
    depot
        .get::<String>("user_id")
        .map(|v| v.to_string())
        .map_err(|_| {
            tracing::error!("No user_id in depot");
            StatusCode::UNAUTHORIZED
        })
}

/// 获取 user_id，不存在时返回 "unknown"
pub fn get_user_id_or_unknown(depot: &Depot) -> String {
    depot
        .get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// 将 "usr_xxx" 格式的 user_id 转为 i64
pub fn parse_user_id_i64(user_id: &str) -> i64 {
    user_id.replace("usr_", "").parse().unwrap_or(0)
}
