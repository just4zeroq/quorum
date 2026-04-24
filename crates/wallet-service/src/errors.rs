//! Wallet Service Errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Address not found: {0}")]
    AddressNotFound(String),

    #[error("Deposit not found: {0}")]
    DepositNotFound(String),

    #[error("Deposit already confirmed: {0}")]
    DepositAlreadyConfirmed(String),

    #[error("Withdraw not found: {0}")]
    WithdrawNotFound(String),

    #[error("Withdraw already confirmed: {0}")]
    WithdrawAlreadyConfirmed(String),

    #[error("Withdraw not pending: {0}")]
    WithdrawNotPending(String),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Insufficient fee")]
    InsufficientFee,

    #[error("Address not whitelisted: {0}")]
    AddressNotWhitelisted(String),

    #[error("Payment password required")]
    PaymentPasswordRequired,

    #[error("Payment password invalid")]
    PaymentPasswordInvalid,

    #[error("Chain not supported: {0}")]
    ChainNotSupported(String),

    #[error("Broadcast failed: {0}")]
    BroadcastFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, WalletError>;
