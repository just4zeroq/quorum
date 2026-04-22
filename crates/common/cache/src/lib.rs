//! Cache common library (Redis)

pub mod config;
pub mod client;

pub use config::{Config, MergedConfig};
pub use client::{CacheError, CacheManager, RedisClient, Result};