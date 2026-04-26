//! gRPC Server Module

use std::sync::Arc;
use tonic::transport::Server as TonicServer;
use crate::config::Config;
use crate::services::MarketService;
use api::prediction_market::prediction_market_service_server::PredictionMarketServiceServer;
use queue::{ProducerManager, Config as QueueConfig};

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

        // 创建 Portfolio Service gRPC 客户端连接
        let portfolio_channel = tonic::transport::Channel::from_shared(self.config.portfolio_service_addr.clone())
            .map_err(|e| format!("Invalid portfolio service address: {}", e))?
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to portfolio service: {}", e))?;
        let portfolio_client =
            api::portfolio::portfolio_service_client::PortfolioServiceClient::new(portfolio_channel);

        // 初始化 queue producer for market events
        let queue_config = QueueConfig::default();
        let merged_config = queue_config.merge();
        let event_producer = ProducerManager::new(merged_config);
        event_producer.init().await.map_err(|e| format!("Failed to init event producer: {}", e))?;

        let market_service = MarketService::new(pool, portfolio_client)
            .with_event_producer(event_producer);

        TonicServer::builder()
            .add_service(PredictionMarketServiceServer::new(market_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}