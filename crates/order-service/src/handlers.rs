//! Order Service HTTP 处理器

use axum::{extract::Path, Json};
use common::*;
use std::sync::Arc;

use crate::OrderManager;

pub type AppState = Arc<OrderManager>;

pub async fn create_order(
    State(state): State<AppState>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<Order>, Error> {
    let order = Order::new(
        req.user_id,
        req.symbol,
        req.side,
        req.order_type,
        req.price,
        req.quantity,
    );
    let order = state.create_order(order)?;
    Ok(Json(order))
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<Order>, Error> {
    state.get_order(&order_id)
}

pub async fn cancel_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<Order>, Error> {
    state.cancel_order(&order_id)
}

#[derive(serde::Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: String,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
}