//! User Service 主程序

use user_service::{Config, Server};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,user_service=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting User Service...");

    // 加载配置
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "crates/user-service/config/user-service.yaml".to_string());

    let config = match Config::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to load config from {}, using default: {}", config_path, e);
            Config::default()
        }
    };

    // 创建并运行服务
    let server = Server::new(config).await?;
    server.run().await?;

    Ok(())
}