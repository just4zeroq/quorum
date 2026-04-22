//! API Gateway 服务
//!
//! 请求入口、身份验证、限流、路由

pub mod router;
pub mod middleware;
pub mod handlers;

pub use router::create_router;
pub use middleware::{Auth, RateLimit};
pub use handlers::*;