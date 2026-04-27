//! 账户/持仓相关处理器

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use crate::grpc::create_portfolio_client;
use crate::grpc::GrpcConfig;
use crate::handlers::get_user_id_or_unknown;

// ========== 请求/响应类型 ==========

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

#[derive(Debug, Deserialize, Default)]
pub struct GetPositionsQuery {
    pub market_id: Option<u64>,
}

// ========== 处理器 ==========

/// 获取账户余额
#[handler]
pub async fn get_balance(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = get_user_id_or_unknown(depot);
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

/// 获取用户持仓
#[handler]
pub async fn get_positions(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = get_user_id_or_unknown(depot);
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
