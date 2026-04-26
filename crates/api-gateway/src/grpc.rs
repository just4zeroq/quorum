//! gRPC 客户端模块
//!
//! 提供对后端微服务的 gRPC 调用能力

use std::time::Duration;
use tonic::transport::Channel;

/// gRPC 客户端配置
#[derive(Clone)]
pub struct GrpcConfig {
    pub user_service_addr: String,
    pub order_service_addr: String,
    pub auth_service_addr: String,
    pub portfolio_service_addr: String,
    pub risk_service_addr: String,
    pub market_data_service_addr: String,
    pub prediction_market_service_addr: String,
    pub wallet_service_addr: String,
    pub timeout: Duration,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            user_service_addr: "http://127.0.0.1:50001".to_string(),
            order_service_addr: "http://127.0.0.1:50004".to_string(),
            auth_service_addr: "http://127.0.0.1:50009".to_string(),
            portfolio_service_addr: "http://127.0.0.1:50003".to_string(),
            risk_service_addr: "http://127.0.0.1:50005".to_string(),
            market_data_service_addr: "http://127.0.0.1:50006".to_string(),
            prediction_market_service_addr: "http://127.0.0.1:50008".to_string(),
            wallet_service_addr: "http://127.0.0.1:50002".to_string(),
            timeout: Duration::from_secs(10),
        }
    }
}

/// 连接到 gRPC 服务
pub async fn connect(addr: String) -> Result<Channel, tonic::transport::Error> {
    let endpoint = tonic::transport::Endpoint::new(addr)?;
    endpoint.timeout(Duration::from_secs(10)).connect().await
}

/// 创建 User Service 客户端
pub async fn create_user_client(
    addr: String,
) -> Result<api::UserServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    api::user::create_user_client(&addr).await
}

/// 创建 Order Service 客户端
pub async fn create_order_client(
    addr: String,
) -> Result<api::OrderServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    api::order::create_order_client(&addr).await
}

/// 创建 Auth Service 客户端
pub async fn create_auth_client(
    addr: String,
) -> Result<api::AuthServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    api::auth::create_auth_client(&addr).await
}

/// 创建 Market Data Service 客户端
pub async fn create_market_data_client(
    addr: String,
) -> Result<api::MarketDataServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    api::market_data::create_market_data_client(&addr).await
}

/// 创建 Prediction Market Service 客户端
pub async fn create_prediction_market_client(
    addr: String,
) -> Result<api::PredictionMarketServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    api::prediction_market::create_prediction_market_client(&addr).await
}

/// 创建 Portfolio Service 客户端
pub async fn create_portfolio_client(
    addr: String,
) -> Result<api::PortfolioServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    api::portfolio::create_portfolio_client(&addr).await
}

/// 创建 Risk Service 客户端
pub async fn create_risk_client(
    addr: String,
) -> Result<api::RiskServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    api::risk::create_risk_client(&addr).await
}

/// 创建 Wallet Service 客户端
pub async fn create_wallet_client(
    addr: String,
) -> Result<api::WalletServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    api::wallet::create_wallet_client(&addr).await
}
