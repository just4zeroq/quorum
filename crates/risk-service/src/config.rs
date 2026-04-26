//! Risk Service Configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub service: ServiceConfig,
    pub etcd_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service: ServiceConfig {
                host: "0.0.0.0".to_string(),
                port: 50005,
            },
            etcd_endpoints: vec!["http://127.0.0.1:2379".to_string()],
        }
    }
}