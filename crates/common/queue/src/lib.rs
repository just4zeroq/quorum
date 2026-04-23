//! Queue common library (支持 Redis 和 Kafka 双后端)

pub mod config;
pub mod producer;
pub mod consumer;
pub mod consumer_handler;

pub mod kafka_producer;
pub mod kafka_consumer;

pub use config::{Backend, Config, MergedConfig};
pub use producer::{Message, MessageProducer, ProducerError, ProducerManager, Result};
pub use consumer::{ConsumeMessage, MessageConsumer, ConsumerError, ConsumerManager, Result as ConsumerResult};
pub use consumer_handler::{ConsumerManagerWithHandler, MessageHandler};
