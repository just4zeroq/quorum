//! Ledger 模块 - 账本流水
//!
//! 职责:
//! - 记录所有资金变动流水（不可变）
//! - 支持审计和追溯

use rust_decimal::Decimal;

use crate::errors::PortfolioError;
use crate::models::{LedgerEntry, LedgerType};
use crate::repository::PortfolioRepository;

/// 账本服务（只追加，不可变）
pub struct LedgerService {
    repo: PortfolioRepository,
}

impl LedgerService {
    pub fn new(repo: PortfolioRepository) -> Self {
        Self { repo }
    }

    /// 记录流水
    pub async fn record(
        &self,
        user_id: &str,
        account_id: &str,
        ledger_type: LedgerType,
        asset: &str,
        amount: Decimal,
        balance_after: Decimal,
        reference_id: &str,
        reference_type: &str,
    ) -> Result<LedgerEntry, PortfolioError> {
        let entry = LedgerEntry {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            account_id: account_id.to_string(),
            ledger_type,
            asset: asset.to_string(),
            amount,
            balance_after,
            reference_id: reference_id.to_string(),
            reference_type: reference_type.to_string(),
            created_at: chrono::Utc::now(),
        };

        self.repo.insert_ledger(&entry).await?;

        tracing::debug!(
            "Ledger recorded: {} {} {} (ref: {}:{}, balance_after: {})",
            entry.ledger_type.as_str(),
            amount,
            asset,
            reference_type,
            reference_id,
            balance_after
        );

        Ok(entry)
    }

    /// 查询用户流水
    pub async fn get_user_entries(
        &self,
        user_id: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<LedgerEntry>, PortfolioError> {
        self.repo.list_ledger_by_user(user_id, limit, offset).await
    }
}
