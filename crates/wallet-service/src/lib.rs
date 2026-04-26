//! Wallet Service - 钱包服务
//!
//! 提供充值、提现、地址管理、白名单、支付密码等功能

pub mod models;
pub mod errors;
pub mod repository;
pub mod services;
pub mod config;

pub use errors::{WalletError, Result};
