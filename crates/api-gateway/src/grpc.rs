//! gRPC 客户端模块
//!
//! 提供对后端微服务的 gRPC 调用能力，使用 etcd 服务发现实现动态路由

use std::collections::HashMap;
use registry::{ServiceDiscovery, ServiceInstance, RegistryError};
use tokio::sync::RwLock;
use std::sync::Arc;

/// GrpcClientManager 使用 etcd 服务发现管理所有 gRPC 客户端
///
/// 通过 ServiceDiscovery 动态获取服务实例地址，无需硬编码服务地址
pub struct GrpcClientManager {
    /// 每个服务名称对应的服务发现实例
    discoveries: HashMap<String, ServiceDiscovery>,
    /// etcd 端点列表
    etcd_endpoints: Vec<String>,
}

impl GrpcClientManager {
    /// 创建新的 GrpcClientManager
    ///
    /// 初始化所有服务的 ServiceDiscovery 实例
    pub async fn new(etcd_endpoints: Vec<String>) -> Result<Self, RegistryError> {
        let service_names = vec![
            "user-service",
            "wallet-service",
            "order-service",
            "portfolio-service",
            "risk-service",
            "market-data-service",
            "prediction-market-service",
        ];

        let mut discoveries = HashMap::new();
        for name in service_names {
            discoveries.insert(
                name.to_string(),
                ServiceDiscovery::new(name, &etcd_endpoints).await?,
            );
        }

        Ok(Self {
            discoveries,
            etcd_endpoints,
        })
    }

    /// 从服务发现中获取第一个可用实例的地址
    async fn get_instance_addr(&self, service_name: &str) -> Result<String, RegistryError> {
        let instances = self.discoveries
            .get(service_name)
            .ok_or_else(|| RegistryError::NotFound(service_name.to_string()))?
            .get_services()
            .await?;

        if instances.is_empty() {
            return Err(RegistryError::NotFound(format!("{} has no instances", service_name)));
        }

        Ok(instances[0].addr.clone())
    }

    /// 获取 User Service 客户端
    pub async fn get_user_client(&self) -> Result<api::UserServiceClient<tonic::transport::Channel>, RegistryError> {
        let addr = self.get_instance_addr("user-service").await?;
        api::user::create_user_client(&addr).await
            .map_err(|e| RegistryError::Etcd(e.to_string()))
    }

    /// 获取 Order Service 客户端
    pub async fn get_order_client(&self) -> Result<api::OrderServiceClient<tonic::transport::Channel>, RegistryError> {
        let addr = self.get_instance_addr("order-service").await?;
        api::order::create_order_client(&addr).await
            .map_err(|e| RegistryError::Etcd(e.to_string()))
    }

    /// 获取 Auth Service 客户端
    pub async fn get_auth_client(&self) -> Result<api::AuthServiceClient<tonic::transport::Channel>, RegistryError> {
        let addr = self.get_instance_addr("auth-service").await?;
        api::auth::create_auth_client(&addr).await
            .map_err(|e| RegistryError::Etcd(e.to_string()))
    }

    /// 获取 Portfolio Service 客户端
    pub async fn get_portfolio_client(&self) -> Result<api::PortfolioServiceClient<tonic::transport::Channel>, RegistryError> {
        let addr = self.get_instance_addr("portfolio-service").await?;
        api::portfolio::create_portfolio_client(&addr).await
            .map_err(|e| RegistryError::Etcd(e.to_string()))
    }

    /// 获取 Risk Service 客户端
    pub async fn get_risk_client(&self) -> Result<api::RiskServiceClient<tonic::transport::Channel>, RegistryError> {
        let addr = self.get_instance_addr("risk-service").await?;
        api::risk::create_risk_client(&addr).await
            .map_err(|e| RegistryError::Etcd(e.to_string()))
    }

    /// 获取 Market Data Service 客户端
    pub async fn get_market_data_client(&self) -> Result<api::MarketDataServiceClient<tonic::transport::Channel>, RegistryError> {
        let addr = self.get_instance_addr("market-data-service").await?;
        api::market_data::create_market_data_client(&addr).await
            .map_err(|e| RegistryError::Etcd(e.to_string()))
    }

    /// 获取 Prediction Market Service 客户端
    pub async fn get_prediction_market_client(&self) -> Result<api::PredictionMarketServiceClient<tonic::transport::Channel>, RegistryError> {
        let addr = self.get_instance_addr("prediction-market-service").await?;
        api::prediction_market::create_prediction_market_client(&addr).await
            .map_err(|e| RegistryError::Etcd(e.to_string()))
    }

    /// 获取 Wallet Service 客户端
    pub async fn get_wallet_client(&self) -> Result<api::WalletServiceClient<tonic::transport::Channel>, RegistryError> {
        let addr = self.get_instance_addr("wallet-service").await?;
        api::wallet::create_wallet_client(&addr).await
            .map_err(|e| RegistryError::Etcd(e.to_string()))
    }
}

/// 连接到 gRPC 服务（保留用于直接连接场景）
pub async fn connect(addr: String) -> Result<tonic::transport::Channel, tonic::transport::Error> {
    let endpoint = tonic::transport::Endpoint::new(addr)?;
    endpoint.timeout(std::time::Duration::from_secs(10)).connect().await
}