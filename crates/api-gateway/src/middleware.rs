//! 中间件

use salvo::prelude::*;
use salvo::cors::{Cors, CorsHandler};
use salvo::hyper::Method;
use serde::{Deserialize, Serialize};

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user_id
    pub sid: String,        // session_id
    pub exp: i64,           // expiration
    pub iat: i64,           // issued at
    pub ttype: String,      // token type: access/refresh
}

/// 认证中间件
#[handler]
pub async fn auth(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    // 公开接口列表
    let public_paths = [
        "/health",
        "/ready",
        "/api/v1/market",
        "/api/v1/users/login",
        "/api/v1/users/register",
    ];

    let path = req.uri().path();

    // 检查是否是公开接口
    if public_paths.iter().any(|p| path.starts_with(p)) {
        return;
    }

    // 检查 Authorization header
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];

            // Auth Service 地址
            let auth_addr = std::env::var("AUTH_SERVICE_ADDR")
                .unwrap_or_else(|_| "http://127.0.0.1:50009".to_string());

            match crate::grpc::create_auth_client(auth_addr).await {
                Ok(mut client) => {
                    let request = tonic::Request::new(api::auth::ValidateTokenRequest {
                        token: token.to_string(),
                    });

                    match client.validate_token(request).await {
                        Ok(response) => {
                            let validate_resp = response.into_inner();
                            if validate_resp.valid {
                                depot.insert("user_id", validate_resp.user_id.clone());
                                depot.insert("session_id", validate_resp.session_id.clone());
                                tracing::debug!("Authenticated user: {}", validate_resp.user_id);
                            } else {
                                res.status_code(StatusCode::UNAUTHORIZED);
                                res.render(Json(serde_json::json!({
                                    "success": false,
                                    "error": "Invalid or expired token"
                                })));
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Auth service validate token failed: {:?}", e);
                            res.status_code(StatusCode::UNAUTHORIZED);
                            res.render(Json(serde_json::json!({
                                "success": false,
                                "error": "Invalid or expired token"
                            })));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to connect to auth service: {:?}", e);
                    res.status_code(StatusCode::SERVICE_UNAVAILABLE);
                    res.render(Json(serde_json::json!({
                        "success": false,
                        "error": "Authentication service unavailable"
                    })));
                }
            }
        }
        _ => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "success": false,
                "error": "Missing or invalid authorization header"
            })));
        }
    }
}

/// 限流中间件
#[handler]
pub async fn rate_limit(req: &mut Request, _depot: &mut Depot, _res: &mut Response) {
    let client_ip = req
        .headers()
        .get("x-real-ip")
        .or_else(|| req.headers().get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    tracing::debug!("Request from IP: {}", client_ip);
}

/// 请求日志中间件
#[handler]
pub async fn log_request(req: &mut Request, _depot: &mut Depot, _res: &mut Response) {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("");

    tracing::info!("{} {}?{}", method, path, query);
}

/// CORS 中间件配置
pub fn cors_handler() -> CorsHandler {
    Cors::new()
        .allow_origin("*")
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers("*")
        .into_handler()
}
