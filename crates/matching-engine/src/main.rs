//! Matching Engine Service
//!
//! 独立的撮合引擎服务，通过 gRPC 和消息队列与其他服务通信

mod server;

// 导入库模块
pub mod api;
pub mod core;
pub mod utils;
pub mod example;
pub mod event_emitter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .init();

    tracing::info!("Starting Matching Engine Service...");

    // 启动 gRPC + Queue 服务
    server::start().await
}
