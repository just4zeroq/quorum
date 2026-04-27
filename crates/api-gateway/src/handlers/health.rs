//! 健康检查处理器

use salvo::prelude::*;

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
