//! 中间件

use salvo::prelude::*;
use salvo::cors::{Cors, CorsHandler};
use salvo::hyper::Method;
use jsonwebtoken::{decode, DecodingKey, Validation};
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

            // JWT secret - 在生产环境应该从配置读取
            let jwt_secret = std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key".to_string());

            match decode::<Claims>(
                token,
                &DecodingKey::from_secret(jwt_secret.as_bytes()),
                &Validation::default(),
            ) {
                Ok(token_data) => {
                    // Token 有效，提取 user_id
                    let claims = token_data.claims;

                    // 检查 token 类型
                    if claims.ttype != "access" {
                        res.status_code(StatusCode::UNAUTHORIZED);
                        res.render(Json(serde_json::json!({
                            "success": false,
                            "error": "Invalid token type"
                        })));
                        return;
                    }

                    // 将用户信息放入 depot
                    depot.insert("user_id", claims.sub.clone());
                    depot.insert("session_id", claims.sid.clone());

                    tracing::debug!("Authenticated user: {}", claims.sub);
                }
                Err(e) => {
                    tracing::warn!("JWT validation failed: {:?}", e);
                    res.status_code(StatusCode::UNAUTHORIZED);
                    res.render(Json(serde_json::json!({
                        "success": false,
                        "error": "Invalid or expired token"
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
