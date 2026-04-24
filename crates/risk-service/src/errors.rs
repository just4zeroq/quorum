//! Risk Service 错误定义

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RiskError {
    #[error("Risk check rejected: {0}")]
    Rejected(String),

    #[error("Invalid parameter: {0}")]
    InvalidParam(String),
}
