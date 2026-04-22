//! Order Service - 订单服务
//!
//! 负责订单管理、状态维护

pub mod config;
pub mod models;
pub mod repository;
pub mod services;
pub mod server;
pub mod pb;

pub use config::Config;
pub use server::OrderServer;
pub use models::{Order, OrderQuery, CreateOrderRequest, OrderEventRecord};