//! CEX 交易系统公共模块
//!
//! 提供所有服务共享的类型、错误定义和工具函数

pub mod error;
pub mod types;
pub mod constants;

pub use error::{Error, Result};
pub use types::*;
pub use constants::*;