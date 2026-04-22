//! Market Data Service Main Entry

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Market Data Service...");

    // 加载配置
    let config = market_data_service::Config::load_default();
    tracing::info!("Config loaded: {:?}", config.service);

    // 创建并启动服务器
    let server = market_data_service::MarketDataServer::new(config).await?;
    server.run().await?;

    Ok(())
}