//! Storage Backends

pub mod memory;
pub mod redis;

pub use memory::MemoryStore;
pub use redis::RedisStore;
