//! API Gateway 主程序

use api_gateway::create_router;
use salvo::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,api_gateway=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting API Gateway on port 8080");

    // 创建路由器
    let router = create_router();

    // 启动服务
    let acceptor = TcpListener::new("0.0.0.0:8080")
        .bind()
        .await;
    Server::new(acceptor).serve(router).await;
}