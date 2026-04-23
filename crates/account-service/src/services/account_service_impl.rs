//! Account Service 实现
//!
//! 实现 gRPC 服务定义的所有方法

use db::DBPool;
use tonic::{Request, Response, Status};

use crate::error::{Error, Result};
use crate::models::{Account, BalanceOperation, BalanceOperationType};
use crate::precision::AssetPrecision;
use crate::repository::{AccountRepository, OperationRepository};
use crate::pb::account_service_server::account_service_server::{
    AccountService, GetBalanceRequest, GetBalanceResponse, GetBalancesRequest, GetBalancesResponse,
    Balance, FreezeRequest, FreezeResponse, UnfreezeRequest, UnfreezeResponse,
    FreezeAndDeductRequest, FreezeAndDeductResponse, CheckAndFreezeRequest, CheckAndFreezeResponse,
    DepositRequest, DepositResponse, WithdrawRequest, WithdrawResponse,
    TransferRequest, TransferResponse, LockRequest, LockResponse,
    UnlockRequest, UnlockResponse, CheckBalanceRequest, CheckBalanceResponse,
    CheckFrozenRequest, CheckFrozenResponse, SettleRequest, SettleResponse,
    BatchGetBalancesRequest, BatchGetBalancesResponse,
};

/// Account Service 实现
pub struct AccountServiceImpl {
    pool: DBPool,
    precision: AssetPrecision,
}

impl AccountServiceImpl {
    /// 创建新的 AccountServiceImpl
    pub fn new(pool: DBPool, precision: AssetPrecision) -> Self {
        Self { pool, precision }
    }

    /// 检查金额是否有效
    fn validate_amount(&self, amount: i64) -> Result<()> {
        if amount <= 0 {
            return Err(Error::AmountInvalid(format!("Amount must be positive: {}", amount)));
        }
        Ok(())
    }

    /// 获取账户，带错误转换
    async fn get_account(&self, user_id: i64, asset: &str) -> Result<Account> {
        AccountRepository::get(&self.pool, user_id, asset)
            .await?
            .ok_or_else(|| Error::AccountNotFound(user_id, asset.to_string()))
    }

    /// 获取或创建账户
    async fn get_or_create_account(&self, user_id: i64, asset: &str, precision: u8) -> Result<Account> {
        Ok(AccountRepository::get_or_create(&self.pool, user_id, asset, precision).await?)
    }

    /// 记录操作
    async fn record_operation(&self, op: &mut BalanceOperation) -> Result<()> {
        let id = OperationRepository::record(&self.pool, op).await?;
        op.id = id;
        Ok(())
    }
}

#[tonic::async_trait]
impl AccountService for AccountServiceImpl {
    /// 获取单个资产余额
    async fn get_balance(
        &self,
        request: Request<GetBalanceRequest>,
    ) -> Result<Response<GetBalanceResponse>, Status> {
        let req = request.into_inner();

        // 获取或创建账户
        let precision = if req.precision > 0 {
            req.precision as u8
        } else {
            self.precision.get_precision(&req.asset)
        };

        let account = self.get_or_create_account(req.user_id, &req.asset, precision).await?;

        Ok(Response::new(GetBalanceResponse {
            success: true,
            message: "OK".to_string(),
            account_id: account.id,
            user_id: account.user_id,
            asset: account.asset,
            available: account.available,
            frozen: account.frozen,
            locked: account.locked,
            precision: account.precision as i32,
        }))
    }

    /// 获取用户所有资产余额
    async fn get_balances(
        &self,
        request: Request<GetBalancesRequest>,
    ) -> Result<Response<GetBalancesResponse>, Status> {
        let req = request.into_inner();

        let accounts = AccountRepository::get_by_user(&self.pool, req.user_id).await?;

        let balances = accounts.into_iter().map(|a| Balance {
            account_id: a.id,
            asset: a.asset,
            available: a.available,
            frozen: a.frozen,
            locked: a.locked,
            precision: a.precision as i32,
        }).collect();

        Ok(Response::new(GetBalancesResponse { balances }))
    }

    /// 冻结余额 (下单)
    async fn freeze(
        &self,
        request: Request<FreezeRequest>,
    ) -> Result<Response<FreezeResponse>, Status> {
        let req = request.into_inner();
        self.validate_amount(req.amount)?;

        let precision = if req.precision > 0 {
            req.precision as u8
        } else {
            self.precision.get_precision(&req.asset)
        };

        // 获取账户
        let mut account = self.get_or_create_account(req.user_id, &req.asset, precision).await?;

        // 检查可用余额
        if !account.has_sufficient_available(req.amount) {
            return Err(Error::InsufficientBalance(account.available, req.amount).into());
        }

        // 冻结
        let available_before = account.available;
        let frozen_before = account.frozen;
        account.available -= req.amount;
        account.frozen += req.amount;

        // 更新数据库
        AccountRepository::update_balance(&self.pool, account.id, account.available, account.frozen, account.locked).await?;

        // 记录操作
        let mut op = BalanceOperation::new(
            req.user_id,
            &req.asset,
            BalanceOperationType::Freeze,
            req.amount,
            available_before,
            account.available,
            frozen_before,
            account.frozen,
            &req.order_id,
        );
        op.account_id = account.id;
        if !req.reason.is_empty() {
            op = op.with_reason(&req.reason);
        }
        self.record_operation(&mut op).await?;

        Ok(Response::new(FreezeResponse {
            success: true,
            message: "OK".to_string(),
            account_id: account.id,
            available_before,
            available_after: account.available,
            frozen_before,
            frozen_after: account.frozen,
        }))
    }

    /// 解冻余额 (撤单)
    async fn unfreeze(
        &self,
        request: Request<UnfreezeRequest>,
    ) -> Result<Response<UnfreezeResponse>, Status> {
        let req = request.into_inner();
        self.validate_amount(req.amount)?;

        // 获取账户
        let mut account = self.get_account(req.user_id, &req.asset).await?;

        // 检查冻结余额
        if !account.has_sufficient_frozen(req.amount) {
            return Err(Error::InsufficientFrozen(account.frozen, req.amount).into());
        }

        // 解冻
        let available_before = account.available;
        let frozen_before = account.frozen;
        account.frozen -= req.amount;
        account.available += req.amount;

        // 更新数据库
        AccountRepository::update_balance(&self.pool, account.id, account.available, account.frozen, account.locked).await?;

        // 记录操作
        let mut op = BalanceOperation::new(
            req.user_id,
            &req.asset,
            BalanceOperationType::Unfreeze,
            req.amount,
            available_before,
            account.available,
            frozen_before,
            account.frozen,
            &req.order_id,
        );
        op.account_id = account.id;
        if !req.reason.is_empty() {
            op = op.with_reason(&req.reason);
        }
        self.record_operation(&mut op).await?;

        Ok(Response::new(UnfreezeResponse {
            success: true,
            message: "OK".to_string(),
            account_id: account.id,
            available_before,
            available_after: account.available,
            frozen_before,
            frozen_after: account.frozen,
        }))
    }

    /// 冻结并扣减 (成交)
    async fn freeze_and_deduct(
        &self,
        request: Request<FreezeAndDeductRequest>,
    ) -> Result<Response<FreezeAndDeductResponse>, Status> {
        let req = request.into_inner();

        // 获取基础资产账户
        let mut base_account = self.get_account(req.user_id, &req.asset).await?;

        // 检查冻结余额是否足够
        if !base_account.has_sufficient_frozen(req.freeze_amount) {
            return Err(Error::InsufficientFrozen(base_account.frozen, req.freeze_amount).into());
        }

        let available_before = base_account.available;
        let frozen_before = base_account.frozen;

        // 从 frozen 中扣减
        base_account.frozen -= req.freeze_amount;

        // 更新基础资产账户
        AccountRepository::update_balance(&self.pool, base_account.id, base_account.available, base_account.frozen, base_account.locked).await?;

        // 如果指定了结果代币，增加买方的结果代币余额
        if !req.outcome_asset.is_empty() {
            let outcome_precision = if req.outcome_precision > 0 {
                req.outcome_precision as u8
            } else {
                self.precision.outcome_precision()
            };

            let mut outcome_account = self.get_or_create_account(req.user_id, &req.outcome_asset, outcome_precision).await?;
            outcome_account.available += req.outcome_amount;

            AccountRepository::update_available(&self.pool, outcome_account.id, outcome_account.available).await?;

            // 记录结果代币增加操作
            let mut op = BalanceOperation::new(
                req.user_id,
                &req.outcome_asset,
                BalanceOperationType::TransferIn,
                req.outcome_amount,
                outcome_account.available - req.outcome_amount,
                outcome_account.available,
                outcome_account.frozen,
                outcome_account.frozen,
                &req.order_id,
            );
            op.account_id = outcome_account.id;
            self.record_operation(&mut op).await?;
        }

        // 记录基础资产扣减操作
        let mut op = BalanceOperation::new(
            req.user_id,
            &req.asset,
            BalanceOperationType::Deduct,
            req.freeze_amount,
            available_before,
            base_account.available,
            frozen_before,
            base_account.frozen,
            &req.order_id,
        );
        op.account_id = base_account.id;
        if !req.reason.is_empty() {
            op = op.with_reason(&req.reason);
        }
        self.record_operation(&mut op).await?;

        Ok(Response::new(FreezeAndDeductResponse {
            success: true,
            message: "OK".to_string(),
            account_id: base_account.id,
            available_before,
            available_after: base_account.available,
            frozen_before,
            frozen_after: base_account.frozen,
        }))
    }

    /// 原子操作：检查余额并冻结 (推荐)
    async fn check_and_freeze(
        &self,
        request: Request<CheckAndFreezeRequest>,
    ) -> Result<Response<CheckAndFreezeResponse>, Status> {
        let req = request.into_inner();
        self.validate_amount(req.amount)?;

        let precision = if req.precision > 0 {
            req.precision as u8
        } else {
            self.precision.get_precision(&req.asset)
        };

        // 获取或创建账户
        let mut account = self.get_or_create_account(req.user_id, &req.asset, precision).await?;

        // 检查余额
        if !account.has_sufficient_available(req.amount) {
            return Err(Error::InsufficientBalance(account.available, req.amount).into());
        }

        // 冻结
        let available_before = account.available;
        let frozen_before = account.frozen;
        account.available -= req.amount;
        account.frozen += req.amount;

        // 更新数据库
        AccountRepository::update_balance(&self.pool, account.id, account.available, account.frozen, account.locked).await?;

        // 记录操作
        let mut op = BalanceOperation::new(
            req.user_id,
            &req.asset,
            BalanceOperationType::Freeze,
            req.amount,
            available_before,
            account.available,
            frozen_before,
            account.frozen,
            &req.order_id,
        );
        op.account_id = account.id;
        if !req.reason.is_empty() {
            op = op.with_reason(&req.reason);
        }
        self.record_operation(&mut op).await?;

        Ok(Response::new(CheckAndFreezeResponse {
            success: true,
            message: "OK".to_string(),
            account_id: account.id,
            available_before,
            available_after: account.available,
            frozen_before,
            frozen_after: account.frozen,
        }))
    }

    /// 充值
    async fn deposit(
        &self,
        request: Request<DepositRequest>,
    ) -> Result<Response<DepositResponse>, Status> {
        let req = request.into_inner();
        self.validate_amount(req.amount)?;

        let precision = self.precision.get_precision(&req.asset);
        let mut account = self.get_or_create_account(req.user_id, &req.asset, precision).await?;

        let available_before = account.available;
        account.available += req.amount;

        // 更新数据库
        AccountRepository::update_balance(&self.pool, account.id, account.available, account.frozen, account.locked).await?;

        // 记录操作
        let mut op = BalanceOperation::new(
            req.user_id,
            &req.asset,
            BalanceOperationType::Deposit,
            req.amount,
            available_before,
            account.available,
            account.frozen,
            account.frozen,
            &req.tx_id,
        );
        op.account_id = account.id;
        if !req.reason.is_empty() {
            op = op.with_reason(&req.reason);
        }
        self.record_operation(&mut op).await?;

        Ok(Response::new(DepositResponse {
            success: true,
            message: "OK".to_string(),
            account_id: account.id,
            available_before,
            available_after: account.available,
        }))
    }

    /// 提现
    async fn withdraw(
        &self,
        request: Request<WithdrawRequest>,
    ) -> Result<Response<WithdrawResponse>, Status> {
        let req = request.into_inner();
        self.validate_amount(req.amount)?;

        let mut account = self.get_account(req.user_id, &req.asset).await?;

        // 检查可用余额
        if !account.has_sufficient_available(req.amount) {
            return Err(Error::InsufficientBalance(account.available, req.amount).into());
        }

        let available_before = account.available;
        account.available -= req.amount;

        // 更新数据库
        AccountRepository::update_balance(&self.pool, account.id, account.available, account.frozen, account.locked).await?;

        // 记录操作
        let mut op = BalanceOperation::new(
            req.user_id,
            &req.asset,
            BalanceOperationType::Withdraw,
            req.amount,
            available_before,
            account.available,
            account.frozen,
            account.frozen,
            &req.withdraw_id,
        );
        op.account_id = account.id;
        if !req.reason.is_empty() {
            op = op.with_reason(&req.reason);
        }
        self.record_operation(&mut op).await?;

        Ok(Response::new(WithdrawResponse {
            success: true,
            message: "OK".to_string(),
            account_id: account.id,
            available_before,
            available_after: account.available,
        }))
    }

    /// 内部划转
    async fn transfer(
        &self,
        request: Request<TransferRequest>,
    ) -> Result<Response<TransferResponse>, Status> {
        let req = request.into_inner();
        self.validate_amount(req.amount)?;

        let precision = self.precision.get_precision(&req.asset);

        // 获取转出账户
        let mut from_account = self.get_account(req.from_user_id, &req.asset).await?;

        // 检查可用余额
        if !from_account.has_sufficient_available(req.amount) {
            return Err(Error::InsufficientBalance(from_account.available, req.amount).into());
        }

        let from_available_before = from_account.available;
        from_account.available -= req.amount;

        // 更新转出账户
        AccountRepository::update_available(&self.pool, from_account.id, from_account.available).await?;

        // 获取转入账户
        let mut to_account = self.get_or_create_account(req.to_user_id, &req.asset, precision).await?;
        let to_available_before = to_account.available;
        to_account.available += req.amount;

        // 更新转入账户
        AccountRepository::update_available(&self.pool, to_account.id, to_account.available).await?;

        // 记录转出操作
        let mut op_out = BalanceOperation::new(
            req.from_user_id,
            &req.asset,
            BalanceOperationType::TransferOut,
            req.amount,
            from_available_before,
            from_account.available,
            from_account.frozen,
            from_account.frozen,
            &req.transfer_id,
        );
        op_out.account_id = from_account.id;
        if !req.reason.is_empty() {
            op_out = op_out.with_reason(&req.reason);
        }
        self.record_operation(&mut op_out).await?;

        // 记录转入操作
        let mut op_in = BalanceOperation::new(
            req.to_user_id,
            &req.asset,
            BalanceOperationType::TransferIn,
            req.amount,
            to_available_before,
            to_account.available,
            to_account.frozen,
            to_account.frozen,
            &req.transfer_id,
        );
        op_in.account_id = to_account.id;
        if !req.reason.is_empty() {
            op_in = op_in.with_reason(&req.reason);
        }
        self.record_operation(&mut op_in).await?;

        Ok(Response::new(TransferResponse {
            success: true,
            message: "OK".to_string(),
            from_available_before,
            from_available_after: from_account.available,
            to_available_before,
            to_available_after: to_account.available,
        }))
    }

    /// 风控锁定
    async fn lock(
        &self,
        request: Request<LockRequest>,
    ) -> Result<Response<LockResponse>, Status> {
        let req = request.into_inner();
        self.validate_amount(req.amount)?;

        let mut account = self.get_account(req.user_id, &req.asset).await?;

        let available_before = account.available;
        let locked_before = account.locked;

        account.available -= req.amount;
        account.locked += req.amount;

        // 更新数据库
        AccountRepository::update_balance(&self.pool, account.id, account.available, account.frozen, account.locked).await?;

        // 记录操作
        let mut op = BalanceOperation::new(
            req.user_id,
            &req.asset,
            BalanceOperationType::Lock,
            req.amount,
            available_before,
            account.available,
            account.frozen,
            account.frozen,
            "",
        );
        op.account_id = account.id;
        if !req.reason.is_empty() {
            op = op.with_reason(&req.reason);
        }
        self.record_operation(&mut op).await?;

        Ok(Response::new(LockResponse {
            success: true,
            message: "OK".to_string(),
            account_id: account.id,
            available_before,
            available_after: account.available,
            locked_before,
            locked_after: account.locked,
        }))
    }

    /// 风控解锁
    async fn unlock(
        &self,
        request: Request<UnlockRequest>,
    ) -> Result<Response<UnlockResponse>, Status> {
        let req = request.into_inner();
        self.validate_amount(req.amount)?;

        let mut account = self.get_account(req.user_id, &req.asset).await?;

        // 检查锁定余额
        if !account.has_sufficient_locked(req.amount) {
            return Err(Error::InsufficientLocked(account.locked, req.amount).into());
        }

        let available_before = account.available;
        let locked_before = account.locked;

        account.locked -= req.amount;
        account.available += req.amount;

        // 更新数据库
        AccountRepository::update_balance(&self.pool, account.id, account.available, account.frozen, account.locked).await?;

        // 记录操作
        let mut op = BalanceOperation::new(
            req.user_id,
            &req.asset,
            BalanceOperationType::Unlock,
            req.amount,
            available_before,
            account.available,
            account.frozen,
            account.frozen,
            "",
        );
        op.account_id = account.id;
        if !req.reason.is_empty() {
            op = op.with_reason(&req.reason);
        }
        self.record_operation(&mut op).await?;

        Ok(Response::new(UnlockResponse {
            success: true,
            message: "OK".to_string(),
            account_id: account.id,
            available_before,
            available_after: account.available,
            locked_before,
            locked_after: account.locked,
        }))
    }

    /// 检查余额是否足够
    async fn check_balance(
        &self,
        request: Request<CheckBalanceRequest>,
    ) -> Result<Response<CheckBalanceResponse>, Status> {
        let req = request.into_inner();
        self.validate_amount(req.amount)?;

        let account = self.get_or_create_account(req.user_id, &req.asset, self.precision.get_precision(&req.asset)).await?;

        Ok(Response::new(CheckBalanceResponse {
            sufficient: account.has_sufficient_available(req.amount),
            available: account.available,
            required: req.amount,
        }))
    }

    /// 检查冻结是否足够
    async fn check_frozen(
        &self,
        request: Request<CheckFrozenRequest>,
    ) -> Result<Response<CheckFrozenResponse>, Status> {
        let req = request.into_inner();

        let account = self.get_account(req.user_id, &req.asset).await?;

        Ok(Response::new(CheckFrozenResponse {
            sufficient: account.has_sufficient_frozen(account.frozen),
            frozen: account.frozen,
            required: account.frozen,
        }))
    }

    /// 结算派彩
    async fn settle(
        &self,
        request: Request<SettleRequest>,
    ) -> Result<Response<SettleResponse>, Status> {
        let req = request.into_inner();
        self.validate_amount(req.outcome_amount)?;

        // 消耗结果代币
        let mut outcome_account = self.get_account(req.user_id, &req.outcome_asset).await?;

        // 检查结果代币余额
        if !outcome_account.has_sufficient_available(req.outcome_amount) {
            return Err(Error::InsufficientBalance(outcome_account.available, req.outcome_amount).into());
        }

        let outcome_available_before = outcome_account.available;
        outcome_account.available -= req.outcome_amount;

        // 更新结果代币账户
        AccountRepository::update_available(&self.pool, outcome_account.id, outcome_account.available).await?;

        // 增加基础资产
        let base_precision = self.precision.get_precision(&req.base_asset);
        let mut base_account = self.get_or_create_account(req.user_id, &req.base_asset, base_precision).await?;
        let base_available_before = base_account.available;
        base_account.available += req.base_amount;

        // 更新基础资产账户
        AccountRepository::update_available(&self.pool, base_account.id, base_account.available).await?;

        // 记录结果代币扣减操作
        let mut op_outcome = BalanceOperation::new(
            req.user_id,
            &req.outcome_asset,
            BalanceOperationType::Settlement,
            req.outcome_amount,
            outcome_available_before,
            outcome_account.available,
            outcome_account.frozen,
            outcome_account.frozen,
            &req.market_id.to_string(),
        );
        op_outcome.account_id = outcome_account.id;
        if !req.reason.is_empty() {
            op_outcome = op_outcome.with_reason(&req.reason);
        }
        self.record_operation(&mut op_outcome).await?;

        // 记录基础资产增加操作
        let mut op_base = BalanceOperation::new(
            req.user_id,
            &req.base_asset,
            BalanceOperationType::Settlement,
            req.base_amount,
            base_available_before,
            base_account.available,
            base_account.frozen,
            base_account.frozen,
            &req.market_id.to_string(),
        );
        op_base.account_id = base_account.id;
        if !req.reason.is_empty() {
            op_base = op_base.with_reason(&req.reason);
        }
        self.record_operation(&mut op_base).await?;

        Ok(Response::new(SettleResponse {
            success: true,
            message: "OK".to_string(),
            outcome_available_after: outcome_account.available,
            base_available_after: base_account.available,
        }))
    }

    /// 批量获取余额
    async fn batch_get_balances(
        &self,
        request: Request<BatchGetBalancesRequest>,
    ) -> Result<Response<BatchGetBalancesResponse>, Status> {
        let req = request.into_inner();

        let accounts = AccountRepository::batch_get(&self.pool, &req.user_ids, &req.assets).await?;

        let balances = accounts.into_iter().map(|a| Balance {
            account_id: a.id,
            asset: a.asset,
            available: a.available,
            frozen: a.frozen,
            locked: a.locked,
            precision: a.precision as i32,
        }).collect();

        Ok(Response::new(BatchGetBalancesResponse { balances }))
    }
}