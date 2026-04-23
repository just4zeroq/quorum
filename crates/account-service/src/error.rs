//! 错误类型定义
//!
//! 定义 account-service 的错误类型，并提供到 tonic::Status 的转换

use thiserror::Error;
use tonic::{Code, Status};

/// Account Service 错误类型
#[derive(Error, Debug)]
pub enum Error {
    #[error("Account not found: user_id={0}, asset={1}")]
    AccountNotFound(i64, String),

    #[error("Asset not supported: {0}")]
    AssetNotSupported(String),

    #[error("Insufficient balance: available={0}, required={1}")]
    InsufficientBalance(i64, i64),

    #[error("Insufficient frozen: frozen={0}, required={1}")]
    InsufficientFrozen(i64, i64),

    #[error("Insufficient locked: locked={0}, required={1}")]
    InsufficientLocked(i64, i64),

    #[error("Amount invalid: {0}")]
    AmountInvalid(String),

    #[error("Amount too small: {0}")]
    AmountTooSmall(i64),

    #[error("Amount too large: {0}")]
    AmountTooLarge(i64),

    #[error("User frozen: user_id={0}")]
    UserFrozen(i64),

    #[error("Account locked: user_id={0}, asset={1}")]
    AccountLocked(i64, String),

    #[error("Outcome asset invalid: {0}")]
    OutcomeAssetInvalid(String),

    #[error("Settlement failed: {0}")]
    SettlementFailed(String),

    #[error("Duplicate operation: {0}")]
    DuplicateOperation(String),

    #[error("Operation timeout: {0}")]
    OperationTimeout(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Database(err.to_string())
    }
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        match err {
            Error::AccountNotFound(_, _) => Status::new(Code::NotFound, err.to_string()),
            Error::AssetNotSupported(_) => Status::new(Code::InvalidArgument, err.to_string()),
            Error::InsufficientBalance(_, _) => {
                Status::new(Code::FailedPrecondition, err.to_string())
            }
            Error::InsufficientFrozen(_, _) => {
                Status::new(Code::FailedPrecondition, err.to_string())
            }
            Error::InsufficientLocked(_, _) => {
                Status::new(Code::FailedPrecondition, err.to_string())
            }
            Error::AmountInvalid(_) => Status::new(Code::InvalidArgument, err.to_string()),
            Error::AmountTooSmall(_) => Status::new(Code::InvalidArgument, err.to_string()),
            Error::AmountTooLarge(_) => Status::new(Code::InvalidArgument, err.to_string()),
            Error::UserFrozen(_) => Status::new(Code::PermissionDenied, err.to_string()),
            Error::AccountLocked(_, _) => Status::new(Code::PermissionDenied, err.to_string()),
            Error::OutcomeAssetInvalid(_) => {
                Status::new(Code::InvalidArgument, err.to_string())
            }
            Error::SettlementFailed(_) => Status::new(Code::Internal, err.to_string()),
            Error::DuplicateOperation(_) => Status::new(Code::AlreadyExists, err.to_string()),
            Error::OperationTimeout(_) => Status::new(Code::DeadlineExceeded, err.to_string()),
            Error::Database(_) => Status::new(Code::Internal, err.to_string()),
            Error::Internal(_) => Status::new(Code::Internal, err.to_string()),
        }
    }
}

impl From<Error> for Code {
    fn from(err: Error) -> Self {
        match err {
            Error::AccountNotFound(_, _) => Code::NotFound,
            Error::AssetNotSupported(_) => Code::InvalidArgument,
            Error::InsufficientBalance(_, _) => Code::FailedPrecondition,
            Error::InsufficientFrozen(_, _) => Code::FailedPrecondition,
            Error::InsufficientLocked(_, _) => Code::FailedPrecondition,
            Error::AmountInvalid(_) => Code::InvalidArgument,
            Error::AmountTooSmall(_) => Code::InvalidArgument,
            Error::AmountTooLarge(_) => Code::InvalidArgument,
            Error::UserFrozen(_) => Code::PermissionDenied,
            Error::AccountLocked(_, _) => Code::PermissionDenied,
            Error::OutcomeAssetInvalid(_) => Code::InvalidArgument,
            Error::SettlementFailed(_) => Code::Internal,
            Error::DuplicateOperation(_) => Code::AlreadyExists,
            Error::OperationTimeout(_) => Code::DeadlineExceeded,
            Error::Database(_) => Code::Internal,
            Error::Internal(_) => Code::Internal,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_status() {
        let err = Error::InsufficientBalance(100, 200);
        let status: Status = err.into();
        assert_eq!(status.code(), Code::FailedPrecondition);
    }

    #[test]
    fn test_error_to_code() {
        let err = Error::AccountNotFound(1, "USDT".to_string());
        let code: Code = err.into();
        assert_eq!(code, Code::NotFound);
    }
}