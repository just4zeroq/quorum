use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use etcd_client::{Client, PutOptions, DeleteOptions};
use tracing::{info, error};

use crate::error::RegistryError;
use crate::types::ServiceRegistryValue;

const SERVICE_PREFIX: &str = "/services";

pub struct ServiceRegistry {
    service_name: String,
    instance_id: String,
    addr: String,
    client: Arc<RwLock<Client>>,
    lease_id: i64,
}

impl Clone for ServiceRegistry {
    fn clone(&self) -> Self {
        Self {
            service_name: self.service_name.clone(),
            instance_id: self.instance_id.clone(),
            addr: self.addr.clone(),
            client: self.client.clone(),
            lease_id: self.lease_id,
        }
    }
}

impl ServiceRegistry {
    pub async fn new(
        service_name: &str,
        addr: &str,
        etcd_endpoints: &[String],
    ) -> Result<Self, RegistryError> {
        let instance_id = Uuid::new_v4().to_string();

        // Convert &[String] to Vec<&str> for etcd-client
        let endpoints: Vec<&str> = etcd_endpoints.iter().map(|s| s.as_str()).collect();
        let client = Client::connect(endpoints, None)
            .await
            .map_err(|e| RegistryError::Etcd(e.to_string()))?;

        Ok(Self {
            service_name: service_name.to_string(),
            instance_id,
            addr: addr.to_string(),
            client: Arc::new(RwLock::new(client)),
            lease_id: 0, // Will be set during register
        })
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }

    pub async fn register(&self, ttl_secs: u64) -> Result<(), RegistryError> {
        let key = format!("{}/{}/{}", SERVICE_PREFIX, self.service_name, self.instance_id);
        let value = ServiceRegistryValue {
            addr: self.addr.clone(),
        };
        let value_json = serde_json::to_string(&value)
            .map_err(|e| RegistryError::Serialization(e))?;

        let mut client = self.client.write().await;

        // Create a lease with the given TTL
        let lease_resp = client.lease_client()
            .grant(ttl_secs as i64, None)
            .await
            .map_err(|e| RegistryError::Etcd(e.to_string()))?;

        let lease_id = lease_resp.id();

        let opts = PutOptions::new()
            .with_lease(lease_id);

        client.put(key, value_json, Some(opts))
            .await
            .map_err(|e| RegistryError::Etcd(e.to_string()))?;

        info!("Service registered: {} at {} with lease {}", self.service_name, self.addr, lease_id);
        Ok(())
    }

    pub async fn deregister(&self) -> Result<(), RegistryError> {
        let key = format!("{}/{}/{}", SERVICE_PREFIX, self.service_name, self.instance_id);
        let mut client = self.client.write().await;

        client.delete(key, Some(DeleteOptions::new()))
            .await
            .map_err(|e| RegistryError::Etcd(e.to_string()))?;

        info!("Service deregistered: {}", self.service_name);
        Ok(())
    }

    pub fn start_heartbeat(self, ttl_secs: u64, interval_secs: u64) -> tokio::task::JoinHandle<()>
    where
        Self: Send + Sync + 'static,
    {
        let instance_id = self.instance_id.clone();
        let service_name = self.service_name.clone();
        let addr = self.addr.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(interval_secs)
            );

            // Create a new lease for heartbeat
            let lease_id = {
                let c = client.write().await;
                let resp = c.lease_client()
                    .grant(ttl_secs as i64, None)
                    .await
                    .map_err(|e| {
                        error!("Failed to create lease for heartbeat: {}", e);
                        e
                    });
                match resp {
                    Ok(resp) => resp.id(),
                    Err(_) => return,
                }
            };

            loop {
                interval.tick().await;
                let key = format!("{}/{}/{}", SERVICE_PREFIX, service_name, instance_id);
                let value = ServiceRegistryValue {
                    addr: addr.clone(),
                };
                let value_json = match serde_json::to_string(&value) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Failed to serialize service value: {}", e);
                        continue;
                    }
                };

                let mut write_client = client.write().await;
                let opts = PutOptions::new()
                    .with_lease(lease_id);

                if let Err(e) = write_client.put(key, value_json, Some(opts)).await {
                    error!("Heartbeat failed: {}", e);
                }
            }
        })
    }
}