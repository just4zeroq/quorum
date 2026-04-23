//! 资产精度管理
//!
//! 管理资产精度 (小数位数)，提供 i64 整数与人类可读金额之间的转换

use crate::config::AssetsConfig;

/// 资产精度管理器
///
/// 根据资产类型自动匹配精度
pub struct AssetPrecision {
    /// 基础资产精度 (USDT)
    base_precision: u8,
    /// 结果代币默认精度
    outcome_precision: u8,
}

impl AssetPrecision {
    /// 创建新的精度管理器
    pub fn new(config: &AssetsConfig) -> Self {
        Self {
            base_precision: config.base.precision,
            outcome_precision: config.outcome.precision,
        }
    }

    /// 获取基础资产精度 (USDT)
    pub fn base_precision(&self) -> u8 {
        self.base_precision
    }

    /// 获取结果代币默认精度
    pub fn outcome_precision(&self) -> u8 {
        self.outcome_precision
    }

    /// 获取资产的精度
    ///
    /// 规则:
    /// - USDT 使用基础资产精度
    /// - 结果代币格式 `{market_id}_{outcome}` 使用结果代币精度
    pub fn get_precision(&self, asset: &str) -> u8 {
        if asset == "USDT" {
            self.base_precision
        } else if Self::is_outcome_asset(asset) {
            self.outcome_precision
        } else {
            // 未知资产类型，默认使用基础资产精度
            self.base_precision
        }
    }

    /// 判断是否为结果代币
    ///
    /// 结果代币格式: `{market_id}_{outcome}`，如 "12345_yes", "67890_no"
    pub fn is_outcome_asset(asset: &str) -> bool {
        // 结果代币包含下划线，且第二部分为 yes/no 或其他 outcome 标识
        if let Some(underscore_pos) = asset.find('_') {
            let outcome = &asset[underscore_pos + 1..];
            matches!(outcome, "yes" | "no" | "true" | "false")
        } else {
            false
        }
    }

    /// 存储整数转换为人类可读金额字符串
    ///
    /// 例: to_human(1500000, 6) → "1.500000"
    /// 例: to_human(1000000, 4) → "100.0000"
    pub fn to_human(stored: i64, precision: u8) -> String {
        let divisor = 10i64.pow(precision as u32);
        let int_part = stored / divisor;
        let frac_part = (stored % divisor).abs();
        format!("{}.{:0>width$}", int_part, frac_part, width = precision as usize)
    }

    /// 人类可读金额转换为存储整数
    ///
    /// 例: from_human("1.5", 6) → 1500000
    /// 注意: 此函数假设输入是有效的数字字符串
    pub fn from_human(amount: &str, precision: u8) -> Option<i64> {
        let parts: Vec<&str> = amount.split('.').collect();
        let multiplier = 10i64.pow(precision as u32);

        match parts.len() {
            1 => {
                // 整数部分
                parts[0].parse::<i64>().ok().map(|n| n * multiplier)
            }
            2 => {
                // 整数部分和小数部分
                let int_part: i64 = parts[0].parse().unwrap_or(0);
                let frac_str = parts[1];
                // 确保小数部分长度不超过 precision
                let frac_part: i64 = if frac_str.len() >= precision as usize {
                    (&frac_str[..precision as usize]).parse().unwrap_or(0)
                } else {
                    let padded = format!("{:0<width$}", frac_str, width = precision as usize);
                    padded.parse().unwrap_or(0)
                };
                Some(int_part * multiplier + frac_part)
            }
            _ => None,
        }
    }

    /// 检查金额是否有效 (非零正整数)
    pub fn is_valid_amount(amount: i64) -> bool {
        amount > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_precision() -> AssetPrecision {
        AssetPrecision {
            base_precision: 6,
            outcome_precision: 4,
        }
    }

    #[test]
    fn test_is_outcome_asset() {
        assert!(AssetPrecision::is_outcome_asset("12345_yes"));
        assert!(AssetPrecision::is_outcome_asset("67890_no"));
        assert!(!AssetPrecision::is_outcome_asset("USDT"));
        assert!(!AssetPrecision::is_outcome_asset("BTC"));
    }

    #[test]
    fn test_get_precision() {
        let precision = create_test_precision();
        assert_eq!(precision.get_precision("USDT"), 6);
        assert_eq!(precision.get_precision("12345_yes"), 4);
        assert_eq!(precision.get_precision("67890_no"), 4);
    }

    #[test]
    fn test_to_human() {
        let precision = create_test_precision();
        assert_eq!(AssetPrecision::to_human(1500000, 6), "1.500000");
        assert_eq!(AssetPrecision::to_human(1000000, 4), "100.0000");
        assert_eq!(AssetPrecision::to_human(0, 6), "0.000000");
    }

    #[test]
    fn test_from_human() {
        let precision = create_test_precision();
        assert_eq!(AssetPrecision::from_human("1.5", 6), Some(1500000));
        assert_eq!(AssetPrecision::from_human("100", 4), Some(1000000));
        assert_eq!(AssetPrecision::from_human("0.123", 4), Some(1230));
    }

    #[test]
    fn test_is_valid_amount() {
        assert!(AssetPrecision::is_valid_amount(100));
        assert!(!AssetPrecision::is_valid_amount(0));
        assert!(!AssetPrecision::is_valid_amount(-100));
    }
}