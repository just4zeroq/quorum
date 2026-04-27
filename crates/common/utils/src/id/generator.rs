//! ID Generator Core - ID 生成器核心

use std::sync::atomic::{AtomicU64, Ordering};
use chrono::Utc;

/// ID 前缀
pub trait IdPrefix: Clone + Copy + Send + Sync + std::fmt::Debug {
    fn prefix() -> &'static str;
}

/// 通用 ID 生成器
pub struct IdGenerator<P: IdPrefix> {
    #[allow(dead_code)]
    prefix: P,
    sequence: AtomicU64,
}

impl<P: IdPrefix> IdGenerator<P> {
    pub fn new(prefix: P) -> Self {
        Self {
            prefix,
            sequence: AtomicU64::new(0),
        }
    }

    /// 生成 ID
    ///
    /// 格式: {前缀}{时间6位}{分片2位}{序列6位}
    /// 例: o26042215304501000001 (订单)
    ///     t26042215304501000001 (成交)
    pub fn generate(&self, shard_id: u64) -> String {
        let timestamp = Utc::now().format("%y%m%d%H%M%S");
        let shard = shard_id % 100;
        let seq = self.next_sequence();

        format!("{}{}{:02}{:06}", P::prefix(), timestamp, shard, seq)
    }

    /// 获取下一个序列号
    fn next_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::Relaxed) % 1_000_000
    }
}

/// 静态 ID 生成 (非线程安全，适合单次生成)
pub struct StaticIdGenerator;

impl StaticIdGenerator {
    /// 生成 ID
    pub fn generate(prefix: &str, shard_id: u64) -> String {
        let timestamp = Utc::now().format("%y%m%d%H%M%S");
        let shard = shard_id % 100;
        let seq = rand::random::<u64>() % 1_000_000;

        format!("{}{}{:02}{:06}", prefix, timestamp, shard, seq)
    }

    /// 生成带自定义序列的 ID
    pub fn generate_with_seq(prefix: &str, shard_id: u64, sequence: u64) -> String {
        let timestamp = Utc::now().format("%y%m%d%H%M%S");
        let shard = shard_id % 100;
        let seq = sequence % 1_000_000;

        format!("{}{}{:02}{:06}", prefix, timestamp, shard, seq)
    }
}

/// ID 解析
pub struct IdParser;

impl IdParser {
    /// 解析 ID 各部分
    ///
    /// 返回: (前缀, 时间, 分片, 序列)
    pub fn parse(id: &str) -> Option<(&str, &str, &str, &str)> {
        if id.len() < 16 {
            return None;
        }

        let prefix = &id[0..1];
        let time = &id[1..13];
        let shard = &id[13..15];
        let seq = &id[15..21];

        Some((prefix, time, shard, seq))
    }

    /// 提取时间
    pub fn extract_time(id: &str) -> Option<&str> {
        if id.len() < 13 {
            return None;
        }
        Some(&id[1..13])
    }

    /// 提取序列号
    pub fn extract_sequence(id: &str) -> Option<u64> {
        if id.len() < 21 {
            return None;
        }
        id[15..21].parse().ok()
    }

    /// 验证 ID 格式
    pub fn validate(id: &str) -> bool {
        Self::parse(id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_generator() {
        let id = StaticIdGenerator::generate("o", 1);
        assert_eq!(id.len(), 21);
        println!("Generated ID: {}", id);

        let parsed = IdParser::parse(&id).unwrap();
        println!("Parsed: prefix={}, time={}, shard={}, seq={}",
            parsed.0, parsed.1, parsed.2, parsed.3);
    }

    #[test]
    fn test_validate() {
        let valid_id = "o26042215304501000001";
        let invalid_id = "short";

        assert!(IdParser::validate(valid_id));
        assert!(!IdParser::validate(invalid_id));
    }

    #[test]
    fn test_parse() {
        let id = "t26042215304599000123";
        let parsed = IdParser::parse(id).unwrap();

        assert_eq!(parsed.0, "t");
        assert_eq!(parsed.1, "260422153045");
        assert_eq!(parsed.2, "99");
        assert_eq!(parsed.3, "000123");
    }
}