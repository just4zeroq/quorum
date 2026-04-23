//! 余额操作记录 Repository
//!
//! 提供余额操作记录的数据库操作

use db::DBPool;
use sqlx::Row;

use crate::error::Result;
use crate::models::{BalanceOperation, BalanceOperationType};

/// 余额操作记录 Repository
pub struct OperationRepository;

impl OperationRepository {
    /// 记录操作
    pub async fn record(
        pool: &DBPool,
        op: &BalanceOperation,
    ) -> Result<i64> {
        match pool {
            DBPool::Sqlite(pool) => {
                Self::sqlite_record(pool, op).await
            }
            DBPool::Postgres(pool) => {
                Self::postgres_record(pool, op).await
            }
        }
    }

    /// 获取账户的操作记录
    pub async fn get_by_account(
        pool: &DBPool,
        account_id: i64,
        limit: i64,
    ) -> Result<Vec<BalanceOperation>> {
        match pool {
            DBPool::Sqlite(pool) => Self::sqlite_get_by_account(pool, account_id, limit).await,
            DBPool::Postgres(pool) => Self::postgres_get_by_account(pool, account_id, limit).await,
        }
    }

    /// 获取用户的操作记录
    pub async fn get_by_user(
        pool: &DBPool,
        user_id: i64,
        limit: i64,
    ) -> Result<Vec<BalanceOperation>> {
        match pool {
            DBPool::Sqlite(pool) => Self::sqlite_get_by_user(pool, user_id, limit).await,
            DBPool::Postgres(pool) => Self::postgres_get_by_user(pool, user_id, limit).await,
        }
    }

    // ==================== SQLite 实现 ====================

    async fn sqlite_record(
        pool: &sqlx::SqlitePool,
        op: &BalanceOperation,
    ) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO balance_operations
            (account_id, user_id, asset, operation_type, amount, balance_before, balance_after, frozen_before, frozen_after, reason, ref_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(op.account_id)
        .bind(op.user_id)
        .bind(&op.asset)
        .bind(op.operation_type.as_str())
        .bind(op.amount)
        .bind(op.balance_before)
        .bind(op.balance_after)
        .bind(op.frozen_before)
        .bind(op.frozen_after)
        .bind(&op.reason)
        .bind(&op.ref_id)
        .bind(op.created_at)
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn sqlite_get_by_account(
        pool: &sqlx::SqlitePool,
        account_id: i64,
        limit: i64,
    ) -> Result<Vec<BalanceOperation>> {
        let rows = sqlx::query(
            "SELECT * FROM balance_operations WHERE account_id = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(Self::row_to_operation).collect()
    }

    async fn sqlite_get_by_user(
        pool: &sqlx::SqlitePool,
        user_id: i64,
        limit: i64,
    ) -> Result<Vec<BalanceOperation>> {
        let rows = sqlx::query(
            "SELECT * FROM balance_operations WHERE user_id = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(Self::row_to_operation).collect()
    }

    // ==================== PostgreSQL 实现 ====================

    async fn postgres_record(
        pool: &sqlx::PgPool,
        op: &BalanceOperation,
    ) -> Result<i64> {
        let row = sqlx::query(
            r#"
            INSERT INTO balance_operations
            (account_id, user_id, asset, operation_type, amount, balance_before, balance_after, frozen_before, frozen_after, reason, ref_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id
            "#,
        )
        .bind(op.account_id)
        .bind(op.user_id)
        .bind(&op.asset)
        .bind(op.operation_type.as_str())
        .bind(op.amount)
        .bind(op.balance_before)
        .bind(op.balance_after)
        .bind(op.frozen_before)
        .bind(op.frozen_after)
        .bind(&op.reason)
        .bind(&op.ref_id)
        .bind(op.created_at)
        .fetch_one(pool)
        .await?;

        Ok(row.get("id"))
    }

    async fn postgres_get_by_account(
        pool: &sqlx::PgPool,
        account_id: i64,
        limit: i64,
    ) -> Result<Vec<BalanceOperation>> {
        let rows = sqlx::query(
            "SELECT * FROM balance_operations WHERE account_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(Self::row_to_operation).collect()
    }

    async fn postgres_get_by_user(
        pool: &sqlx::PgPool,
        user_id: i64,
        limit: i64,
    ) -> Result<Vec<BalanceOperation>> {
        let rows = sqlx::query(
            "SELECT * FROM balance_operations WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(Self::row_to_operation).collect()
    }

    // ==================== 辅助方法 ====================

    fn row_to_operation(row: impl Row) -> Result<BalanceOperation> {
        let op_type_str: String = row.get("operation_type");
        let operation_type = BalanceOperationType::from_str(&op_type_str)
            .ok_or_else(|| crate::error::Error::Internal(format!("Invalid operation type: {}", op_type_str)))?;

        Ok(BalanceOperation {
            id: row.get("id"),
            account_id: row.get("account_id"),
            user_id: row.get("user_id"),
            asset: row.get("asset"),
            operation_type,
            amount: row.get("amount"),
            balance_before: row.get("balance_before"),
            balance_after: row.get("balance_after"),
            frozen_before: row.get("frozen_before"),
            frozen_after: row.get("frozen_after"),
            reason: row.get("reason"),
            ref_id: row.get("ref_id"),
            created_at: row.get("created_at"),
        })
    }
}