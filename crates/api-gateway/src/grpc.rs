//! gRPC 客户端模块
//!
//! 提供对后端微服务的 gRPC 调用能力

use tonic::transport::Channel;
use std::time::Duration;

/// gRPC 客户端配置
#[derive(Clone)]
pub struct GrpcConfig {
    pub user_service_addr: String,
    pub order_service_addr: String,
    pub auth_service_addr: String,
    pub portfolio_service_addr: String,
    pub market_data_service_addr: String,
    pub prediction_market_service_addr: String,
    pub timeout: Duration,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            user_service_addr: "http://127.0.0.1:50001".to_string(),
            order_service_addr: "http://127.0.0.1:50004".to_string(),
            auth_service_addr: "http://127.0.0.1:50009".to_string(),
            portfolio_service_addr: "http://127.0.0.1:50003".to_string(),
            market_data_service_addr: "http://127.0.0.1:50006".to_string(),
            prediction_market_service_addr: "http://127.0.0.1:50008".to_string(),
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
) -> Result<api::user::user_service_client::UserServiceClient<Channel>, tonic::transport::Error> {
    let channel = connect(addr).await?;
    Ok(api::user::user_service_client::UserServiceClient::new(channel))
}

/// 创建 Order Service 客户端
pub async fn create_order_client(
    addr: String,
) -> Result<api::order::order_service_client::OrderServiceClient<Channel>, tonic::transport::Error> {
    let channel = connect(addr).await?;
    Ok(api::order::order_service_client::OrderServiceClient::new(channel))
}

/// 创建 Auth Service 客户端
pub async fn create_auth_client(
    addr: String,
) -> Result<api::auth::auth_service_client::AuthServiceClient<Channel>, tonic::transport::Error> {
    let channel = connect(addr).await?;
    Ok(api::auth::auth_service_client::AuthServiceClient::new(channel))
}

/// 创建 Market Data Service 客户端
pub async fn create_market_data_client(
    addr: String,
) -> Result<api::market_data::market_data_service_client::MarketDataServiceClient<Channel>, tonic::transport::Error> {
    let channel = connect(addr).await?;
    Ok(api::market_data::market_data_service_client::MarketDataServiceClient::new(channel))
}

/// 创建 Prediction Market Service 客户端
pub async fn create_prediction_market_client(
    addr: String,
) -> Result<api::prediction_market::prediction_market_service_client::PredictionMarketServiceClient<Channel>, tonic::transport::Error> {
    let channel = connect(addr).await?;
    Ok(api::prediction_market::prediction_market_service_client::PredictionMarketServiceClient::new(channel))
}
