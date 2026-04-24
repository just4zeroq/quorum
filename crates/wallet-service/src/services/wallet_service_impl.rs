//! Wallet Service gRPC Implementation

use std::sync::Arc;
use sha2::{Sha256, Digest};
use tonic::{Request, Response, Status};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::errors::WalletError;
use crate::models::{WithdrawStatus, WithdrawRecord, DepositAddress, WhitelistAddress};
use crate::repository::{
    DepositRepository, WithdrawRepository, WhitelistRepository, PaymentPasswordRepository,
};
use crate::pb::wallet_service_server::WalletService;
use crate::pb::*;

pub struct WalletServiceImpl {
    deposit_repo: Arc<DepositRepository>,
    withdraw_repo: Arc<WithdrawRepository>,
    whitelist_repo: Arc<WhitelistRepository>,
    payment_password_repo: Arc<PaymentPasswordRepository>,
    require_whitelist: bool,
    require_payment_password: bool,
    supported_chains: Vec<String>,
    default_fee: String,
}

impl WalletServiceImpl {
    pub fn new(
        deposit_repo: DepositRepository,
        withdraw_repo: WithdrawRepository,
        whitelist_repo: WhitelistRepository,
        payment_password_repo: PaymentPasswordRepository,
    ) -> Self {
        Self {
            deposit_repo: Arc::new(deposit_repo),
            withdraw_repo: Arc::new(withdraw_repo),
            whitelist_repo: Arc::new(whitelist_repo),
            payment_password_repo: Arc::new(payment_password_repo),
            require_whitelist: false,
            require_payment_password: true,
            supported_chains: vec!["ETH".to_string(), "BSC".to_string(), "ARBITRUM".to_string()],
            default_fee: "0.001".to_string(),
        }
    }

    /// Hash password using SHA256
    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate deposit address
    fn generate_address(chain: &str) -> String {
        format!("0x{}_{:016x}", chain.to_lowercase(), uuid::Uuid::new_v4().as_u128())
    }

    /// Validate chain
    fn validate_chain(&self, chain: &str) -> Result<(), WalletError> {
        if self.supported_chains.contains(&chain.to_uppercase()) {
            Ok(())
        } else {
            Err(WalletError::ChainNotSupported(chain.to_string()))
        }
    }
}

#[tonic::async_trait]
impl WalletService for WalletServiceImpl {
    // ========== 充值地址 ==========

    async fn get_deposit_address(
        &self,
        request: Request<GetDepositAddressRequest>,
    ) -> Result<Response<GetDepositAddressResponse>, Status> {
        let req = request.into_inner();

        self.validate_chain(&req.chain)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // Check if address already exists
        if let Ok(Some(addr)) = self.deposit_repo.get_address(req.user_id, &req.chain).await {
            return Ok(Response::new(GetDepositAddressResponse {
                address: addr.address,
                chain: addr.chain,
            }));
        }

        // Generate new address
        let address = Self::generate_address(&req.chain);
        self.deposit_repo.create_address(req.user_id, &req.chain, &address)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetDepositAddressResponse {
            address,
            chain: req.chain,
        }))
    }

    async fn list_deposit_addresses(
        &self,
        request: Request<ListDepositAddressesRequest>,
    ) -> Result<Response<ListDepositAddressesResponse>, Status> {
        let req = request.into_inner();

        let addresses = self.deposit_repo.list_addresses(req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let summaries: Vec<DepositAddressSummary> = addresses.into_iter().map(|a| {
            DepositAddressSummary {
                address: a.address,
                chain: a.chain,
                created_at: a.created_at,
            }
        }).collect();

        Ok(Response::new(ListDepositAddressesResponse {
            addresses: summaries,
        }))
    }

    // ========== 充值 ==========

    async fn confirm_deposit(
        &self,
        request: Request<ConfirmDepositRequest>,
    ) -> Result<Response<ConfirmDepositResponse>, Status> {
        let req = request.into_inner();

        // Validate amount
        let _amount = Decimal::from_str(&req.amount)
            .map_err(|_| Status::invalid_argument("Invalid amount"))?;

        // Create deposit record
        self.deposit_repo.create_record(req.user_id, &req.tx_id, &req.chain, &req.amount, &req.address)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Note: In production, should call Portfolio Service to credit balance
        tracing::info!("Deposit confirmed: user={}, tx={}, amount={}", req.user_id, req.tx_id, req.amount);

        Ok(Response::new(ConfirmDepositResponse {
            success: true,
            message: "Deposit confirmed".to_string(),
        }))
    }

    async fn get_deposit_history(
        &self,
        request: Request<GetDepositHistoryRequest>,
    ) -> Result<Response<GetDepositHistoryResponse>, Status> {
        let req = request.into_inner();

        let page = if req.page <= 0 { 1 } else { req.page };
        let page_size = if req.page_size <= 0 { 20 } else { req.page_size };

        let (records, total) = self.deposit_repo.get_history(req.user_id, &req.chain, page, page_size)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let deposits: Vec<crate::pb::DepositRecord> = records.into_iter().map(|r| {
            crate::pb::DepositRecord {
                tx_id: r.tx_id,
                chain: r.chain,
                amount: r.amount,
                address: r.address,
                created_at: r.created_at,
            }
        }).collect();

        Ok(Response::new(GetDepositHistoryResponse {
            deposits,
            total,
        }))
    }

    // ========== 提现 ==========

    async fn create_withdraw(
        &self,
        request: Request<CreateWithdrawRequest>,
    ) -> Result<Response<CreateWithdrawResponse>, Status> {
        let req = request.into_inner();

        // Validate chain
        self.validate_chain(&req.chain)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // Validate amount
        let amount = Decimal::from_str(&req.amount)
            .map_err(|_| Status::invalid_argument("Invalid amount"))?;
        if amount <= Decimal::ZERO {
            return Err(Status::invalid_argument("Amount must be positive"));
        }

        // Verify payment password if required
        if self.require_payment_password {
            let has_password = self.payment_password_repo.has(req.user_id)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            if has_password {
                let password_hash = Self::hash_password(&req.payment_password);
                let stored = self.payment_password_repo.get(req.user_id)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                if let Some(stored) = stored {
                    if stored.password_hash != password_hash {
                        return Err(Status::permission_denied("Invalid payment password"));
                    }
                }
            }
        }

        // Check whitelist if required
        if self.require_whitelist {
            let is_whitelisted = self.whitelist_repo.is_whitelisted(req.user_id, &req.to_address)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            if !is_whitelisted {
                return Err(Status::permission_denied("Address not whitelisted"));
            }
        }

        // Create withdraw record
        let withdraw_id = self.withdraw_repo.create(
            req.user_id,
            &req.asset,
            &req.amount,
            &self.default_fee,
            &req.to_address,
            &req.chain
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        tracing::info!("Withdraw created: id={}, user={}, amount={}", withdraw_id, req.user_id, req.amount);

        Ok(Response::new(CreateWithdrawResponse {
            success: true,
            message: "Withdraw created".to_string(),
            withdraw_id: withdraw_id.to_string(),
        }))
    }

    async fn confirm_withdraw(
        &self,
        request: Request<ConfirmWithdrawRequest>,
    ) -> Result<Response<ConfirmWithdrawResponse>, Status> {
        let req = request.into_inner();

        let withdraw_id = req.withdraw_id.parse::<i64>()
            .map_err(|_| Status::invalid_argument("Invalid withdraw_id"))?;

        let record = self.withdraw_repo.get_by_id(withdraw_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Withdraw not found"))?;

        if record.status != WithdrawStatus::Pending.to_string() {
            return Err(Status::failed_precondition("Withdraw not pending"));
        }

        // Simulate broadcast
        let tx_id = format!("0x{:064x}", uuid::Uuid::new_v4().as_u128());

        self.withdraw_repo.update_status(withdraw_id, &WithdrawStatus::Confirmed.to_string(), Some(&tx_id))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ConfirmWithdrawResponse {
            success: true,
            message: "Withdraw confirmed".to_string(),
            tx_id,
        }))
    }

    async fn cancel_withdraw(
        &self,
        request: Request<CancelWithdrawRequest>,
    ) -> Result<Response<CancelWithdrawResponse>, Status> {
        let req = request.into_inner();

        let withdraw_id = req.withdraw_id.parse::<i64>()
            .map_err(|_| Status::invalid_argument("Invalid withdraw_id"))?;

        let record = self.withdraw_repo.get_by_id(withdraw_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Withdraw not found"))?;

        if record.user_id != req.user_id {
            return Err(Status::permission_denied("Withdraw does not belong to user"));
        }

        if record.status != WithdrawStatus::Pending.to_string() {
            return Err(Status::failed_precondition("Withdraw not pending"));
        }

        self.withdraw_repo.update_status(withdraw_id, &WithdrawStatus::Cancelled.to_string(), None)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CancelWithdrawResponse {
            success: true,
            message: "Withdraw cancelled".to_string(),
        }))
    }

    async fn get_withdraw_history(
        &self,
        request: Request<GetWithdrawHistoryRequest>,
    ) -> Result<Response<GetWithdrawHistoryResponse>, Status> {
        let req = request.into_inner();

        let page = if req.page <= 0 { 1 } else { req.page };
        let page_size = if req.page_size <= 0 { 20 } else { req.page_size };

        let (records, total) = self.withdraw_repo.get_history(req.user_id, page, page_size)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let withdrawals: Vec<WithdrawRecordSummary> = records.into_iter().map(|r| {
            WithdrawRecordSummary {
                withdraw_id: r.id.to_string(),
                asset: r.asset,
                amount: r.amount,
                fee: r.fee,
                to_address: r.to_address,
                status: r.status,
                tx_id: r.tx_id.unwrap_or_default(),
                created_at: r.created_at,
            }
        }).collect();

        Ok(Response::new(GetWithdrawHistoryResponse {
            withdrawals,
            total,
        }))
    }

    async fn get_pending_withdraws(
        &self,
        request: Request<GetPendingWithdrawsRequest>,
    ) -> Result<Response<GetPendingWithdrawsResponse>, Status> {
        let req = request.into_inner();

        let records = self.withdraw_repo.get_pending(req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let withdrawals: Vec<WithdrawRecordSummary> = records.into_iter().map(|r| {
            WithdrawRecordSummary {
                withdraw_id: r.id.to_string(),
                asset: r.asset,
                amount: r.amount,
                fee: r.fee,
                to_address: r.to_address,
                status: r.status,
                tx_id: r.tx_id.unwrap_or_default(),
                created_at: r.created_at,
            }
        }).collect();

        Ok(Response::new(GetPendingWithdrawsResponse {
            withdrawals,
        }))
    }

    // ========== 地址白名单 ==========

    async fn add_whitelist_address(
        &self,
        request: Request<AddWhitelistAddressRequest>,
    ) -> Result<Response<AddWhitelistAddressResponse>, Status> {
        let req = request.into_inner();

        self.validate_chain(&req.chain)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        self.whitelist_repo.add(req.user_id, &req.chain, &req.address, Some(&req.label))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AddWhitelistAddressResponse {
            success: true,
            message: "Address added to whitelist".to_string(),
        }))
    }

    async fn remove_whitelist_address(
        &self,
        request: Request<RemoveWhitelistAddressRequest>,
    ) -> Result<Response<RemoveWhitelistAddressResponse>, Status> {
        let req = request.into_inner();

        self.whitelist_repo.remove(req.user_id, &req.address)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RemoveWhitelistAddressResponse {
            success: true,
            message: "Address removed from whitelist".to_string(),
        }))
    }

    async fn list_whitelist_addresses(
        &self,
        request: Request<ListWhitelistAddressesRequest>,
    ) -> Result<Response<ListWhitelistAddressesResponse>, Status> {
        let req = request.into_inner();

        let addresses = self.whitelist_repo.list(req.user_id, &req.chain)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let summaries: Vec<WhitelistAddressSummary> = addresses.into_iter().map(|a| {
            WhitelistAddressSummary {
                chain: a.chain,
                address: a.address,
                label: a.label.unwrap_or_default(),
                created_at: a.created_at,
            }
        }).collect();

        Ok(Response::new(ListWhitelistAddressesResponse {
            addresses: summaries,
        }))
    }

    async fn is_whitelisted(
        &self,
        request: Request<IsWhitelistedRequest>,
    ) -> Result<Response<IsWhitelistedResponse>, Status> {
        let req = request.into_inner();

        let is_whitelisted = self.whitelist_repo.is_whitelisted(req.user_id, &req.address)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(IsWhitelistedResponse {
            is_whitelisted,
        }))
    }

    // ========== 支付密码 ==========

    async fn set_payment_password(
        &self,
        request: Request<SetPaymentPasswordRequest>,
    ) -> Result<Response<SetPaymentPasswordResponse>, Status> {
        let req = request.into_inner();

        if req.password.len() < 6 {
            return Err(Status::invalid_argument("Password must be at least 6 characters"));
        }

        let password_hash = Self::hash_password(&req.password);
        self.payment_password_repo.set(req.user_id, &password_hash)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SetPaymentPasswordResponse {
            success: true,
            message: "Payment password set".to_string(),
        }))
    }

    async fn verify_payment_password(
        &self,
        request: Request<VerifyPaymentPasswordRequest>,
    ) -> Result<Response<VerifyPaymentPasswordResponse>, Status> {
        let req = request.into_inner();

        let stored = self.payment_password_repo.get(req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if let Some(stored) = stored {
            let password_hash = Self::hash_password(&req.password);
            let valid = stored.password_hash == password_hash;
            Ok(Response::new(VerifyPaymentPasswordResponse {
                valid,
                message: if valid { "Valid".to_string() } else { "Invalid".to_string() },
            }))
        } else {
            Ok(Response::new(VerifyPaymentPasswordResponse {
                valid: false,
                message: "Payment password not set".to_string(),
            }))
        }
    }

    async fn reset_payment_password(
        &self,
        request: Request<ResetPaymentPasswordRequest>,
    ) -> Result<Response<ResetPaymentPasswordResponse>, Status> {
        let req = request.into_inner();

        let stored = self.payment_password_repo.get(req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if let Some(stored) = stored {
            let old_hash = Self::hash_password(&req.old_password);
            if stored.password_hash != old_hash {
                return Err(Status::permission_denied("Invalid old password"));
            }
        }

        if req.new_password.len() < 6 {
            return Err(Status::invalid_argument("Password must be at least 6 characters"));
        }

        let new_hash = Self::hash_password(&req.new_password);
        self.payment_password_repo.set(req.user_id, &new_hash)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ResetPaymentPasswordResponse {
            success: true,
            message: "Payment password reset".to_string(),
        }))
    }

    async fn has_payment_password(
        &self,
        request: Request<HasPaymentPasswordRequest>,
    ) -> Result<Response<HasPaymentPasswordResponse>, Status> {
        let req = request.into_inner();

        let has_password = self.payment_password_repo.has(req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(HasPaymentPasswordResponse {
            has_password,
        }))
    }
}
