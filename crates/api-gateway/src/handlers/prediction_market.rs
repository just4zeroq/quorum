//! 预测市场处理器

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use crate::grpc::GrpcConfig;

// ========== 请求/响应类型 ==========

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

/// 结算市场请求体
#[derive(Debug, Deserialize)]
pub struct ResolveMarketBody {
    pub outcome_id: i64,
}

// ========== 处理器 ==========

/// 获取市场列表
#[handler]
pub async fn list_markets(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<ListMarketsQuery>().unwrap_or_default();
    let category = query.category.unwrap_or_default();
    let status = query.status.unwrap_or_default();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let config = GrpcConfig::default();
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

/// 获取市场详情
#[handler]
pub async fn get_market(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let market_id = req.param::<i64>("market_id").unwrap_or(0);

    if market_id == 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({"error": "Invalid market_id"})));
        return Ok(());
    }

    let config = GrpcConfig::default();
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
#[handler]
pub async fn resolve_market(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let market_id = req.param::<i64>("market_id").unwrap_or(0);

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

    let config = GrpcConfig::default();
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

/// 获取市场 outcomes
#[handler]
pub async fn get_market_outcomes(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let market_id = req.param::<i64>("market_id").unwrap_or(0);

    if market_id == 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({"error": "Invalid market_id"})));
        return Ok(());
    }

    let config = GrpcConfig::default();
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
                    res.render(Json(serde_json::json!({ "outcomes": outcomes })));
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

/// 获取市场价格
#[handler]
pub async fn get_market_price(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let market_id = req.param::<i64>("market_id").unwrap_or(0);

    if market_id == 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({"error": "Invalid market_id"})));
        return Ok(());
    }

    let config = GrpcConfig::default();
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
