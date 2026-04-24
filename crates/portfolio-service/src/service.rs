//! Portfolio Service gRPC 实现
//!
//! 实现 proto 定义的 PortfolioService trait，将 gRPC 请求转发到领域服务
//! (AccountService / PositionService / ClearingService / LedgerService)

use rust_decimal::Decimal;
use tonic::{Request, Response, Status};

use crate::account::AccountService;
use crate::clearing::ClearingService;
use crate::errors::PortfolioError;
use crate::ledger::LedgerService;
use crate::models::{LedgerType, PositionSide};
use crate::position::PositionService;
use crate::repository::PortfolioRepository;
use crate::clearing::TradeInfo;

use api::portfolio::portfolio_service_server::PortfolioService;
use api::portfolio::{
    GetBalanceRequest, GetBalanceResponse,
    CreditRequest, CreditResponse,
    DebitRequest, DebitResponse,
    FreezeRequest, FreezeResponse,
    UnfreezeRequest, UnfreezeResponse,
    GetPositionsRequest, GetPositionsResponse,
    GetPositionRequest, GetPositionResponse,
    SettleTradeRequest, SettleTradeResponse,
    SettleMarketRequest, SettleMarketResponse,
    GetLedgerRequest, GetLedgerResponse,
    Position as ProtoPosition,
    LedgerEntry as ProtoLedgerEntry,
};

/// PortfolioService gRPC 实现
#[derive(Debug)]
pub struct PortfolioServiceImpl {
    repo: PortfolioRepository,
}

impl PortfolioServiceImpl {
    pub fn new(repo: PortfolioRepository) -> Self {
        Self { repo }
    }
}

// ==================== Helpers ====================

fn parse_decimal(s: &str, field: &str) -> Result<Decimal, Status> {
    s.parse::<Decimal>()
        .map_err(|e| Status::invalid_argument(format!("Invalid {}: {}", field, e)))
}

fn parse_side(s: &str) -> Result<PositionSide, Status> {
    match s.to_lowercase().as_str() {
        "long" | "yes" => Ok(PositionSide::Long),
        "short" | "no" => Ok(PositionSide::Short),
        _ => Err(Status::invalid_argument(format!("Invalid side: {}", s))),
    }
}

fn map_error(e: PortfolioError) -> Status {
    match &e {
        PortfolioError::AccountNotFound(_) => Status::not_found(e.to_string()),
        PortfolioError::InsufficientBalance { .. } => Status::failed_precondition(e.to_string()),
        PortfolioError::PositionNotFound(_) => Status::not_found(e.to_string()),
        PortfolioError::InsufficientPosition { .. } => Status::failed_precondition(e.to_string()),
        PortfolioError::SettlementFailed(_) => Status::internal(e.to_string()),
        PortfolioError::LedgerFailed(_) => Status::internal(e.to_string()),
        PortfolioError::InvalidOperation(_) => Status::invalid_argument(e.to_string()),
        PortfolioError::Database(_) => Status::internal(e.to_string()),
    }
}

fn pos_to_proto(pos: crate::models::Position) -> ProtoPosition {
    ProtoPosition {
        id: pos.id,
        market_id: pos.market_id,
        outcome_id: pos.outcome_id,
        side: pos.side.as_str().to_string(),
        size: pos.size.to_string(),
        entry_price: pos.entry_price.to_string(),
    }
}

fn ledger_to_proto(entry: crate::models::LedgerEntry) -> ProtoLedgerEntry {
    ProtoLedgerEntry {
        id: entry.id,
        ledger_type: entry.ledger_type.as_str().to_string(),
        asset: entry.asset,
        amount: entry.amount.to_string(),
        balance_after: entry.balance_after.to_string(),
        reference_id: entry.reference_id,
        created_at: entry.created_at.to_rfc3339(),
    }
}

// ==================== gRPC Trait Implementation ====================

#[tonic::async_trait]
impl PortfolioService for PortfolioServiceImpl {
    async fn get_balance(
        &self,
        request: Request<GetBalanceRequest>,
    ) -> Result<Response<GetBalanceResponse>, Status> {
        let req = request.into_inner();
        let svc = AccountService::new(self.repo.clone());

        let account = svc
            .get_or_create_account(&req.user_id, &req.asset)
            .await
            .map_err(map_error)?;

        Ok(Response::new(GetBalanceResponse {
            account_id: account.id,
            asset: account.asset,
            available: account.available.to_string(),
            frozen: account.frozen.to_string(),
        }))
    }

    async fn credit(
        &self,
        request: Request<CreditRequest>,
    ) -> Result<Response<CreditResponse>, Status> {
        let req = request.into_inner();
        let amount = parse_decimal(&req.amount, "amount")?;
        let svc = AccountService::new(self.repo.clone());

        let account = svc
            .credit(&req.user_id, &req.asset, amount)
            .await
            .map_err(map_error)?;

        Ok(Response::new(CreditResponse {
            success: true,
            available_after: account.available.to_string(),
        }))
    }

    async fn debit(
        &self,
        request: Request<DebitRequest>,
    ) -> Result<Response<DebitResponse>, Status> {
        let req = request.into_inner();
        let amount = parse_decimal(&req.amount, "amount")?;
        let svc = AccountService::new(self.repo.clone());

        let account = svc
            .debit_available(&req.user_id, &req.asset, amount)
            .await
            .map_err(map_error)?;

        Ok(Response::new(DebitResponse {
            success: true,
            available_after: account.available.to_string(),
        }))
    }

    async fn freeze(
        &self,
        request: Request<FreezeRequest>,
    ) -> Result<Response<FreezeResponse>, Status> {
        let req = request.into_inner();
        let amount = parse_decimal(&req.amount, "amount")?;
        let svc = AccountService::new(self.repo.clone());

        let account = svc
            .freeze(&req.user_id, &req.asset, amount, &req.order_id)
            .await
            .map_err(map_error)?;

        Ok(Response::new(FreezeResponse {
            success: true,
            available_after: account.available.to_string(),
            frozen_after: account.frozen.to_string(),
        }))
    }

    async fn unfreeze(
        &self,
        request: Request<UnfreezeRequest>,
    ) -> Result<Response<UnfreezeResponse>, Status> {
        let req = request.into_inner();
        let amount = parse_decimal(&req.amount, "amount")?;
        let svc = AccountService::new(self.repo.clone());

        svc.unfreeze(&req.user_id, &req.asset, amount, &req.order_id)
            .await
            .map_err(map_error)?;

        Ok(Response::new(UnfreezeResponse { success: true }))
    }

    async fn get_positions(
        &self,
        request: Request<GetPositionsRequest>,
    ) -> Result<Response<GetPositionsResponse>, Status> {
        let req = request.into_inner();
        let positions = self.repo
            .list_positions(&req.user_id, req.market_id)
            .await
            .map_err(map_error)?;

        Ok(Response::new(GetPositionsResponse {
            positions: positions.into_iter().map(pos_to_proto).collect(),
        }))
    }

    async fn get_position(
        &self,
        request: Request<GetPositionRequest>,
    ) -> Result<Response<GetPositionResponse>, Status> {
        let req = request.into_inner();
        let side = parse_side(&req.side)?;
        let svc = PositionService::new(self.repo.clone());

        let position = svc
            .get_position(&req.user_id, req.market_id, req.outcome_id, side)
            .await
            .map_err(map_error)?;

        Ok(Response::new(GetPositionResponse {
            position: position.map(pos_to_proto),
        }))
    }

    /// 结算一笔成交
    ///
    /// 完整流程:
    /// 1. 买家扣款 (price * size + taker_fee)
    /// 2. 卖家收款 (price * size - maker_fee)
    /// 3. 买家开仓 (Long)
    /// 4. 卖家开仓 (Short)
    /// 5. 记录 Settlement
    /// 6. 记录账本流水
    ///
    /// TODO: 需要数据库事务保证原子性
    async fn settle_trade(
        &self,
        request: Request<SettleTradeRequest>,
    ) -> Result<Response<SettleTradeResponse>, Status> {
        let req = request.into_inner();
        let price = parse_decimal(&req.price, "price")?;
        let size = parse_decimal(&req.size, "size")?;
        let taker_fee_rate = parse_decimal(&req.taker_fee_rate, "taker_fee_rate")?;
        let maker_fee_rate = parse_decimal(&req.maker_fee_rate, "maker_fee_rate")?;

        let amount = price * size;
        let taker_fee = amount * taker_fee_rate;
        let maker_fee = amount * maker_fee_rate;

        let account_svc = AccountService::new(self.repo.clone());
        let pos_svc = PositionService::new(self.repo.clone());
        let clearing_svc = ClearingService::new(self.repo.clone());

        // 1. 买家扣款
        let buyer_account = account_svc
            .debit(&req.buyer_id, "USDC", amount + taker_fee)
            .await
            .map_err(map_error)?;

        // 2. 卖家收款
        let seller_account = account_svc
            .credit(&req.seller_id, "USDC", amount - maker_fee)
            .await
            .map_err(map_error)?;

        // 3. 买家开仓 (Long)
        pos_svc
            .open_or_add_position(
                &req.buyer_id,
                req.market_id,
                req.outcome_id,
                PositionSide::Long,
                size,
                price,
            )
            .await
            .map_err(map_error)?;

        // 4. 卖家开仓 (Short)
        pos_svc
            .open_or_add_position(
                &req.seller_id,
                req.market_id,
                req.outcome_id,
                PositionSide::Short,
                size,
                price,
            )
            .await
            .map_err(map_error)?;

        // 5. 记录 Settlement
        let trade_info = TradeInfo {
            trade_id: req.trade_id.clone(),
            market_id: req.market_id,
            outcome_id: req.outcome_id,
            buyer_id: req.buyer_id.clone(),
            seller_id: req.seller_id.clone(),
            price,
            size,
            taker_fee_rate,
            maker_fee_rate,
        };
        let settlements = clearing_svc
            .settle_trade(&trade_info)
            .await
            .map_err(map_error)?;

        // 6. 记录账本流水
        let ledger_svc = LedgerService::new(self.repo.clone());

        // 买家流水（支出）
        ledger_svc
            .record(
                &req.buyer_id,
                &buyer_account.id,
                LedgerType::Trade,
                "USDC",
                amount + taker_fee,
                buyer_account.available,
                &req.trade_id,
                "trade",
            )
            .await
            .map_err(map_error)?;

        // 卖家流水（收入）
        ledger_svc
            .record(
                &req.seller_id,
                &seller_account.id,
                LedgerType::Trade,
                "USDC",
                amount - maker_fee,
                seller_account.available,
                &req.trade_id,
                "trade",
            )
            .await
            .map_err(map_error)?;

        let settlement_id = settlements
            .first()
            .map(|s| s.id.clone())
            .unwrap_or_default();

        Ok(Response::new(SettleTradeResponse {
            success: true,
            settlement_id,
        }))
    }

    /// 结算市场赔付
    ///
    /// 向所有获胜用户分发赔付资金（1 USDC / 份）
    async fn settle_market(
        &self,
        request: Request<SettleMarketRequest>,
    ) -> Result<Response<SettleMarketResponse>, Status> {
        let req = request.into_inner();
        let account_svc = AccountService::new(self.repo.clone());
        let ledger_svc = LedgerService::new(self.repo.clone());

        let mut credited = 0i32;

        for payout in req.payouts {
            let payout_amount: Decimal = payout.payout_amount.parse()
                .map_err(|e| Status::invalid_argument(format!("Invalid payout amount: {}", e)))?;

            if payout_amount <= Decimal::ZERO {
                continue;
            }

            // 1. 创建或获取 USDC 账户
            let _account = account_svc
                .get_or_create_account(&payout.user_id, "USDC")
                .await
                .map_err(map_error)?;

            // 2. 信用入账
            account_svc
                .credit(&payout.user_id, "USDC", payout_amount)
                .await
                .map_err(map_error)?;

            // 3. 获取更新后的账户
            let updated = account_svc
                .get_or_create_account(&payout.user_id, "USDC")
                .await
                .map_err(map_error)?;

            // 4. 记录 Settlement
            let settlement = crate::models::Settlement {
                id: uuid::Uuid::new_v4().to_string(),
                trade_id: format!("market_{}_settle", req.market_id),
                market_id: req.market_id,
                user_id: payout.user_id.clone(),
                outcome_id: req.winning_outcome_id,
                side: crate::models::PositionSide::Long,
                amount: payout_amount,
                fee: Decimal::ZERO,
                payout: payout_amount,
                status: crate::models::SettlementStatus::Completed,
                created_at: chrono::Utc::now(),
            };
            self.repo.insert_settlement(&settlement).await
                .map_err(map_error)?;

            // 5. 记录账本流水
            ledger_svc
                .record(
                    &payout.user_id,
                    &updated.id,
                    crate::models::LedgerType::Settle,
                    "USDC",
                    payout_amount,
                    updated.available,
                    &format!("market_{}", req.market_id),
                    "market_settlement",
                )
                .await
                .map_err(map_error)?;

            credited += 1;
        }

        tracing::info!(
            "Market {} settled: credited {} users, winning outcome {}",
            req.market_id, credited, req.winning_outcome_id
        );

        Ok(Response::new(SettleMarketResponse {
            success: true,
            users_credited: credited,
        }))
    }

    async fn get_ledger(
        &self,
        request: Request<GetLedgerRequest>,
    ) -> Result<Response<GetLedgerResponse>, Status> {
        let req = request.into_inner();
        let svc = LedgerService::new(self.repo.clone());

        let entries = svc
            .get_user_entries(&req.user_id, req.limit, req.offset)
            .await
            .map_err(map_error)?;

        Ok(Response::new(GetLedgerResponse {
            entries: entries.into_iter().map(ledger_to_proto).collect(),
        }))
    }
}
