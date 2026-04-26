//! Wallet Service Configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub service: ServiceConfig,
    pub database: DatabaseConfig,
    pub wallet: WalletConfig,
    pub portfolio_service_addr: String,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletConfig {
    pub supported_chains: Vec<String>,
    pub require_whitelist: bool,
    pub require_payment_password: bool,
    pub default_fee: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service: ServiceConfig {
                host: "0.0.0.0".to_string(),
                port: 50002,
            },
            database: DatabaseConfig {
                driver: "sqlite".to_string(),
                url: "sqlite:./data/wallet.db".to_string(),
                max_connections: 20,
            },
            wallet: WalletConfig {
                supported_chains: vec!["ETH".to_string(), "BSC".to_string(), "ARBITRUM".to_string()],
                require_whitelist: false,
                require_payment_password: true,
                default_fee: "0.001".to_string(),
            },
            portfolio_service_addr: "http://[::1]:50003".to_string(),
            etcd_endpoints: vec!["http://127.0.0.1:2379".to_string()],
        }
    }
}
