//! gRPC Server Module

use std::sync::Arc;
use tonic::transport::Server as TonicServer;
use crate::config::Config;
use crate::services::MarketService;
use crate::pb::prediction_market_service_server::PredictionMarketServiceServer;

pub struct PredictionMarketServer {
    config: Arc<Config>,
}

impl PredictionMarketServer {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(config);

        Ok(Self {
            config,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.service.host, self.config.service.port)
            .parse()?;

        tracing::info!("Starting prediction-market-service gRPC on {}", addr);

        // 创建数据库连接池
        let pool = sqlx::PgPool::connect(&format!(
            "postgres://{}:{}@{}:{}/{}",
            self.config.db.username.as_ref().unwrap_or(&"postgres".to_string()),
            self.config.db.password.as_ref().unwrap_or(&"postgres".to_string()),
            self.config.db.host.as_ref().unwrap_or(&"localhost".to_string()),
            self.config.db.port.unwrap_or(5432),
            self.config.db.database.as_ref().unwrap_or(&"prediction_market".to_string())
        )).await?;

        let market_service = MarketService::new(pool);

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(include_bytes!("pb/prediction_market.desc"))
            .build()?;

        TonicServer::builder()
            .add_service(reflection_service)
            .add_service(PredictionMarketServiceServer::new(market_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}