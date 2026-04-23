//! Account Service
//!
//! 账户余额管理服务
//! 支持: USDT 基础资产 + {market_id}_{outcome} 结果代币

pub mod config;
pub mod error;
pub mod models;
pub mod precision;
pub mod pb;
pub mod repository;
pub mod server;
pub mod services;

pub use config::Config;
pub use error::{Error, Result};
pub use server::AccountServer;