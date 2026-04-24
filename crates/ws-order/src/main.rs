//! WebSocket Order Service
//!
//! 负责实时推送用户订单状态变更

mod server;
mod session;
mod queue_consumer;

use std::sync::Arc;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use queue::Config as QueueConfig;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,ws_order=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .ok();

    tracing::info!("Starting WebSocket Order Service on port 50017");

    // 初始化 queue 配置
    let queue_config = QueueConfig::default();
    let merged_config = queue_config.merge();

    // 创建 session manager
    let session_manager = Arc::new(server::create_session_manager());

    // 创建订单事件处理器
    let topics = vec!["order_events".to_string()];
    let handler = Arc::new(queue_consumer::OrderEventHandler::new(
        session_manager.clone(),
        topics,
    ));

    // 初始化消费者
    if let Err(e) = handler.init(merged_config).await {
        tracing::error!("Failed to init consumer: {}", e);
    }

    // 启动消费者
    let handler_clone = handler.clone();
    tokio::spawn(async move {
        handler_clone.start().await;
    });

    // 启动 WebSocket 服务
    if let Err(e) = server::start("0.0.0.0:50017", session_manager).await {
        tracing::error!("Server error: {}", e);
    }
}
