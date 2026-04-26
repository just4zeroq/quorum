//! Prediction Market Service
//!
//! 预测市场服务 - 管理预测市场事件、结果选项、行情数据和结算

pub mod config;
pub mod models;
pub mod repository;
pub mod services;
pub mod server;

pub use config::Config;
pub use server::PredictionMarketServer;