//! Market Data Service Configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub service: ServiceConfig,
    pub db: DbConfig,
    pub redis: RedisConfig,
    pub prediction_market: PredictionMarketConfig,
    pub etcd_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DbConfig {
    pub db_type: String,
    pub file_path: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub db: u8,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PredictionMarketConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_yaml::from_str(&content).ok()
    }

    pub fn load_default() -> Self {
        let config_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("config/market-data-service.yaml"))
            .unwrap_or_else(|| std::path::Path::new("config/market-data-service.yaml").to_path_buf());

        Self::from_file(&config_path).unwrap_or_default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service: ServiceConfig {
                name: "market-data-service".to_string(),
                host: "0.0.0.0".to_string(),
                port: 50006,
            },
            db: DbConfig {
                db_type: "sqlite".to_string(),
                file_path: Some("data/market-data-service.db".to_string()),
                host: Some("localhost".to_string()),
                port: Some(5432),
                username: Some("postgres".to_string()),
                password: Some("postgres".to_string()),
                database: Some("market_data".to_string()),
            },
            redis: RedisConfig {
                host: "localhost".to_string(),
                port: 6379,
                db: 0,
                password: None,
            },
            prediction_market: PredictionMarketConfig {
                host: "localhost".to_string(),
                port: 50010,
            },
            etcd_endpoints: vec!["http://127.0.0.1:2379".to_string()],
        }
    }
}