//! 预测市场服务 - gRPC 实现

use tonic::{Request, Response, Status};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::models::{PredictionMarket, MarketOutcome, MarketStatus};
use crate::repository::{MarketRepository, OutcomeRepository, PositionRepository};
use crate::pb::{
    prediction_market_service_server::PredictionMarketService,
    CreateMarketRequest, CreateMarketResponse, UpdateMarketRequest, UpdateMarketResponse,
    CloseMarketRequest, CloseMarketResponse, GetMarketRequest, GetMarketResponse,
    ListMarketsRequest, ListMarketsResponse, AddOutcomeRequest, AddOutcomeResponse,
    GetOutcomesRequest, GetOutcomesResponse, GetMarketPriceRequest, GetMarketPriceResponse,
    GetMarketDepthRequest, GetMarketDepthResponse, ResolveMarketRequest, ResolveMarketResponse,
    CalculatePayoutRequest, CalculatePayoutResponse, GetUserPositionsRequest, GetUserPositionsResponse,
    PredictionMarket as PbPredictionMarket, MarketOutcome as PbMarketOutcome,
    UserPosition as PbUserPosition, MarketResolution as PbMarketResolution, OutcomePrice,
    OrderBookLevel, UserPositionPayout,
};

pub struct MarketService {
    pool: sqlx::PgPool,
    portfolio_client: api::portfolio::portfolio_service_client::PortfolioServiceClient<tonic::transport::Channel>,
    event_producer: Option<queue::ProducerManager>,
}

impl MarketService {
    pub fn new(
        pool: sqlx::PgPool,
        portfolio_client: api::portfolio::portfolio_service_client::PortfolioServiceClient<tonic::transport::Channel>,
    ) -> Self {
        Self { pool, portfolio_client, event_producer: None }
    }

    /// 设置事件生产者
    pub fn with_event_producer(mut self, producer: queue::ProducerManager) -> Self {
        self.event_producer = Some(producer);
        self
    }
}

#[tonic::async_trait]
impl PredictionMarketService for MarketService {
    async fn create_market(
        &self,
        request: Request<CreateMarketRequest>,
    ) -> Result<Response<CreateMarketResponse>, Status> {
        let req = request.into_inner();

        if req.question.is_empty() {
            return Err(Status::invalid_argument("Question cannot be empty"));
        }
        if req.outcomes.is_empty() {
            return Err(Status::invalid_argument("At least one outcome is required"));
        }

        let mut market = PredictionMarket::new(
            req.question,
            if req.description.is_empty() { None } else { Some(req.description) },
            req.category,
            if req.image_url.is_empty() { None } else { Some(req.image_url) },
            req.start_time,
            req.end_time,
        );

        let market_id = MarketRepository::create(&self.pool, &market)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        market.id = market_id;

        let mut outcomes = Vec::new();
        for outcome_req in req.outcomes {
            let mut outcome = MarketOutcome::new(
                market_id,
                outcome_req.name,
                if outcome_req.description.is_empty() { None } else { Some(outcome_req.description) },
                if outcome_req.image_url.is_empty() { None } else { Some(outcome_req.image_url) },
            );

            let outcome_id = OutcomeRepository::create(&self.pool, &outcome)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            outcome.id = outcome_id;
            outcomes.push(outcome);
        }

        let response = CreateMarketResponse {
            market_id,
            market: Some(self.model_to_pb_market(market)),
            outcomes: outcomes.into_iter().map(|o| self.model_to_pb_outcome(o)).collect(),
        };

        Ok(Response::new(response))
    }

    async fn get_market(
        &self,
        request: Request<GetMarketRequest>,
    ) -> Result<Response<GetMarketResponse>, Status> {
        let req = request.into_inner();

        let market = MarketRepository::find_by_id(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Market not found"))?;

        let outcomes = OutcomeRepository::find_by_market(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = GetMarketResponse {
            market: Some(self.model_to_pb_market(market)),
            outcomes: outcomes.into_iter().map(|o| self.model_to_pb_outcome(o)).collect(),
        };

        Ok(Response::new(response))
    }

    async fn list_markets(
        &self,
        request: Request<ListMarketsRequest>,
    ) -> Result<Response<ListMarketsResponse>, Status> {
        let req = request.into_inner();
        let limit = req.page_size as i64;
        let offset = ((req.page - 1) * req.page_size) as i64;

        let markets = MarketRepository::list(
            &self.pool,
            if req.category.is_empty() { None } else { Some(&req.category) },
            if req.status.is_empty() { None } else { Some(&req.status) },
            limit,
            offset,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let total = markets.len() as i64;

        let response = ListMarketsResponse {
            markets: markets.into_iter().map(|m| self.model_to_pb_market(m)).collect(),
            total,
            page: req.page,
            page_size: req.page_size,
        };

        Ok(Response::new(response))
    }

    async fn get_outcomes(
        &self,
        request: Request<GetOutcomesRequest>,
    ) -> Result<Response<GetOutcomesResponse>, Status> {
        let req = request.into_inner();

        let outcomes = OutcomeRepository::find_by_market(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = GetOutcomesResponse {
            outcomes: outcomes.into_iter().map(|o| self.model_to_pb_outcome(o)).collect(),
        };

        Ok(Response::new(response))
    }

    async fn update_market(
        &self,
        request: Request<UpdateMarketRequest>,
    ) -> Result<Response<UpdateMarketResponse>, Status> {
        let req = request.into_inner();

        // 检查市场是否存在
        let market = MarketRepository::find_by_id(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Market not found"))?;

        // 只有开放状态的市场才能更新
        if market.status != MarketStatus::Open {
            return Err(Status::failed_precondition("Only open markets can be updated"));
        }

        let question = if req.question.is_empty() { None } else { Some(req.question.as_str()) };
        let description = if req.description.is_empty() { None } else { Some(req.description.as_str()) };
        let image_url = if req.image_url.is_empty() { None } else { Some(req.image_url.as_str()) };
        let end_time = if req.end_time == 0 { None } else { Some(req.end_time) };

        MarketRepository::update(
            &self.pool,
            req.market_id,
            question,
            description,
            image_url,
            end_time,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的市场
        let updated_market = MarketRepository::find_by_id(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .unwrap();

        Ok(Response::new(UpdateMarketResponse {
            success: true,
            market: Some(self.model_to_pb_market(updated_market)),
        }))
    }

    async fn close_market(
        &self,
        request: Request<CloseMarketRequest>,
    ) -> Result<Response<CloseMarketResponse>, Status> {
        let req = request.into_inner();

        let market = MarketRepository::find_by_id(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Market not found"))?;

        if market.status != MarketStatus::Open {
            return Err(Status::failed_precondition("Only open markets can be closed"));
        }

        MarketRepository::update_status(
            &self.pool,
            req.market_id,
            "closed",
            None,
            None,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CloseMarketResponse { success: true }))
    }

    async fn add_outcome(
        &self,
        request: Request<AddOutcomeRequest>,
    ) -> Result<Response<AddOutcomeResponse>, Status> {
        let req = request.into_inner();

        let market = MarketRepository::find_by_id(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Market not found"))?;

        if market.status != MarketStatus::Open {
            return Err(Status::failed_precondition("Only open markets can add outcomes"));
        }

        let mut outcome = MarketOutcome::new(
            req.market_id,
            req.name,
            if req.description.is_empty() { None } else { Some(req.description) },
            if req.image_url.is_empty() { None } else { Some(req.image_url) },
        );

        let outcome_id = OutcomeRepository::create(&self.pool, &outcome)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        outcome.id = outcome_id;

        Ok(Response::new(AddOutcomeResponse {
            success: true,
            outcome: Some(self.model_to_pb_outcome(outcome)),
        }))
    }

    async fn get_market_price(
        &self,
        request: Request<GetMarketPriceRequest>,
    ) -> Result<Response<GetMarketPriceResponse>, Status> {
        let req = request.into_inner();

        let outcomes = OutcomeRepository::find_by_market(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let prices: Vec<OutcomePrice> = outcomes
            .into_iter()
            .map(|o| OutcomePrice {
                outcome_id: o.id,
                name: o.name,
                price: o.price.to_string(),
                volume: o.volume.to_string(),
                probability: o.probability.to_string(),
            })
            .collect();

        Ok(Response::new(GetMarketPriceResponse {
            market_id: req.market_id,
            prices,
        }))
    }

    async fn get_market_depth(
        &self,
        request: Request<GetMarketDepthRequest>,
    ) -> Result<Response<GetMarketDepthResponse>, Status> {
        let req = request.into_inner();

        // 简化的深度数据，从选项价格生成模拟订单簿
        // 实际生产环境应该从 Matching Engine 获取真实订单簿
        let outcomes = OutcomeRepository::find_by_market(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for outcome in outcomes {
            let price: Decimal = outcome.price;
            let volume: Decimal = outcome.volume;

            if price > Decimal::ZERO {
                // Bid: 略低于当前价格
                let bid_price = (price - Decimal::new(1, 2)).max(Decimal::ZERO);
                if bid_price > Decimal::ZERO {
                    bids.push(OrderBookLevel {
                        price: bid_price.to_string(),
                        quantity: volume.to_string(),
                    });
                }

                // Ask: 略高于当前价格
                let ask_price = (price + Decimal::new(1, 2)).min(Decimal::ONE);
                if ask_price <= Decimal::ONE {
                    asks.push(OrderBookLevel {
                        price: ask_price.to_string(),
                        quantity: volume.to_string(),
                    });
                }
            }
        }

        // 按价格排序
        bids.sort_by(|a, b| {
            let pa: Decimal = a.price.parse().unwrap_or_default();
            let pb: Decimal = b.price.parse().unwrap_or_default();
            pb.cmp(&pa) // 价格降序
        });
        asks.sort_by(|a, b| {
            let pa: Decimal = a.price.parse().unwrap_or_default();
            let pb: Decimal = b.price.parse().unwrap_or_default();
            pa.cmp(&pb) // 价格升序
        });

        // 限制深度
        let depth = req.depth as usize;
        let bids = bids.into_iter().take(depth).collect();
        let asks = asks.into_iter().take(depth).collect();

        Ok(Response::new(GetMarketDepthResponse {
            market_id: req.market_id,
            bids,
            asks,
        }))
    }

    async fn resolve_market(
        &self,
        request: Request<ResolveMarketRequest>,
    ) -> Result<Response<ResolveMarketResponse>, Status> {
        let req = request.into_inner();

        let market = MarketRepository::find_by_id(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Market not found"))?;

        if market.status != MarketStatus::Open && market.status != MarketStatus::Closed {
            return Err(Status::failed_precondition("Market cannot be resolved in current status"));
        }

        let outcomes = OutcomeRepository::find_by_market(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let _winning_outcome = outcomes.iter()
            .find(|o| o.id == req.outcome_id)
            .ok_or_else(|| Status::not_found("Outcome not found"))?;

        let now = chrono::Utc::now().timestamp_millis();
        MarketRepository::update_status(
            &self.pool,
            req.market_id,
            "resolved",
            Some(req.outcome_id),
            Some(now),
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        // ========== 派彩：将赢家持仓结算到 Portfolio Service ==========
        let winning_positions = PositionRepository::find_by_market_and_outcome(
            &self.pool,
            req.market_id,
            req.outcome_id,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        // 按 user_id 汇总数量
        let mut user_quantities: HashMap<i64, Decimal> = HashMap::new();
        for pos in &winning_positions {
            *user_quantities.entry(pos.user_id).or_insert(Decimal::ZERO) += pos.quantity;
        }

        // 构建 Portfolio Service 的 UserPayout 列表
        let mut total_payout = Decimal::ZERO;
        let mut total_quantity = Decimal::ZERO;
        let payouts: Vec<api::portfolio::UserPayout> = user_quantities
            .iter()
            .map(|(user_id, quantity)| {
                total_payout += quantity;
                total_quantity += quantity;
                api::portfolio::UserPayout {
                    user_id: user_id.to_string(),
                    quantity: quantity.to_string(),
                    payout_amount: quantity.to_string(), // 1 USDC / 份
                }
            })
            .collect();

        if !payouts.is_empty() {
            let settle_req = tonic::Request::new(api::portfolio::SettleMarketRequest {
                market_id: req.market_id as u64,
                winning_outcome_id: req.outcome_id as u64,
                payouts,
            });

            self.portfolio_client
                .clone()
                .settle_market(settle_req)
                .await
                .map_err(|e| Status::internal(format!("Failed to settle market: {}", e)))?;

            tracing::info!(
                "Market {} settled: {} users, total payout {}",
                req.market_id,
                user_quantities.len(),
                total_payout,
            );
        } else {
            tracing::info!("Market {} resolved: no winning positions to settle", req.market_id);
        }

        // 发布市场事件到消息队列
        self.publish_market_event(
            req.market_id,
            req.outcome_id,
            &total_payout.to_string(),
            &total_quantity.to_string(),
            user_quantities.len() as i64,
            now,
        ).await;

        let resolution = PbMarketResolution {
            id: req.market_id,
            market_id: req.market_id,
            outcome_id: req.outcome_id,
            total_payout: total_payout.to_string(),
            winning_quantity: total_quantity.to_string(),
            payout_ratio: "1".to_string(),
            resolved_at: now,
        };

        let response = ResolveMarketResponse {
            success: true,
            message: "Market resolved successfully".to_string(),
            resolution: Some(resolution),
        };

        Ok(Response::new(response))
    }

    async fn calculate_payout(
        &self,
        request: Request<CalculatePayoutRequest>,
    ) -> Result<Response<CalculatePayoutResponse>, Status> {
        let req = request.into_inner();

        let market = MarketRepository::find_by_id(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Market not found"))?;

        if market.status != MarketStatus::Resolved {
            return Err(Status::failed_precondition("Market is not resolved yet"));
        }

        let winning_outcome_id = market.resolved_outcome_id
            .ok_or_else(|| Status::failed_precondition("Winning outcome not set"))?;

        let positions = PositionRepository::find_by_user(
            &self.pool,
            req.user_id,
            Some(req.market_id),
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let mut total_payout = Decimal::ZERO;
        let mut position_payouts = Vec::new();

        for position in positions {
            let quantity = position.quantity;
            let avg_price = position.avg_price;

            let payout = if position.outcome_id == winning_outcome_id {
                // 获胜选项：按 1 USDC/份 赔付
                // 赔付 = 数量 * (1 - 平均价格) + 数量 * 平均价格 = 数量
                // 实际上预测市场赢家获得 1 USDC/份
                quantity
            } else {
                // 失败选项：赔付为 0
                Decimal::ZERO
            };

            if payout > Decimal::ZERO {
                total_payout += payout;
            }

            position_payouts.push(UserPositionPayout {
                outcome_id: position.outcome_id,
                outcome_name: format!("Outcome {}", position.outcome_id),
                quantity: quantity.to_string(),
                avg_price: avg_price.to_string(),
                payout: payout.to_string(),
            });
        }

        Ok(Response::new(CalculatePayoutResponse {
            market_id: req.market_id,
            user_id: req.user_id,
            total_payout: total_payout.to_string(),
            positions: position_payouts,
        }))
    }

    async fn get_user_positions(
        &self,
        request: Request<GetUserPositionsRequest>,
    ) -> Result<Response<GetUserPositionsResponse>, Status> {
        let req = request.into_inner();

        let positions = PositionRepository::find_by_user(
            &self.pool,
            req.user_id,
            if req.market_id == 0 { None } else { Some(req.market_id) },
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let pb_positions: Vec<PbUserPosition> = positions
            .into_iter()
            .map(|p| PbUserPosition {
                id: p.id,
                user_id: p.user_id,
                market_id: p.market_id,
                outcome_id: p.outcome_id,
                quantity: p.quantity.to_string(),
                avg_price: p.avg_price.to_string(),
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
            .collect();

        Ok(Response::new(GetUserPositionsResponse {
            positions: pb_positions,
        }))
    }
}

// 辅助方法：模型转换 + 事件发布
impl MarketService {
    fn model_to_pb_market(&self, m: PredictionMarket) -> PbPredictionMarket {
        PbPredictionMarket {
            id: m.id,
            question: m.question,
            description: m.description.unwrap_or_default(),
            category: m.category,
            image_url: m.image_url.unwrap_or_default(),
            start_time: m.start_time,
            end_time: m.end_time,
            status: m.status.to_string(),
            resolved_outcome_id: m.resolved_outcome_id.unwrap_or(0),
            resolved_at: m.resolved_at.unwrap_or(0),
            total_volume: m.total_volume.to_string(),
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }

    fn model_to_pb_outcome(&self, o: MarketOutcome) -> PbMarketOutcome {
        PbMarketOutcome {
            id: o.id,
            market_id: o.market_id,
            name: o.name,
            description: o.description.unwrap_or_default(),
            image_url: o.image_url.unwrap_or_default(),
            price: o.price.to_string(),
            volume: o.volume.to_string(),
            probability: o.probability.to_string(),
            created_at: o.created_at,
            updated_at: o.updated_at,
        }
    }

    /// 发布市场事件到消息队列
    async fn publish_market_event(
        &self,
        market_id: i64,
        outcome_id: i64,
        total_payout: &str,
        winning_quantity: &str,
        payout_count: i64,
        resolved_at: i64,
    ) {
        if let Some(ref producer) = self.event_producer {
            // market_status 事件
            let status_event = serde_json::json!({
                "type": "market_status",
                "market_id": market_id,
                "data": {
                    "status": "resolved",
                    "winning_outcome_id": outcome_id,
                    "resolved_at": resolved_at,
                }
            });
            let msg = queue::Message {
                key: Some(market_id.to_string()),
                value: serde_json::to_string(&status_event).unwrap_or_default(),
            };
            if let Err(e) = producer.send("market_events", msg).await {
                tracing::warn!("Failed to publish market_status event: {}", e);
            }

            // settlement 事件
            let settlement_event = serde_json::json!({
                "type": "settlement",
                "market_id": market_id,
                "data": {
                    "winning_outcome_id": outcome_id,
                    "total_payout": total_payout,
                    "winning_quantity": winning_quantity,
                    "payout_count": payout_count,
                    "resolved_at": resolved_at,
                }
            });
            let msg = queue::Message {
                key: Some(market_id.to_string()),
                value: serde_json::to_string(&settlement_event).unwrap_or_default(),
            };
            if let Err(e) = producer.send("settlement_events", msg).await {
                tracing::warn!("Failed to publish settlement event: {}", e);
            }

            tracing::info!("Published market events for market {}", market_id);
        }
    }
}
