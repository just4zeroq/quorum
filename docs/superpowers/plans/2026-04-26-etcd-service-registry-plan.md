# etcd 服务注册与发现实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 quorum 项目实现基于 etcd 的服务注册与发现机制，移除硬编码配置

**Architecture:** 新建 `crates/common/registry` crate 提供统一的服务注册/发现能力，各服务启动时注册到 etcd，API Gateway 通过 Watch 模式动态发现服务地址

**Tech Stack:** etcd-client, tokio, serde_json, uuid

---

## 文件结构

```
crates/common/registry/           # 新建
├── Cargo.toml
└── src/
    ├── lib.rs                    # 核心接口
    ├── error.rs                  # 错误类型
    ├── registry.rs               # ServiceRegistry 实现
    ├── discovery.rs              # ServiceDiscovery 实现
    └── types.rs                  # 共享类型

crates/common/Cargo.toml           # 修改
Cargo.toml                        # 修改 (workspace dependencies)

crates/user-service/src/server.rs           # 修改
crates/wallet-service/src/main.rs           # 修改
crates/order-service/src/server.rs           # 修改
crates/risk-service/src/main.rs              # 修改
crates/portfolio-service/src/server.rs       # 修改
crates/market-data-service/src/server.rs     # 修改
crates/prediction-market-service/src/server.rs # 修改
crates/api-gateway/src/grpc.rs               # 修改
crates/api-gateway/src/lib.rs                 # 修改
```

---

## Task 1: 创建 registry crate 基础结构

**Files:**
- Create: `crates/common/registry/Cargo.toml`
- Create: `crates/common/registry/src/lib.rs`
- Create: `crates/common/registry/src/error.rs`
- Create: `crates/common/registry/src/types.rs`

- [ ] **Step 1: 创建 Cargo.toml**

```toml
[package]
name = "registry"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tracing.workspace = true
uuid.workspace = true
```

- [ ] **Step 2: 创建 error.rs**

```rust
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
```

- [ ] **Step 3: 创建 types.rs**

```rust
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
```

- [ ] **Step 4: 创建 lib.rs 框架**

```rust
pub mod error;
pub mod types;
pub mod registry;
pub mod discovery;

pub use error::RegistryError;
pub use types::{ServiceInstance, ServiceWatchEvent};
pub use registry::ServiceRegistry;
pub use discovery::ServiceDiscovery;
```

- [ ] **Step 5: 提交**

```bash
git add crates/common/registry/
git commit -m "feat(registry): create registry crate with basic structure"
```

---

## Task 2: 实现 ServiceRegistry

**Files:**
- Modify: `crates/common/registry/Cargo.toml` (添加 etcd-client 依赖)
- Create: `crates/common/registry/src/registry.rs`
- Modify: `crates/common/registry/src/lib.rs`

- [ ] **Step 1: 添加 etcd-client 依赖到 Cargo.toml**

```toml
# etcd
etcd-client = "0.14"
```

- [ ] **Step 2: 实现 registry.rs**

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use etcd_client::{Client, kv_ops::PutOptions, kv_ops::DeleteOptions};
use etcd_client::Watcher;
use tracing::{info, error};

use crate::error::RegistryError;
use crate::types::{ServiceInstance, ServiceRegistryValue};

const SERVICE_PREFIX: &str = "/services";

pub struct ServiceRegistry {
    service_name: String,
    instance_id: String,
    addr: String,
    client: Arc<RwLock<Client>>,
}

impl ServiceRegistry {
    pub async fn new(
        service_name: &str,
        addr: &str,
        etcd_endpoints: &[String],
    ) -> Result<Self, RegistryError> {
        let instance_id = Uuid::new_v4().to_string();
        let client = Client::connect(etcd_endpoints, None)
            .await
            .map_err(|e| RegistryError::Etcd(e.to_string()))?;

        Ok(Self {
            service_name: service_name.to_string(),
            instance_id,
            addr: addr.to_string(),
            client: Arc::new(RwLock::new(client)),
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
        let opts = PutOptions::new()
            .with_lease_id(ttl_secs);

        client.put(key, value_json, Some(opts))
            .await
            .map_err(|e| RegistryError::Etcd(e.to_string()))?;

        info!("Service registered: {} at {}", self.service_name, self.addr);
        Ok(())
    }

    pub async fn deregister(&self) -> Result<(), RegistryError> {
        let key = format!("{}/{}/{}", SERVICE_PREFIX, self.service_name, self.instance_id);
        let client = self.client.read().await;

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

                let read_client = client.read().await;
                let opts = PutOptions::new()
                    .with_lease_id(ttl_secs);

                if let Err(e) = read_client.put(key, value_json, Some(opts)).await {
                    error!("Heartbeat failed: {}", e);
                }
            }
        })
    }
}
```

- [ ] **Step 3: 更新 lib.rs**

```rust
pub mod error;
pub mod types;
pub mod registry;
pub mod discovery;

pub use error::RegistryError;
pub use types::{ServiceInstance, ServiceWatchEvent};
pub use registry::ServiceRegistry;
pub use discovery::ServiceDiscovery;
```

- [ ] **Step 4: 编译验证**

```bash
cd D:/code/github/quorum && cargo build -p registry
```

- [ ] **Step 5: 提交**

```bash
git add crates/common/registry/
git commit -m "feat(registry): implement ServiceRegistry with TTL heartbeat"
```

---

## Task 3: 实现 ServiceDiscovery

**Files:**
- Create: `crates/common/registry/src/discovery.rs`
- Modify: `crates/common/registry/src/lib.rs`

- [ ] **Step 1: 实现 discovery.rs**

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use etcd_client::{Client, kv_ops::GetOptions};
use etcd_client::Watcher;
use futures_util::StreamExt;
use tracing::{info, error};

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

        let mut response = client.get(prefix.as_str(), Some(GetOptions::new().with_prefix()))
            .await
            .map_err(|e| RegistryError::Etcd(e.to_string()))?;

        let mut instances = Vec::new();

        while let Some(kv) = response.next().await {
            match kv {
                Ok(kv) => {
                    let key = String::from_utf8_lossy(kv.key()).to_string();
                    let value_json = String::from_utf8_lossy(kv.value()).to_string();

                    if let Ok(value) = serde_json::from_str::<ServiceRegistryValue>(&value_json) {
                        let instance_id = key.split('/').last().unwrap_or("").to_string();
                        instances.push(ServiceInstance {
                            instance_id,
                            addr: value.addr,
                        });
                    }
                }
                Err(e) => {
                    error!("Failed to parse service instance: {}", e);
                }
            }
        }

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
            let client = self.client.read().await;
            let mut watcher = client.watcher(self.prefix.as_str(), Some(etcd_client::WatchOptions::new().with_prefix()))
                .await
                .ok()?;

            if let Some(response) = watcher.next().await {
                match response {
                    Ok(watch_response) => {
                        for event in watch_response.events() {
                            match event {
                                etcd_client::Event::Put(put) => {
                                    let key = String::from_utf8_lossy(put.key()).to_string();
                                    let value_json = String::from_utf8_lossy(put.value()).to_string();

                                    if let Ok(value) = serde_json::from_str::<ServiceRegistryValue>(&value_json) {
                                        let instance_id = key.split('/').last().unwrap_or("").to_string();
                                        return Some(ServiceWatchEvent::Added(ServiceInstance {
                                            instance_id,
                                            addr: value.addr,
                                        }));
                                    }
                                }
                                etcd_client::Event::Delete(delete) => {
                                    let key = String::from_utf8_lossy(delete.key()).to_string();
                                    let instance_id = key.split('/').last().unwrap_or("").to_string();
                                    return Some(ServiceWatchEvent::Removed(instance_id));
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        error!("Watch error: {}", e);
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: 更新 lib.rs**

```rust
pub mod error;
pub mod types;
pub mod registry;
pub mod discovery;

pub use error::RegistryError;
pub use types::{ServiceInstance, ServiceWatchEvent};
pub use registry::ServiceRegistry;
pub use discovery::{ServiceDiscovery, ServiceWatchStream};
```

- [ ] **Step 3: 编译验证**

```bash
cd D:/code/github/quorum && cargo build -p registry
```

- [ ] **Step 4: 提交**

```bash
git add crates/common/registry/
git commit -m "feat(registry): implement ServiceDiscovery with watch support"
```

---

## Task 4: Workspace 添加 registry 依赖

**Files:**
- Modify: `Cargo.toml` (workspace)
- Modify: `crates/common/Cargo.toml`

- [ ] **Step 1: 在 workspace Cargo.toml 添加 registry**

在 `[workspace.dependencies]` 添加:
```toml
# Registry
registry = { path = "crates/common/registry" }
```

- [ ] **Step 2: 在 crates/common/Cargo.toml 添加 members**

```toml
[package]
name = "common"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
# 现有依赖...
registry.workspace = true
```

- [ ] **Step 3: 编译验证**

```bash
cd D:/code/github/quorum && cargo build
```

- [ ] **Step 4: 提交**

```bash
git add Cargo.toml crates/common/Cargo.toml
git commit -m "feat: add registry crate to workspace"
```

---

## Task 5: 改造 user-service

**Files:**
- Modify: `crates/user-service/Cargo.toml`
- Modify: `crates/user-service/src/server.rs`

- [ ] **Step 1: 添加 registry 依赖到 Cargo.toml**

```toml
registry.workspace = true
```

- [ ] **Step 2: 改造 server.rs**

在 server.rs 中:

```rust
use registry::{ServiceRegistry, RegistryError};

pub struct UserGrpcServer {
    config: Arc<Config>,
    user_service: Arc<UserServiceImpl>,
    auth_service: Option<Arc<AuthServiceImpl>>,
    registry: Option<ServiceRegistry>,  // 添加
    heartbeat_handle: Option<tokio::task::JoinHandle<()>>,  // 添加
}
```

在 `run()` 方法开始处添加注册逻辑:

```rust
pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("[::1]:{}", self.config.service.port).parse()?;

    // 注册到 etcd
    let registry = ServiceRegistry::new(
        "user-service",
        &format!("http://{}", addr),
        &self.config.etcd_endpoints,
    ).await?;

    registry.register(30).await?;
    let heartbeat_handle = registry.clone().start_heartbeat(30, 10);

    tracing::info!("User service registered to etcd");
    tracing::info!("User service gRPC server listening on {}", addr);

    // ... 现有代码 ...
}
```

- [ ] **Step 3: 添加 etcd 配置到 Config**

在 `src/config.rs` 中添加:

```rust
#[derive(Debug, Clone)]
pub struct Config {
    // ... 现有字段 ...
    pub etcd_endpoints: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... 现有默认值 ...
            etcd_endpoints: vec!["http://127.0.0.1:2379".to_string()],
        }
    }
}
```

- [ ] **Step 4: 编译验证**

```bash
cd D:/code/github/quorum && cargo build -p user-service
```

- [ ] **Step 5: 提交**

```bash
git add crates/user-service/
git commit -m "feat(user-service): add etcd service registry"
```

---

## Task 6: 改造 wallet-service

**Files:**
- Modify: `crates/wallet-service/Cargo.toml`
- Modify: `crates/wallet-service/src/main.rs`

- [ ] **Step 1: 添加 registry 依赖到 Cargo.toml**

```toml
registry.workspace = true
```

- [ ] **Step 2: 改造 main.rs**

在 main 函数中添加服务注册:

```rust
use registry::{ServiceRegistry, RegistryError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... 现有初始化代码 ...

    // 注册到 etcd
    let registry = ServiceRegistry::new(
        "wallet-service",
        &format!("http://{}", addr),
        &config.etcd_endpoints,
    ).await?;

    registry.register(30).await?;
    let heartbeat_handle = registry.clone().start_heartbeat(30, 10);

    // ... 现有服务器启动代码 ...
}
```

- [ ] **Step 3: 添加 Config etcd 支持**

在 `src/config.rs` 中添加 `etcd_endpoints` 字段

- [ ] **Step 4: 编译验证**

```bash
cd D:/code/github/quorum && cargo build -p wallet-service
```

- [ ] **Step 5: 提交**

```bash
git add crates/wallet-service/
git commit -m "feat(wallet-service): add etcd service registry"
```

---

## Task 7: 改造 order-service

**Files:**
- Modify: `crates/order-service/Cargo.toml`
- Modify: `crates/order-service/src/server.rs`
- Modify: `crates/order-service/src/config.rs`

- [ ] **Step 1-4: 类似于 wallet-service 的改造**

添加 registry 依赖，改造 server.rs 添加注册逻辑，添加 Config etcd 支持

- [ ] **Step 5: 编译验证**

```bash
cd D:/code/github/quorum && cargo build -p order-service
```

- [ ] **Step 6: 提交**

```bash
git add crates/order-service/
git commit -m "feat(order-service): add etcd service registry"
```

---

## Task 8: 改造 portfolio-service

**Files:**
- Modify: `crates/portfolio-service/Cargo.toml`
- Modify: `crates/portfolio-service/src/server.rs`
- Modify: `crates/portfolio-service/src/config.rs`

- [ ] **Step 1-4: 改造方式同上**

- [ ] **Step 5: 编译验证**

```bash
cd D:/code/github/quorum && cargo build -p portfolio-service
```

- [ ] **Step 6: 提交**

```bash
git add crates/portfolio-service/
git commit -m "feat(portfolio-service): add etcd service registry"
```

---

## Task 9: 改造 risk-service

**Files:**
- Modify: `crates/risk-service/Cargo.toml`
- Modify: `crates/risk-service/src/main.rs`
- Modify: `crates/risk-service/src/config.rs`

- [ ] **Step 1-4: 改造方式同上**

- [ ] **Step 5: 编译验证**

```bash
cd D:/code/github/quorum && cargo build -p risk-service
```

- [ ] **Step 6: 提交**

```bash
git add crates/risk-service/
git commit -m "feat(risk-service): add etcd service registry"
```

---

## Task 10: 改造 market-data-service

**Files:**
- Modify: `crates/market-data-service/Cargo.toml`
- Modify: `crates/market-data-service/src/server.rs`
- Modify: `crates/market-data-service/src/config.rs`

- [ ] **Step 1-4: 改造方式同上**

- [ ] **Step 5: 编译验证**

```bash
cd D:/code/github/quorum && cargo build -p market-data-service
```

- [ ] **Step 6: 提交**

```bash
git add crates/market-data-service/
git commit -m "feat(market-data-service): add etcd service registry"
```

---

## Task 11: 改造 prediction-market-service

**Files:**
- Modify: `crates/prediction-market-service/Cargo.toml`
- Modify: `crates/prediction-market-service/src/server.rs`
- Modify: `crates/prediction-market-service/src/config.rs`

- [ ] **Step 1-4: 改造方式同上**

- [ ] **Step 5: 编译验证**

```bash
cd D:/code/github/quorum && cargo build -p prediction-market-service
```

- [ ] **Step 6: 提交**

```bash
git add crates/prediction-market-service/
git commit -m "feat(prediction-market-service): add etcd service registry"
```

---

## Task 12: 改造 API Gateway

**Files:**
- Modify: `crates/api-gateway/Cargo.toml`
- Modify: `crates/api-gateway/src/grpc.rs`
- Modify: `crates/api-gateway/src/lib.rs`

- [ ] **Step 1: 添加 registry 依赖到 Cargo.toml**

```toml
registry.workspace = true
```

- [ ] **Step 2: 改造 grpc.rs**

替换静态 GrpcConfig 为动态 ServiceDiscovery:

```rust
use registry::{ServiceDiscovery, ServiceInstance};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct GrpcClientManager {
    discoveries: HashMap<String, ServiceDiscovery>,
    clients: Arc<RwLock<HashMap<String, api::UserServiceClient<tonic::transport::Channel>>>>,
    etcd_endpoints: Vec<String>,
}

impl GrpcClientManager {
    pub async fn new(etcd_endpoints: Vec<String>) -> Result<Self, RegistryError> {
        let service_names = vec![
            "user-service",
            "wallet-service",
            "order-service",
            "portfolio-service",
            "risk-service",
            "market-data-service",
            "prediction-market-service",
        ];

        let mut discoveries = HashMap::new();
        for name in service_names {
            discoveries.insert(
                name.to_string(),
                ServiceDiscovery::new(name, &etcd_endpoints).await?,
            );
        }

        Ok(Self {
            discoveries,
            clients: Arc::new(RwLock::new(HashMap::new())),
            etcd_endpoints,
        })
    }

    pub async fn get_user_client(&self) -> Result<api::UserServiceClient<tonic::transport::Channel>, RegistryError> {
        let instances = self.discoveries.get("user-service")
            .ok_or_else(|| RegistryError::NotFound("user-service".to_string()))?
            .get_services()
            .await?;

        if instances.is_empty() {
            return Err(RegistryError::NotFound("user-service has no instances".to_string()));
        }

        let addr = &instances[0].addr;
        api::user::create_user_client(addr).await
            .map_err(|e| RegistryError::Etcd(e.to_string()))
    }

    // 类似方法为其他服务...
}
```

- [ ] **Step 3: 更新 lib.rs 导出**

```rust
pub mod grpc;
pub use grpc::GrpcClientManager;
```

- [ ] **Step 4: 编译验证**

```bash
cd D:/code/github/quorum && cargo build -p api-gateway
```

- [ ] **Step 5: 提交**

```bash
git add crates/api-gateway/
git commit -m "feat(api-gateway): add etcd service discovery"
```

---

## Task 13: 全量编译验证

**Files:**
- None (仅验证)

- [ ] **Step 1: 编译所有 crate**

```bash
cd D:/code/github/quorum && cargo build --all
```

- [ ] **Step 2: 验证通过后提交**

```bash
git add -A
git commit -m "feat: implement etcd service registry and discovery

- Add crates/common/registry crate with ServiceRegistry and ServiceDiscovery
- All 7 backend services register to etcd on startup with TTL heartbeat
- API Gateway dynamically discovers services via etcd watch
- Remove hardcoded service addresses from configuration"
```

---

## 实施检查清单

- [ ] Task 1: registry crate 基础结构
- [ ] Task 2: ServiceRegistry 实现
- [ ] Task 3: ServiceDiscovery 实现
- [ ] Task 4: Workspace 添加 registry
- [ ] Task 5: user-service 改造
- [ ] Task 6: wallet-service 改造
- [ ] Task 7: order-service 改造
- [ ] Task 8: portfolio-service 改造
- [ ] Task 9: risk-service 改造
- [ ] Task 10: market-data-service 改造
- [ ] Task 11: prediction-market-service 改造
- [ ] Task 12: API Gateway 改造
- [ ] Task 13: 全量编译验证
