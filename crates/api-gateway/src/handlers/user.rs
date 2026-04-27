//! 用户相关处理器

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use crate::grpc::{create_user_client, create_auth_client, GrpcConfig};
use crate::handlers::parse_json;

// ========== 请求/响应类型 ==========

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

#[derive(Debug, Deserialize, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
}

// ========== 处理器 ==========

/// 用户注册
#[handler]
pub async fn register(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let payload = parse_json::<RegisterRequest>(req).await?;

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

/// 用户登录（两步：User Service 验证身份 → Auth Service 签发 Token）
#[handler]
pub async fn login(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let payload = parse_json::<LoginRequest>(req).await?;

    let config = GrpcConfig::default();

    // 步骤1: 调用 User Service 验证邮箱/密码
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
                password: String::new(),
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

/// 退出登录
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
            let grpc_request = api::auth::LogoutRequest {
                session_id: session_id.clone(),
            };
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

/// 刷新 Token
#[handler]
pub async fn refresh_token(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let payload = parse_json::<RefreshTokenRequest>(req).await?;

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

/// 获取当前用户信息
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
                        res.render(Json(serde_json::json!({ "error": "User not found" })));
                    }
                }
                Err(e) => {
                    tracing::error!("User service get_user failed: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(serde_json::json!({ "error": format!("Failed to get user: {:?}", e) })));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to user service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({ "error": "User service unavailable" })));
        }
    }

    Ok(())
}
