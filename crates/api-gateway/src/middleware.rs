//! 中间件

use salvo::prelude::*;
use common::Result;

/// 认证中间件
#[handler]
pub async fn auth(req: &mut Request, depot: &mut Depot, res: &mut Response, next: &mut Next) -> Result<()> {
    // 检查 Authorization header
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];
            // TODO: 验证 JWT token
            // 验证通过后将 user_id 放入 depot
            depot.insert("user_id", "mock_user_id".to_string());
            next.next(req, depot, res).await;
        }
        _ => {
            // 公开接口不需要认证
            let path = req.uri().path();
            if path.starts_with("/health")
                || path.starts_with("/ready")
                || path.starts_with("/api/v1/market")
                || path == "/api/v1/users/login"
                || path == "/api/v1/users/register"
            {
                next.next(req, depot, res).await;
            } else {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Text::Plain("Missing or invalid authorization header"));
            }
        }
    }
}

/// 限流中间件
#[handler]
pub async fn rate_limit(req: &mut Request, _depot: &mut Depot, _res: &mut Response, next: &mut Next) -> Result<()> {
    // TODO: 实现基于 token bucket 的限流
    // 目前简单实现：检查 X-Forwarded-For 或 X-Real-IP
    let client_ip = req
        .headers()
        .get("x-real-ip")
        .or_else(|| req.headers().get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    tracing::debug!("Request from IP: {}", client_ip);
    next.next(req, _depot, _res).await
}

/// 请求日志中间件
#[handler]
pub async fn log_request(req: &mut Request, _depot: &mut Depot, _res: &mut Response, next: &mut Next) -> Result<()> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    tracing::debug!("{} {}", method, path);

    next.next(req, _depot, _res).await;

    tracing::debug!("Request completed: {} {}", method, path);
}

/// CORS 中间件配置
pub fn cors_handler() -> Cors {
    Cors::new()
        .allow_origin("*")
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_headers(vec!["*"])
}