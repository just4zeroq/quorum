//! Market Data Service Implementation

use std::sync::Arc;
use tonic::{Request, Response, Status};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::models::{PredictionMarket, MarketOutcome, Kline, Trade, OrderBook, Market24hStats};
use crate::repository::{MarketRepository, KlineRepository, TradeRepository};
use api::market_data::{
    market_data_service_server::MarketDataService, GetMarketsRequest, GetMarketsResponse, MarketSummary, OutcomeSummary,
    GetMarketDetailRequest, GetMarketDetailResponse, MarketDetail, OutcomeDetail,
    GetOutcomePricesRequest, GetOutcomePricesResponse, OutcomePrice,
    GetOrderBookRequest, GetOrderBookResponse, OrderBookLevel,
    GetKlinesRequest, GetKlinesResponse, KlineData,
    GetLatestKlineRequest, GetLatestKlineResponse,
    GetRecentTradesRequest, GetRecentTradesResponse, TradeData,
    Get24hStatsRequest, Get24hStatsResponse,
};

pub struct MarketDataServiceImpl {
    pool: sqlx::PgPool,
}

impl MarketDataServiceImpl {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl MarketDataService for MarketDataServiceImpl {
    async fn get_markets(
        &self,
        request: Request<GetMarketsRequest>,
    ) -> Result<Response<GetMarketsResponse>, Status> {
        let req = request.into_inner();

        let sort_by = if req.sort_by.is_empty() { "created_at" } else { &req.sort_by };
        let page = if req.page <= 0 { 1 } else { req.page };
        let page_size = if req.page_size <= 0 { 20 } else { req.page_size };

        let markets = MarketRepository::get_markets(
            &self.pool,
            if req.category.is_empty() { None } else { Some(&req.category) },
            if req.status.is_empty() { None } else { Some(&req.status) },
            sort_by,
            req.descending,
            page,
            page_size,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let total = MarketRepository::count_markets(
            &self.pool,
            if req.category.is_empty() { None } else { Some(&req.category) },
            if req.status.is_empty() { None } else { Some(&req.status) },
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        // 获取每个市场的选项
        let mut market_summaries = Vec::new();
        for market in markets {
            let outcomes = MarketRepository::get_outcomes(&self.pool, market.id)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            let outcome_summaries: Vec<OutcomeSummary> = outcomes
                .iter()
                .map(|o| OutcomeSummary {
                    id: o.id,
                    name: o.name.clone(),
                    price: o.price.to_string(),
                    volume: o.volume.to_string(),
                    probability: o.probability.to_string(),
                })
                .collect();

            market_summaries.push(MarketSummary {
                id: market.id,
                question: market.question,
                category: market.category,
                end_time: market.end_time,
                status: market.status.to_string(),
                total_volume: market.total_volume.to_string(),
                outcomes: outcome_summaries,
                created_at: market.created_at,
            });
        }

        Ok(Response::new(GetMarketsResponse {
            markets: market_summaries,
            total,
            page,
            page_size,
        }))
    }

    async fn get_market_detail(
        &self,
        request: Request<GetMarketDetailRequest>,
    ) -> Result<Response<GetMarketDetailResponse>, Status> {
        let req = request.into_inner();

        let market = MarketRepository::get_market_by_id(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Market not found"))?;

        let outcomes = MarketRepository::get_outcomes(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let market_detail = MarketDetail {
            id: market.id,
            question: market.question,
            description: market.description.unwrap_or_default(),
            category: market.category,
            image_url: market.image_url.unwrap_or_default(),
            start_time: market.start_time,
            end_time: market.end_time,
            status: market.status.to_string(),
            total_volume: market.total_volume.to_string(),
            resolved_outcome_id: market.resolved_outcome_id.unwrap_or(0),
            resolved_at: market.resolved_at.unwrap_or(0),
            created_at: market.created_at,
            updated_at: market.updated_at,
        };

        let outcome_details: Vec<OutcomeDetail> = outcomes
            .iter()
            .map(|o| OutcomeDetail {
                id: o.id,
                name: o.name.clone(),
                description: o.description.clone().unwrap_or_default(),
                image_url: o.image_url.clone().unwrap_or_default(),
                price: o.price.to_string(),
                volume: o.volume.to_string(),
                probability: o.probability.to_string(),
            })
            .collect();

        Ok(Response::new(GetMarketDetailResponse {
            market: Some(market_detail),
            outcomes: outcome_details,
        }))
    }

    async fn get_outcome_prices(
        &self,
        request: Request<GetOutcomePricesRequest>,
    ) -> Result<Response<GetOutcomePricesResponse>, Status> {
        let req = request.into_inner();

        let outcome_ids: Vec<i64> = req.outcome_ids.clone();
        let outcomes = MarketRepository::get_outcome_prices(&self.pool, req.market_id, &outcome_ids)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let prices: Vec<OutcomePrice> = outcomes
            .iter()
            .map(|o| OutcomePrice {
                outcome_id: o.id,
                name: o.name.clone(),
                price: o.price.to_string(),
                volume: o.volume.to_string(),
                probability: o.probability.to_string(),
                change_24h: "0".to_string(),
            })
            .collect();

        Ok(Response::new(GetOutcomePricesResponse {
            market_id: req.market_id,
            prices,
        }))
    }

    async fn get_order_book(
        &self,
        request: Request<GetOrderBookRequest>,
    ) -> Result<Response<GetOrderBookResponse>, Status> {
        let req = request.into_inner();
        let depth = if req.depth <= 0 { 10 } else { req.depth };

        let orderbook = MarketRepository::get_orderbook(&self.pool, req.market_id, depth)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let bids: Vec<OrderBookLevel> = orderbook
            .bids
            .iter()
            .map(|b| OrderBookLevel {
                price: b.price.to_string(),
                quantity: b.quantity.to_string(),
                orders: b.orders.to_string(),
            })
            .collect();

        let asks: Vec<OrderBookLevel> = orderbook
            .asks
            .iter()
            .map(|a| OrderBookLevel {
                price: a.price.to_string(),
                quantity: a.quantity.to_string(),
                orders: a.orders.to_string(),
            })
            .collect();

        Ok(Response::new(GetOrderBookResponse {
            market_id: req.market_id,
            bids,
            asks,
            timestamp: orderbook.timestamp,
        }))
    }

    async fn get_klines(
        &self,
        request: Request<GetKlinesRequest>,
    ) -> Result<Response<GetKlinesResponse>, Status> {
        let req = request.into_inner();

        let klines = KlineRepository::get_klines(
            &self.pool,
            req.market_id,
            &req.interval,
            req.start_time,
            req.end_time,
            req.limit,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let kline_data: Vec<KlineData> = klines
            .iter()
            .map(|k| KlineData {
                timestamp: k.timestamp,
                open: k.open.to_string(),
                high: k.high.to_string(),
                low: k.low.to_string(),
                close: k.close.to_string(),
                volume: k.volume.to_string(),
                quote_volume: k.quote_volume.to_string(),
            })
            .collect();

        Ok(Response::new(GetKlinesResponse {
            market_id: req.market_id,
            interval: req.interval,
            klines: kline_data,
        }))
    }

    async fn get_latest_kline(
        &self,
        request: Request<GetLatestKlineRequest>,
    ) -> Result<Response<GetLatestKlineResponse>, Status> {
        let req = request.into_inner();

        let kline = KlineRepository::get_latest_kline(&self.pool, req.market_id, &req.interval)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match kline {
            Some(k) => Ok(Response::new(GetLatestKlineResponse {
                kline: Some(KlineData {
                    timestamp: k.timestamp,
                    open: k.open.to_string(),
                    high: k.high.to_string(),
                    low: k.low.to_string(),
                    close: k.close.to_string(),
                    volume: k.volume.to_string(),
                    quote_volume: k.quote_volume.to_string(),
                }),
            })),
            None => Ok(Response::new(GetLatestKlineResponse { kline: None })),
        }
    }

    async fn get_recent_trades(
        &self,
        request: Request<GetRecentTradesRequest>,
    ) -> Result<Response<GetRecentTradesResponse>, Status> {
        let req = request.into_inner();
        let limit = if req.limit <= 0 { 50 } else { req.limit };

        let trades = TradeRepository::get_recent_trades(
            &self.pool,
            req.market_id,
            if req.outcome_id == 0 { None } else { Some(req.outcome_id) },
            limit,
            if req.from_trade_id == 0 { None } else { Some(req.from_trade_id) },
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let trade_data: Vec<TradeData> = trades
            .iter()
            .map(|t| TradeData {
                id: t.id.parse().unwrap_or(0),
                market_id: t.market_id,
                outcome_id: t.outcome_id,
                user_id: t.taker_user_id,
                side: t.side.to_string(),
                price: t.price.to_string(),
                quantity: t.quantity.to_string(),
                amount: t.amount.to_string(),
                fee: t.taker_fee.to_string(),
                timestamp: t.created_at,
            })
            .collect();

        let next_id = trade_data.last().map(|t| t.id).unwrap_or(0);

        Ok(Response::new(GetRecentTradesResponse {
            trades: trade_data,
            next_from_trade_id: next_id,
            has_more: false,
        }))
    }

    async fn get_24h_stats(
        &self,
        request: Request<Get24hStatsRequest>,
    ) -> Result<Response<Get24hStatsResponse>, Status> {
        let req = request.into_inner();

        let stats = MarketRepository::get_24h_stats(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(Get24hStatsResponse {
            market_id: req.market_id,
            volume_24h: stats.volume_24h.to_string(),
            amount_24h: stats.amount_24h.to_string(),
            high_24h: stats.high_24h.to_string(),
            low_24h: stats.low_24h.to_string(),
            price_change: stats.price_change.to_string(),
            price_change_percent: stats.price_change_percent.to_string(),
            trade_count_24h: stats.trade_count_24h,
            timestamp: stats.timestamp,
        }))
    }
}