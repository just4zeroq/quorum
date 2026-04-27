//! Account 模块 - 账户余额管理
//!
//! 职责:
//! - 余额查询
//! - 冻结/解冻（乐观锁）
//! - 划转

use rust_decimal::Decimal;
use std::time::Duration;

use crate::errors::PortfolioError;
use crate::models::{Account, AccountType};
use crate::repository::PortfolioRepository;

/// 账户服务
pub struct AccountService {
    repo: PortfolioRepository,
}

impl AccountService {
    pub fn new(repo: PortfolioRepository) -> Self {
        Self { repo }
    }

    /// 获取或创建账户
    pub async fn get_or_create_account(
        &self,
        user_id: &str,
        asset: &str,
    ) -> Result<Account, PortfolioError> {
        if let Some(account) = self.repo.get_account(user_id, asset).await? {
            return Ok(account);
        }

        let account = Account {
            id: format!("acc_{}_{}", user_id, asset),
            user_id: user_id.to_string(),
            asset: asset.to_string(),
            account_type: AccountType::Spot,
            available: Decimal::ZERO,
            frozen: Decimal::ZERO,
            version: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.repo.create_account(&account).await?;
        Ok(account)
    }

    /// 获取账户余额
    pub async fn get_balance(
        &self,
        user_id: &str,
        asset: &str,
    ) -> Result<Account, PortfolioError> {
        self.get_or_create_account(user_id, asset).await
    }

    /// 冻结资金（乐观锁重试）
    pub async fn freeze(
        &self,
        user_id: &str,
        asset: &str,
        amount: Decimal,
        _order_id: &str,
    ) -> Result<Account, PortfolioError> {
        if amount <= Decimal::ZERO {
            return Err(PortfolioError::InvalidOperation(
                "Freeze amount must be positive".into(),
            ));
        }

        for attempt in 0..3 {
            let account = self.get_or_create_account(user_id, asset).await?;

            if account.available < amount {
                return Err(PortfolioError::InsufficientBalance {
                    available: account.available.to_string(),
                    required: amount.to_string(),
                });
            }

            let rows = self
                .repo
                .freeze_with_version(user_id, asset, amount, account.version)
                .await?;

            if rows == 1 {
                // 记录流水
                let updated = self.repo.get_account(user_id, asset).await?.unwrap();
                return Ok(updated);
            }

            tokio::time::sleep(Duration::from_millis(10 * (attempt + 1))).await;
        }

        Err(PortfolioError::InvalidOperation(
            "Freeze failed after retries".into(),
        ))
    }

    /// 解冻资金（乐观锁重试）
    pub async fn unfreeze(
        &self,
        user_id: &str,
        asset: &str,
        amount: Decimal,
        _order_id: &str,
    ) -> Result<Account, PortfolioError> {
        if amount <= Decimal::ZERO {
            return Err(PortfolioError::InvalidOperation(
                "Unfreeze amount must be positive".into(),
            ));
        }

        for attempt in 0..3 {
            let account = match self.repo.get_account(user_id, asset).await? {
                Some(a) => a,
                None => {
                    return Err(PortfolioError::AccountNotFound(format!(
                        "{}:{}",
                        user_id, asset
                    )))
                }
            };

            if account.frozen < amount {
                return Err(PortfolioError::InvalidOperation(
                    format!(
                        "Insufficient frozen balance: frozen={}, required={}",
                        account.frozen, amount
                    )
                    .into(),
                ));
            }

            let rows = self
                .repo
                .unfreeze_with_version(user_id, asset, amount, account.version)
                .await?;

            if rows == 1 {
                let updated = self.repo.get_account(user_id, asset).await?.unwrap();
                return Ok(updated);
            }

            tokio::time::sleep(Duration::from_millis(10 * (attempt + 1))).await;
        }

        Err(PortfolioError::InvalidOperation(
            "Unfreeze failed after retries".into(),
        ))
    }

    /// 扣除资金（从 available 中扣）
    pub async fn debit(
        &self,
        user_id: &str,
        asset: &str,
        amount: Decimal,
    ) -> Result<Account, PortfolioError> {
        if amount <= Decimal::ZERO {
            return Err(PortfolioError::InvalidOperation(
                "Debit amount must be positive".into(),
            ));
        }

        for attempt in 0..3 {
            let account = match self.repo.get_account(user_id, asset).await? {
                Some(a) => a,
                None => {
                    return Err(PortfolioError::AccountNotFound(format!(
                        "{}:{}",
                        user_id, asset
                    )))
                }
            };

            if account.available < amount {
                return Err(PortfolioError::InsufficientBalance {
                    available: account.available.to_string(),
                    required: amount.to_string(),
                });
            }

            let rows = self
                .repo
                .debit_available_with_version(user_id, asset, amount, account.version)
                .await?;

            if rows == 1 {
                let updated = self.repo.get_account(user_id, asset).await?.unwrap();
                return Ok(updated);
            }

            tokio::time::sleep(Duration::from_millis(10 * (attempt + 1))).await;
        }

        Err(PortfolioError::InvalidOperation(
            "Debit failed after retries".into(),
        ))
    }

    /// 增加资金
    pub async fn credit(
        &self,
        user_id: &str,
        asset: &str,
        amount: Decimal,
    ) -> Result<Account, PortfolioError> {
        if amount <= Decimal::ZERO {
            return Err(PortfolioError::InvalidOperation(
                "Credit amount must be positive".into(),
            ));
        }

        for attempt in 0..3 {
            let account = self.get_or_create_account(user_id, asset).await?;

            let rows = self
                .repo
                .credit_with_version(user_id, asset, amount, account.version)
                .await?;

            if rows == 1 {
                let updated = self.repo.get_account(user_id, asset).await?.unwrap();
                return Ok(updated);
            }

            tokio::time::sleep(Duration::from_millis(10 * (attempt + 1))).await;
        }

        Err(PortfolioError::InvalidOperation(
            "Credit failed after retries".into(),
        ))
    }

    /// 扣除可用余额
    pub async fn debit_available(
        &self,
        user_id: &str,
        asset: &str,
        amount: Decimal,
    ) -> Result<Account, PortfolioError> {
        if amount <= Decimal::ZERO {
            return Err(PortfolioError::InvalidOperation(
                "Debit amount must be positive".into(),
            ));
        }

        for attempt in 0..3 {
            let account = match self.repo.get_account(user_id, asset).await? {
                Some(a) => a,
                None => {
                    return Err(PortfolioError::AccountNotFound(format!(
                        "{}:{}",
                        user_id, asset
                    )))
                }
            };

            if account.available < amount {
                return Err(PortfolioError::InsufficientBalance {
                    available: account.available.to_string(),
                    required: amount.to_string(),
                });
            }

            let rows = self
                .repo
                .debit_available_with_version(user_id, asset, amount, account.version)
                .await?;

            if rows == 1 {
                let updated = self.repo.get_account(user_id, asset).await?.unwrap();
                return Ok(updated);
            }

            tokio::time::sleep(Duration::from_millis(10 * (attempt + 1))).await;
        }

        Err(PortfolioError::InvalidOperation(
            "Debit failed after retries".into(),
        ))
    }
}
