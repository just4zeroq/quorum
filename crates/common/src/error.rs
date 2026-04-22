//! 公共错误类型定义

use thiserror::Error;

/// 系统统一错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// 业务错误
    #[error("业务错误: {0}")]
    Business(String),

    /// 参数错误
    #[error("参数错误: {0}")]
    InvalidParam(String),

    /// 资金不足
    #[error("资金不足: {0}")]
    InsufficientBalance(String),

    /// 余额冻结失败
    #[error("余额冻结失败: {0}")]
    FreezeFailed(String),

    /// 订单不存在
    #[error("订单不存在: {0}")]
    OrderNotFound(String),

    /// 订单已存在
    #[error("订单已存在: {0}")]
    OrderAlreadyExists(String),

    /// 撮合错误
    #[error("撮合错误: {0}")]
    MatchingError(String),

    /// 风控拒绝
    #[error("风控拒绝: {0}")]
    RiskRejected(String),

    /// 持仓不存在
    #[error("持仓不存在: {0}")]
    PositionNotFound(String),

    /// 账户不存在
    #[error("账户不存在: {0}")]
    AccountNotFound(String),

    /// 账本错误
    #[error("账本错误: {0}")]
    LedgerError(String),

    /// 结算错误
    #[error("结算错误: {0}")]
    ClearingError(String),

    /// 网络错误
    #[error("网络错误: {0}")]
    NetworkError(String),

    /// 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    SerializationError(String),

    /// 认证错误
    #[error("认证错误: {0}")]
    AuthError(String),

    /// 权限错误
    #[error("权限错误: {0}")]
    PermissionDenied(String),

    /// 限流错误
    #[error("限流错误: {0}")]
    RateLimitExceeded(String),

    /// 强平错误
    #[error("强平错误: {0}")]
    LiquidationError(String),

    /// 钱包错误
    #[error("钱包错误: {0}")]
    WalletError(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),

    /// 未实现
    #[error("未实现: {0}")]
    Unimplemented(String),
}

/// 系统统一结果类型
pub type Result<T> = std::result::Result<T, Error>;

/// 将标准错误转换为系统错误
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Internal(e.to_string())
    }
}

/// 将序列化错误转换为系统错误
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerializationError(e.to_string())
    }
}