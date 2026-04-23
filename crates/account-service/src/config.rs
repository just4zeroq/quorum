//! 配置模块
//!
//! 管理 account-service 的配置，包括数据库配置和资产精度配置

use serde::Deserialize;
use std::fs;
use std::path::Path;
use db::Config as DBConfig;

/// 服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// 服务配置
    pub service: ServiceConfig,
    /// 数据库配置
    pub database: DBConfig,
    /// 资产精度配置
    pub assets: AssetsConfig,
    /// 风控配置
    pub risk: Option<RiskConfig>,
}

/// 服务元信息
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "account-service".to_string(),
            host: "0.0.0.0".to_string(),
            port: 50019,
        }
    }
}

/// 资产精度配置
#[derive(Debug, Clone, Deserialize)]
pub struct AssetsConfig {
    /// 基础资产 (USDT) 配置
    pub base: BaseAssetConfig,
    /// 结果代币配置
    pub outcome: OutcomeAssetConfig,
}

/// 基础资产配置
#[derive(Debug, Clone, Deserialize)]
pub struct BaseAssetConfig {
    pub name: String,
    pub precision: u8,
}

/// 结果代币配置
#[derive(Debug, Clone, Deserialize)]
pub struct OutcomeAssetConfig {
    pub precision: u8,
}

/// 风控配置
#[derive(Debug, Clone, Deserialize)]
pub struct RiskConfig {
    pub max_deposit: Option<String>,
    pub max_withdraw: Option<String>,
}

impl Default for AssetsConfig {
    fn default() -> Self {
        Self {
            base: BaseAssetConfig {
                name: "USDT".to_string(),
                precision: 6,
            },
            outcome: OutcomeAssetConfig {
                precision: 4,
            },
        }
    }
}

impl Config {
    /// 从配置文件加载
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 加载默认配置
    pub fn load_default() -> Self {
        Self::load("config/account_service.yaml").unwrap_or_else(|_| {
            Config {
                service: ServiceConfig::default(),
                database: DBConfig::default(),
                assets: AssetsConfig::default(),
                risk: None,
            }
        })
    }

    /// 获取合并后的数据库配置
    pub fn merged_db_config(&self) -> db::MergedConfig {
        self.database.merge()
    }

    /// 获取基础资产精度
    pub fn base_precision(&self) -> u8 {
        self.assets.base.precision
    }

    /// 获取结果代币默认精度
    pub fn outcome_precision(&self) -> u8 {
        self.assets.outcome.precision
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::load_default();
        assert_eq!(config.service.port, 50019);
        assert_eq!(config.assets.base.precision, 6);
        assert_eq!(config.assets.outcome.precision, 4);
    }
}