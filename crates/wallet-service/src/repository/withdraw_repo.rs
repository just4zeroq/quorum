//! Withdraw Repository

use sqlx::SqlitePool;
use crate::errors::{WalletError, Result};
use crate::models::{WithdrawRecord, WithdrawStatus};

pub struct WithdrawRepository {
    pool: SqlitePool,
}

impl WithdrawRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create withdraw record
    pub async fn create(&self, user_id: i64, asset: &str, amount: &str, fee: &str, to_address: &str, chain: &str) -> Result<i64> {
        let created_at = chrono::Utc::now().timestamp();
        let status = WithdrawStatus::Pending.to_string();
        let result = sqlx::query(
            "INSERT INTO withdraw_records (user_id, asset, amount, fee, to_address, chain, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(asset)
        .bind(amount)
        .bind(fee)
        .bind(to_address)
        .bind(chain)
        .bind(&status)
        .bind(created_at)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(result.last_insert_rowid())
    }

    /// Get withdraw by id
    pub async fn get_by_id(&self, id: i64) -> Result<Option<WithdrawRecord>> {
        let record = sqlx::query_as::<_, WithdrawRecord>(
            "SELECT * FROM withdraw_records WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(record)
    }

    /// Update withdraw status
    pub async fn update_status(&self, id: i64, status: &str, tx_id: Option<&str>) -> Result<()> {
        let updated_at = chrono::Utc::now().timestamp();
        sqlx::query(
            "UPDATE withdraw_records SET status = ?, tx_id = ?, updated_at = ? WHERE id = ?"
        )
        .bind(status)
        .bind(tx_id)
        .bind(updated_at)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get withdraw history
    pub async fn get_history(&self, user_id: i64, page: i32, page_size: i32) -> Result<(Vec<WithdrawRecord>, i64)> {
        let offset = (page - 1).max(0) as i64 * page_size as i64;
        let limit = page_size as i64;

        let records = sqlx::query_as::<_, WithdrawRecord>(
            "SELECT * FROM withdraw_records WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM withdraw_records WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok((records, total))
    }

    /// Get pending withdraws
    pub async fn get_pending(&self, user_id: i64) -> Result<Vec<WithdrawRecord>> {
        let records = sqlx::query_as::<_, WithdrawRecord>(
            "SELECT * FROM withdraw_records WHERE user_id = ? AND status = ? ORDER BY created_at DESC"
        )
        .bind(user_id)
        .bind(WithdrawStatus::Pending.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(records)
    }
}
