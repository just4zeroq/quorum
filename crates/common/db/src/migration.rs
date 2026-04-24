//! Database Migration Runner
//!
//! 按顺序执行指定目录下的 SQL 迁移脚本。
//! 迁移文件名格式: `{version}_{description}.sql`
//! 通过 `_migrations` 表记录已执行的版本。

use std::path::Path;

use tracing::{info, warn, error};

use crate::{DBError, DBPool, Result};

/// 迁移记录
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub sql: String,
}

/// 迁移执行器
pub struct MigrationRunner;

impl MigrationRunner {
    /// 执行指定目录下的所有未执行迁移
    ///
    /// # 目录结构
    /// ```text
    /// migrations/
    /// ├── 001_init.sql
    /// ├── 002_add_index.sql
    /// └── ...
    /// ```
    pub async fn run_migrations(pool: &DBPool, migrations_dir: impl AsRef<Path>) -> Result<()> {
        let dir = migrations_dir.as_ref();
        if !dir.exists() {
            warn!("Migrations directory does not exist: {:?}", dir);
            return Ok(());
        }

        // 1. 创建迁移记录表
        Self::ensure_migration_table(pool).await?;

        // 2. 读取迁移文件
        let mut migrations = Self::load_migrations(dir)?;
        migrations.sort_by_key(|m| m.version);

        // 3. 获取已执行版本
        let executed = Self::get_executed_versions(pool).await?;

        // 4. 执行未执行的迁移
        for migration in migrations {
            if executed.contains(&migration.version) {
                info!("Migration {} already applied, skipping", migration.version);
                continue;
            }

            info!(
                "Applying migration {}: {} ...",
                migration.version, migration.name
            );

            Self::execute_migration(pool, &migration).await?;

            info!(
                "Migration {} applied successfully",
                migration.version
            );
        }

        Ok(())
    }

    /// 创建迁移记录表
    async fn ensure_migration_table(pool: &DBPool) -> Result<()> {
        match pool {
            DBPool::Postgres(pg) => {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS _migrations (
                        version BIGINT PRIMARY KEY,
                        name VARCHAR(255) NOT NULL,
                        applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                    )
                    "#,
                )
                .execute(pg)
                .await?;
            }
            DBPool::Sqlite(sq) => {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS _migrations (
                        version INTEGER PRIMARY KEY,
                        name TEXT NOT NULL,
                        applied_at TEXT NOT NULL DEFAULT (datetime('now'))
                    )
                    "#,
                )
                .execute(sq)
                .await?;
            }
        }
        Ok(())
    }

    /// 读取迁移文件
    fn load_migrations(dir: &Path) -> Result<Vec<Migration>> {
        let mut migrations = Vec::new();

        for entry in std::fs::read_dir(dir)
            .map_err(|e| DBError::Config(format!("Read migrations dir failed: {}", e)))?
        {
            let entry = entry
                .map_err(|e| DBError::Config(format!("Read dir entry failed: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("sql") {
                continue;
            }

            let file_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| DBError::Config("Invalid migration file name".to_string()))?;

            // Parse version from filename like "001_init"
            let (version_str, name) = match file_name.find('_') {
                Some(idx) => (&file_name[..idx], &file_name[idx + 1..]),
                None => {
                    warn!("Skipping migration file without version prefix: {}", file_name);
                    continue;
                }
            };

            let version: i64 = version_str
                .parse()
                .map_err(|_| DBError::Config(format!("Invalid version in: {}", file_name)))?;

            let sql = std::fs::read_to_string(&path)
                .map_err(|e| DBError::Config(format!("Read migration {} failed: {}", file_name, e)))?;

            migrations.push(Migration {
                version,
                name: name.to_string(),
                sql,
            });
        }

        Ok(migrations)
    }

    /// 获取已执行版本
    async fn get_executed_versions(pool: &DBPool) -> Result<Vec<i64>> {
        match pool {
            DBPool::Postgres(pg) => {
                let rows: Vec<(i64,)> = sqlx::query_as("SELECT version FROM _migrations")
                    .fetch_all(pg)
                    .await?;
                Ok(rows.into_iter().map(|r| r.0).collect())
            }
            DBPool::Sqlite(sq) => {
                let rows: Vec<(i64,)> = sqlx::query_as("SELECT version FROM _migrations")
                    .fetch_all(sq)
                    .await?;
                Ok(rows.into_iter().map(|r| r.0).collect())
            }
        }
    }

    /// 在事务中执行单个迁移
    async fn execute_migration(pool: &DBPool, migration: &Migration) -> Result<()> {
        match pool {
            DBPool::Postgres(pg) => {
                let mut tx = pg.begin().await?;

                // 执行迁移 SQL
                sqlx::query(&migration.sql).execute(&mut *tx).await.map_err(|e| {
                    error!("Migration {} failed: {}", migration.version, e);
                    DBError::Sqlx(e)
                })?;

                // 记录迁移
                sqlx::query("INSERT INTO _migrations (version, name) VALUES ($1, $2)")
                    .bind(migration.version)
                    .bind(&migration.name)
                    .execute(&mut *tx)
                    .await?;

                tx.commit().await?;
            }
            DBPool::Sqlite(sq) => {
                let mut tx = sq.begin().await?;

                sqlx::query(&migration.sql).execute(&mut *tx).await.map_err(|e| {
                    error!("Migration {} failed: {}", migration.version, e);
                    DBError::Sqlx(e)
                })?;

                sqlx::query("INSERT INTO _migrations (version, name) VALUES ($1, $2)")
                    .bind(migration.version)
                    .bind(&migration.name)
                    .execute(&mut *tx)
                    .await?;

                tx.commit().await?;
            }
        }
        Ok(())
    }
}
