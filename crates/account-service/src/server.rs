//! Server 模块
//!
//! gRPC Server 启动和初始化

use db::{DBManager, DBPool};
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic_reflection::server::Builder;

use crate::config::Config;
use crate::precision::AssetPrecision;
use crate::services::AccountServiceImpl;
use crate::pb::account_service_server::account_service_server::AccountServiceServer;

/// Account Service Server
pub struct AccountServer {
    config: Config,
}

impl AccountServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 启动服务
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 使用 common/db: 通过 DBManager 初始化
        let merged_config = self.config.merged_db_config();
        let db_manager = DBManager::new(merged_config);
        db_manager.init().await?;

        let pool = db_manager.get_pool()
            .await
            .ok_or("Failed to get DB pool")?;

        // 自行建表 (不使用 DBPool::create_tables，因为那是 user-service 的表)
        Self::init_tables(&pool).await?;

        // 创建精度管理器
        let asset_precision = AssetPrecision::new(&self.config.assets);

        // 创建服务
        let account_service = AccountServiceImpl::new(pool, asset_precision);
        let addr: SocketAddr = format!("{}:{}", self.config.service.host, self.config.service.port)
            .parse()?;

        tracing::info!("Starting Account Service on {}", addr);

        // 添加反射服务
        let reflection_service = Builder::configure()
            .register_encoded_file_descriptor_set(include_bytes!("pb/account.desc"))
            .build_v1()?;

        // 构建 gRPC 服务器
        Server::builder()
            .add_service(reflection_service)
            .add_service(AccountServiceServer::new(account_service))
            .serve(addr)
            .await?;

        Ok(())
    }

    /// 初始化数据库表
    async fn init_tables(pool: &DBPool) -> Result<(), sqlx::Error> {
        match pool {
            DBPool::Sqlite(pool) => {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS accounts (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        user_id INTEGER NOT NULL,
                        asset TEXT NOT NULL,
                        precision INTEGER NOT NULL,
                        available INTEGER NOT NULL DEFAULT 0,
                        frozen INTEGER NOT NULL DEFAULT 0,
                        locked INTEGER NOT NULL DEFAULT 0,
                        created_at INTEGER NOT NULL,
                        updated_at INTEGER NOT NULL
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_user_asset ON accounts(user_id, asset)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_id ON accounts(user_id)")
                    .execute(pool)
                    .await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS balance_operations (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        account_id INTEGER NOT NULL,
                        user_id INTEGER NOT NULL,
                        asset TEXT NOT NULL,
                        operation_type TEXT NOT NULL,
                        amount INTEGER NOT NULL,
                        balance_before INTEGER NOT NULL,
                        balance_after INTEGER NOT NULL,
                        frozen_before INTEGER NOT NULL,
                        frozen_after INTEGER NOT NULL,
                        reason TEXT,
                        ref_id TEXT,
                        created_at INTEGER NOT NULL
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_account_id ON balance_operations(account_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_user_id ON balance_operations(user_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_ref_id ON balance_operations(ref_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_created_at ON balance_operations(created_at)")
                    .execute(pool)
                    .await?;
            }
            DBPool::Postgres(pool) => {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS accounts (
                        id BIGSERIAL PRIMARY KEY,
                        user_id BIGINT NOT NULL,
                        asset VARCHAR(50) NOT NULL,
                        precision INTEGER NOT NULL,
                        available BIGINT NOT NULL DEFAULT 0,
                        frozen BIGINT NOT NULL DEFAULT 0,
                        locked BIGINT NOT NULL DEFAULT 0,
                        created_at BIGINT NOT NULL,
                        updated_at BIGINT NOT NULL
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_user_asset ON accounts(user_id, asset)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_id ON accounts(user_id)")
                    .execute(pool)
                    .await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS balance_operations (
                        id BIGSERIAL PRIMARY KEY,
                        account_id BIGINT NOT NULL,
                        user_id BIGINT NOT NULL,
                        asset VARCHAR(50) NOT NULL,
                        operation_type VARCHAR(30) NOT NULL,
                        amount BIGINT NOT NULL,
                        balance_before BIGINT NOT NULL,
                        balance_after BIGINT NOT NULL,
                        frozen_before BIGINT NOT NULL,
                        frozen_after BIGINT NOT NULL,
                        reason TEXT,
                        ref_id TEXT,
                        created_at BIGINT NOT NULL
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_account_id ON balance_operations(account_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_user_id ON balance_operations(user_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_ref_id ON balance_operations(ref_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_created_at ON balance_operations(created_at)")
                    .execute(pool)
                    .await?;
            }
        }

        tracing::info!("Account tables initialized");
        Ok(())
    }
}