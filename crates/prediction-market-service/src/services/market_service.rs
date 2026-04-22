//! 预测市场服务 - gRPC 实现

use std::sync::Arc;
use tonic::{Request, Response, Status};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::models::{PredictionMarket, MarketOutcome, MarketStatus};
use crate::repository::{MarketRepository, OutcomeRepository};
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
};

pub struct MarketService {
    pool: sqlx::PgPool,
}

impl MarketService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl PredictionMarketService for MarketService {
    async fn create_market(
        &self,
        request: Request<CreateMarketRequest>,
    ) -> Result<Response<CreateMarketResponse>, Status> {
        let req = request.into_inner();

        // 验证参数
        if req.question.is_empty() {
            return Err(Status::invalid_argument("Question cannot be empty"));
        }
        if req.outcomes.is_empty() {
            return Err(Status::invalid_argument("At least one outcome is required"));
        }

        // 创建市场
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

        // 创建选项
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

    async fn resolve_market(
        &self,
        request: Request<ResolveMarketRequest>,
    ) -> Result<Response<ResolveMarketResponse>, Status> {
        let req = request.into_inner();

        // 检查市场是否存在
        let market = MarketRepository::find_by_id(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Market not found"))?;

        // 检查市场状态
        if market.status != MarketStatus::Open {
            return Err(Status::failed_precondition("Market is not open"));
        }

        // 检查选项是否存在
        let outcomes = OutcomeRepository::find_by_market(&self.pool, req.market_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let _winning_outcome = outcomes.iter()
            .find(|o| o.id == req.outcome_id)
            .ok_or_else(|| Status::not_found("Outcome not found"))?;

        // 更新市场状态
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

        let response = ResolveMarketResponse {
            success: true,
            message: "Market resolved successfully".to_string(),
            resolution: None,
        };

        Ok(Response::new(response))
    }

    // 未实现的接口返回默认值
    async fn update_market(
        &self,
        _request: Request<UpdateMarketRequest>,
    ) -> Result<Response<UpdateMarketResponse>, Status> {
        Err(Status::unimplemented("UpdateMarket not implemented"))
    }

    async fn close_market(
        &self,
        _request: Request<CloseMarketRequest>,
    ) -> Result<Response<CloseMarketResponse>, Status> {
        Err(Status::unimplemented("CloseMarket not implemented"))
    }

    async fn add_outcome(
        &self,
        _request: Request<AddOutcomeRequest>,
    ) -> Result<Response<AddOutcomeResponse>, Status> {
        Err(Status::unimplemented("AddOutcome not implemented"))
    }

    async fn get_market_price(
        &self,
        _request: Request<GetMarketPriceRequest>,
    ) -> Result<Response<GetMarketPriceResponse>, Status> {
        Err(Status::unimplemented("GetMarketPrice not implemented"))
    }

    async fn get_market_depth(
        &self,
        _request: Request<GetMarketDepthRequest>,
    ) -> Result<Response<GetMarketDepthResponse>, Status> {
        Err(Status::unimplemented("GetMarketDepth not implemented"))
    }

    async fn calculate_payout(
        &self,
        _request: Request<CalculatePayoutRequest>,
    ) -> Result<Response<CalculatePayoutResponse>, Status> {
        Err(Status::unimplemented("CalculatePayout not implemented"))
    }

    async fn get_user_positions(
        &self,
        _request: Request<GetUserPositionsRequest>,
    ) -> Result<Response<GetUserPositionsResponse>, Status> {
        Err(Status::unimplemented("GetUserPositions not implemented"))
    }
}

// 辅助方法：模型转换
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
}