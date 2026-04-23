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
    pub queue: QueueConfig,
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

#[derive(Debug, Clone, Deserialize)]
pub struct QueueConfig {
    pub backend: String,
    pub brokers: Vec<String>,
    pub group_id: String,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_default() -> Self {
        Self::load("config/config.yaml").unwrap_or_else(|_| {
            Self {
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
                queue: QueueConfig {
                    backend: "redis".to_string(),
                    brokers: vec!["localhost:9092".to_string()],
                    group_id: "order-service".to_string(),
                },
            }
        })
    }
}

impl From<DatabaseConfig> for db::config::Config {
    fn from(db: DatabaseConfig) -> Self {
        let is_sqlite = db.url.starts_with("sqlite:");
        db::config::Config {
            db_type: Some(if is_sqlite { "sqlite".to_string() } else { "postgres".to_string() }),
            host: if is_sqlite { None } else { Some("localhost".to_string()) },
            port: if is_sqlite { None } else { Some(5432) },
            user: if is_sqlite { None } else { Some("postgres".to_string()) },
            password: if is_sqlite { None } else { Some("postgres".to_string()) },
            database: if is_sqlite { None } else { Some("cex_dex".to_string()) },
            max_connections: Some(db.max_connections),
            min_connections: Some(1),
            connection_timeout_ms: Some(5000),
            idle_timeout_ms: Some(60000),
            file_path: if is_sqlite {
                // 从 sqlite:data/order_service.db 提取路径
                let path = db.url.trim_start_matches("sqlite:");
                Some(path.to_string())
            } else {
                None
            },
        }
    }
}
