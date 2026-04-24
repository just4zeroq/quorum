//! Deposit Repository

use sqlx::SqlitePool;
use crate::errors::{WalletError, Result};
use crate::models::{DepositAddress, DepositRecord};

pub struct DepositRepository {
    pool: SqlitePool,
}

impl DepositRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create deposit address
    pub async fn create_address(&self, user_id: i64, chain: &str, address: &str) -> Result<i64> {
        let created_at = chrono::Utc::now().timestamp();
        let result = sqlx::query(
            "INSERT INTO deposit_addresses (user_id, chain, address, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(chain)
        .bind(address)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(result.last_insert_rowid())
    }

    /// Get deposit address by user and chain
    pub async fn get_address(&self, user_id: i64, chain: &str) -> Result<Option<DepositAddress>> {
        let address = sqlx::query_as::<_, DepositAddress>(
            "SELECT * FROM deposit_addresses WHERE user_id = ? AND chain = ?"
        )
        .bind(user_id)
        .bind(chain)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(address)
    }

    /// List all deposit addresses for user
    pub async fn list_addresses(&self, user_id: i64) -> Result<Vec<DepositAddress>> {
        let addresses = sqlx::query_as::<_, DepositAddress>(
            "SELECT * FROM deposit_addresses WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(addresses)
    }

    /// Create deposit record
    pub async fn create_record(&self, user_id: i64, tx_id: &str, chain: &str, amount: &str, address: &str) -> Result<i64> {
        let created_at = chrono::Utc::now().timestamp();
        let result = sqlx::query(
            "INSERT INTO deposit_records (user_id, tx_id, chain, amount, address, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(tx_id)
        .bind(chain)
        .bind(amount)
        .bind(address)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(result.last_insert_rowid())
    }

    /// Get deposit history
    pub async fn get_history(&self, user_id: i64, _chain: &str, page: i32, page_size: i32) -> Result<(Vec<DepositRecord>, i64)> {
        let offset = (page - 1).max(0) as i64 * page_size as i64;
        let limit = page_size as i64;

        let records = sqlx::query_as::<_, DepositRecord>(
            "SELECT * FROM deposit_records WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM deposit_records WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok((records, total))
    }
}
