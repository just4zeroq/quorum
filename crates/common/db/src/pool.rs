use std::sync::Arc;
use sqlx::{postgres::PgPoolOptions, sqlite::SqlitePoolOptions, PgPool, SqlitePool};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::config::MergedConfig;

#[derive(Error, Debug)]
pub enum DBError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Unsupported database type: {0}")]
    UnsupportedDBType(String),
}

pub type Result<T> = std::result::Result<T, DBError>;

/// 数据库连接池
#[derive(Clone)]
pub enum DBPool {
    /// PostgreSQL 连接池
    Postgres(PgPool),
    /// SQLite 连接池
    Sqlite(SqlitePool),
}

impl DBPool {
    /// 创建数据库连接池
    pub async fn new(config: &MergedConfig) -> Result<Self> {
        let url = config
            .url()
            .ok_or_else(|| DBError::Config("Database URL not found".to_string()))?;

        if config.is_sqlite {
            let pool = SqlitePoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .connect(&url)
                .await?;
            Ok(DBPool::Sqlite(pool))
        } else {
            let pool = PgPoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .acquire_timeout(std::time::Duration::from_millis(config.connection_timeout_ms))
                .idle_timeout(std::time::Duration::from_millis(config.idle_timeout_ms))
                .connect(&url)
                .await?;
            Ok(DBPool::Postgres(pool))
        }
    }

    /// 是否是 SQLite
    pub fn is_sqlite(&self) -> bool {
        matches!(self, DBPool::Sqlite(_))
    }

    /// 获取 PostgreSQL 连接池引用
    pub fn pg_pool(&self) -> Option<&PgPool> {
        match self {
            DBPool::Postgres(pool) => Some(pool),
            _ => None,
        }
    }

    /// 获取 SQLite 连接池引用
    pub fn sqlite_pool(&self) -> Option<&SqlitePool> {
        match self {
            DBPool::Sqlite(pool) => Some(pool),
            _ => None,
        }
    }

    /// 创建表
    pub async fn create_tables(&self) -> Result<()> {
        match self {
            DBPool::Sqlite(pool) => {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS users (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        username TEXT NOT NULL UNIQUE,
                        email TEXT NOT NULL UNIQUE,
                        password_hash TEXT,
                        phone TEXT,
                        kyc_status TEXT NOT NULL DEFAULT 'none',
                        kyc_level INTEGER NOT NULL DEFAULT 0,
                        kyc_submitted_at TEXT,
                        kyc_verified_at TEXT,
                        two_factor_enabled INTEGER NOT NULL DEFAULT 0,
                        two_factor_secret TEXT,
                        status TEXT NOT NULL DEFAULT 'active',
                        status_reason TEXT,
                        frozen_at TEXT,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL,
                        last_login_at TEXT
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS wallet_addresses (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        user_id INTEGER NOT NULL,
                        wallet_address TEXT NOT NULL,
                        wallet_type TEXT NOT NULL,
                        chain_type TEXT NOT NULL,
                        is_primary INTEGER NOT NULL DEFAULT 0,
                        verified_at TEXT,
                        created_at TEXT NOT NULL,
                        FOREIGN KEY (user_id) REFERENCES users(id)
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS user_sessions (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        user_id INTEGER NOT NULL,
                        token TEXT NOT NULL UNIQUE,
                        refresh_token TEXT,
                        ip_address TEXT,
                        user_agent TEXT,
                        device_id TEXT,
                        login_method TEXT NOT NULL,
                        expires_at TEXT NOT NULL,
                        created_at TEXT NOT NULL,
                        last_active_at TEXT NOT NULL,
                        FOREIGN KEY (user_id) REFERENCES users(id)
                    )
                    "#,
                )
                .execute(pool)
                .await?;
            }
            DBPool::Postgres(pool) => {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS users (
                        id BIGSERIAL PRIMARY KEY,
                        username VARCHAR(50) NOT NULL UNIQUE,
                        email VARCHAR(255) NOT NULL UNIQUE,
                        password_hash VARCHAR(255),
                        phone VARCHAR(50),
                        kyc_status VARCHAR(20) NOT NULL DEFAULT 'none',
                        kyc_level INTEGER NOT NULL DEFAULT 0,
                        kyc_submitted_at TIMESTAMP,
                        kyc_verified_at TIMESTAMP,
                        two_factor_enabled BOOLEAN NOT NULL DEFAULT false,
                        two_factor_secret VARCHAR(255),
                        status VARCHAR(20) NOT NULL DEFAULT 'active',
                        status_reason TEXT,
                        frozen_at TIMESTAMP,
                        created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                        updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
                        last_login_at TIMESTAMP
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS wallet_addresses (
                        id BIGSERIAL PRIMARY KEY,
                        user_id BIGINT NOT NULL REFERENCES users(id),
                        wallet_address VARCHAR(100) NOT NULL,
                        wallet_type VARCHAR(20) NOT NULL,
                        chain_type VARCHAR(20) NOT NULL,
                        is_primary BOOLEAN NOT NULL DEFAULT false,
                        verified_at TIMESTAMP,
                        created_at TIMESTAMP NOT NULL DEFAULT NOW()
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS user_sessions (
                        id BIGSERIAL PRIMARY KEY,
                        user_id BIGINT NOT NULL REFERENCES users(id),
                        token TEXT NOT NULL UNIQUE,
                        refresh_token TEXT,
                        ip_address VARCHAR(50),
                        user_agent TEXT,
                        device_id VARCHAR(100),
                        login_method VARCHAR(20) NOT NULL,
                        expires_at TIMESTAMP NOT NULL,
                        created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                        last_active_at TIMESTAMP NOT NULL DEFAULT NOW()
                    )
                    "#,
                )
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    /// 关闭连接池
    pub async fn close(&self) {
        match self {
            DBPool::Postgres(pool) => pool.close().await,
            DBPool::Sqlite(pool) => pool.close().await,
        }
    }
}

/// 数据库连接池管理器
pub struct DBManager {
    pool: Arc<RwLock<Option<DBPool>>>,
    config: MergedConfig,
}

impl DBManager {
    pub fn new(config: MergedConfig) -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// 初始化连接池
    pub async fn init(&self) -> Result<()> {
        let pool = DBPool::new(&self.config).await?;
        let mut guard = self.pool.write().await;
        *guard = Some(pool);
        Ok(())
    }

    /// 获取连接池
    pub async fn get_pool(&self) -> Option<DBPool> {
        let guard = self.pool.read().await;
        guard.clone()
    }

    /// 关闭连接池
    pub async fn close(&self) {
        let mut guard = self.pool.write().await;
        if let Some(pool) = guard.take() {
            pool.close().await;
        }
    }
}