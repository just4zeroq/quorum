//! Payment Password Repository

use sqlx::SqlitePool;
use crate::errors::{WalletError, Result};
use crate::models::PaymentPassword;

pub struct PaymentPasswordRepository {
    pool: SqlitePool,
}

impl PaymentPasswordRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Set payment password
    pub async fn set(&self, user_id: i64, password_hash: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO payment_passwords (user_id, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?) ON CONFLICT(user_id) DO UPDATE SET password_hash = EXCLUDED.password_hash, updated_at = EXCLUDED.updated_at"
        )
        .bind(user_id)
        .bind(password_hash)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get payment password
    pub async fn get(&self, user_id: i64) -> Result<Option<PaymentPassword>> {
        let record = sqlx::query_as::<_, PaymentPassword>(
            "SELECT * FROM payment_passwords WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(record)
    }

    /// Check if user has payment password
    pub async fn has(&self, user_id: i64) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM payment_passwords WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }
}
