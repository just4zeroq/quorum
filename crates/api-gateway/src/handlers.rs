//! API 处理器

use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grpc::{create_user_client, create_order_client, create_auth_client, create_portfolio_client, create_risk_client, create_wallet_client, GrpcConfig};

/// 健康检查
#[handler]
pub async fn health_check(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "ok",
        "service": "api-gateway",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })));
}

/// 就绪检查
#[handler]
pub async fn ready_check(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "ready",
        "service": "api-gateway",
    })));
}

// ========== 用户相关 ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub user_id: Option<String>,
    pub message: String,
}

#[handler]
pub async fn register(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let payload = req.parse_json::<RegisterRequest>().await
        .map_err(|e| {
            tracing::error!("Failed to parse register request: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // 调用 User Service gRPC 注册
    let config = GrpcConfig::default();
    match create_user_client(config.user_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::user::RegisterRequest {
                username: payload.username.clone(),
                email: payload.email.clone(),
                password: payload.password,
                invite_code: String::new(),
                ip_address: String::new(),
                user_agent: String::new(),
            };

            match client.register(grpc_request).await {
                Ok(resp) => {
                    let user_data = resp.into_inner();
                    tracing::info!("User registered: {}", payload.email);

                    res.render(Json(RegisterResponse {
                        success: true,
                        user_id: Some(user_data.user_id),
                        message: "Registration successful. Please login.".to_string(),
                    }));
                }
                Err(e) => {
                    tracing::error!("User service register failed: {:?}", e);
                    res.status_code(StatusCode::BAD_REQUEST);
                    res.render(Json(RegisterResponse {
                        success: false,
                        user_id: None,
                        message: format!("Registration failed: {:?}", e),
                    }));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to user service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(RegisterResponse {
                success: false,
                user_id: None,
                message: "Service unavailable".to_string(),
            }));
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: Option<String>,
    pub user_id: Option<String>,
    pub message: Option<String>,
}

#[handler]
pub async fn login(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let payload = req.parse_json::<LoginRequest>().await
        .map_err(|e| {
            tracing::error!("Failed to parse login request: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // 步骤1: 调用 User Service 验证邮箱/密码并获取 user_id
    let config = GrpcConfig::default();
    let user_id = match create_user_client(config.user_service_addr.clone()).await {
        Ok(mut client) => {
            let grpc_request = api::user::LoginRequest {
                email: payload.email.clone(),
                password: payload.password,
                code_2fa: String::new(),
                ip_address: String::new(),
                user_agent: String::new(),
                device_id: String::new(),
            };

            match client.login(grpc_request).await {
                Ok(resp) => {
                    let user_data = resp.into_inner();
                    tracing::info!("User verified: {}", payload.email);
                    user_data.user_id
                }
                Err(e) => {
                    tracing::error!("User service login failed: {:?}", e);
                    res.status_code(StatusCode::UNAUTHORIZED);
                    res.render(Json(LoginResponse {
                        success: false,
                        token: None,
                        refresh_token: None,
                        expires_in: None,
                        token_type: None,
                        user_id: None,
                        message: Some("Invalid credentials".to_string()),
                    }));
                    return Ok(());
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to user service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(LoginResponse {
                success: false,
                token: None,
                refresh_token: None,
                expires_in: None,
                token_type: None,
                user_id: None,
                message: Some("Service unavailable".to_string()),
            }));
            return Ok(());
        }
    };

    // 步骤2: 调用 Auth Service 生成 JWT Token
    match create_auth_client(config.auth_service_addr).await {
        Ok(mut client) => {
            let auth_request = api::auth::LoginRequest {
                user_id: user_id.clone(),
                password: String::new(), // Auth Service MVP 不校验密码，由 User Service 校验
                device_id: String::new(),
                ip_address: String::new(),
                user_agent: String::new(),
            };

            match client.login(auth_request).await {
                Ok(resp) => {
                    let auth_data = resp.into_inner();
                    tracing::info!("Auth tokens issued for user: {}", user_id);

                    res.render(Json(LoginResponse {
                        success: true,
                        token: Some(auth_data.access_token),
                        refresh_token: Some(auth_data.refresh_token),
                        expires_in: Some(auth_data.expires_in),
                        token_type: Some(auth_data.token_type),
                        user_id: Some(user_id),
                        message: None,
                    }));
                }
                Err(e) => {
                    tracing::error!("Auth service login failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(LoginResponse {
                        success: false,
                        token: None,
                        refresh_token: None,
                        expires_in: None,
                        token_type: None,
                        user_id: None,
                        message: Some("Authentication service error".to_string()),
                    }));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to auth service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(LoginResponse {
                success: false,
                token: None,
                refresh_token: None,
                expires_in: None,
                token_type: None,
                user_id: None,
                message: Some("Auth service unavailable".to_string()),
            }));
        }
    }

    Ok(())
}

// ========== 认证相关 (Auth Service) ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

#[handler]
pub async fn logout(_req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let session_id = match depot.get::<String>("session_id") {
        Ok(sid) => sid.clone(),
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(LogoutResponse {
                success: false,
                message: "Unauthorized".to_string(),
            }));
            return Ok(());
        }
    };

    let config = GrpcConfig::default();
    match create_auth_client(config.auth_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::auth::LogoutRequest { session_id: session_id.clone() };
            match client.logout(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    res.render(Json(LogoutResponse {
                        success: data.success,
                        message: if data.success { "Logged out successfully".to_string() } else { "Logout failed".to_string() },
                    }));
                }
                Err(e) => {
                    tracing::error!("Auth service logout failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(LogoutResponse {
                        success: false,
                        message: "Auth service error".to_string(),
                    }));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to auth service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(LogoutResponse {
                success: false,
                message: "Auth service unavailable".to_string(),
            }));
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub success: bool,
    pub access_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: Option<String>,
    pub message: Option<String>,
}

#[handler]
pub async fn refresh_token(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let payload = req.parse_json::<RefreshTokenRequest>().await
        .map_err(|e| {
            tracing::error!("Failed to parse refresh request: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let config = GrpcConfig::default();
    match create_auth_client(config.auth_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::auth::RefreshTokenRequest {
                refresh_token: payload.refresh_token,
            };

            match client.refresh_token(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    res.render(Json(RefreshTokenResponse {
                        success: true,
                        access_token: Some(data.access_token),
                        expires_in: Some(data.expires_in),
                        token_type: Some("Bearer".to_string()),
                        message: None,
                    }));
                }
                Err(e) => {
                    tracing::error!("Auth service refresh_token failed: {:?}", e);
                    res.status_code(StatusCode::UNAUTHORIZED);
                    res.render(Json(RefreshTokenResponse {
                        success: false,
                        access_token: None,
                        expires_in: None,
                        token_type: None,
                        message: Some("Invalid or expired refresh token".to_string()),
                    }));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to auth service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(RefreshTokenResponse {
                success: false,
                access_token: None,
                expires_in: None,
                token_type: None,
                message: Some("Auth service unavailable".to_string()),
            }));
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[handler]
pub async fn get_current_user(_req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = match depot.get::<String>("user_id") {
        Ok(uid) => uid.clone(),
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "success": false,
                "error": "Unauthorized"
            })));
            return Ok(());
        }
    };

    // 调用 User Service gRPC 获取用户信息
    let config = GrpcConfig::default();
    match create_user_client(config.user_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::user::GetUserRequest {
                user_id: user_id.clone(),
            };

            match client.get_user(grpc_request).await {
                Ok(resp) => {
                    let user_data = resp.into_inner().user;
                    if let Some(user) = user_data {
                        res.render(Json(UserResponse {
                            id: user.id,
                            username: user.username,
                            email: user.email,
                        }));
                    } else {
                        res.status_code(StatusCode::NOT_FOUND);
                        res.render(Json(serde_json::json!({
                            "error": "User not found"
                        })));
                    }
                }
                Err(e) => {
                    tracing::error!("User service get_user failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({
                        "error": format!("Failed to get user: {:?}", e)
                    })));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to user service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({
                "error": "User service unavailable"
            })));
        }
    }

    Ok(())
}

// ========== 订单相关 ==========

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

#[handler]
pub async fn create_order(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let payload = req.parse_json::<CreateOrderRequest>().await
        .map_err(|e| {
            tracing::error!("Failed to parse order request: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // 步骤1: 调用 Risk Service 风控检查
    let config = GrpcConfig::default();
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
                    // 风险服务不可用时，继续下单（熔断降级）
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to connect to risk service, proceeding without risk check: {:?}", e);
            // 熔断降级
        }
    }

    // 步骤2: 调用 Order Service gRPC
    let user_id_i64: i64 = user_id.replace("usr_", "").parse().unwrap_or(0);
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

#[handler]
pub async fn get_order(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let order_id = req.param::<String>("order_id")
        .unwrap_or_default();

    // 调用 Order Service gRPC
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
                        res.render(Json(serde_json::json!({
                            "error": "Order not found"
                        })));
                    }
                }
                Err(e) => {
                    tracing::error!("Order service get_order failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({
                        "error": format!("Failed to get order: {:?}", e)
                    })));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to order service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({
                "error": "Order service unavailable"
            })));
        }
    }

    Ok(())
}

#[handler]
pub async fn cancel_order(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let order_id = req.param::<String>("order_id")
        .unwrap_or_default();

    // 解析 user_id 为 i64
    let user_id_i64: i64 = user_id.replace("usr_", "").parse().unwrap_or(0);

    // 调用 Order Service gRPC 取消订单
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

#[derive(Debug, Deserialize, Default)]
pub struct GetOrdersQuery {
    pub market_id: Option<u64>,
    pub status: Option<String>,
    pub limit: Option<usize>,
}

#[handler]
pub async fn get_orders(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let query = req.parse_queries::<GetOrdersQuery>().unwrap_or_default();
    let user_id_i64: i64 = user_id.replace("usr_", "").parse().unwrap_or(0);

    // 调用 Order Service gRPC
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
                    let orders: Vec<OrderResponse> = orders_data.orders.into_iter().map(|o| {
                        OrderResponse {
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
                        }
                    }).collect();

                    res.render(Json(orders));
                }
                Err(e) => {
                    tracing::error!("Order service get_orders failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({
                        "error": format!("Failed to get orders: {:?}", e)
                    })));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to order service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({
                "error": "Order service unavailable"
            })));
        }
    }

    Ok(())
}

// ========== 账户相关 ==========

#[derive(Debug, Deserialize, Default)]
pub struct GetBalanceQuery {
    pub asset: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub account_id: String,
    pub asset: String,
    pub available: String,
    pub frozen: String,
}

#[handler]
pub async fn get_balance(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let query = req.parse_queries::<GetBalanceQuery>().unwrap_or_default();
    let asset = query.asset.unwrap_or_else(|| "USDC".to_string());

    let config = GrpcConfig::default();
    match create_portfolio_client(config.portfolio_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::portfolio::GetBalanceRequest {
                user_id: user_id.clone(),
                asset: asset.clone(),
            };

            match client.get_balance(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    res.render(Json(BalanceResponse {
                        account_id: data.account_id,
                        asset: data.asset,
                        available: data.available,
                        frozen: data.frozen,
                    }));
                }
                Err(e) => {
                    tracing::error!("Portfolio service get_balance failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to portfolio service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Portfolio service unavailable"})));
        }
    }

    Ok(())
}

// ========== 持仓相关 ==========

#[derive(Debug, Deserialize, Default)]
pub struct GetPositionsQuery {
    pub market_id: Option<u64>,
}

#[handler]
pub async fn get_positions(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let query = req.parse_queries::<GetPositionsQuery>().unwrap_or_default();
    let market_id = query.market_id.unwrap_or(0);

    let config = GrpcConfig::default();
    match create_portfolio_client(config.portfolio_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::portfolio::GetPositionsRequest {
                user_id: user_id.clone(),
                market_id,
            };

            match client.get_positions(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    let positions: Vec<serde_json::Value> = data.positions.into_iter().map(|p| {
                        serde_json::json!({
                            "id": p.id,
                            "market_id": p.market_id,
                            "outcome_id": p.outcome_id,
                            "side": p.side,
                            "size": p.size,
                            "entry_price": p.entry_price,
                        })
                    }).collect();
                    res.render(Json(positions));
                }
                Err(e) => {
                    tracing::error!("Portfolio service get_positions failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to portfolio service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Portfolio service unavailable"})));
        }
    }

    Ok(())
}

// ========== 行情相关 ==========

#[derive(Debug, Deserialize, Default)]
pub struct DepthQuery {
    pub market_id: Option<u64>,
    pub depth: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct DepthResponse {
    pub asks: Vec<Vec<String>>,
    pub bids: Vec<Vec<String>>,
}

#[handler]
pub async fn get_depth(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<DepthQuery>().unwrap_or_default();
    let market_id = query.market_id.unwrap_or(1) as i64;
    let depth = query.depth.unwrap_or(20);

    let config = crate::grpc::GrpcConfig::default();
    match crate::grpc::create_market_data_client(config.market_data_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::market_data::GetOrderBookRequest {
                market_id,
                depth,
            };

            match client.get_order_book(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    let asks: Vec<Vec<String>> = data.asks.iter().map(|a| vec![a.price.clone(), a.quantity.clone()]).collect();
                    let bids: Vec<Vec<String>> = data.bids.iter().map(|b| vec![b.price.clone(), b.quantity.clone()]).collect();
                    res.render(Json(DepthResponse { asks, bids }));
                }
                Err(e) => {
                    tracing::error!("Market data get_order_book failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to market data service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Market data service unavailable"})));
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Default)]
pub struct TickerQuery {
    pub market_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TickerResponse {
    pub market_id: u64,
    pub volume_24h: String,
    pub amount_24h: String,
    pub high_24h: String,
    pub low_5h: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub trade_count_24h: i64,
    pub timestamp: i64,
}

#[handler]
pub async fn get_ticker(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<TickerQuery>().unwrap_or_default();
    let market_id = query.market_id.unwrap_or(1) as i64;

    let config = crate::grpc::GrpcConfig::default();
    match crate::grpc::create_market_data_client(config.market_data_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::market_data::Get24hStatsRequest {
                market_id,
            };

            match client.get24h_stats(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    res.render(Json(TickerResponse {
                        market_id: market_id as u64,
                        volume_24h: data.volume_24h,
                        amount_24h: data.amount_24h,
                        high_24h: data.high_24h,
                        low_5h: data.low_5h,
                        price_change: data.price_change,
                        price_change_percent: data.price_change_percent,
                        trade_count_24h: data.trade_count_24h,
                        timestamp: data.timestamp,
                    }));
                }
                Err(e) => {
                    tracing::error!("Market data get24h_stats failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to market data service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Market data service unavailable"})));
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Default)]
pub struct KlineQuery {
    pub market_id: Option<u64>,
    pub interval: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct KlineResponse {
    pub timestamp: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

#[handler]
pub async fn get_kline(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<KlineQuery>().unwrap_or_default();
    let market_id = query.market_id.unwrap_or(1) as i64;
    let interval = query.interval.unwrap_or_else(|| "1h".to_string());
    let start_time = query.start_time.unwrap_or(0);
    let end_time = query.end_time.unwrap_or(0);
    let limit = query.limit.unwrap_or(100);

    let config = crate::grpc::GrpcConfig::default();
    match crate::grpc::create_market_data_client(config.market_data_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::market_data::GetKlinesRequest {
                market_id,
                interval,
                start_time,
                end_time,
                limit,
            };

            match client.get_klines(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    let klines: Vec<KlineResponse> = data.klines.into_iter().map(|k| KlineResponse {
                        timestamp: k.timestamp,
                        open: k.open,
                        high: k.high,
                        low: k.low,
                        close: k.close,
                        volume: k.volume,
                    }).collect();
                    res.render(Json(klines));
                }
                Err(e) => {
                    tracing::error!("Market data get_klines failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to market data service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Market data service unavailable"})));
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Default)]
pub struct TradesQuery {
    pub market_id: Option<u64>,
    pub outcome_id: Option<u64>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TradeResponse {
    pub trade_id: i64,
    pub market_id: i64,
    pub outcome_id: i64,
    pub price: String,
    pub quantity: String,
    pub side: String,
    pub timestamp: i64,
}

#[handler]
pub async fn get_recent_trades(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<TradesQuery>().unwrap_or_default();
    let market_id = query.market_id.unwrap_or(1) as i64;
    let outcome_id = query.outcome_id.unwrap_or(0) as i64;
    let limit = query.limit.unwrap_or(50);

    let config = crate::grpc::GrpcConfig::default();
    match crate::grpc::create_market_data_client(config.market_data_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::market_data::GetRecentTradesRequest {
                market_id,
                outcome_id,
                limit,
                from_trade_id: 0,
            };

            match client.get_recent_trades(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    let trades: Vec<TradeResponse> = data.trades.into_iter().map(|t| TradeResponse {
                        trade_id: t.id,
                        market_id: t.market_id,
                        outcome_id: t.outcome_id,
                        price: t.price,
                        quantity: t.quantity,
                        side: t.side,
                        timestamp: t.timestamp,
                    }).collect();
                    res.render(Json(trades));
                }
                Err(e) => {
                    tracing::error!("Market data get_recent_trades failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to market data service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Market data service unavailable"})));
        }
    }

    Ok(())
}

// ========== 钱包相关 ==========

#[derive(Debug, Deserialize, Default)]
pub struct DepositAddressQuery {
    pub asset: String,
    pub network: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DepositAddressResponse {
    pub address: String,
    pub asset: String,
    pub network: String,
    pub memo: Option<String>,
}

#[handler]
pub async fn get_deposit_address(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let _user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let query = req.parse_queries::<DepositAddressQuery>().unwrap_or_default();
    let asset = query.asset.to_uppercase();
    let network = if let Some(ref net) = query.network {
        net.clone()
    } else {
        match asset.as_str() {
            "USDT" => "TRC20".to_string(),
            "TRX" => "TRC20".to_string(),
            "BTC" => "BTC".to_string(),
            _ => "ERC20".to_string(),
        }
    };

    // TODO: 调用 Wallet Service gRPC
    // 模拟生成充值地址
    let address = match asset.as_str() {
        "USDT" | "TRX" => format!("TN{}W", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(32).collect::<String>()),
        "BTC" => format!("bc1q{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(38).collect::<String>()),
        _ => format!("0x{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(40).collect::<String>()),
    };

    let memo = if network == "TRC20" {
        Some(uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(8).collect())
    } else {
        None
    };

    res.render(Json(DepositAddressResponse {
        address,
        asset,
        network,
        memo,
    }));

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WithdrawRequest {
    pub asset: String,
    pub amount: String,
    pub address: String,
    pub network: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WithdrawResponse {
    pub success: bool,
    pub withdrawal_id: String,
    pub asset: String,
    pub amount: String,
    pub fee: String,
    pub net_amount: String,
    pub status: String,
}

#[handler]
pub async fn withdraw(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let payload = req.parse_json::<WithdrawRequest>().await
        .map_err(|e| {
            tracing::error!("Failed to parse withdraw request: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let config = crate::grpc::GrpcConfig::default();
    match create_wallet_client(config.wallet_service_addr).await {
        Ok(mut client) => {
            let grpc_req = api::wallet::CreateWithdrawRequest {
                user_id: user_id.parse::<i64>().unwrap_or(0),
                asset: payload.asset.clone(),
                amount: payload.amount.clone(),
                to_address: payload.address.clone(),
                chain: payload.network.clone().unwrap_or_else(|| "ETH".to_string()),
                payment_password: String::new(),
            };

            match client.create_withdraw(grpc_req).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    let amt = payload.amount.clone();
                    res.render(Json(WithdrawResponse {
                        success: data.success,
                        withdrawal_id: data.withdraw_id,
                        asset: payload.asset.to_uppercase(),
                        amount: payload.amount,
                        fee: "0.001".to_string(),
                        net_amount: amt,
                        status: "pending".to_string(),
                    }));
                }
                Err(e) => {
                    tracing::error!("Wallet withdraw failed: {:?}", e);
                    if e.code() == tonic::Code::FailedPrecondition {
                        res.status_code(StatusCode::BAD_REQUEST);
                        res.render(Json(serde_json::json!({"error": "Insufficient balance"})));
                    } else {
                        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                        res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to wallet service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Wallet service unavailable"})));
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Default)]
pub struct WalletHistoryQuery {
    pub asset: Option<String>,
    pub r#type: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct WalletHistoryResponse {
    pub deposits: Vec<DepositRecord>,
    pub withdrawals: Vec<WithdrawRecord>,
}

#[derive(Debug, Serialize)]
pub struct DepositRecord {
    pub id: String,
    pub asset: String,
    pub amount: String,
    pub address: String,
    pub tx_hash: String,
    pub confirmations: u32,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct WithdrawRecord {
    pub id: String,
    pub asset: String,
    pub amount: String,
    pub fee: String,
    pub address: String,
    pub tx_hash: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[handler]
pub async fn get_wallet_history(_req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id_str = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let user_id = user_id_str.parse::<i64>().unwrap_or(0);
    if user_id == 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({"error": "Invalid user"})));
        return Ok(());
    }

    let config = crate::grpc::GrpcConfig::default();
    match create_wallet_client(config.wallet_service_addr).await {
        Ok(mut client) => {
            // Get deposit history
            let deposit_req = api::wallet::GetDepositHistoryRequest {
                user_id,
                chain: String::new(),
                page: 1,
                page_size: 20,
            };
            let deposits = match client.get_deposit_history(deposit_req).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    data.deposits.into_iter().map(|d| DepositRecord {
                        id: d.tx_id.clone(),
                        asset: d.chain.clone(),
                        amount: d.amount,
                        address: d.address,
                        tx_hash: d.tx_id,
                        confirmations: 0,
                        status: "completed".to_string(),
                        created_at: d.created_at.to_string(),
                    }).collect::<Vec<_>>()
                }
                Err(e) => {
                    tracing::error!("Failed to get deposit history: {:?}", e);
                    Vec::new()
                }
            };

            // Get withdraw history
            let withdraw_req = api::wallet::GetWithdrawHistoryRequest {
                user_id,
                page: 1,
                page_size: 20,
            };
            let withdrawals = match client.get_withdraw_history(withdraw_req).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    data.withdrawals.into_iter().map(|w| WithdrawRecord {
                        id: w.withdraw_id,
                        asset: w.asset,
                        amount: w.amount,
                        fee: w.fee,
                        address: w.to_address,
                        tx_hash: if w.tx_id.is_empty() { None } else { Some(w.tx_id) },
                        status: w.status,
                        created_at: w.created_at.to_string(),
                    }).collect::<Vec<_>>()
                }
                Err(e) => {
                    tracing::error!("Failed to get withdraw history: {:?}", e);
                    Vec::new()
                }
            };

            res.render(Json(WalletHistoryResponse {
                deposits,
                withdrawals,
            }));
        }
        Err(e) => {
            tracing::error!("Failed to connect to wallet service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Wallet service unavailable"})));
        }
    }

    Ok(())
}

// ========== 预测市场相关 ==========

#[derive(Debug, Deserialize, Default)]
pub struct ListMarketsQuery {
    pub category: Option<String>,
    pub status: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct MarketSummaryResponse {
    pub id: i64,
    pub question: String,
    pub description: String,
    pub category: String,
    pub image_url: String,
    pub start_time: i64,
    pub end_time: i64,
    pub status: String,
    pub resolved_outcome_id: i64,
    pub total_volume: String,
    pub created_at: i64,
}

#[handler]
pub async fn list_markets(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<ListMarketsQuery>().unwrap_or_default();
    let category = query.category.unwrap_or_default();
    let status = query.status.unwrap_or_default();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let config = crate::grpc::GrpcConfig::default();
    match crate::grpc::create_prediction_market_client(config.prediction_market_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::prediction_market::ListMarketsRequest {
                category: category.clone(),
                status: status.clone(),
                page,
                page_size,
            };

            match client.list_markets(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    let markets: Vec<MarketSummaryResponse> = data.markets.into_iter().map(|m| MarketSummaryResponse {
                        id: m.id,
                        question: m.question,
                        description: m.description,
                        category: m.category,
                        image_url: m.image_url,
                        start_time: m.start_time,
                        end_time: m.end_time,
                        status: m.status,
                        resolved_outcome_id: m.resolved_outcome_id,
                        total_volume: m.total_volume,
                        created_at: m.created_at,
                    }).collect();

                    res.render(Json(serde_json::json!({
                        "markets": markets,
                        "total": data.total,
                        "page": data.page,
                        "page_size": data.page_size,
                    })));
                }
                Err(e) => {
                    tracing::error!("Prediction market list_markets failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to prediction market service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Prediction market service unavailable"})));
        }
    }

    Ok(())
}

#[handler]
pub async fn get_market(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let market_id = req.param::<i64>("market_id")
        .unwrap_or(0);

    if market_id == 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({"error": "Invalid market_id"})));
        return Ok(());
    }

    let config = crate::grpc::GrpcConfig::default();
    match crate::grpc::create_prediction_market_client(config.prediction_market_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::prediction_market::GetMarketRequest { market_id };

            match client.get_market(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    let market = data.market.as_ref().map(|m| serde_json::json!({
                        "id": m.id,
                        "question": m.question,
                        "description": m.description,
                        "category": m.category,
                        "image_url": m.image_url,
                        "start_time": m.start_time,
                        "end_time": m.end_time,
                        "status": m.status,
                        "resolved_outcome_id": m.resolved_outcome_id,
                        "resolved_at": m.resolved_at,
                        "total_volume": m.total_volume,
                        "created_at": m.created_at,
                    }));
                    let outcomes: Vec<serde_json::Value> = data.outcomes.iter().map(|o| serde_json::json!({
                        "id": o.id,
                        "market_id": o.market_id,
                        "name": o.name,
                        "description": o.description,
                        "image_url": o.image_url,
                        "price": o.price,
                        "volume": o.volume,
                        "probability": o.probability,
                        "created_at": o.created_at,
                        "updated_at": o.updated_at,
                    })).collect();
                    res.render(Json(serde_json::json!({
                        "market": market,
                        "outcomes": outcomes,
                    })));
                }
                Err(e) => {
                    tracing::error!("Prediction market get_market failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to prediction market service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Prediction market service unavailable"})));
        }
    }

    Ok(())
}

/// 结算市场（管理员）
#[derive(Debug, Deserialize)]
pub struct ResolveMarketBody {
    pub outcome_id: i64,
}

#[handler]
pub async fn resolve_market(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let market_id = req.param::<i64>("market_id")
        .unwrap_or(0);

    if market_id == 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({"error": "Invalid market_id"})));
        return Ok(());
    }

    let body = req.parse_json::<ResolveMarketBody>().await
        .map_err(|_| {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({"error": "Invalid request body: outcome_id required"})));
            StatusCode::BAD_REQUEST
        })?;

    let config = crate::grpc::GrpcConfig::default();
    match crate::grpc::create_prediction_market_client(config.prediction_market_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::prediction_market::ResolveMarketRequest {
                market_id,
                outcome_id: body.outcome_id,
            };

            match client.resolve_market(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    res.render(Json(serde_json::json!({
                        "success": true,
                        "message": data.message,
                        "resolution": data.resolution.map(|r| serde_json::json!({
                            "id": r.id,
                            "market_id": r.market_id,
                            "outcome_id": r.outcome_id,
                            "total_payout": r.total_payout,
                            "winning_quantity": r.winning_quantity,
                            "payout_ratio": r.payout_ratio,
                            "resolved_at": r.resolved_at,
                        })),
                    })));
                }
                Err(e) => {
                    tracing::error!("Prediction market resolve_market failed: {:?}", e);
                    let msg = if e.code() == tonic::Code::NotFound {
                        "Market not found"
                    } else if e.code() == tonic::Code::FailedPrecondition {
                        "Market cannot be resolved in current status"
                    } else {
                        "Internal error resolving market"
                    };
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": msg, "detail": format!("{}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to prediction market service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Prediction market service unavailable"})));
        }
    }

    Ok(())
}

#[handler]
pub async fn get_market_outcomes(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let market_id = req.param::<i64>("market_id")
        .unwrap_or(0);

    if market_id == 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({"error": "Invalid market_id"})));
        return Ok(());
    }

    let config = crate::grpc::GrpcConfig::default();
    match crate::grpc::create_prediction_market_client(config.prediction_market_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::prediction_market::GetOutcomesRequest { market_id };

            match client.get_outcomes(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    let outcomes: Vec<serde_json::Value> = data.outcomes.iter().map(|o| serde_json::json!({
                        "id": o.id,
                        "market_id": o.market_id,
                        "name": o.name,
                        "description": o.description,
                        "image_url": o.image_url,
                        "price": o.price,
                        "volume": o.volume,
                        "probability": o.probability,
                        "created_at": o.created_at,
                        "updated_at": o.updated_at,
                    })).collect();
                    res.render(Json(serde_json::json!({
                        "outcomes": outcomes,
                    })));
                }
                Err(e) => {
                    tracing::error!("Prediction market get_outcomes failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to prediction market service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Prediction market service unavailable"})));
        }
    }

    Ok(())
}

#[handler]
pub async fn get_market_price(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let market_id = req.param::<i64>("market_id")
        .unwrap_or(0);

    if market_id == 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({"error": "Invalid market_id"})));
        return Ok(());
    }

    let config = crate::grpc::GrpcConfig::default();
    match crate::grpc::create_prediction_market_client(config.prediction_market_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::prediction_market::GetMarketPriceRequest { market_id };

            match client.get_market_price(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    let prices: Vec<serde_json::Value> = data.prices.iter().map(|p| serde_json::json!({
                        "outcome_id": p.outcome_id,
                        "name": p.name,
                        "price": p.price,
                        "volume": p.volume,
                        "probability": p.probability,
                    })).collect();
                    res.render(Json(serde_json::json!({
                        "market_id": data.market_id,
                        "prices": prices,
                    })));
                }
                Err(e) => {
                    tracing::error!("Prediction market get_market_price failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to prediction market service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Prediction market service unavailable"})));
        }
    }

    Ok(())
}
