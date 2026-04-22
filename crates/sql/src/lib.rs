//! SQL Scripts Module
//!
//! 统一管理所有服务的数据库表创建脚本

pub mod user;
pub mod prediction_market;

pub use user::*;
pub use prediction_market::*;