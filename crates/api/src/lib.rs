//! API - 统一接口定义包
//!
//! 包含所有微服务的 gRPC 接口定义和数据类型
//! 服务实现和 API Gateway 都应依赖此包
//!
//! ## 设计原则
//! - **纯 Rust 定义** - 接口类型直接用 Rust struct 定义，使用 prost 序列化
//! - **契约优先** - 接口定义与实现分离
//! - **单向依赖** - api 包不依赖任何业务包
//!
//! ## 模块结构
//! - `user` - 用户服务接口 (注册、登录、用户信息)
//! - `auth` - 鉴权服务接口 (Token 验证、刷新)
//! - `order` - 订单服务接口 (下单、撤单、查询)
//! - `portfolio` - 账户+持仓+清算+账本服务接口
//! - `market_data` - 行情服务接口 (订单簿、Ticker、K线)
//! - `prediction_market` - 预测市场服务接口 (市场管理、结算)
//! - `risk` - 风控服务接口
//! - `wallet` - 钱包服务接口
//! - `matching` - 撮合引擎接口
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

pub mod auth;
pub mod matching;
pub mod market_data;
pub mod order;
pub mod portfolio;
pub mod prediction_market;
pub mod risk;
pub mod user;
pub mod wallet;

// Re-export client types for convenience
pub use auth::auth_service_client::AuthServiceClient;
pub use matching::matching_service_client::MatchingServiceClient;
pub use market_data::market_data_service_client::MarketDataServiceClient;
pub use order::order_service_client::OrderServiceClient;
pub use portfolio::portfolio_service_client::PortfolioServiceClient;
pub use prediction_market::prediction_market_service_client::PredictionMarketServiceClient;
pub use risk::risk_service_client::RiskServiceClient;
pub use user::user_service_client::UserServiceClient;
pub use wallet::wallet_service_client::WalletServiceClient;

// Re-export server traits for service implementations
pub use auth::auth_service_server::AuthService;
pub use matching::matching_service_server::MatchingService;
pub use market_data::market_data_service_server::MarketDataService;
pub use order::order_service_server::OrderService;
pub use portfolio::portfolio_service_server::PortfolioService;
pub use prediction_market::prediction_market_service_server::PredictionMarketService;
pub use risk::risk_service_server::RiskService;
pub use user::user_service_server::UserService;
pub use wallet::wallet_service_server::WalletService;
