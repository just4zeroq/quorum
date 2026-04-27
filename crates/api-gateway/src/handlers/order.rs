//! 订单相关处理器

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use crate::grpc::{create_order_client, create_risk_client, create_portfolio_client, GrpcConfig};
use crate::handlers::{parse_json, get_user_id_or_unknown, parse_user_id_i64};

// ========== 请求/响应类型 ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateOrderRequest {
    pub market_id: u64,
    pub outcome_id: u64,
    pub side: String,
    pub order_type: String,
    pub price: String,
    pub quantity: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub market_id: u64,
    pub outcome_id: u64,
    pub side: String,
    pub order_type: String,
    pub price: String,
    pub quantity: String,
    pub filled_quantity: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct GetOrdersQuery {
    pub market_id: Option<u64>,
    pub status: Option<String>,
    pub limit: Option<usize>,
}

// ========== 处理器 ==========

/// 创建订单（三步：风控检查 → 冻结保证金 → Order Service 下单）
#[handler]
pub async fn create_order(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = get_user_id_or_unknown(depot);
    let payload = parse_json::<CreateOrderRequest>(req).await?;

    let config = GrpcConfig::default();
    let is_buy = matches!(payload.side.to_lowercase().as_str(), "buy" | "yes" | "bid");

    // 步骤1: 调用 Risk Service 风控检查
    match create_risk_client(config.risk_service_addr.clone()).await {
        Ok(mut client) => {
            let risk_request = api::risk::CheckRiskRequest {
                user_id: user_id.clone(),
                market_id: payload.market_id,
                outcome_id: payload.outcome_id,
                side: payload.side.clone(),
                order_type: payload.order_type.clone(),
                price: payload.price.clone(),
                quantity: payload.quantity.clone(),
            };

            match client.check_risk(risk_request).await {
                Ok(resp) => {
                    let risk_data = resp.into_inner();
                    if !risk_data.accepted {
                        tracing::warn!("Risk check rejected for user {}: {}", user_id, risk_data.reason);
                        res.status_code(StatusCode::BAD_REQUEST);
                        res.render(Json(serde_json::json!({
                            "success": false,
                            "error": risk_data.reason,
                        })));
                        return Ok(());
                    }
                    tracing::debug!("Risk check passed for user {}", user_id);
                }
                Err(e) => {
                    tracing::warn!("Risk service unavailable, proceeding without risk check: {:?}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to connect to risk service, proceeding without risk check: {:?}", e);
        }
    }

    // 步骤2: Portfolio.Freeze - 冻结保证金
    let freeze_ref = format!("fr_{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(20).collect::<String>());

    let freeze_amount = if is_buy {
        let p: rust_decimal::Decimal = payload.price.parse().unwrap_or_default();
        let q: rust_decimal::Decimal = payload.quantity.parse().unwrap_or_default();
        (p * q).to_string()
    } else {
        payload.quantity.clone()
    };

    let mut freeze_ok = false;
    match create_portfolio_client(config.portfolio_service_addr.clone()).await {
        Ok(mut client) => {
            let asset = if is_buy { "USDC" } else { &payload.side.to_uppercase() };
            let grpc_req = api::portfolio::FreezeRequest {
                user_id: user_id.clone(),
                asset: asset.to_string(),
                amount: freeze_amount.clone(),
                order_id: freeze_ref.clone(),
            };
            match client.freeze(grpc_req).await {
                Ok(_) => {
                    tracing::debug!("Portfolio freeze success for user {}: {} {}", user_id, freeze_amount, asset);
                    freeze_ok = true;
                }
                Err(e) => {
                    tracing::warn!("Portfolio freeze failed for user {}: {:?}", user_id, e);
                    res.status_code(StatusCode::BAD_REQUEST);
                    res.render(Json(serde_json::json!({
                        "success": false,
                        "error": "Insufficient balance"
                    })));
                    return Ok(());
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to connect to portfolio service, proceeding without freeze: {:?}", e);
        }
    }

    // 步骤3: 调用 Order Service gRPC
    let user_id_i64 = parse_user_id_i64(&user_id);
    match create_order_client(config.order_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::order::CreateOrderRequest {
                user_id: user_id_i64,
                market_id: payload.market_id as i64,
                outcome_id: payload.outcome_id as i64,
                side: payload.side.clone(),
                order_type: payload.order_type.clone(),
                price: payload.price.clone(),
                quantity: payload.quantity.clone(),
                client_order_id: String::new(),
            };

            match client.create_order(grpc_request).await {
                Ok(resp) => {
                    let order_data = resp.into_inner();
                    tracing::info!("Order created: {} by user {}", order_data.order_id, user_id);

                    res.status_code(StatusCode::CREATED);
                    res.render(Json(OrderResponse {
                        order_id: order_data.order_id,
                        market_id: payload.market_id,
                        outcome_id: payload.outcome_id,
                        side: payload.side,
                        order_type: payload.order_type,
                        price: payload.price,
                        quantity: payload.quantity,
                        filled_quantity: order_data.order.as_ref().map(|o| o.filled_quantity.clone()).unwrap_or_default(),
                        status: order_data.order.as_ref().map(|o| o.status.clone()).unwrap_or_default(),
                        created_at: chrono::Utc::now().to_rfc3339(),
                    }));
                }
                Err(e) => {
                    // 如果之前 freeze 成功了，需要解冻
                    if freeze_ok {
                        if let Ok(mut client) = create_portfolio_client(config.portfolio_service_addr.clone()).await {
                            let unfreeze_req = api::portfolio::UnfreezeRequest {
                                user_id: user_id.clone(),
                                asset: if is_buy { "USDC".to_string() } else { payload.side.to_uppercase() },
                                amount: freeze_amount.clone(),
                                order_id: freeze_ref.clone(),
                            };
                            let _ = client.unfreeze(unfreeze_req).await;
                        }
                    }

                    tracing::error!("Order service create_order failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to create order: {:?}", e)
                    })));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to order service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({
                "success": false,
                "error": "Order service unavailable"
            })));
        }
    }

    Ok(())
}

/// 获取订单详情
#[handler]
pub async fn get_order(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let order_id = req.param::<String>("order_id").unwrap_or_default();

    let config = GrpcConfig::default();
    match create_order_client(config.order_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::order::GetOrderRequest {
                order_id: order_id.clone(),
            };

            match client.get_order(grpc_request).await {
                Ok(resp) => {
                    let order_data = resp.into_inner().order;
                    if let Some(order) = order_data {
                        res.render(Json(OrderResponse {
                            order_id: order.id,
                            market_id: order.market_id as u64,
                            outcome_id: order.outcome_id as u64,
                            side: order.side,
                            order_type: order.order_type,
                            price: order.price,
                            quantity: order.quantity,
                            filled_quantity: order.filled_quantity,
                            status: order.status,
                            created_at: chrono::DateTime::from_timestamp(order.created_at, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_default(),
                        }));
                    } else {
                        res.status_code(StatusCode::NOT_FOUND);
                        res.render(Json(serde_json::json!({ "error": "Order not found" })));
                    }
                }
                Err(e) => {
                    tracing::error!("Order service get_order failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({ "error": format!("Failed to get order: {:?}", e) })));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to order service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({ "error": "Order service unavailable" })));
        }
    }

    Ok(())
}

/// 取消订单
#[handler]
pub async fn cancel_order(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = get_user_id_or_unknown(depot);
    let order_id = req.param::<String>("order_id").unwrap_or_default();
    let user_id_i64 = parse_user_id_i64(&user_id);

    let config = GrpcConfig::default();
    match create_order_client(config.order_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::order::CancelOrderRequest {
                order_id: order_id.clone(),
                user_id: user_id_i64,
            };

            match client.cancel_order(grpc_request).await {
                Ok(resp) => {
                    let resp_data = resp.into_inner();
                    tracing::info!("Order cancelled: {} by user {}", order_id, user_id);
                    res.render(Json(serde_json::json!({
                        "success": resp_data.success,
                        "order_id": order_id,
                        "status": if resp_data.success { "cancelled" } else { "failed" },
                        "message": resp_data.message
                    })));
                }
                Err(e) => {
                    tracing::error!("Order service cancel_order failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to cancel order: {:?}", e)
                    })));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to order service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({
                "success": false,
                "error": "Order service unavailable"
            })));
        }
    }

    Ok(())
}

/// 获取用户订单列表
#[handler]
pub async fn get_orders(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = get_user_id_or_unknown(depot);
    let query = req.parse_queries::<GetOrdersQuery>().unwrap_or_default();
    let user_id_i64 = parse_user_id_i64(&user_id);

    let config = GrpcConfig::default();
    match create_order_client(config.order_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::order::GetUserOrdersRequest {
                user_id: user_id_i64,
                market_id: query.market_id.map(|m| m as i64).unwrap_or(0),
                status: query.status.unwrap_or_default(),
                page: 1,
                page_size: query.limit.unwrap_or(50) as i32,
            };

            match client.get_user_orders(grpc_request).await {
                Ok(resp) => {
                    let orders_data = resp.into_inner();
                    let orders: Vec<OrderResponse> = orders_data.orders.into_iter().map(|o| OrderResponse {
                        order_id: o.id,
                        market_id: o.market_id as u64,
                        outcome_id: o.outcome_id as u64,
                        side: o.side,
                        order_type: o.order_type,
                        price: o.price,
                        quantity: o.quantity,
                        filled_quantity: o.filled_quantity,
                        status: o.status,
                        created_at: chrono::DateTime::from_timestamp(o.created_at, 0)
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_default(),
                    }).collect();

                    res.render(Json(orders));
                }
                Err(e) => {
                    tracing::error!("Order service get_orders failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({ "error": format!("Failed to get orders: {:?}", e) })));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to order service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({ "error": "Order service unavailable" })));
        }
    }

    Ok(())
}
