pub mod error;
pub mod types;
pub mod registry;
pub mod discovery;

pub use error::RegistryError;
pub use types::{ServiceInstance, ServiceWatchEvent};
pub use registry::ServiceRegistry;
pub use discovery::ServiceDiscovery;