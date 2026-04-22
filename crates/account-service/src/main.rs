//! Account Service 主程序

use account_service::create_account_manager;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use account_service::handlers::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,account_service=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Account Service on port 8083");

    let state = create_account_manager();

    // 创建测试账户
    state.create_test_account("user1", "USDT", Decimal::new(100000, 0));
    state.create_test_account("user1", "BTC", Decimal::new(10, 0));
    state.create_test_account("user2", "USDT", Decimal::new(50000, 0));
    state.create_test_account("user2", "ETH", Decimal::new(100, 0));

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/account/balance", get(get_balance))
        .route("/api/v1/account/freeze", post(freeze))
        .route("/api/v1/account/unfreeze", post(unfreeze))
        .route("/api/v1/account/deduct", post(deduct))
        .route("/api/v1/account/credit", post(credit))
        .route("/api/v1/account/transfer", post(transfer))
        .route("/api/v1/account/update-equity", post(update_equity))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8083").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "account-service",
    }))
}