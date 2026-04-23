//! 账户 Repository
//!
//! 提供账户的数据库操作

use db::DBPool;
use sqlx::{Row, FromRow};

use crate::error::{Error, Result};
use crate::models::Account;

/// 账户 Repository
pub struct AccountRepository;

impl AccountRepository {
    /// 获取或创建账户 (无则自动创建，余额为0)
    ///
    /// 如果账户不存在，自动创建一个新的账户
    pub async fn get_or_create(
        pool: &DBPool,
        user_id: i64,
        asset: &str,
        precision: u8,
    ) -> Result<Account> {
        match pool {
            DBPool::Sqlite(pool) => {
                Self::sqlite_get_or_create(pool, user_id, asset, precision).await
            }
            DBPool::Postgres(pool) => {
                Self::postgres_get_or_create(pool, user_id, asset, precision).await
            }
        }
    }

    /// 获取账户 (必须存在)
    pub async fn get(pool: &DBPool, user_id: i64, asset: &str) -> Result<Option<Account>> {
        match pool {
            DBPool::Sqlite(pool) => Self::sqlite_get(pool, user_id, asset).await,
            DBPool::Postgres(pool) => Self::postgres_get(pool, user_id, asset).await,
        }
    }

    /// 获取用户所有账户
    pub async fn get_by_user(pool: &DBPool, user_id: i64) -> Result<Vec<Account>> {
        match pool {
            DBPool::Sqlite(pool) => Self::sqlite_get_by_user(pool, user_id).await,
            DBPool::Postgres(pool) => Self::postgres_get_by_user(pool, user_id).await,
        }
    }

    /// 更新余额
    pub async fn update_balance(
        pool: &DBPool,
        account_id: i64,
        available: i64,
        frozen: i64,
        locked: i64,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();
        match pool {
            DBPool::Sqlite(pool) => {
                sqlx::query(
                    "UPDATE accounts SET available = ?, frozen = ?, locked = ?, updated_at = ? WHERE id = ?"
                )
                .bind(available)
                .bind(frozen)
                .bind(locked)
                .bind(now)
                .bind(account_id)
                .execute(pool)
                .await?;
            }
            DBPool::Postgres(pool) => {
                sqlx::query(
                    "UPDATE accounts SET available = $1, frozen = $2, locked = $3, updated_at = $4 WHERE id = $5"
                )
                .bind(available)
                .bind(frozen)
                .bind(locked)
                .bind(now)
                .bind(account_id)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    /// 更新可用余额
    pub async fn update_available(
        pool: &DBPool,
        account_id: i64,
        available: i64,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();
        match pool {
            DBPool::Sqlite(pool) => {
                sqlx::query("UPDATE accounts SET available = ?, updated_at = ? WHERE id = ?")
                    .bind(available)
                    .bind(now)
                    .bind(account_id)
                    .execute(pool)
                    .await?;
            }
            DBPool::Postgres(pool) => {
                sqlx::query("UPDATE accounts SET available = $1, updated_at = $2 WHERE id = $3")
                    .bind(available)
                    .bind(now)
                    .bind(account_id)
                    .execute(pool)
                    .await?;
            }
        }
        Ok(())
    }

    /// 批量获取余额
    pub async fn batch_get(
        pool: &DBPool,
        user_ids: &[i64],
        assets: &[String],
    ) -> Result<Vec<Account>> {
        if user_ids.is_empty() || assets.is_empty() {
            return Ok(vec![]);
        }

        match pool {
            DBPool::Sqlite(pool) => Self::sqlite_batch_get(pool, user_ids, assets).await,
            DBPool::Postgres(pool) => Self::postgres_batch_get(pool, user_ids, assets).await,
        }
    }

    // ==================== SQLite 实现 ====================

    async fn sqlite_get_or_create(
        pool: &sqlx::SqlitePool,
        user_id: i64,
        asset: &str,
        precision: u8,
    ) -> Result<Account> {
        // 先尝试查询
        let row = sqlx::query("SELECT * FROM accounts WHERE user_id = ? AND asset = ?")
            .bind(user_id)
            .bind(asset)
            .fetch_optional(pool)
            .await?;

        if let Some(row) = row {
            return Self::row_to_account(row);
        }

        // 不存在则创建
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO accounts (user_id, asset, precision, available, frozen, locked, created_at, updated_at) VALUES (?, ?, ?, 0, 0, 0, ?, ?)"
        )
        .bind(user_id)
        .bind(asset)
        .bind(precision)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // 返回创建的账户
        let row = sqlx::query("SELECT * FROM accounts WHERE user_id = ? AND asset = ?")
            .bind(user_id)
            .bind(asset)
            .fetch_one(pool)
            .await?;
        Self::row_to_account(row)
    }

    async fn sqlite_get(
        pool: &sqlx::SqlitePool,
        user_id: i64,
        asset: &str,
    ) -> Result<Option<Account>> {
        let row = sqlx::query("SELECT * FROM accounts WHERE user_id = ? AND asset = ?")
            .bind(user_id)
            .bind(asset)
            .fetch_optional(pool)
            .await?;

        match row {
            Some(row) => Ok(Some(Self::row_to_account(row)?)),
            None => Ok(None),
        }
    }

    async fn sqlite_get_by_user(pool: &sqlx::SqlitePool, user_id: i64) -> Result<Vec<Account>> {
        let rows = sqlx::query("SELECT * FROM accounts WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await?;

        rows.into_iter().map(Self::row_to_account).collect()
    }

    async fn sqlite_batch_get(
        pool: &sqlx::SqlitePool,
        user_ids: &[i64],
        assets: &[String],
    ) -> Result<Vec<Account>> {
        // 构建 IN 子句
        let user_ids_str: Vec<String> = user_ids.iter().map(|id| id.to_string()).collect();
        let assets_str: Vec<String> = assets.iter().map(|a| format!("'{}'", a)).collect();

        let query = format!(
            "SELECT * FROM accounts WHERE user_id IN ({}) AND asset IN ({})",
            user_ids_str.join(","),
            assets_str.join(",")
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;
        rows.into_iter().map(Self::row_to_account).collect()
    }

    // ==================== PostgreSQL 实现 ====================

    async fn postgres_get_or_create(
        pool: &sqlx::PgPool,
        user_id: i64,
        asset: &str,
        precision: u8,
    ) -> Result<Account> {
        // 先尝试查询
        let row = sqlx::query("SELECT * FROM accounts WHERE user_id = $1 AND asset = $2")
            .bind(user_id)
            .bind(asset)
            .fetch_optional(pool)
            .await?;

        if let Some(row) = row {
            return Self::row_to_account(row);
        }

        // 不存在则创建
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO accounts (user_id, asset, precision, available, frozen, locked, created_at, updated_at) VALUES ($1, $2, $3, 0, 0, 0, $4, $5)"
        )
        .bind(user_id)
        .bind(asset)
        .bind(precision)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // 返回创建的账户
        let row = sqlx::query("SELECT * FROM accounts WHERE user_id = $1 AND asset = $2")
            .bind(user_id)
            .bind(asset)
            .fetch_one(pool)
            .await?;
        Self::row_to_account(row)
    }

    async fn postgres_get(
        pool: &sqlx::PgPool,
        user_id: i64,
        asset: &str,
    ) -> Result<Option<Account>> {
        let row = sqlx::query("SELECT * FROM accounts WHERE user_id = $1 AND asset = $2")
            .bind(user_id)
            .bind(asset)
            .fetch_optional(pool)
            .await?;

        match row {
            Some(row) => Ok(Some(Self::row_to_account(row)?)),
            None => Ok(None),
        }
    }

    async fn postgres_get_by_user(pool: &sqlx::PgPool, user_id: i64) -> Result<Vec<Account>> {
        let rows = sqlx::query("SELECT * FROM accounts WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(pool)
            .await?;

        rows.into_iter().map(Self::row_to_account).collect()
    }

    async fn postgres_batch_get(
        pool: &sqlx::PgPool,
        user_ids: &[i64],
        assets: &[String],
    ) -> Result<Vec<Account>> {
        let rows = sqlx::query(
            "SELECT * FROM accounts WHERE user_id = ANY($1) AND asset = ANY($2)"
        )
        .bind(user_ids)
        .bind(assets)
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(Self::row_to_account).collect()
    }

    // ==================== 辅助方法 ====================

    fn row_to_account(row: impl Row) -> Result<Account> {
        Ok(Account {
            id: row.get("id"),
            user_id: row.get("user_id"),
            asset: row.get("asset"),
            precision: row.get("precision"),
            available: row.get("available"),
            frozen: row.get("frozen"),
            locked: row.get("locked"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}