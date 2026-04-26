use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub instance_id: String,
    pub addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistryValue {
    pub addr: String,
}

#[derive(Debug, Clone)]
pub enum ServiceWatchEvent {
    Added(ServiceInstance),
    Removed(String),
    Modified(ServiceInstance),
}