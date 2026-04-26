//! gRPC Server Module

use std::sync::Arc;
use tonic::transport::Server as TonicServer;
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