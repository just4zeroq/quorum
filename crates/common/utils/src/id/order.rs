//! Order ID Generator - 订单 ID 生成器

use super::{StaticIdGenerator, IdParser};

/// 订单 ID 前缀
pub const ORDER_PREFIX: &str = "o";

/// 生成订单 ID
///
/// 格式: o{时间6位}{市场后2位}{用户后2位}{序列6位}
///
/// 例: o260422153045010123456789
///     │││││││││││││││││││└── 序列号 (0-999999)
///     ││││││││││││││││││└─── 用户ID后2位 (01)
///     │││││││││││││││││└─── 市场ID后2位 (01)
///     │││││││││││││││└──── 22年
///     ││││││││││││││└───── 04月
///     │││││││││││││└────── 22日
///     ││││││││││││└─────── 15时
///     │││││││││││└──────── 30分
///     ││││││││││└───────── 45秒
///     │└────────────────────── 前缀 (o = order)
///
/// 总长度: 21位
pub fn generate_order_id(market_id: i64, user_id: i64) -> String {
    let shard = (market_id * 100 + user_id % 100) as u64;
    StaticIdGenerator::generate(ORDER_PREFIX, shard)
}

/// 生成订单 ID (带自定义序列)
pub fn generate_order_id_with_seq(market_id: i64, user_id: i64, sequence: u64) -> String {
    let shard = (market_id * 100 + user_id % 100) as u64;
    StaticIdGenerator::generate_with_seq(ORDER_PREFIX, shard, sequence)
}

/// 验证订单 ID
pub fn validate_order_id(id: &str) -> bool {
    match IdParser::parse(id) {
        Some((prefix, _, _, _)) => prefix == ORDER_PREFIX,
        None => false,
    }
}

/// 从订单 ID 提取时间
pub fn extract_order_time(id: &str) -> Option<&str> {
    IdParser::extract_time(id)
}

/// 从订单 ID 提取序列
pub fn extract_order_sequence(id: &str) -> Option<u64> {
    IdParser::extract_sequence(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_order_id() {
        let id = generate_order_id(1, 100);
        assert_eq!(id.len(), 21);
        assert!(id.starts_with('o'));
        println!("Order ID: {}", id);

        assert!(validate_order_id(&id));
    }

    #[test]
    fn test_order_id_format() {
        let id = generate_order_id(1, 100);
        let parsed = IdParser::parse(&id).unwrap();

        assert_eq!(parsed.0, "o");
        println!("Parsed: {:?}", parsed);
    }
}