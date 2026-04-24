# CEX DEX 项目

## 项目概述

这是一个 Rust 实现的中心化交易所 (CEX) 项目，采用微服务架构。
**只支持现货交易，用于预测市场 (Prediction Market)。**

## 市场定位

- **交易类型**: 现货交易 (Spot Trading)
- **应用场景**: 预测市场 (Prediction Market) - 交易事件结果的概率价格
- **订单模型**: CLOB (中央限价订单簿)

> 注意: 不支持合约、杠杆、期货、交割等衍生品交易

## 服务实现状态

| 服务 | Proto | Server | Tests | 文档 |
|------|:------:|:------:|:-----:|:----:|
| User Service | ✅ | ✅ | - | ✅ |
| Portfolio Service | ✅ | ✅ | - | ✅ |
| Order Service | ✅ | ✅ | - | ✅ |
| Prediction Market Service | ✅ | ✅ | ✅ | ✅ |
| Matching Engine | - | ✅ Core + Queue | ✅ | ✅ |
| Market Data Service | ✅ | ✅ | ✅ | ✅ |
| Risk Service | ✅ | ✅ | - | ✅ |
| Wallet Service | ✅ | ✅ | - | ✅ |
| Auth Service | ✅ | ✅ | - | - |
| API Gateway | - | ✅ | - | ✅ |
| ws-market-data | - | ✅ | - | ✅ |
| ws-order | - | ✅ | - | ✅ |
| ws-prediction | - | ✅ | - | ✅ |

> ✅ = 已完成实现/文档 | - = 未实现 | Core = 仅核心逻辑

## 撮合引擎架构 (Matching Engine)

### 预测市场订单簿设计 (方案1: 单一 YES 订单簿)

- **只维护 YES 订单簿**，NO 价格通过互补计算 (SCALE - YES 价格)
- 每个订单记录 `original_asset` 标识原始下单品种 (YES/NO)
- 撤单时通过 `order_id` 定位原始订单

**价格转换:**
```
买 YES → YES 簿 Bid      买 NO  → YES 簿 Ask (价格取反)
卖 YES → YES 簿 Ask      卖 NO  → YES 簿 Bid (价格取反)
```

**优势:** 数据量减少 50%，撤单逻辑简单，撮合算法统一

## 公共组件

| 组件 | 路径 | 说明 |
|------|------|------|
| `domain` | `crates/domain` | 共享领域模型，按服务拆分 (order/trade/user/market_data/prediction_market)，每个领域含 model/event/shared |
| `api` | `crates/api` | 统一 gRPC 接口定义包，各服务 proto 编译输出至此 |
| `db` | `crates/common/db` | 数据库连接池，支持 PostgreSQL / SQLite |
| `cache` | `crates/common/cache` | Redis 客户端，含分布式锁 |
| `queue` | `crates/common/queue` | 消息队列，支持 Redis Streams / Kafka 双后端 |
| `rate_limiter` | `crates/common/rate_limiter` | 限流组件，支持 TokenBucket/SlidingWindow/FixedWindow |
| `utils` | `crates/common/utils` | 通用工具 (cipher/wallet/id/token) |

## 技术栈

- **语言**: Rust
- **Web 框架**: Salvo (API Gateway), Tonic (gRPC)
- **数据库**: PostgreSQL / SQLite (测试)
- **缓存**: Redis
- **消息队列**: common/queue (Redis Streams / Kafka 双后端)
- **ORM**: SQLx

## 目录结构

```
rust-cex/
├── Cargo.toml                    # Workspace 配置
├── crates/
│   ├── common/                   # 公共组件 (db/cache/queue/rate_limiter/utils)
│   ├── domain/                   # 领域模型共享包
│   ├── api/                      # 统一 gRPC 接口定义
│   ├── api-gateway/              # API 网关 (HTTP/WS 入口)
│   ├── user-service/             # 用户服务
│   ├── wallet-service/           # 钱包服务
│   ├── portfolio-service/        # 账户+持仓+清算+账本
│   ├── order-service/            # 订单服务
│   ├── risk-service/             # 风控服务
│   ├── auth-service/             # 鉴权服务 (JWT/API Key)
│   ├── market-data-service/      # 行情服务
│   ├── matching-engine/          # 撮合引擎 (内存 CLOB + WAL)
│   ├── prediction-market-service/ # 预测市场服务
│   └── ws-market-data/           # 行情 WebSocket 服务
```

## 编译命令

```bash
export PATH="/home/ubuntu/.cargo/bin:$PATH"

# 编译所有服务
cargo build

# 编译特定服务
cargo build -p user-service
cargo build -p market-data-service

# 运行测试
cargo test -p user-service

# 运行服务
cargo run -p user-service
```

## 服务列表

| 服务 | 端口 | 数据库 | 说明 |
|------|------|--------|------|
| User Service | 50001 | 独立 | 用户注册/登录/KYC/2FA |
| Wallet Service | 50002 | 独立 | 充值/提现/地址管理 |
| Portfolio Service | 50003 | 独立 | 账户+持仓+清算+账本 |
| Order Service | 50004 | 独立 | 订单管理 |
| Risk Service | 50005 | 独立 | 风控规则/限额 |
| Market Data Service | 50006 | 共享 PM DB | 行情/K线/订单簿/24h统计 |
| Matching Engine | 50007 | **无** | CLOB 撮合 (内存 + WAL) |
| Prediction Market Service | 50008 | 主数据库 | 市场管理/结算/派彩 |
| Auth Service | 50009 | 独立 | JWT/API Key 鉴权 |
| ws-market-data | 50016 | - | WebSocket 行情推送 |
| API Gateway | 8080 | - | HTTP/WS 统一入口 |

### Portfolio Service 模块

合并自: Account Service + Position Service + Clearing Service + Ledger Service

### Auth Service 模块

- JWT Token 管理 (Access/Refresh)
- API Key 管理
- Session 管理

### 已移除的服务 (现货不需要)

| 服务 | 原因 |
|------|------|
| Liquidation Engine | 合约强平机制，现货不需要 |
| Mark Price Service | 合约标记价格，现货不需要 |
| Funding Service | 合约资金费率，现货不需要 |

## 交易模式

采用 **CLOB (中央限价订单簿)** 模型：
- 用户下单到订单簿
- Matching Engine 实时撮合
- 成交后更新持仓和账户

## 整体架构

```
用户 → API Gateway(8080) → gRPC → 各业务服务
                            ↓
                    Matching Engine
                            ↓
                      消息队列 (common/queue)
                            ↓
              ws-market-data / Portfolio / Order / ...
```

## 数据库规划

| 服务 | SQLite (测试) | PostgreSQL (生产) | 备注 |
|------|:-------------:|:-----------------:|------|
| User Service | ✅ | ✅ | 独立 |
| Wallet Service | ✅ | ✅ | 独立 |
| Order Service | ✅ | ✅ | 独立 |
| Risk Service | ✅ | ✅ | 独立 |
| Market Data Service | ✅ | ✅ | 共享 Prediction Market DB |
| Matching Engine | ❌ | 内存 + WAL | 无 DB |
| Prediction Market Service | ✅ | ✅ | 主数据库 |
| Portfolio Service | ✅ | ✅ | 独立 |

## 服务间通信

| 源服务 | 目标服务 | 通信方式 | 说明 |
|--------|----------|----------|------|
| API Gateway | 所有业务服务 | gRPC | HTTP 入口转发 |
| Order Service | Matching Engine | gRPC | 下单/撤单 |
| Matching Engine | 消息队列 | Queue Producer | 发布成交/订单事件 |
| 消息队列 | Order Service | Queue Consumer | 订单状态更新 |
| 消息队列 | Portfolio Service | Queue Consumer | 持仓/余额更新 |
| 消息队列 | ws-market-data | Queue Consumer | 行情推送 |
| Prediction Market Service | Matching Engine | gRPC | 创建市场/结算 |
| Prediction Market Service | Market Data Service | 共享 DB | 市场数据 |

## MVP 最小闭环

```
用户 -> 注册/登录 -> 查行情 -> 下单 -> 撮合 -> 持仓/账户
```

## 配置说明

- 配置文件在各服务包内: `crates/{service}/config/{service}.yaml`
- 公共组件配置: `crates/common/{db,cache,queue}/config/config.yaml`
- 组件配置优先级: 服务配置 > 组件默认配置 > 硬编码默认值

## gRPC 服务代码结构

```
{service}/
├── Cargo.toml                  # 依赖: tonic, prost, tonic-build
├── build.rs                    # Proto 编译配置 (输出到 crates/api/src/)
├── config/
│   └── {service}.yaml
└── src/
    ├── lib.rs                  # 模块导出
    ├── main.rs                 # 入口
    ├── models.rs               # 数据模型
    ├── pb/
    │   └── {service}.proto     # Proto 定义 (源文件)
    ├── repository/
    └── services/
        └── {service}.rs        # gRPC 服务实现
```

### Proto 定义示例

```protobuf
syntax = "proto3";
package order;

service OrderService {
    rpc CreateOrder(CreateOrderRequest) returns (CreateOrderResponse);
}

message CreateOrderRequest {
    int64 user_id = 1;
    string price = 2;    // Decimal 用 string 传输
}

message CreateOrderResponse {
    bool success = 1;
    string order_id = 2;
}
```

### gRPC 服务实现示例

```rust
use tonic::{Request, Response, Status};
use api::order::{order_service_server::OrderService, *};

pub struct OrderServiceImpl { pool: sqlx::PgPool }

#[tonic::async_trait]
impl OrderService for OrderServiceImpl {
    async fn create_order(&self, request: Request<CreateOrderRequest>)
        -> Result<Response<CreateOrderResponse>, Status> {
        Ok(Response::new(CreateOrderResponse { success: true, order_id: "o_xxx".to_string() }))
    }
}
```

---

## API 接口定义规范

### 架构原则

```
domain (纯业务模型)
  ↓
api (接口定义 + 序列化)
  ↓
services (服务实现)
  ↓
gateway (API 网关)
```

**设计原则:**
1. Proto 文件放在各自服务 `src/pb/*.proto`
2. 生成代码输出到 `crates/api/src/` (通过 build.rs)
3. 统一依赖: `api = { path = "../api" }`
4. 单向依赖: domain → api → services → gateway

### 服务 build.rs 模板

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let api_src = manifest_dir
        .parent().unwrap().parent().unwrap()
        .join("crates/api/src");

    std::fs::create_dir_all(&api_src)?;

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(&api_src.join("{service}.desc"))
        .out_dir(&api_src)
        .compile_protos(
            &[manifest_dir.join("src/pb/{service}.proto")],
            &[manifest_dir.join("src/pb")],
        )?;
    Ok(())
}
```

### crates/api/src/lib.rs 结构

```rust
pub mod user { include!("user.rs"); }
pub mod order { include!("order.rs"); }
pub mod auth { include!("auth.rs"); }

pub use user::user_service_client::UserServiceClient;
```

### 类型映射规范

| Rust 类型 | Proto 类型 | 说明 |
|-----------|-----------|------|
| `i64` | `int64` | ID、时间戳 |
| `String` | `string` | 字符串 |
| `Decimal` | `string` | 金额用字符串传输 |
| `Vec<T>` | `repeated T` | 数组 |
| `Option<T>` | `optional` | 可选字段 |
| `bool` | `bool` | 布尔值 |

### 新增服务流程

1. 在 `src/pb/` 创建 `*.proto` 文件
2. 配置 `build.rs` 输出到 `crates/api/src/`
3. 服务 `Cargo.toml` 添加 `api = { path = "../api" }`
4. 在 `crates/api/src/lib.rs` 添加 `pub mod {service};`

### 注意事项

- 服务端使用 tonic，不依赖 salvo
- API Gateway 负责 HTTP -> gRPC 转换
- `Decimal` 类型在 proto 中使用 `string` 传输
- 生成文件: `{service}.rs` (代码) + `{service}.desc` (描述符)
