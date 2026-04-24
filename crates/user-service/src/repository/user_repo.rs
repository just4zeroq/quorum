//! 用户 Repository

use std::sync::Arc;
use crate::config::Config;
use db::{DBPool, DBError};
use sqlx::Row;

pub struct UserRepository {
    pool: DBPool,
    _config: Arc<Config>,
}

impl UserRepository {
    pub async fn new(config: &Config) -> Result<Self, String> {
        let db_config = config.db.clone().unwrap_or_default();
        let merged = db_config.merge();

        let pool = DBPool::new(&merged)
            .await
            .map_err(|e| e.to_string())?;

        // 创建表
        pool.create_tables().await.map_err(|e| e.to_string())?;

        Ok(Self {
            pool,
            _config: Arc::new(config.clone()),
        })
    }

    /// 创建用户
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<i64, DBError> {
        if self.pool.is_sqlite() {
            let pool = self.pool.sqlite_pool().unwrap();
            let row = sqlx::query(
                "INSERT INTO users (username, email, password_hash, kyc_status, kyc_level, two_factor_enabled, status, created_at, updated_at) VALUES (?, ?, ?, 'none', 0, 0, 'active', datetime('now'), datetime('now')) RETURNING id"
            )
            .bind(username)
            .bind(email)
            .bind(password_hash)
            .fetch_one(pool)
            .await?;
            Ok(row.get::<i64, _>("id"))
        } else {
            let pool = self.pool.pg_pool().unwrap();
            let row = sqlx::query(
                "INSERT INTO users (username, email, password_hash, kyc_status, kyc_level, two_factor_enabled, status, created_at, updated_at) VALUES ($1, $2, $3, 'none', 0, false, 'active', NOW(), NOW()) RETURNING id"
            )
            .bind(username)
            .bind(email)
            .bind(password_hash)
            .fetch_one(pool)
            .await?;
            Ok(row.get::<i64, _>("id"))
        }
    }

    /// 根据 ID 查找用户
    pub async fn find_by_id(&self, id: i64) -> Result<Option<UserRow>, DBError> {
        if self.pool.is_sqlite() {
            let pool = self.pool.sqlite_pool().unwrap();
            let row = sqlx::query_as::<_, UserRowSqlite>(
                "SELECT id, username, email, kyc_status, kyc_level, two_factor_enabled, status FROM users WHERE id = ?"
            )
            .bind(id)
            .fetch_optional(pool)
            .await?;

            Ok(row.map(|r| UserRow {
                id: r.id,
                username: r.username,
                email: r.email,
                kyc_status: r.kyc_status,
                kyc_level: r.kyc_level,
                two_factor_enabled: r.two_factor_enabled,
                status: r.status,
            }))
        } else {
            let pool = self.pool.pg_pool().unwrap();
            let row = sqlx::query_as::<_, UserRowPg>(
                "SELECT id, username, email, kyc_status, kyc_level, two_factor_enabled, status FROM users WHERE id = $1"
            )
            .bind(id)
            .fetch_optional(pool)
            .await?;

            Ok(row.map(|r| UserRow {
                id: r.id,
                username: r.username,
                email: r.email,
                kyc_status: r.kyc_status,
                kyc_level: r.kyc_level,
                two_factor_enabled: r.two_factor_enabled,
                status: r.status,
            }))
        }
    }

    /// 根据邮箱查找用户
    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserRow>, DBError> {
        if self.pool.is_sqlite() {
            let pool = self.pool.sqlite_pool().unwrap();
            let row = sqlx::query_as::<_, UserRowSqlite>(
                "SELECT id, username, email, kyc_status, kyc_level, two_factor_enabled, status FROM users WHERE email = ?"
            )
            .bind(email)
            .fetch_optional(pool)
            .await?;

            Ok(row.map(|r| UserRow {
                id: r.id,
                username: r.username,
                email: r.email,
                kyc_status: r.kyc_status,
                kyc_level: r.kyc_level,
                two_factor_enabled: r.two_factor_enabled,
                status: r.status,
            }))
        } else {
            let pool = self.pool.pg_pool().unwrap();
            let row = sqlx::query_as::<_, UserRowPg>(
                "SELECT id, username, email, kyc_status, kyc_level, two_factor_enabled, status FROM users WHERE email = $1"
            )
            .bind(email)
            .fetch_optional(pool)
            .await?;

            Ok(row.map(|r| UserRow {
                id: r.id,
                username: r.username,
                email: r.email,
                kyc_status: r.kyc_status,
                kyc_level: r.kyc_level,
                two_factor_enabled: r.two_factor_enabled,
                status: r.status,
            }))
        }
    }
}

/// 用户行
#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub kyc_status: String,
    pub kyc_level: i32,
    pub two_factor_enabled: bool,
    pub status: String,
}

// SQLite 行类型
#[derive(sqlx::FromRow)]
struct UserRowSqlite {
    id: i64,
    username: String,
    email: String,
    kyc_status: String,
    kyc_level: i32,
    two_factor_enabled: bool,
    status: String,
}

// PostgreSQL 行类型
#[derive(sqlx::FromRow)]
struct UserRowPg {
    id: i64,
    username: String,
    email: String,
    kyc_status: String,
    kyc_level: i32,
    two_factor_enabled: bool,
    status: String,
}