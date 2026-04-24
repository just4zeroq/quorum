//! Database common library
//!
//! Support for PostgreSQL and SQLite

pub mod config;
pub mod pool;
pub mod migration;

pub use config::{Config, MergedConfig};
pub use pool::{DBError, DBManager, DBPool, Result};
pub use migration::{MigrationRunner, Migration};

#[cfg(feature = "postgres")]
pub use sqlx::postgres::{PgPool, PgRow};

#[cfg(feature = "sqlite")]
pub use sqlx::sqlite::{SqlitePool, SqliteRow};

#[cfg(feature = "postgres")]
pub use sqlx::Row as AnyRow;

#[cfg(not(feature = "postgres"))]
pub use sqlx::sqlite::SqliteRow as AnyRow;