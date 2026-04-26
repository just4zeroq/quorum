//! Risk Service - 风控服务
//!
//! 适用于预测市场 (Prediction Market) 的现货风控规则
//! 不涉及合约/杠杆/强平等期货概念
//!
//! ## gRPC 接口
//! - CheckRisk: Pre-trade 风控检查

pub mod config;
pub mod errors;
pub mod rules;
pub mod service;

pub use service::RiskServiceImpl;
