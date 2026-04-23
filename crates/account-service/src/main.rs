//! Account Service 主程序
//!
//! gRPC 服务入口

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,account_service=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = if let Ok(config) = account_service::Config::load("config/account_service.yaml") {
        config
    } else {
        tracing::warn!("Failed to load config, using default");
        account_service::Config::load_default()
    };

    tracing::info!("Starting Account Service...");
    tracing::info!("Service config: {:?}", config.service);
    tracing::info!("Database config: db_type={:?}", config.database.db_type);

    // 创建并启动服务
    let server = account_service::AccountServer::new(config);
    server.run().await?;

    Ok(())
}