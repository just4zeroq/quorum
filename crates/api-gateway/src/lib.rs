//! API Gateway 服务
//!
//! 请求入口、身份验证、限流、路由

pub mod router;
pub mod middleware;
pub mod handlers;
pub mod grpc;

pub use router::create_router;
pub use middleware::{auth, rate_limit, log_request, cors_handler};
pub use handlers::*;
pub use grpc::{GrpcConfig, connect, create_user_client, create_order_client, create_auth_client};