//! API - 统一接口定义包
//!
//! 包含所有微服务的 gRPC 接口定义和数据类型
//! 服务实现和 API Gateway 都应依赖此包
//!
//! ## 设计原则
//! - **服务定义输出** - 各服务的 proto 编译后输出到本包
//! - **契约优先** - 接口定义与实现分离
//! - **单向依赖** - api 包不依赖任何业务包
//!
//! ## 模块结构
//! - `user` - 用户服务接口 (注册、登录、用户信息)
//! - `order` - 订单服务接口 (下单、撤单、查询)
//! - `auth` - 鉴权服务接口 (Token 验证、刷新)
//! - `market_data` - 行情服务接口 (订单簿、Ticker、K线)
//! - `prediction_market` - 预测市场服务接口 (市场管理、结算)
//! - `matching` - 撮合引擎接口 (订单簿操作)
//!
//! ## 依赖关系
//! ```
//! domain (纯业务模型)
//!   ↓
//! api (接口定义 + 序列化)
//!   ↓
//! services (服务实现)
//!   ↓
//! gateway (API 网关)
//! ```

// User service types
pub mod user {
    include!("user.rs");
}

// Order service types
pub mod order {
    include!("order.rs");
}

// Auth service types
pub mod auth {
    include!("auth.rs");
}

// Market data service types
pub mod market_data {
    include!("market_data.rs");
}

// Prediction market service types
pub mod prediction_market {
    include!("prediction_market.rs");
}

// Matching engine types
pub mod matching {
    include!("matching.rs");
}

// Re-export client traits for convenience
pub use user::user_service_client::UserServiceClient;
pub use order::order_service_client::OrderServiceClient;
pub use auth::auth_service_client::AuthServiceClient;
