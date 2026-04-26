use std::sync::Arc;
use tokio::sync::RwLock;
use etcd_client::{Client, GetOptions, EventType};
use tracing::error;

use crate::error::RegistryError;
use crate::types::{ServiceInstance, ServiceRegistryValue, ServiceWatchEvent};

const SERVICE_PREFIX: &str = "/services";

pub struct ServiceDiscovery {
    service_name: String,
    client: Arc<RwLock<Client>>,
}

impl ServiceDiscovery {
    pub async fn new(
        service_name: &str,
        etcd_endpoints: &[String],
    ) -> Result<Self, RegistryError> {
        let client = Client::connect(etcd_endpoints, None)
            .await
            .map_err(|e| RegistryError::Etcd(e.to_string()))?;

        Ok(Self {
            service_name: service_name.to_string(),
            client: Arc::new(RwLock::new(client)),
        })
    }

    pub async fn get_services(&self) -> Result<Vec<ServiceInstance>, RegistryError> {
        let prefix = format!("{}/{}", SERVICE_PREFIX, self.service_name);
        let client = self.client.read().await;
        let mut kv_client = client.kv_client();

        let response = kv_client
            .get(prefix.as_str(), Some(GetOptions::new().with_prefix()))
            .await
            .map_err(|e| RegistryError::Etcd(e.to_string()))?;

        let instances: Vec<ServiceInstance> = response
            .kvs()
            .iter()
            .filter_map(|kv| {
                let key = String::from_utf8_lossy(kv.key()).to_string();
                let value_json = String::from_utf8_lossy(kv.value()).to_string();

                if let Ok(value) =
                    serde_json::from_str::<ServiceRegistryValue>(&value_json)
                {
                    let instance_id = key.split('/').last().unwrap_or("").to_string();
                    Some(ServiceInstance {
                        instance_id,
                        addr: value.addr,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(instances)
    }

    pub fn watch(&self) -> ServiceWatchStream {
        let prefix = format!("{}/{}", SERVICE_PREFIX, self.service_name);
        ServiceWatchStream {
            service_name: self.service_name.clone(),
            client: self.client.clone(),
            prefix,
        }
    }
}

pub struct ServiceWatchStream {
    service_name: String,
    client: Arc<RwLock<Client>>,
    prefix: String,
}

impl ServiceWatchStream {
    pub async fn next(&mut self) -> Option<ServiceWatchEvent> {
        loop {
            let mut client = self.client.write().await;
            let result = client
                .watch(
                    self.prefix.as_str(),
                    Some(etcd_client::WatchOptions::new().with_prefix()),
                )
                .await;

            let (_, mut watch_stream) = match result {
                Ok(pair) => pair,
                Err(e) => {
                    error!("Watch connection error: {}", e);
                    continue;
                }
            };

            loop {
                match watch_stream.message().await {
                    Ok(Some(watch_response)) => {
                        for event in watch_response.events() {
                            match event.event_type() {
                                EventType::Put => {
                                    if let Some(kv) = event.kv() {
                                        let key =
                                            String::from_utf8_lossy(kv.key()).to_string();
                                        let value_json =
                                            String::from_utf8_lossy(kv.value()).to_string();

                                        if let Ok(value) =
                                            serde_json::from_str::<ServiceRegistryValue>(
                                                &value_json,
                                            )
                                        {
                                            let instance_id = key
                                                .split('/')
                                                .last()
                                                .unwrap_or("")
                                                .to_string();
                                            return Some(ServiceWatchEvent::Added(
                                                ServiceInstance {
                                                    instance_id,
                                                    addr: value.addr,
                                                },
                                            ));
                                        }
                                    }
                                }
                                EventType::Delete => {
                                    if let Some(kv) = event.kv() {
                                        let key =
                                            String::from_utf8_lossy(kv.key()).to_string();
                                        let instance_id = key
                                            .split('/')
                                            .last()
                                            .unwrap_or("")
                                            .to_string();
                                        return Some(ServiceWatchEvent::Removed(
                                            instance_id,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        // Stream ended, break inner loop and reconnect
                        break;
                    }
                    Err(e) => {
                        error!("Watch error: {}", e);
                        break;
                    }
                }
            }
        }
    }
}