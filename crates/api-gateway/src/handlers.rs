//! API 处理器

use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grpc::{create_user_client, create_order_client, GrpcConfig};

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

    // 调用 User Service gRPC 验证登录
    let config = GrpcConfig::default();
    match create_user_client(config.user_service_addr).await {
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
                    tracing::info!("User logged in: {}", payload.email);

                    res.render(Json(LoginResponse {
                        success: true,
                        token: Some(user_data.token),
                        refresh_token: Some(user_data.refresh_token),
                        expires_in: Some(user_data.expires_at),
                        token_type: Some("Bearer".to_string()),
                        user_id: Some(user_data.user_id),
                        message: None,
                    }));
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
    let user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

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

    // 解析 user_id 为 i64
    let user_id_i64: i64 = user_id.replace("usr_", "").parse().unwrap_or(0);

    // 调用 Order Service gRPC
    let config = GrpcConfig::default();
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
    pub account_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub available: String,
    pub frozen: String,
    pub equity: String,
}

#[handler]
pub async fn get_balance(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let _user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let _query = req.parse_queries::<GetBalanceQuery>().unwrap_or_default();

    // TODO: 调用 Portfolio Service gRPC
    res.render(Json(BalanceResponse {
        available: "10000.00".to_string(),
        frozen: "1000.00".to_string(),
        equity: "11000.00".to_string(),
    }));

    Ok(())
}

// ========== 持仓相关 ==========

#[handler]
pub async fn get_positions(_req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let _user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // TODO: 调用 Portfolio Service gRPC
    res.render(Json(Vec::<serde_json::Value>::new()));

    Ok(())
}

// ========== 行情相关 ==========

#[derive(Debug, Deserialize, Default)]
pub struct DepthQuery {
    pub market_id: Option<u64>,
    pub outcome_id: Option<u64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct DepthResponse {
    pub asks: Vec<Vec<String>>,
    pub bids: Vec<Vec<String>>,
}

#[handler]
pub async fn get_depth(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<DepthQuery>().unwrap_or_default();
    let _market_id = query.market_id.unwrap_or(1);
    let _outcome_id = query.outcome_id.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    // TODO: 调用 Market Data Service gRPC
    // 模拟订单簿数据
    let asks: Vec<Vec<String>> = (0..limit.min(10))
        .map(|i| vec![
            format!("{:.2}", 0.50 + i as f64 * 0.01),
            format!("{}", 100 + i * 50)
        ])
        .collect();

    let bids: Vec<Vec<String>> = (0..limit.min(10))
        .map(|i| vec![
            format!("{:.2}", 0.49 - i as f64 * 0.01),
            format!("{}", 100 + i * 50)
        ])
        .collect();

    res.render(Json(DepthResponse { asks, bids }));

    Ok(())
}

#[derive(Debug, Deserialize, Default)]
pub struct TickerQuery {
    pub market_id: Option<u64>,
    pub outcome_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TickerResponse {
    pub market_id: u64,
    pub outcome_id: u64,
    pub last_price: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub quote_volume: String,
    pub timestamp: String,
}

#[handler]
pub async fn get_ticker(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<TickerQuery>().unwrap_or_default();
    let market_id = query.market_id.unwrap_or(1);
    let outcome_id = query.outcome_id.unwrap_or(1);

    // TODO: 调用 Market Data Service gRPC
    res.render(Json(TickerResponse {
        market_id,
        outcome_id,
        last_price: "0.55".to_string(),
        price_change: "+0.05".to_string(),
        price_change_percent: "+10.00%".to_string(),
        high_price: "0.60".to_string(),
        low_price: "0.45".to_string(),
        volume: "1000000".to_string(),
        quote_volume: "550000".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }));

    Ok(())
}

#[handler]
pub async fn get_kline(_req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    // TODO: 调用 Market Data Service gRPC
    res.render(Json(Vec::<serde_json::Value>::new()));

    Ok(())
}

#[handler]
pub async fn get_recent_trades(_req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    // TODO: 调用 Market Data Service gRPC
    res.render(Json(Vec::<serde_json::Value>::new()));

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

    // TODO: 调用 Wallet Service gRPC
    let withdrawal_id = format!("wd_{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(16).collect::<String>());

    tracing::info!("Withdrawal requested: {} {} {} by user {}",
        payload.amount, payload.asset, withdrawal_id, user_id);

    // 计算手续费 (假设 1 USDT 或 1%)
    let amount: f64 = payload.amount.parse().unwrap_or(0.0);
    let fee = if payload.asset.to_uppercase() == "USDT" {
        1.0f64.max(amount * 0.01)
    } else {
        amount * 0.01
    };

    res.render(Json(WithdrawResponse {
        success: true,
        withdrawal_id,
        asset: payload.asset.to_uppercase(),
        amount: payload.amount,
        fee: format!("{:.2}", fee),
        net_amount: format!("{:.2}", amount - fee),
        status: "pending".to_string(),
    }));

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
pub async fn get_wallet_history(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let _user_id = depot.get::<String>("user_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let _query = req.parse_queries::<WalletHistoryQuery>().unwrap_or_default();

    // TODO: 调用 Wallet Service gRPC
    res.render(Json(WalletHistoryResponse {
        deposits: Vec::new(),
        withdrawals: Vec::new(),
    }));

    Ok(())
}
