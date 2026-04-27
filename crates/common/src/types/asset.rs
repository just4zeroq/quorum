//! 资产类型

use serde::{Deserialize, Serialize};

/// 资产类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Asset {
    USDT,
    BTC,
    ETH,
    BNB,
    SOL,
    /// 其他资产
    Other(String),
}

impl Asset {
    pub fn as_str(&self) -> &str {
        match self {
            Asset::USDT => "USDT",
            Asset::BTC => "BTC",
            Asset::ETH => "ETH",
            Asset::BNB => "BNB",
            Asset::SOL => "SOL",
            Asset::Other(s) => s,
        }
    }
}

impl From<&str> for Asset {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "USDT" => Asset::USDT,
            "BTC" => Asset::BTC,
            "ETH" => Asset::ETH,
            "BNB" => Asset::BNB,
            "SOL" => Asset::SOL,
            other => Asset::Other(other.to_string()),
        }
    }
}
