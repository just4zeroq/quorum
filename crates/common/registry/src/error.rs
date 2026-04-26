use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("etcd error: {0}")]
    Etcd(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("service not found: {0}")]
    NotFound(String),

    #[error("invalid configuration: {0}")]
    Config(String),
}