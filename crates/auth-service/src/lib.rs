//! Auth Service - Authentication & Authorization
//!
//! 提供 API 和 WebSocket 鉴权服务

pub mod pb;
pub mod models;
pub mod errors;
pub mod service;
pub mod repository;

pub use errors::{AuthError, Result};
pub use service::AuthServiceImpl;
