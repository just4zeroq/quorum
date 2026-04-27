//! Portfolio Service Repository
//!
//! 统一的数据访问层，封装所有 SQL 操作。
//! 所有余额/持仓更新使用乐观锁 (version 字段)。

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::{PgPool, SqlitePool};

use crate::errors::PortfolioError;
use crate::models::{Account, AccountType, Position, PositionSide, Settlement, LedgerEntry, LedgerType};

/// 数据库连接池包装
#[derive(Debug, Clone)]
pub enum PortfolioPool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

impl PortfolioPool {
    pub fn from_db_pool(pool: db::DBPool) -> Self {
        match pool {
            db::DBPool::Postgres(p) => PortfolioPool::Postgres(p),
            db::DBPool::Sqlite(p) => PortfolioPool::Sqlite(p),
        }
    }

    pub async fn begin_tx(&self) -> Result<PortfolioTransaction<'_>, PortfolioError> {
        match self {
            PortfolioPool::Postgres(pool) => {
                let tx = pool.begin().await?;
                Ok(PortfolioTransaction::Postgres(tx))
            }
            PortfolioPool::Sqlite(pool) => {
                let tx = pool.begin().await?;
                Ok(PortfolioTransaction::Sqlite(tx))
            }
        }
    }
}

/// 数据库事务包装
///
/// 提供与 PortfolioRepository 相同的数据操作，但运行在事务中。
/// 用于需要原子性的批量操作（如 settle_trade）。
pub enum PortfolioTransaction<'a> {
    Postgres(sqlx::Transaction<'a, sqlx::Postgres>),
    Sqlite(sqlx::Transaction<'a, sqlx::Sqlite>),
}

impl PortfolioTransaction<'_> {
    pub async fn commit(self) -> Result<(), PortfolioError> {
        match self {
            PortfolioTransaction::Postgres(tx) => tx.commit().await?,
            PortfolioTransaction::Sqlite(tx) => tx.commit().await?,
        }
        Ok(())
    }

    pub async fn get_account(&mut self, user_id: &str, asset: &str) -> Result<Option<Account>, PortfolioError> {
        match self {
            PortfolioTransaction::Postgres(tx) => {
                let row = sqlx::query_as::<_, AccountRow>(
                    "SELECT id, user_id, asset, account_type, available, frozen, version, created_at, updated_at FROM accounts WHERE user_id = $1 AND asset = $2"
                )
                .bind(user_id)
                .bind(asset)
                .fetch_optional(tx.as_mut())
                .await?;
                Ok(row.map(Into::into))
            }
            PortfolioTransaction::Sqlite(tx) => {
                let row = sqlx::query_as::<_, AccountRow>(
                    "SELECT id, user_id, asset, account_type, available, frozen, version, created_at, updated_at FROM accounts WHERE user_id = ?1 AND asset = ?2"
                )
                .bind(user_id)
                .bind(asset)
                .fetch_optional(tx.as_mut())
                .await?;
                Ok(row.map(Into::into))
            }
        }
    }

    pub async fn debit_available_with_version(&mut self, user_id: &str, asset: &str, amount: Decimal, version: i64) -> Result<u64, PortfolioError> {
        let rows = match self {
            PortfolioTransaction::Postgres(tx) => {
                sqlx::query(
                    "UPDATE accounts SET available = available - $1, version = version + 1, updated_at = NOW() WHERE user_id = $2 AND asset = $3 AND version = $4 AND available >= $1"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(tx.as_mut())
                .await?
                .rows_affected()
            }
            PortfolioTransaction::Sqlite(tx) => {
                sqlx::query(
                    "UPDATE accounts SET available = available - ?1, version = version + 1, updated_at = datetime('now') WHERE user_id = ?2 AND asset = ?3 AND version = ?4 AND available >= ?1"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(tx.as_mut())
                .await?
                .rows_affected()
            }
        };
        Ok(rows)
    }

    pub async fn credit_with_version(&mut self, user_id: &str, asset: &str, amount: Decimal, version: i64) -> Result<u64, PortfolioError> {
        let rows = match self {
            PortfolioTransaction::Postgres(tx) => {
                sqlx::query(
                    "UPDATE accounts SET available = available + $1, version = version + 1, updated_at = NOW() WHERE user_id = $2 AND asset = $3 AND version = $4"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(tx.as_mut())
                .await?
                .rows_affected()
            }
            PortfolioTransaction::Sqlite(tx) => {
                sqlx::query(
                    "UPDATE accounts SET available = available + ?1, version = version + 1, updated_at = datetime('now') WHERE user_id = ?2 AND asset = ?3 AND version = ?4"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(tx.as_mut())
                .await?
                .rows_affected()
            }
        };
        Ok(rows)
    }

    pub async fn create_account(&mut self, account: &Account) -> Result<(), PortfolioError> {
        match self {
            PortfolioTransaction::Postgres(tx) => {
                sqlx::query(
                    "INSERT INTO accounts (id, user_id, asset, account_type, available, frozen, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
                )
                .bind(&account.id)
                .bind(&account.user_id)
                .bind(&account.asset)
                .bind(account.account_type.as_str())
                .bind(account.available.to_string())
                .bind(account.frozen.to_string())
                .bind(account.version)
                .bind(account.created_at)
                .bind(account.updated_at)
                .execute(tx.as_mut())
                .await?;
            }
            PortfolioTransaction::Sqlite(tx) => {
                sqlx::query(
                    "INSERT INTO accounts (id, user_id, asset, account_type, available, frozen, version, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
                )
                .bind(&account.id)
                .bind(&account.user_id)
                .bind(&account.asset)
                .bind(account.account_type.as_str())
                .bind(account.available.to_string())
                .bind(account.frozen.to_string())
                .bind(account.version)
                .bind(account.created_at)
                .bind(account.updated_at)
                .execute(tx.as_mut())
                .await?;
            }
        }
        Ok(())
    }

    pub async fn get_or_create_account(&mut self, user_id: &str, asset: &str) -> Result<Account, PortfolioError> {
        if let Some(account) = self.get_account(user_id, asset).await? {
            return Ok(account);
        }
        let account = Account {
            id: format!("acc_{}_{}", user_id, asset),
            user_id: user_id.to_string(),
            asset: asset.to_string(),
            account_type: AccountType::Spot,
            available: Decimal::ZERO,
            frozen: Decimal::ZERO,
            version: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.create_account(&account).await?;
        Ok(account)
    }

    pub async fn get_position(&mut self, user_id: &str, market_id: i64, outcome_id: i64, side: &str) -> Result<Option<Position>, PortfolioError> {
        match self {
            PortfolioTransaction::Postgres(tx) => {
                let row = sqlx::query_as::<_, PositionRow>(
                    "SELECT id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at FROM positions WHERE user_id = $1 AND market_id = $2 AND outcome_id = $3 AND side = $4"
                )
                .bind(user_id)
                .bind(market_id)
                .bind(outcome_id)
                .bind(side)
                .fetch_optional(tx.as_mut())
                .await?;
                Ok(row.map(Into::into))
            }
            PortfolioTransaction::Sqlite(tx) => {
                let row = sqlx::query_as::<_, PositionRow>(
                    "SELECT id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at FROM positions WHERE user_id = ?1 AND market_id = ?2 AND outcome_id = ?3 AND side = ?4"
                )
                .bind(user_id)
                .bind(market_id)
                .bind(outcome_id)
                .bind(side)
                .fetch_optional(tx.as_mut())
                .await?;
                Ok(row.map(Into::into))
            }
        }
    }

    /// 带乐观锁的持仓 upsert（事务内）
    pub async fn upsert_position_with_version(&mut self, pos: &mut Position) -> Result<bool, PortfolioError> {
        let rows = match self {
            PortfolioTransaction::Postgres(tx) => {
                sqlx::query(
                    r#"
                    INSERT INTO positions (id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    ON CONFLICT (user_id, market_id, outcome_id, side) DO UPDATE SET
                        size = EXCLUDED.size,
                        entry_price = EXCLUDED.entry_price,
                        version = EXCLUDED.version,
                        updated_at = NOW()
                    WHERE positions.version = EXCLUDED.version - 1
                    "#
                )
                .bind(&pos.id)
                .bind(&pos.user_id)
                .bind(pos.market_id as i64)
                .bind(pos.outcome_id as i64)
                .bind(pos.side.as_str())
                .bind(pos.size.to_string())
                .bind(pos.entry_price.to_string())
                .bind(pos.version)
                .bind(pos.created_at)
                .bind(pos.updated_at)
                .execute(tx.as_mut())
                .await?
                .rows_affected()
            }
            PortfolioTransaction::Sqlite(tx) => {
                sqlx::query(
                    r#"
                    INSERT INTO positions (id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    ON CONFLICT (user_id, market_id, outcome_id, side) DO UPDATE SET
                        size = EXCLUDED.size,
                        entry_price = EXCLUDED.entry_price,
                        version = EXCLUDED.version,
                        updated_at = datetime('now')
                    WHERE positions.version = EXCLUDED.version - 1
                    "#
                )
                .bind(&pos.id)
                .bind(&pos.user_id)
                .bind(pos.market_id as i64)
                .bind(pos.outcome_id as i64)
                .bind(pos.side.as_str())
                .bind(pos.size.to_string())
                .bind(pos.entry_price.to_string())
                .bind(pos.version)
                .bind(pos.created_at)
                .bind(pos.updated_at)
                .execute(tx.as_mut())
                .await?
                .rows_affected()
            }
        };
        Ok(rows == 1)
    }

    pub async fn insert_settlement(&mut self, s: &Settlement) -> Result<(), PortfolioError> {
        match self {
            PortfolioTransaction::Postgres(tx) => {
                sqlx::query(
                    "INSERT INTO settlements (id, trade_id, market_id, user_id, outcome_id, side, amount, fee, payout, status, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
                )
                .bind(&s.id)
                .bind(&s.trade_id)
                .bind(s.market_id as i64)
                .bind(&s.user_id)
                .bind(s.outcome_id as i64)
                .bind(s.side.as_str())
                .bind(s.amount.to_string())
                .bind(s.fee.to_string())
                .bind(s.payout.to_string())
                .bind(s.status.as_str())
                .bind(s.created_at)
                .execute(tx.as_mut())
                .await?;
            }
            PortfolioTransaction::Sqlite(tx) => {
                sqlx::query(
                    "INSERT INTO settlements (id, trade_id, market_id, user_id, outcome_id, side, amount, fee, payout, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
                )
                .bind(&s.id)
                .bind(&s.trade_id)
                .bind(s.market_id as i64)
                .bind(&s.user_id)
                .bind(s.outcome_id as i64)
                .bind(s.side.as_str())
                .bind(s.amount.to_string())
                .bind(s.fee.to_string())
                .bind(s.payout.to_string())
                .bind(s.status.as_str())
                .bind(s.created_at)
                .execute(tx.as_mut())
                .await?;
            }
        }
        Ok(())
    }

    pub async fn insert_ledger(&mut self, entry: &LedgerEntry) -> Result<(), PortfolioError> {
        match self {
            PortfolioTransaction::Postgres(tx) => {
                sqlx::query(
                    "INSERT INTO ledger_entries (id, user_id, account_id, ledger_type, asset, amount, balance_after, reference_id, reference_type, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
                )
                .bind(&entry.id)
                .bind(&entry.user_id)
                .bind(&entry.account_id)
                .bind(entry.ledger_type.as_str())
                .bind(&entry.asset)
                .bind(entry.amount.to_string())
                .bind(entry.balance_after.to_string())
                .bind(&entry.reference_id)
                .bind(&entry.reference_type)
                .bind(entry.created_at)
                .execute(tx.as_mut())
                .await?;
            }
            PortfolioTransaction::Sqlite(tx) => {
                sqlx::query(
                    "INSERT INTO ledger_entries (id, user_id, account_id, ledger_type, asset, amount, balance_after, reference_id, reference_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
                )
                .bind(&entry.id)
                .bind(&entry.user_id)
                .bind(&entry.account_id)
                .bind(entry.ledger_type.as_str())
                .bind(&entry.asset)
                .bind(entry.amount.to_string())
                .bind(entry.balance_after.to_string())
                .bind(&entry.reference_id)
                .bind(&entry.reference_type)
                .bind(entry.created_at)
                .execute(tx.as_mut())
                .await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PortfolioRepository {
    pool: PortfolioPool,
}

impl PortfolioRepository {
    pub fn new(pool: PortfolioPool) -> Self {
        Self { pool }
    }

    pub fn from_db_pool(pool: db::DBPool) -> Self {
        Self::new(PortfolioPool::from_db_pool(pool))
    }

    pub async fn begin_tx(&self) -> Result<PortfolioTransaction<'_>, PortfolioError> {
        self.pool.begin_tx().await
    }

    // ==================== Account ====================

    pub async fn get_account(&self, user_id: &str, asset: &str) -> Result<Option<Account>, PortfolioError> {
        match &self.pool {
            PortfolioPool::Postgres(pool) => {
                let row = sqlx::query_as::
                    <_, AccountRow>(
                    "SELECT id, user_id, asset, account_type, available, frozen, version, created_at, updated_at FROM accounts WHERE user_id = $1 AND asset = $2"
                )
                .bind(user_id)
                .bind(asset)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(Into::into))
            }
            PortfolioPool::Sqlite(pool) => {
                let row = sqlx::query_as::
                    <_, AccountRow>(
                    "SELECT id, user_id, asset, account_type, available, frozen, version, created_at, updated_at FROM accounts WHERE user_id = ?1 AND asset = ?2"
                )
                .bind(user_id)
                .bind(asset)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(Into::into))
            }
        }
    }

    pub async fn get_account_by_id(&self, id: &str) -> Result<Option<Account>, PortfolioError> {
        match &self.pool {
            PortfolioPool::Postgres(pool) => {
                let row = sqlx::query_as::
                    <_, AccountRow>(
                    "SELECT id, user_id, asset, account_type, available, frozen, version, created_at, updated_at FROM accounts WHERE id = $1"
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(Into::into))
            }
            PortfolioPool::Sqlite(pool) => {
                let row = sqlx::query_as::
                    <_, AccountRow>(
                    "SELECT id, user_id, asset, account_type, available, frozen, version, created_at, updated_at FROM accounts WHERE id = ?1"
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(Into::into))
            }
        }
    }

    pub async fn create_account(&self, account: &Account) -> Result<(), PortfolioError> {
        match &self.pool {
            PortfolioPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO accounts (id, user_id, asset, account_type, available, frozen, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
                )
                .bind(&account.id)
                .bind(&account.user_id)
                .bind(&account.asset)
                .bind(account.account_type.as_str())
                .bind(account.available.to_string())
                .bind(account.frozen.to_string())
                .bind(account.version)
                .bind(account.created_at)
                .bind(account.updated_at)
                .execute(pool)
                .await?;
            }
            PortfolioPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO accounts (id, user_id, asset, account_type, available, frozen, version, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
                )
                .bind(&account.id)
                .bind(&account.user_id)
                .bind(&account.asset)
                .bind(account.account_type.as_str())
                .bind(account.available.to_string())
                .bind(account.frozen.to_string())
                .bind(account.version)
                .bind(account.created_at)
                .bind(account.updated_at)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    /// 乐观锁冻结资金
    pub async fn freeze_with_version(&self, user_id: &str, asset: &str, amount: Decimal, version: i64) -> Result<u64, PortfolioError> {
        let rows = match &self.pool {
            PortfolioPool::Postgres(pool) => {
                sqlx::query(
                    "UPDATE accounts SET available = available - $1, frozen = frozen + $1, version = version + 1, updated_at = NOW() WHERE user_id = $2 AND asset = $3 AND version = $4 AND available >= $1"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(pool)
                .await?
                .rows_affected()
            }
            PortfolioPool::Sqlite(pool) => {
                sqlx::query(
                    "UPDATE accounts SET available = available - ?1, frozen = frozen + ?1, version = version + 1, updated_at = datetime('now') WHERE user_id = ?2 AND asset = ?3 AND version = ?4 AND available >= ?1"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(pool)
                .await?
                .rows_affected()
            }
        };
        Ok(rows)
    }

    /// 增加可用余额（乐观锁）
    pub async fn credit_with_version(&self, user_id: &str, asset: &str, amount: Decimal, version: i64) -> Result<u64, PortfolioError> {
        let rows = match &self.pool {
            PortfolioPool::Postgres(pool) => {
                sqlx::query(
                    "UPDATE accounts SET available = available + $1, version = version + 1, updated_at = NOW() WHERE user_id = $2 AND asset = $3 AND version = $4"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(pool)
                .await?
                .rows_affected()
            }
            PortfolioPool::Sqlite(pool) => {
                sqlx::query(
                    "UPDATE accounts SET available = available + ?1, version = version + 1, updated_at = datetime('now') WHERE user_id = ?2 AND asset = ?3 AND version = ?4"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(pool)
                .await?
                .rows_affected()
            }
        };
        Ok(rows)
    }

    /// 扣除可用余额（乐观锁，带余额检查）
    pub async fn debit_available_with_version(&self, user_id: &str, asset: &str, amount: Decimal, version: i64) -> Result<u64, PortfolioError> {
        let rows = match &self.pool {
            PortfolioPool::Postgres(pool) => {
                sqlx::query(
                    "UPDATE accounts SET available = available - $1, version = version + 1, updated_at = NOW() WHERE user_id = $2 AND asset = $3 AND version = $4 AND available >= $1"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(pool)
                .await?
                .rows_affected()
            }
            PortfolioPool::Sqlite(pool) => {
                sqlx::query(
                    "UPDATE accounts SET available = available - ?1, version = version + 1, updated_at = datetime('now') WHERE user_id = ?2 AND asset = ?3 AND version = ?4 AND available >= ?1"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(pool)
                .await?
                .rows_affected()
            }
        };
        Ok(rows)
    }

    /// 乐观锁解冻资金
    pub async fn unfreeze_with_version(&self, user_id: &str, asset: &str, amount: Decimal, version: i64) -> Result<u64, PortfolioError> {
        let rows = match &self.pool {
            PortfolioPool::Postgres(pool) => {
                sqlx::query(
                    "UPDATE accounts SET available = available + $1, frozen = frozen - $1, version = version + 1, updated_at = NOW() WHERE user_id = $2 AND asset = $3 AND version = $4 AND frozen >= $1"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(pool)
                .await?
                .rows_affected()
            }
            PortfolioPool::Sqlite(pool) => {
                sqlx::query(
                    "UPDATE accounts SET available = available + ?1, frozen = frozen - ?1, version = version + 1, updated_at = datetime('now') WHERE user_id = ?2 AND asset = ?3 AND version = ?4 AND frozen >= ?1"
                )
                .bind(amount.to_string())
                .bind(user_id)
                .bind(asset)
                .bind(version)
                .execute(pool)
                .await?
                .rows_affected()
            }
        };
        Ok(rows)
    }

    // ==================== Position ====================

    pub async fn get_position(&self, user_id: &str, market_id: i64, outcome_id: i64, side: &str) -> Result<Option<Position>, PortfolioError> {
        match &self.pool {
            PortfolioPool::Postgres(pool) => {
                let row = sqlx::query_as::
                    <_, PositionRow>(
                    "SELECT id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at FROM positions WHERE user_id = $1 AND market_id = $2 AND outcome_id = $3 AND side = $4"
                )
                .bind(user_id)
                .bind(market_id)
                .bind(outcome_id)
                .bind(side)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(Into::into))
            }
            PortfolioPool::Sqlite(pool) => {
                let row = sqlx::query_as::
                    <_, PositionRow>(
                    "SELECT id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at FROM positions WHERE user_id = ?1 AND market_id = ?2 AND outcome_id = ?3 AND side = ?4"
                )
                .bind(user_id)
                .bind(market_id)
                .bind(outcome_id)
                .bind(side)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(Into::into))
            }
        }
    }

    /// 带乐观锁的持仓 upsert
    /// 返回 true 表示写入成功, false 表示版本冲突需重试
    pub async fn upsert_position_with_version(&self, pos: &Position) -> Result<bool, PortfolioError> {
        let rows = match &self.pool {
            PortfolioPool::Postgres(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO positions (id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    ON CONFLICT (user_id, market_id, outcome_id, side) DO UPDATE SET
                        size = EXCLUDED.size,
                        entry_price = EXCLUDED.entry_price,
                        version = EXCLUDED.version,
                        updated_at = NOW()
                    WHERE positions.version = EXCLUDED.version - 1
                    "#
                )
                .bind(&pos.id)
                .bind(&pos.user_id)
                .bind(pos.market_id as i64)
                .bind(pos.outcome_id as i64)
                .bind(pos.side.as_str())
                .bind(pos.size.to_string())
                .bind(pos.entry_price.to_string())
                .bind(pos.version)
                .bind(pos.created_at)
                .bind(pos.updated_at)
                .execute(pool)
                .await?
                .rows_affected()
            }
            PortfolioPool::Sqlite(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO positions (id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    ON CONFLICT (user_id, market_id, outcome_id, side) DO UPDATE SET
                        size = EXCLUDED.size,
                        entry_price = EXCLUDED.entry_price,
                        version = EXCLUDED.version,
                        updated_at = datetime('now')
                    WHERE positions.version = EXCLUDED.version - 1
                    "#
                )
                .bind(&pos.id)
                .bind(&pos.user_id)
                .bind(pos.market_id as i64)
                .bind(pos.outcome_id as i64)
                .bind(pos.side.as_str())
                .bind(pos.size.to_string())
                .bind(pos.entry_price.to_string())
                .bind(pos.version)
                .bind(pos.created_at)
                .bind(pos.updated_at)
                .execute(pool)
                .await?
                .rows_affected()
            }
        };
        Ok(rows == 1)
    }

    pub async fn upsert_position(&self, pos: &Position) -> Result<(), PortfolioError> {
        match &self.pool {
            PortfolioPool::Postgres(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO positions (id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    ON CONFLICT (user_id, market_id, outcome_id, side) DO UPDATE SET
                        size = EXCLUDED.size,
                        entry_price = EXCLUDED.entry_price,
                        version = EXCLUDED.version,
                        updated_at = NOW()
                    "#
                )
                .bind(&pos.id)
                .bind(&pos.user_id)
                .bind(pos.market_id as i64)
                .bind(pos.outcome_id as i64)
                .bind(pos.side.as_str())
                .bind(pos.size.to_string())
                .bind(pos.entry_price.to_string())
                .bind(pos.version)
                .bind(pos.created_at)
                .bind(pos.updated_at)
                .execute(pool)
                .await?;
            }
            PortfolioPool::Sqlite(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO positions (id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    ON CONFLICT (user_id, market_id, outcome_id, side) DO UPDATE SET
                        size = EXCLUDED.size,
                        entry_price = EXCLUDED.entry_price,
                        version = EXCLUDED.version,
                        updated_at = datetime('now')
                    "#
                )
                .bind(&pos.id)
                .bind(&pos.user_id)
                .bind(pos.market_id as i64)
                .bind(pos.outcome_id as i64)
                .bind(pos.side.as_str())
                .bind(pos.size.to_string())
                .bind(pos.entry_price.to_string())
                .bind(pos.version)
                .bind(pos.created_at)
                .bind(pos.updated_at)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    // ==================== Ledger ====================

    pub async fn insert_ledger(&self, entry: &LedgerEntry) -> Result<(), PortfolioError> {
        match &self.pool {
            PortfolioPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO ledger_entries (id, user_id, account_id, ledger_type, asset, amount, balance_after, reference_id, reference_type, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
                )
                .bind(&entry.id)
                .bind(&entry.user_id)
                .bind(&entry.account_id)
                .bind(entry.ledger_type.as_str())
                .bind(&entry.asset)
                .bind(entry.amount.to_string())
                .bind(entry.balance_after.to_string())
                .bind(&entry.reference_id)
                .bind(&entry.reference_type)
                .bind(entry.created_at)
                .execute(pool)
                .await?;
            }
            PortfolioPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO ledger_entries (id, user_id, account_id, ledger_type, asset, amount, balance_after, reference_id, reference_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
                )
                .bind(&entry.id)
                .bind(&entry.user_id)
                .bind(&entry.account_id)
                .bind(entry.ledger_type.as_str())
                .bind(&entry.asset)
                .bind(entry.amount.to_string())
                .bind(entry.balance_after.to_string())
                .bind(&entry.reference_id)
                .bind(&entry.reference_type)
                .bind(entry.created_at)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn list_ledger_by_user(&self, user_id: &str, limit: i32, offset: i32) -> Result<Vec<LedgerEntry>, PortfolioError> {
        match &self.pool {
            PortfolioPool::Postgres(pool) => {
                let rows: Vec<LedgerRow> = sqlx::query_as(
                    "SELECT id, user_id, account_id, ledger_type, asset, amount, balance_after, reference_id, reference_type, created_at FROM ledger_entries WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
                )
                .bind(user_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(Into::into).collect())
            }
            PortfolioPool::Sqlite(pool) => {
                let rows: Vec<LedgerRow> = sqlx::query_as(
                    "SELECT id, user_id, account_id, ledger_type, asset, amount, balance_after, reference_id, reference_type, created_at FROM ledger_entries WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
                )
                .bind(user_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(Into::into).collect())
            }
        }
    }

    pub async fn list_positions(&self, user_id: &str, market_id: u64) -> Result<Vec<Position>, PortfolioError> {
        match &self.pool {
            PortfolioPool::Postgres(pool) => {
                let rows: Vec<PositionRow> = sqlx::query_as(
                    "SELECT id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at FROM positions WHERE user_id = $1 AND market_id = $2"
                )
                .bind(user_id)
                .bind(market_id as i64)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(Into::into).collect())
            }
            PortfolioPool::Sqlite(pool) => {
                let rows: Vec<PositionRow> = sqlx::query_as(
                    "SELECT id, user_id, market_id, outcome_id, side, size, entry_price, version, created_at, updated_at FROM positions WHERE user_id = ?1 AND market_id = ?2"
                )
                .bind(user_id)
                .bind(market_id as i64)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(Into::into).collect())
            }
        }
    }

    // ==================== Settlement ====================

    pub async fn insert_settlement(&self, s: &Settlement) -> Result<(), PortfolioError> {
        match &self.pool {
            PortfolioPool::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO settlements (id, trade_id, market_id, user_id, outcome_id, side, amount, fee, payout, status, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
                )
                .bind(&s.id)
                .bind(&s.trade_id)
                .bind(s.market_id as i64)
                .bind(&s.user_id)
                .bind(s.outcome_id as i64)
                .bind(s.side.as_str())
                .bind(s.amount.to_string())
                .bind(s.fee.to_string())
                .bind(s.payout.to_string())
                .bind(s.status.as_str())
                .bind(s.created_at)
                .execute(pool)
                .await?;
            }
            PortfolioPool::Sqlite(pool) => {
                sqlx::query(
                    "INSERT INTO settlements (id, trade_id, market_id, user_id, outcome_id, side, amount, fee, payout, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
                )
                .bind(&s.id)
                .bind(&s.trade_id)
                .bind(s.market_id as i64)
                .bind(&s.user_id)
                .bind(s.outcome_id as i64)
                .bind(s.side.as_str())
                .bind(s.amount.to_string())
                .bind(s.fee.to_string())
                .bind(s.payout.to_string())
                .bind(s.status.as_str())
                .bind(s.created_at)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
}

// ==================== SQLx Row Mappings ====================

#[derive(sqlx::FromRow)]
struct AccountRow {
    id: String,
    user_id: String,
    asset: String,
    account_type: String,
    available: String,
    frozen: String,
    version: i64,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<AccountRow> for Account {
    fn from(r: AccountRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            asset: r.asset,
            account_type: r.account_type.parse().unwrap_or(AccountType::Spot),
            available: r.available.parse().unwrap_or_default(),
            frozen: r.frozen.parse().unwrap_or_default(),
            version: r.version,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct PositionRow {
    id: String,
    user_id: String,
    market_id: i64,
    outcome_id: i64,
    side: String,
    size: String,
    entry_price: String,
    version: i64,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<PositionRow> for Position {
    fn from(r: PositionRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            market_id: r.market_id as u64,
            outcome_id: r.outcome_id as u64,
            side: r.side.parse().unwrap_or(PositionSide::Long),
            size: r.size.parse().unwrap_or_default(),
            entry_price: r.entry_price.parse().unwrap_or_default(),
            version: r.version,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LedgerRow {
    id: String,
    user_id: String,
    account_id: String,
    ledger_type: String,
    asset: String,
    amount: String,
    balance_after: String,
    reference_id: String,
    reference_type: String,
    created_at: chrono::DateTime<Utc>,
}

impl From<LedgerRow> for LedgerEntry {
    fn from(r: LedgerRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            account_id: r.account_id,
            ledger_type: r.ledger_type.parse().unwrap_or(LedgerType::Trade),
            asset: r.asset,
            amount: r.amount.parse().unwrap_or_default(),
            balance_after: r.balance_after.parse().unwrap_or_default(),
            reference_id: r.reference_id,
            reference_type: r.reference_type,
            created_at: r.created_at,
        }
    }
}
