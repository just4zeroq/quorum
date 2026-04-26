//! Portfolio Service Configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub service: ServiceConfig,
    pub database: DatabaseConfig,
    pub etcd_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub driver: String,
    pub url: String,
    pub max_connections: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service: ServiceConfig {
                host: "0.0.0.0".to_string(),
                port: 50003,
            },
            database: DatabaseConfig {
                driver: "sqlite".to_string(),
                url: "sqlite:./data/portfolio.db".to_string(),
                max_connections: 20,
            },
            etcd_endpoints: vec!["http://127.0.0.1:2379".to_string()],
        }
    }
}