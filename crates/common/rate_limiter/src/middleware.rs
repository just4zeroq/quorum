//! Rate Limiter Middleware

use std::sync::Arc;
use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderValue},
    middleware::Next,
    response::Response,
};

use crate::traits::{RateLimiter, RateLimitKey};

/// Rate limit headers names
pub const X_RATELIMIT_LIMIT: &str = "X-RateLimit-Limit";
pub const X_RATELIMIT_REMAINING: &str = "X-RateLimit-Remaining";
pub const X_RATELIMIT_RESET: &str = "X-RateLimit-Reset";

/// Rate limit middleware for Axum
pub async fn rate_limit_middleware<R: RateLimiter + 'static>(
    State(limiter): State<Arc<R>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let key = key_from_request(None, &request);

    match limiter.check_and_consume(&key, 1).await {
        Ok(result) => {
            if result.allowed {
                let mut response = next.run(request).await;

                // Add rate limit headers
                let headers = response.headers_mut();
                headers.insert(X_RATELIMIT_LIMIT, HeaderValue::from_static("100"));
                headers.insert(X_RATELIMIT_REMAINING, result.remaining.to_string().parse().unwrap());
                headers.insert(X_RATELIMIT_RESET, result.reset_at_ms.to_string().parse().unwrap());

                Ok(response)
            } else {
                Err(StatusCode::TOO_MANY_REQUESTS)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Build rate limit key from request
pub fn key_from_request(user_id: Option<String>, request: &Request) -> RateLimitKey {
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());

    let endpoint = request.uri().path().to_string();

    let mut key = RateLimitKey::new();
    if let Some(uid) = user_id {
        key = key.with_user(uid);
    }
    if let Some(ip) = ip {
        key = key.with_ip(ip);
    }
    key.with_endpoint(endpoint)
}
