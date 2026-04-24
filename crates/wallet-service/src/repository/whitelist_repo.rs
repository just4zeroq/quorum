//! Whitelist Repository

use sqlx::SqlitePool;
use crate::errors::{WalletError, Result};
use crate::models::WhitelistAddress;

pub struct WhitelistRepository {
    pool: SqlitePool,
}

impl WhitelistRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Add whitelist address
    pub async fn add(&self, user_id: i64, chain: &str, address: &str, label: Option<&str>) -> Result<()> {
        let created_at = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO whitelist_addresses (user_id, chain, address, label, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(chain)
        .bind(address)
        .bind(label)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Remove whitelist address
    pub async fn remove(&self, user_id: i64, address: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM whitelist_addresses WHERE user_id = ? AND address = ?"
        )
        .bind(user_id)
        .bind(address)
        .execute(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// List whitelist addresses
    pub async fn list(&self, user_id: i64, chain: &str) -> Result<Vec<WhitelistAddress>> {
        let addresses = if chain.is_empty() {
            sqlx::query_as::<_, WhitelistAddress>(
                "SELECT * FROM whitelist_addresses WHERE user_id = ?"
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, WhitelistAddress>(
                "SELECT * FROM whitelist_addresses WHERE user_id = ? AND chain = ?"
            )
            .bind(user_id)
            .bind(chain)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(addresses)
    }

    /// Check if address is whitelisted
    pub async fn is_whitelisted(&self, user_id: i64, address: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM whitelist_addresses WHERE user_id = ? AND address = ?"
        )
        .bind(user_id)
        .bind(address)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }
}
