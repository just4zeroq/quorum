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
| Portfolio Service | ✅ | - | - | ✅ |
| Order Service | ✅ | ✅ | - | ✅ |
| Prediction Market Service | ✅ | ✅ | ✅ | ✅ |
| Matching Engine | - | ✅ Core + Kafka | ✅ | ✅ |
| Market Data Service | ✅ | ✅ | ✅ | ✅ |
| Risk Service | ✅ | - | - | ✅ |
| Wallet Service | ✅ | - | - | ✅ |
| Auth Service | ✅ | - | - | - |
| API Gateway | - | - | - | ✅ |
| ws-market-data | - | - | - | ✅ |
| ws-order | - | - | - | ✅ |
| ws-prediction | - | - | - | ✅ |

> ✅ = 已完成实现/文档 | - = 未实现 | Core = 仅核心逻辑

## 撮合引擎架构 (Matching Engine)

### 预测市场订单簿设计 (方案1: 单一 YES 订单簿)

撮合引擎采用**方案1**设计预测市场订单簿：

#### 核心原理
- **只维护 YES 订单簿**，NO 价格通过互补计算 (SCALE - YES 价格)
- 每个订单记录 `original_asset` 标识原始下单品种 (YES/NO)
- 撤单时通过 `order_id` 定位原始订单

#### 价格转换
```
前端展示时:
  YES 价格: 直接展示
  NO 价格: SCALE - YES 价格 (互补)

内部处理时:
  买 YES → YES 簿 Bid
  卖 YES → YES 簿 Ask
  买 NO  → YES 簿 Ask (价格取反)
  卖 NO  → YES 簿 Bid (价格取反)
```

#### 数据结构
```rust
struct OrderEntry {
    order_id: OrderId,
    uid: UserId,
    price: Price,                    // YES 簿价格
    action: OrderAction,             // YES 簿方向
    original_asset: String,          // "1_yes" 或 "1_no"
    original_action: OrderAction,    // 原始下单方向
}
```

#### 优势
- 数据量减少 50%（vs 双簿方案）
- 撤单逻辑简单（无需处理镜像订单）
- 撮合算法统一（只需维护一个订单簿）

### 核心组件

```
matching-engine/
├── src/
│   ├── api/                    # API 类型定义
│   │   ├── commands.rs         # OrderCommand 命令结构
│   │   ├── events.rs          # MatcherTradeEvent 成交事件
│   │   └── types.rs            # 价格/数量/订单类型
│   ├── core/                   # 核心撮合逻辑
│   │   ├── exchange.rs         # ExchangeCore 交易所核心
│   │   ├── pipeline.rs          # Pipeline 处理器流水线
│   │   ├── journal.rs          # WAL 日志 (rkyv 序列化)
│   │   ├── snapshot.rs         # 快照管理 (bincode 序列化)
│   │   ├── processors/          # 处理器
│   │   │   ├── matching_engine.rs  # 撮合引擎路由
│   │   │   └── risk_engine.rs     # 风控引擎
│   │   └── orderbook/          # 订单簿实现
│   │       ├── direct.rs        # DirectOrderBook (高性能链表)
│   │       ├── prediction.rs    # PredictionOrderBook (方案1)
│   │       └── ...
│   ├── server.rs               # Kafka 消费入口
│   └── event_emitter.rs        # Kafka 事件发布
```

### 服务间通信

| 源服务 | 目标服务 | 通信方式 | 说明 |
|--------|----------|----------|------|
| Matching Engine | Kafka | Producer | 发布成交/订单事件 |
| Kafka | Matching Engine | Consumer | 消费 order.commands |

## 公共组件

### domain - 领域模型共享包 (crates/domain)

服务对齐的目录结构，按服务拆分模块：

- `order/model` - 订单模型 (Order, OrderStatus, OrderType, OrderSide)
- `order/event` - 订单事件 (Created, Submitted, Filled, Cancelled, Rejected)
- `trade/model` - 成交模型 (Trade, TradeSide, TradeQuery)
- `trade/event` - 成交事件 (Executed, Rollback)
- `user/model` - 用户模型 (User, UserStatus, UserSession)
- `user/event` - 用户事件 (Registered, Login, Logout, Frozen)
- `market_data/model` - 行情模型 (Market, Outcome, OrderBook, Kline, KlineInterval)
- `market_data/event` - 行情事件 (PriceUpdated, OrderBookUpdated, TradeExecuted, KlineUpdated)
- `prediction_market/model` - 预测市场模型 (PredictionMarket, MarketOutcome, MarketStatus, Resolution)
- `prediction_market/event` - 预测市场事件 (MarketCreated, MarketClosed, MarketResolved, OutcomeAdded)

### common/auth - 鉴权组件 (crates/common/auth)

统一鉴权接口，支持多种鉴权方式：

```
auth/
├── lib.rs
├── jwt.rs           # JWT Token 验证
├── api_key.rs       # API Key 验证
├── context.rs       # AuthContext (用户信息提取)
├── error.rs         # AuthError 错误类型
└── traits.rs        # AuthService trait
```

### common/rate_limiter - 限流组件 (crates/common/rate_limiter)

支持多种限流算法和存储后端：

```
rate_limiter/
├── lib.rs
├── algorithm/       # 限流算法
│   ├── token_bucket.rs   # Token Bucket
│   ├── sliding_window.rs  # 滑动窗口
│   └── fixed_window.rs   # 固定窗口
├── store/          # 存储实现
│   ├── memory.rs        # 内存存储 (测试)
│   └── redis.rs         # Redis 存储 (生产)
├── traits.rs       # RateLimiter trait
└── middleware.rs   # Middleware 辅助
```

### common/utils - 通用工具模块 (crates/common/utils)

- `token.rs` - JWT Token 生成与验证 (已迁移到 auth)
- `cipher.rs` - AES-256-GCM 加解密, HMAC, SHA, PBKDF2
- `wallet.rs` - Ethereum 钱包签名验证 (EIP-191)
- `id/` - ID 生成器
  - `order.rs` - 订单 ID 生成 (前缀 `o`)
  - `trade.rs` - 成交 ID 生成 (前缀 `t`)
  - `generator.rs` - ID 生成核心

### common/db - 数据库组件 (crates/common/db)
### common/cache - Redis 缓存组件 (crates/common/cache)
### common/queue - Kafka 消息队列组件 (crates/common/queue)

详细架构文档见: `docs/ARCHITECTURE.md`

## 技术栈

- **语言**: Rust
- **Web 框架**: Salvo (API Gateway), Tonic (gRPC)
- **数据库**: PostgreSQL / SQLite (测试)
- **缓存**: Redis
- **消息队列**: Kafka
- **ORM**: SQLx

## 目录结构

```
rust-cex/
├── Cargo.toml                    # Workspace 配置
├── crates/
│   ├── common/                   # 公共组件
│   │   ├── db/                   # 数据库组件 (支持 SQLite/PG)
│   │   ├── cache/               # Redis 缓存组件
│   │   ├── queue/               # Kafka 消息队列组件
│   │   ├── utils/               # 通用工具 (cipher/wallet/id)
│   │   ├── auth/                # 鉴权组件 (JWT/API Key)
│   │   └── rate_limiter/        # 限流组件 (Token Bucket/Sliding Window)
│   ├── domain/                   # 领域模型共享包
│   ├── api-gateway/              # API 网关 (HTTP/WS 入口)
│   ├── user-service/             # 用户服务
│   ├── wallet-service/           # 钱包服务
│   ├── portfolio-service/         # 账户+持仓+清算+账本
│   ├── order-service/            # 订单服务
│   ├── risk-service/             # 风控服务
│   ├── auth-service/             # 鉴权服务 (JWT/API Key)
│   ├── market-data-service/      # 行情服务
│   ├── matching-engine/          # 撮合引擎 (内存 CLOB + WAL)
│   ├── prediction-market-service/ # 预测市场服务
│   └── ws-market-data/           # 行情 WebSocket 服务
```
```

## 编译命令

```bash
# Rust 工具链路径
export PATH="/home/ubuntu/.cargo/bin:$PATH"
# 或
export PATH="/home/ubuntu/.cargo/bin/rustc:$PATH"

# 编译所有服务
cargo build

# 编译特定服务
cargo build -p user-service
cargo build -p market-data-service

# 运行测试 (SQLite)
cargo test -p user-service

# 运行服务
cargo run -p user-service
cargo run -p api-gateway
```

## 服务列表

### 核心业务服务

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

### 网关

| 服务 | 端口 | 说明 |
|------|------|------|
| API Gateway | 8080 | HTTP/WS 统一入口 |

### Portfolio Service 模块

合并自:
- Account Service (账户余额)
- Position Service (持仓管理)
- Clearing Service (结算清算)
- Ledger Service (账本流水)

### Auth Service 模块

提供 API 和 WebSocket 鉴权:
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
┌─────────────────────────────────────────────────────────────────┐
│                         用户                                      │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                    ┌──────▼──────┐
                    │ API Gateway │ 8080
                    │ HTTP/WS     │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   gRPC ▼             gRPC ▼             gRPC ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   User       │  │   Market     │  │   Order      │
│  Service     │  │   Data Svc   │  │  Service     │
│   50001      │  │   50006      │  │   50003      │
└──────────────┘  └──────┬───────┘  └──────┬───────┘
                         │                  │
   ┌─────────────────────┼──────────────────┼─────────────────────┐
   │                     │                  │                     │
   │               ┌─────▼─────────────────▼─────┐               │
   │               │       Matching Engine         │               │
   │               │          50009               │               │
   │               │    ┌───────────────────┐     │               │
   │               │    │   内存 CLOB       │     │               │
   │               │    │   + WAL 持久化    │     │               │
   │               │    └───────────────────┘     │               │
   │               └─────────────┬─────────────────┘               │
   │                             │                               │
   │              ┌──────────────┼──────────────┐                │
   │              ▼              ▼              ▼                │
   │      ┌────────────┐ ┌────────────┐ ┌────────────┐           │
   │      │  Market   │ │  Account   │ │ Position   │           │
   │      │  Data Svc │ │  Service   │ │  Service   │           │
   │      └─────┬─────┘ └─────┬──────┘ └─────┬──────┘           │
   │            │             │             │                    │
   │            │             └──────┬──────┘                    │
   │            │                    │                           │
   │      ┌─────▼─────┐      ┌──────▼──────┐                   │
   │      │ ws-market │      │   Ledger    │                   │
   │      │ -data     │      │   Service   │                   │
   │      │  50016    │      │   50011     │                   │
   │      └───────────┘      └─────────────┘                   │
   │                                                              │
   └──────────────────────────────────────────────────────────────┘
                                    │
                         ┌──────────▼──────────┐
                         │  Kafka             │
                         │  事件总线           │
                         └─────────────────────┘
```

## 数据库规划

| 服务 | SQLite (测试) | PostgreSQL (生产) | 备注 |
|------|:-------------:|:-----------------:|------|
| User Service | ✅ | ✅ | 独立 |
| Wallet Service | ✅ | ✅ | 独立 |
| Order Service | ✅ | ✅ | 独立 |
| Risk Service | ✅ | ✅ | 独立 |
| Position Service | ✅ | ✅ | 独立 |
| Market Data Service | ✅ | ✅ | 共享 Prediction Market DB |
| Admin Service | ✅ | ✅ | 独立 |
| Clearing Service | ✅ | ✅ | 独立 |
| Matching Engine | ❌ | 内存 + WAL | 无 DB |
| Prediction Market Service | ✅ | ✅ | 主数据库 |
| Ledger Service | ✅ | ✅ | 独立 |
| Trade Service | ✅ | ✅ | 独立 |
| Account Service | ✅ | ✅ | 独立 |
| Reconciliation Service | ✅ | ✅ | 独立 |

## 服务间通信

| 源服务 | 目标服务 | 通信方式 | 说明 |
|--------|----------|----------|------|
| API Gateway | 所有业务服务 | gRPC | HTTP 入口转发 |
| Order Service | Matching Engine | gRPC | 下单/撤单 |
| Matching Engine | Kafka | Kafka | 发布成交/订单事件 |
| Kafka | Order Service | Kafka Consumer | 订单状态更新 |
| Kafka | Position Service | Kafka Consumer | 持仓更新 |
| Kafka | Account Service | Kafka Consumer | 余额更新 |
| Kafka | ws-market-data | Kafka Consumer | 行情推送 |
| Kafka | ws-order | Kafka Consumer | 订单状态推送 |
| Kafka | ws-prediction | Kafka Consumer | 市场事件推送 |
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

所有微服务使用 Tonic 实现 gRPC，API Gateway 统一提供 HTTP 接口。

```
{service}/
├── Cargo.toml                  # 依赖: tonic, prost, tonic-build
├── build.rs                    # Proto 编译配置
├── config/
│   └── {service}.yaml          # 服务配置
└── src/
    ├── lib.rs                  # 模块导出
    ├── main.rs                 # 入口
    ├── config.rs               # 配置加载
    ├── models.rs               # 数据模型
    ├── pb.rs                   # Proto 生成代码引入
    ├── pb/
    │   ├── {service}.proto     # Proto 定义
    │   ├── {service}.rs        # 生成代码 (auto)
    │   └── {service}.desc      # 描述符 (auto)
    ├── repository/
    │   ├── mod.rs
    │   └── {entity}_repo.rs    # 数据库操作
    └── services/
        ├── mod.rs
        └── {service}.rs        # gRPC 服务实现
```

### Proto 定义示例

```protobuf
syntax = "proto3";

package {service};

service {ServiceName}Service {
    rpc Method1(Method1Request) returns (Method1Response);
    rpc Method2(Method2Request) returns (Method2Response);
}

message Method1Request {
    string param = 1;
}

message Method1Response {
    bool success = 1;
    string message = 2;
}
```

### gRPC 服务实现示例

```rust
// services/{service}.rs
use tonic::{Request, Response, Status};
use crate::pb::{service_name_service_server::ServiceNameService, *};

pub struct ServiceNameServiceImpl {
    pool: sqlx::PgPool,
}

#[tonic::async_trait]
impl ServiceNameService for ServiceNameServiceImpl {
    async fn method1(
        &self,
        request: Request<Method1Request>,
    ) -> Result<Response<Method1Response>, Status> {
        // 业务逻辑
        Ok(Response::new(Method1Response {
            success: true,
            message: "OK".to_string(),
        }))
    }
}
```

### 注意事项

- 服务端使用 tonic，不依赖 salvo
- API Gateway 负责 HTTP -> gRPC 转换
- Proto 文件放在 `src/pb/` 目录
- build.rs 配置 tonic-build 生成代码

---

## API 接口定义规范

### 架构原则

```
┌─────────────────────────────────────────────────────────────────┐
│                         API Gateway                               │
│              依赖 crates/api (接口定义)                          │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                         crates/api                               │
│              统一接口定义，所有服务依赖此包                       │
│   ├── user.rs (来自 user-service)                               │
│   ├── order.rs (来自 order-service)                             │
│   ├── auth.rs (来自 auth-service)                              │
│   └── ...                                                      │
└──────────────────────────┬──────────────────────────────────────┘
                           │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│user-service │   │order-service│   │auth-service │
│  src/pb/   │   │   src/pb/   │   │   src/pb/   │
│ user.proto  │   │order.proto │   │ auth.proto  │
└─────────────┘   └─────────────┘   └─────────────┘
```

### 设计原则

1. **Proto 文件放在各自服务** - 源文件 `src/pb/*.proto` 保留在服务目录
2. **生成代码输出到 crates/api** - build.rs 配置输出到 `crates/api/src/`
3. **统一依赖** - 所有组件依赖 `api = { path = "../api" }`
4. **单向依赖** - domain → api → services → gateway

### 服务 Proto 定义模板

```rust
// {service}/build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let api_src = manifest_dir
        .parent().unwrap()
        .parent().unwrap()
        .join("crates/api/src");

    std::fs::create_dir_all(&api_src)?;

    tonic_build::configure()
        .build_server(true)           // 生成服务端代码
        .build_client(true)          // 生成客户端代码
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
pub mod user {
    include!("user.rs");
}

pub mod order {
    include!("order.rs");
}

pub mod auth {
    include!("auth.rs");
}

// ... 其他服务模块

// 便捷导出
pub use user::user_service_client::UserServiceClient;
pub use order::order_service_client::OrderServiceClient;
pub use auth::auth_service_client::AuthServiceClient;
```

### 服务间依赖关系

| 包 | 依赖 | 说明 |
|----|------|------|
| `domain` | 无 | 纯业务模型，无序列化 |
| `api` | 无 | 接口定义 + 序列化注解 |
| `user-service` | domain, api | 服务实现 |
| `order-service` | domain, api | 服务实现 |
| `api-gateway` | api, common | 依赖接口，不依赖服务实现 |
| `ws-*` | api | WebSocket 服务 |

### 新增服务流程

1. 在 `src/pb/` 创建 `*.proto` 文件
2. 配置 `build.rs` 输出到 `crates/api/src/`
3. 服务 `Cargo.toml` 添加 `api = { path = "../api" }`
4. 在 `crates/api/src/lib.rs` 添加 `pub mod {service};`
5. 其他服务/Gateway 依赖 `api` 使用接口

### Proto 命名规范

- 服务名使用下划线: `user_service.proto`
- 生成模块使用下划线: `pub mod user { include!("user.rs"); }`
- 生成的 Client: `UserServiceClient`