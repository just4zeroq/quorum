//! Ledger 模块 - 账本流水
//!
//! 职责:
//! - 记录所有资金变动流水（不可变）
//! - 支持审计和追溯

use rust_decimal::Decimal;
use crate::errors::PortfolioError;
use crate::models::{LedgerEntry, LedgerType};

/// 账本服务（只追加，不可变）
pub struct LedgerService;

impl LedgerService {
    /// 记录流水
    ///
    /// 账本是追加-only，不可修改或删除
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

        tracing::debug!(
            "Ledger record: {} {} {} (ref: {}:{})",
            ledger_type_to_string(entry.ledger_type),
            amount,
            asset,
            reference_type,
            reference_id
        );

        // TODO: 写入数据库

        Ok(entry)
    }

    /// 查询用户流水
    pub async fn get_user_entries(
        &self,
        user_id: &str,
        limit: i32,
    ) -> Result<Vec<LedgerEntry>, PortfolioError> {
        tracing::info!("Get ledger entries for user: {}", user_id);
        // TODO: 从数据库查询
        Ok(vec![])
    }

    /// 查询账户流水
    pub async fn get_account_entries(
        &self,
        account_id: &str,
        limit: i32,
    ) -> Result<Vec<LedgerEntry>, PortfolioError> {
        tracing::info!("Get ledger entries for account: {}", account_id);
        // TODO: 从数据库查询
        Ok(vec![])
    }
}

fn ledger_type_to_string(t: LedgerType) -> &'static str {
    match t {
        LedgerType::Deposit => "Deposit",
        LedgerType::Withdraw => "Withdraw",
        LedgerType::Freeze => "Freeze",
        LedgerType::Unfreeze => "Unfreeze",
        LedgerType::Trade => "Trade",
        LedgerType::Settle => "Settle",
        LedgerType::TransferIn => "TransferIn",
        LedgerType::TransferOut => "TransferOut",
    }
}
