//! Risk Service gRPC 实现

use rust_decimal::Decimal;
use tonic::{Request, Response, Status};

use crate::rules::{self, RiskCheckContext};

use api::risk::risk_service_server::RiskService;
use api::risk::{CheckRiskRequest, CheckRiskResponse};

/// RiskService gRPC 实现
#[derive(Debug, Default)]
pub struct RiskServiceImpl;

#[tonic::async_trait]
impl RiskService for RiskServiceImpl {
    async fn check_risk(
        &self,
        request: Request<CheckRiskRequest>,
    ) -> Result<Response<CheckRiskResponse>, Status> {
        let req = request.into_inner();

        let price: Decimal = req
            .price
            .parse()
            .map_err(|e| Status::invalid_argument(format!("Invalid price: {}", e)))?;
        let quantity: Decimal = req
            .quantity
            .parse()
            .map_err(|e| Status::invalid_argument(format!("Invalid quantity: {}", e)))?;

        let ctx = RiskCheckContext {
            user_id: req.user_id,
            market_id: req.market_id,
            outcome_id: req.outcome_id,
            side: req.side,
            order_type: req.order_type,
            price,
            quantity,
        };

        let result = rules::evaluate(&ctx);

        Ok(Response::new(CheckRiskResponse {
            accepted: result.accepted,
            reason: result.reason,
        }))
    }
}
