//! gRPC Server Module

use std::sync::Arc;
use tonic::transport::Server as TonicServer;
use registry::ServiceRegistry;
use crate::config::Config;
use crate::services::MarketDataServiceImpl;
use api::market_data::market_data_service_server::MarketDataServiceServer;

pub struct MarketDataServer {
    config: Arc<Config>,
}

impl MarketDataServer {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(config);
        Ok(Self { config })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.service.host, self.config.service.port)
            .parse()?;

        tracing::info!("Starting market-data-service gRPC on {}", addr);

        // 注册到 etcd
        let registry = ServiceRegistry::new(
            "market-data-service",
            &format!("http://{}", addr),
            &self.config.etcd_endpoints,
        ).await?;

        registry.register(30).await?;
        let _heartbeat_handle = registry.clone().start_heartbeat(30, 10);

        tracing::info!("Market data service registered to etcd");

        // 创建数据库连接池
        let pool = sqlx::PgPool::connect(&format!(
            "postgres://{}:{}@{}:{}/{}",
            self.config.db.username.as_ref().unwrap_or(&"postgres".to_string()),
            self.config.db.password.as_ref().unwrap_or(&"postgres".to_string()),
            self.config.db.host.as_ref().unwrap_or(&"localhost".to_string()),
            self.config.db.port.unwrap_or(5432),
            self.config.db.database.as_ref().unwrap_or(&"market_data".to_string())
        )).await?;

        let market_data_service = MarketDataServiceImpl::new(pool);

        TonicServer::builder()
            .add_service(MarketDataServiceServer::new(market_data_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}