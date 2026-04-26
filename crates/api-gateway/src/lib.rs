//! API Gateway 服务
//!
//! 请求入口、身份验证、限流、路由

pub mod router;
pub mod middleware;
pub mod handlers;
pub mod grpc;
pub mod ws_proxy;

pub use router::create_router;
pub use middleware::{auth, rate_limit, log_request, cors_handler};
pub use handlers::*;
pub use grpc::{GrpcClientManager, connect, GrpcConfig};