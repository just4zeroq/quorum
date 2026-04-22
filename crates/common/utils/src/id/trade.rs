//! Trade ID Generator - 成交 ID 生成器

use super::{StaticIdGenerator, IdParser};

/// 成交 ID 前缀
pub const TRADE_PREFIX: &str = "t";

/// 生成成交 ID
///
/// 格式: t{时间6位}{市场后2位}{序列6位}
///
/// 例: t26042215304501000001
///     ││││││││││││││││││└── 序列号 (0-999999)
///     │││││││││││││││││└─── 市场ID后2位 (01)
///     │││││││││││││││└──── 22年
///     ││││││││││││││└───── 04月
///     │││││││││││││└────── 22日
///     ││││││││││││└─────── 15时
///     │││││││││││└──────── 30分
///     ││││││││││└───────── 45秒
///     │└────────────────────── 前缀 (t = trade)
///
/// 总长度: 21位
pub fn generate_trade_id(market_id: i64) -> String {
    StaticIdGenerator::generate(TRADE_PREFIX, market_id as u64)
}

/// 生成成交 ID (带自定义序列)
pub fn generate_trade_id_with_seq(market_id: i64, sequence: u64) -> String {
    StaticIdGenerator::generate_with_seq(TRADE_PREFIX, market_id as u64, sequence)
}

/// 验证成交 ID
pub fn validate_trade_id(id: &str) -> bool {
    match IdParser::parse(id) {
        Some((prefix, _, _, _)) => prefix == TRADE_PREFIX,
        None => false,
    }
}

/// 从成交 ID 提取时间
pub fn extract_trade_time(id: &str) -> Option<&str> {
    IdParser::extract_time(id)
}

/// 从成交 ID 提取序列
pub fn extract_trade_sequence(id: &str) -> Option<u64> {
    IdParser::extract_sequence(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_trade_id() {
        let id = generate_trade_id(1);
        assert_eq!(id.len(), 21);
        assert!(id.starts_with('t'));
        println!("Trade ID: {}", id);

        assert!(validate_trade_id(&id));
    }

    #[test]
    fn test_trade_id_format() {
        let id = generate_trade_id(1);
        let parsed = IdParser::parse(&id).unwrap();

        assert_eq!(parsed.0, "t");
        println!("Parsed: {:?}", parsed);
    }
}