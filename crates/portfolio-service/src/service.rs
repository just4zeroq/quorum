//! Portfolio Service gRPC 实现
//!
//! 实现 proto 定义的 PortfolioService trait，将 gRPC 请求转发到领域服务
//! (AccountService / PositionService / ClearingService / LedgerService)

use rust_decimal::Decimal;
use tonic::{Request, Response, Status};

use crate::account::AccountService;
use crate::errors::PortfolioError;
use crate::ledger::LedgerService;
use crate::models::{LedgerType, PositionSide, Position, Settlement, SettlementStatus, LedgerEntry};
use crate::position::PositionService;
use crate::repository::PortfolioRepository;

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
        PortfolioError::OptimisticLockFailed(_) => Status::aborted(e.to_string()),
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
    /// 完整流程（原子事务）:
    /// 1. 买家扣款 (price * size + taker_fee)
    /// 2. 卖家收款 (price * size - maker_fee)
    /// 3. 买家开仓 (Long)
    /// 4. 卖家开仓 (Short)
    /// 5. 记录 Settlement
    /// 6. 记录账本流水
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
        let buyer_debit = amount + taker_fee;
        let seller_credit = amount - maker_fee;
        let outcome_id = req.outcome_id;
        let now = chrono::Utc::now();

        // 所有操作在一个数据库事务中
        let mut tx = self.repo.begin_tx().await.map_err(map_error)?;

        // 1. 买家扣款（乐观锁）
        let buyer_acct = tx.get_or_create_account(&req.buyer_id, "USDC").await.map_err(map_error)?;
        if buyer_acct.available < buyer_debit {
            return Err(map_error(PortfolioError::InsufficientBalance {
                available: buyer_acct.available.to_string(),
                required: buyer_debit.to_string(),
            }));
        }
        let rows = tx
            .debit_available_with_version(&req.buyer_id, "USDC", buyer_debit, buyer_acct.version)
            .await
            .map_err(map_error)?;
        if rows == 0 {
            return Err(map_error(PortfolioError::OptimisticLockFailed(
                "buyer debit conflict in settle_trade".into(),
            )));
        }
        let buyer_balance_after = buyer_acct.available - buyer_debit;

        // 2. 卖家收款（乐观锁）
        let seller_acct = tx.get_or_create_account(&req.seller_id, "USDC").await.map_err(map_error)?;
        let rows = tx
            .credit_with_version(&req.seller_id, "USDC", seller_credit, seller_acct.version)
            .await
            .map_err(map_error)?;
        if rows == 0 {
            return Err(map_error(PortfolioError::OptimisticLockFailed(
                "seller credit conflict in settle_trade".into(),
            )));
        }
        let seller_balance_after = seller_acct.available + seller_credit;

        // 3. 买家开仓 (Long)
        {
            let existing = tx
                .get_position(&req.buyer_id, req.market_id as i64, outcome_id as i64, "long")
                .await
                .map_err(map_error)?;
            let mut pos = if let Some(mut p) = existing {
                let total_cost = p.entry_price * p.size + price * size;
                p.size += size;
                p.entry_price = total_cost / p.size;
                p.version += 1;
                p.updated_at = now;
                p
            } else {
                Position {
                    id: uuid::Uuid::new_v4().to_string(),
                    user_id: req.buyer_id.clone(),
                    market_id: req.market_id,
                    outcome_id,
                    side: PositionSide::Long,
                    size,
                    entry_price: price,
                    version: 0,
                    created_at: now,
                    updated_at: now,
                }
            };
            if !tx.upsert_position_with_version(&mut pos).await.map_err(map_error)? {
                return Err(map_error(PortfolioError::OptimisticLockFailed(
                    "buyer position conflict in settle_trade".into(),
                )));
            }
        }

        // 4. 卖家开仓 (Short)
        {
            let existing = tx
                .get_position(&req.seller_id, req.market_id as i64, outcome_id as i64, "short")
                .await
                .map_err(map_error)?;
            let mut pos = if let Some(mut p) = existing {
                let total_cost = p.entry_price * p.size + price * size;
                p.size += size;
                p.entry_price = total_cost / p.size;
                p.version += 1;
                p.updated_at = now;
                p
            } else {
                Position {
                    id: uuid::Uuid::new_v4().to_string(),
                    user_id: req.seller_id.clone(),
                    market_id: req.market_id,
                    outcome_id,
                    side: PositionSide::Short,
                    size,
                    entry_price: price,
                    version: 0,
                    created_at: now,
                    updated_at: now,
                }
            };
            if !tx.upsert_position_with_version(&mut pos).await.map_err(map_error)? {
                return Err(map_error(PortfolioError::OptimisticLockFailed(
                    "seller position conflict in settle_trade".into(),
                )));
            }
        }

        // 5. 记录 Settlements
        let buyer_settlement = Settlement {
            id: uuid::Uuid::new_v4().to_string(),
            trade_id: req.trade_id.clone(),
            market_id: req.market_id,
            user_id: req.buyer_id.clone(),
            outcome_id,
            side: PositionSide::Long,
            amount,
            fee: taker_fee,
            payout: Decimal::ZERO,
            status: SettlementStatus::Completed,
            created_at: now,
        };
        let seller_settlement = Settlement {
            id: uuid::Uuid::new_v4().to_string(),
            trade_id: req.trade_id.clone(),
            market_id: req.market_id,
            user_id: req.seller_id.clone(),
            outcome_id,
            side: PositionSide::Short,
            amount,
            fee: maker_fee,
            payout: Decimal::ZERO,
            status: SettlementStatus::Completed,
            created_at: now,
        };
        tx.insert_settlement(&buyer_settlement).await.map_err(map_error)?;
        tx.insert_settlement(&seller_settlement).await.map_err(map_error)?;

        // 6. 记录账本流水
        tx.insert_ledger(&LedgerEntry {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: req.buyer_id.clone(),
            account_id: buyer_acct.id.clone(),
            ledger_type: LedgerType::Trade,
            asset: "USDC".to_string(),
            amount: buyer_debit,
            balance_after: buyer_balance_after,
            reference_id: req.trade_id.clone(),
            reference_type: "trade".to_string(),
            created_at: now,
        }).await.map_err(map_error)?;

        tx.insert_ledger(&LedgerEntry {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: req.seller_id.clone(),
            account_id: seller_acct.id.clone(),
            ledger_type: LedgerType::Trade,
            asset: "USDC".to_string(),
            amount: seller_credit,
            balance_after: seller_balance_after,
            reference_id: req.trade_id.clone(),
            reference_type: "trade".to_string(),
            created_at: now,
        }).await.map_err(map_error)?;

        // Commit 事务
        tx.commit().await.map_err(map_error)?;

        let settlement_id = buyer_settlement.id;

        Ok(Response::new(SettleTradeResponse {
            success: true,
            settlement_id,
        }))
    }

    /// 结算市场赔付
    ///
    /// 向所有获胜用户分发赔付资金（1 USDC / 份）
    /// 所有 payout 在同一个数据库事务中处理
    async fn settle_market(
        &self,
        request: Request<SettleMarketRequest>,
    ) -> Result<Response<SettleMarketResponse>, Status> {
        let req = request.into_inner();

        // 解析所有 payout 金额，提前校验
        let mut payouts = Vec::new();
        for payout in &req.payouts {
            let payout_amount: Decimal = payout.payout_amount.parse()
                .map_err(|e| Status::invalid_argument(format!("Invalid payout amount: {}", e)))?;
            if payout_amount > Decimal::ZERO {
                payouts.push((payout.user_id.clone(), payout_amount));
            }
        }

        let now = chrono::Utc::now();

        // 所有 payout 在同一事务中处理
        let mut tx = self.repo.begin_tx().await.map_err(map_error)?;

        let mut credited = 0i32;
        for (user_id, payout_amount) in &payouts {
            // 1. 创建或获取 USDC 账户
            let account = tx
                .get_or_create_account(user_id, "USDC")
                .await
                .map_err(map_error)?;

            // 2. 信用入账（乐观锁）
            let rows = tx
                .credit_with_version(user_id, "USDC", *payout_amount, account.version)
                .await
                .map_err(map_error)?;
            if rows == 0 {
                return Err(map_error(PortfolioError::OptimisticLockFailed(
                    format!("credit conflict for user {} in settle_market", user_id),
                )));
            }
            let balance_after = account.available + payout_amount;

            // 3. 记录 Settlement
            let settlement = Settlement {
                id: uuid::Uuid::new_v4().to_string(),
                trade_id: format!("market_{}_settle", req.market_id),
                market_id: req.market_id,
                user_id: user_id.clone(),
                outcome_id: req.winning_outcome_id,
                side: PositionSide::Long,
                amount: *payout_amount,
                fee: Decimal::ZERO,
                payout: *payout_amount,
                status: SettlementStatus::Completed,
                created_at: now,
            };
            tx.insert_settlement(&settlement).await.map_err(map_error)?;

            // 4. 记录账本流水
            tx.insert_ledger(&LedgerEntry {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.clone(),
                account_id: account.id.clone(),
                ledger_type: LedgerType::Settle,
                asset: "USDC".to_string(),
                amount: *payout_amount,
                balance_after,
                reference_id: format!("market_{}", req.market_id),
                reference_type: "market_settlement".to_string(),
                created_at: now,
            }).await.map_err(map_error)?;

            credited += 1;
        }

        // Commit 事务
        tx.commit().await.map_err(map_error)?;

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
