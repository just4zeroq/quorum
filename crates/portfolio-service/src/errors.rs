//! Portfolio Service 错误定义

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PortfolioError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Insufficient balance: available={available}, required={required}")]
    InsufficientBalance {
        available: String,
        required: String,
    },

    #[error("Position not found: {0}")]
    PositionNotFound(String),

    #[error("Insufficient position size: available={available}, required={required}")]
    InsufficientPosition {
        available: String,
        required: String,
    },

    #[error("Settlement failed: {0}")]
    SettlementFailed(String),

    #[error("Ledger entry failed: {0}")]
    LedgerFailed(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Optimistic lock failed after retries: {0}")]
    OptimisticLockFailed(String),
}

impl From<sqlx::Error> for PortfolioError {
    fn from(e: sqlx::Error) -> Self {
        PortfolioError::Database(e.to_string())
    }
}
