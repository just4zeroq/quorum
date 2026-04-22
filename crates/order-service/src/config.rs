//! Configuration for Order Service

use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Service configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub service: ServiceConfig,
    pub database: DatabaseConfig,
    pub matching_engine: MatchingEngineConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchingEngineConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_default() -> Self {
        Self::load("config/config.yaml").unwrap_or_else(|_| {
            Self::load("/home/ubuntu/code/cex-dex/rust-cex/crates/order-service/config/config.yaml")
                .unwrap_or_else(|_| {
                    Config {
                        service: ServiceConfig {
                            name: "order-service".to_string(),
                            host: "0.0.0.0".to_string(),
                            port: 50003,
                        },
                        database: DatabaseConfig {
                            url: "sqlite:data/order_service.db".to_string(),
                            max_connections: 10,
                        },
                        matching_engine: MatchingEngineConfig {
                            host: "127.0.0.1".to_string(),
                            port: 50009,
                        },
                    }
                })
        })
    }
}