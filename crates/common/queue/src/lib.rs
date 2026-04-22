//! Queue common library (支持 Redis 和 Kafka 双后端)

pub mod config;
pub mod producer;
pub mod consumer;

pub use config::{Backend, Config, MergedConfig};
pub use producer::{Message, MessageProducer, ProducerError, ProducerManager, Result};