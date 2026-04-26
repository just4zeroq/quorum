# etcd 服务注册与发现设计

**日期**: 2026-04-26
**状态**: 待实现

## 目标

将 quorum 项目的 gRPC 微服务改造为支持 etcd 做服务注册与发现，实现服务地址的动态感知，移除硬编码配置。

## 背景

当前 quorum 项目存在以下问题：
- 8 个 gRPC 微服务端口硬编码（如 `127.0.0.1:50001`）
- API Gateway 通过 `GrpcConfig` 静态配置连接各服务
- 服务地址变更时需要手动修改配置
- 无法感知服务上下线

## 方案选择

| 选项 | 选择 |
|------|------|
| 注册粒度 | **单实例注册** - 每个服务实例独立注册，支持多实例部署 |
| 生命周期 | **TTL 自动过期** - 服务注册携带 TTL，心跳续约，崩溃自动清除 |
| 发现机制 | **Watch 模式** - API Gateway 监听 etcd 变化，实时感知服务上下线 |
| 存储格式 | **完整地址存储** - 存储 `http://127.0.0.1:50001` |

## 架构设计

```
etcd cluster
  │
  ├── /services/user-service/uuid-1  → {"addr": "http://127.0.0.1:50001"}
  ├── /services/user-service/uuid-2  → {"addr": "http://127.0.0.1:50011"}
  ├── /services/order-service/uuid-3 → {"addr": "http://127.0.0.1:50004"}
  └── ...

各服务启动时 ──────────────────────────────────────────────
    │
    ▼
ServiceRegistry::new("user-service", "http://127.0.0.1:50001")
    │
    ├── connect(etcd_endpoints)
    ├── register(ttl=30s)
    └── start_heartbeat(interval=10s)  // 后台任务定期续约
                                              │
API Gateway 启动时 ─────────────────────────────
    │
    ▼
ServiceDiscovery::new("user-service", etcd_endpoints)
    │
    ├── get_services() → 实时获取所有实例
    └── watch() → WatchStream 监听服务变化事件
```

## 存储格式

**Key 格式**: `/services/{service_name}/{instance_id}`
- 示例: `/services/user-service/550e8400-e29b-41d4-a716-446655440000`

**Value 格式**: JSON
```json
{"addr": "http://127.0.0.1:50001"}
```

**TTL**: 30 秒（服务心跳间隔 10 秒，续约频率保证 TTL 不会过期）

## 新建组件

### `crates/common/registry` crate

新建 `crates/common/registry/` 目录，提供服务注册与发现能力。

#### 目录结构

```
crates/common/registry/
├── Cargo.toml
└── src/
    └── lib.rs
```

#### Cargo.toml 依赖

```toml
[package]
name = "registry"
version.workspace = true
edition.workspace = true

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tracing.workspace = true
uuid.workspace = true
```

#### 核心接口

```rust
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("etcd error: {0}")]
    Etcd(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub instance_id: String,
    pub addr: String,
}

// ============ 服务注册（供各服务调用）============

pub struct ServiceRegistry {
    service_name: String,
    instance_id: String,
    addr: String,
}

impl ServiceRegistry {
    /// 创建服务注册器
    pub async fn new(
        service_name: &str,
        addr: &str,
        etcd_endpoints: &[String],
    ) -> Result<Self, RegistryError>;

    /// 注册服务到 etcd
    pub async fn register(&self, ttl_secs: u64) -> Result<(), RegistryError>;

    /// 从 etcd 注销服务
    pub async fn deregister(&self) -> Result<(), RegistryError>;

    /// 启动心跳续约任务（后台异步执行）
    pub fn start_heartbeat(self, ttl_secs: u64, interval_secs: u64) -> tokio::task::JoinHandle<()>;
}

// ============ 服务发现（供 API Gateway 调用）============

pub struct ServiceDiscovery {
    service_name: String,
    etcd_client: etcd_client::Client,
}

impl ServiceDiscovery {
    /// 创建服务发现器
    pub async fn new(
        service_name: &str,
        etcd_endpoints: &[String],
    ) -> Result<Self, RegistryError>;

    /// 获取所有服务实例
    pub async fn get_services(&self) -> Result<Vec<ServiceInstance>, RegistryError>;

    /// 监听服务变化（返回 watch stream）
    pub fn watch(&self) -> ServiceWatchStream;
}

pub struct ServiceWatchStream {
    // ...
}

impl ServiceWatchStream {
    /// 异步迭代器，返回服务变化事件
    pub async fn next(&mut self) -> Option<ServiceWatchEvent>;
}

#[derive(Debug, Clone)]
pub enum ServiceWatchEvent {
    Added(ServiceInstance),
    Removed(String),  // instance_id
    Modified(ServiceInstance),
}
```

## 改造范围

### 1. Workspace 依赖

**`Cargo.toml`**
```toml
[workspace.dependencies]
# etcd
etcd-client = "0.14"
```

### 2. 新建 common/registry

| 文件 | 说明 |
|------|------|
| `crates/common/registry/Cargo.toml` | crate 配置 |
| `crates/common/registry/src/lib.rs` | 核心逻辑 |

### 3. 各服务启动改造

| 服务 | 文件 | 改动 |
|------|------|------|
| user-service | `src/server.rs` | 启动时调用 `ServiceRegistry::new().register()`，关闭时 `deregister()` |
| wallet-service | `src/main.rs` | 同上 |
| order-service | `src/server.rs` | 同上 |
| risk-service | `src/main.rs` | 同上 |
| portfolio-service | `src/server.rs` | 同上 |
| market-data-service | `src/server.rs` | 同上 |
| prediction-market-service | `src/server.rs` | 同上 |

### 4. API Gateway 改造

**`crates/api-gateway/src/grpc.rs`**

```rust
// 旧：静态配置
pub struct GrpcConfig {
    pub user_service_addr: String,
    pub order_service_addr: String,
    // ...
}

// 新：动态发现
pub struct GrpcDiscovery {
    discovery: ServiceDiscovery,
}

impl GrpcDiscovery {
    pub async fn new(etcd_endpoints: &[String]) -> Result<Self, RegistryError> {
        // 为每个服务创建 ServiceDiscovery
    }

    pub async fn get_user_client(&self) -> Result<api::UserServiceClient<Channel>, ...> {
        let instances = self.user_discovery.get_services().await?;
        // 选择一个实例（可扩展为负载均衡）
        let addr = instances.first().ok_or(...)?;
        api::user::create_user_client(&addr.addr).await
    }
}
```

### 5. 服务配置

各服务 `config/{service}.yaml` 新增配置：

```yaml
registry:
  enabled: true
  etcd_endpoints:
    - "http://127.0.0.1:2379"
  ttl_secs: 30
  heartbeat_interval_secs: 10
```

## 实现步骤

1. 创建 `crates/common/registry` crate，实现核心接口
2. 在 workspace `Cargo.toml` 添加 etcd-client 依赖
3. 各服务添加 registry 依赖，改造启动逻辑
4. API Gateway 改造为使用 `ServiceDiscovery` 动态获取地址
5. 添加配置支持
6. 测试验证

## 注意事项

1. **优雅退出**: 各服务需要捕获 SIGTERM 信号，确保关闭时调用 `deregister()`
2. **连接失败**: etcd 连接失败时，服务应能继续启动（注册为可选功能）
3. **Watch 恢复**: API Gateway 重连时需要重新获取完整服务列表
4. **实例 ID**: 使用 UUID v4 确保唯一性
5. **向后兼容**: 可选择保留静态配置作为 fallback

## 未来扩展

- 负载均衡策略（轮询、随机、最小连接数）
- 健康检查集成
- 服务元数据（版本、区域、权重）
