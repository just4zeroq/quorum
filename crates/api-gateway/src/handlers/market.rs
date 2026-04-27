//! 行情数据处理器

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use crate::grpc::GrpcConfig;

// ========== 请求/响应类型 ==========

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
    pub low_24h: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub trade_count_24h: i64,
    pub timestamp: i64,
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

// ========== 处理器 ==========

/// 获取订单簿深度
#[handler]
pub async fn get_depth(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<DepthQuery>().unwrap_or_default();
    let market_id = query.market_id.unwrap_or(1) as i64;
    let depth = query.depth.unwrap_or(20);

    let config = GrpcConfig::default();
    match crate::grpc::create_market_data_client(config.market_data_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::market_data::GetOrderBookRequest { market_id, depth };

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

/// 获取 24 小时行情
#[handler]
pub async fn get_ticker(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<TickerQuery>().unwrap_or_default();
    let market_id = query.market_id.unwrap_or(1) as i64;

    let config = GrpcConfig::default();
    match crate::grpc::create_market_data_client(config.market_data_service_addr).await {
        Ok(mut client) => {
            let grpc_request = api::market_data::Get24hStatsRequest { market_id };

            match client.get_24h_stats(grpc_request).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    res.render(Json(TickerResponse {
                        market_id: market_id as u64,
                        volume_24h: data.volume_24h,
                        amount_24h: data.amount_24h,
                        high_24h: data.high_24h,
                        low_24h: data.low_24h,
                        price_change: data.price_change,
                        price_change_percent: data.price_change_percent,
                        trade_count_24h: data.trade_count_24h,
                        timestamp: data.timestamp,
                    }));
                }
                Err(e) => {
                    tracing::error!("Market data get_24h_stats failed: {:?}", e);
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

/// 获取 K 线数据
#[handler]
pub async fn get_kline(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<KlineQuery>().unwrap_or_default();
    let market_id = query.market_id.unwrap_or(1) as i64;
    let interval = query.interval.unwrap_or_else(|| "1h".to_string());
    let start_time = query.start_time.unwrap_or(0);
    let end_time = query.end_time.unwrap_or(0);
    let limit = query.limit.unwrap_or(100);

    let config = GrpcConfig::default();
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

/// 获取最近成交
#[handler]
pub async fn get_recent_trades(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<TradesQuery>().unwrap_or_default();
    let market_id = query.market_id.unwrap_or(1) as i64;
    let outcome_id = query.outcome_id.unwrap_or(0) as i64;
    let limit = query.limit.unwrap_or(50);

    let config = GrpcConfig::default();
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
