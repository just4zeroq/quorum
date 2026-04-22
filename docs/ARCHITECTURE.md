# 预测市场 CEX 架构文档

## 一、项目概述

### 1.1 项目定位

这是一个基于 **Rust** 实现的**中心化交易所 (CEX)** 项目，专门用于**预测市场 (Prediction Market)** 交易。

- **交易类型**: 现货交易 (Spot Trading)
- **核心机制**: CLOB (中央限价订单簿)
- **支持资产**: 预测市场事件的各种结果选项 (如 "A队获胜" / "B队获胜")

### 1.2 技术栈

| 类别 | 技术选型 |
|------|----------|
| 语言 | Rust |
| 服务框架 | Tonic (gRPC) |
| Web 框架 | Salvo (API Gateway HTTP/WS) |
| 数据库 | PostgreSQL (生产) / SQLite (测试) |
| 缓存 | Redis |
| 消息队列 | Kafka |
| ORM | SQLx |

---

## 二、服务架构

### 2.1 服务列表

#### 核心业务服务

| 服务 | 端口 | 数据库 | 职责 | 状态 |
|------|------|--------|------|------|
| **User Service** | 50001 | 独立 | 用户注册、登录、KYC、2FA | ✅ Proto + Server |
| **Wallet Service** | 50002 | 独立 | 钱包地址、充值、提现 | 框架 |
| **Order Service** | 50003 | 独立 | 订单 CRUD、冻结请求、事件记录 | ✅ Proto + Server |
| **Risk Service** | 50004 | 独立 | 风控检查、限额 (简化版) | 待实现 |
| **Position Service** | 50005 | 独立 | 用户持仓管理 | 待实现 |
| **Market Data Service** | 50006 | 共享 PM DB | 行情、K线、订单簿、24h 统计 | ✅ Proto + Server + Tests |
| **Admin Service** | 50007 | 独立 | 管理后台 | 框架 |
| **Clearing Service** | 50008 | 独立 | 成交结算、派彩计算 | 待实现 |
| **Matching Engine** | 50009 | 无 | CLOB 撮合 (内存 + WAL) | ✅ Core Logic |
| **Prediction Market Service** | 50010 | 主数据库 | 预测市场管理、结算、派彩 | ✅ Proto + Server + Tests |
| **Ledger Service** | 50011 | 独立 | 账本服务 (不可变) | 待实现 |
| **Trade Service** | 50013 | 独立 | 成交记录管理 | 待实现 |
| **Account Service** | - | 独立 | 余额管理 (Available/Frozen) | 待实现 |
| **Reconciliation Service** | 50014 | 独立 | 数据对账 | 待实现 |

#### WebSocket 服务

| 服务 | 端口 | 推送数据 |
|------|------|----------|
| **ws-market-data** | 50016 | K线、成交、深度、24h ticker |
| **ws-order** | 50017 | 用户订单状态变更 |
| **ws-prediction** | 50018 | 市场事件 (结算、关闭) |

#### 网关

| 服务 | 端口 | 职责 |
|------|------|------|
| **API Gateway** | 8080 | HTTP 入口、WS 连接管理 |

#### 已移除的服务 (现货不需要)

| 服务 | 原因 |
|------|------|
| Liquidation Engine | 合约强平机制，现货不需要 |
| Mark Price Service | 合约标记价格，现货不需要 |
| Funding Service | 合约资金费率，现货不需要 |

### 2.2 架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              用户 / 客户端                                    │
│                                                                             │
│   WebSocket 连接 1 ──▶ ws-market-data:50016 (行情订阅)                     │
│   WebSocket 连接 2 ──▶ ws-order:50017 (订单订阅)                            │
│   WebSocket 连接 3 ──▶ ws-prediction:50018 (市场事件订阅)                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        │ HTTP/gRPC
                                        ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         API Gateway (8080)                                  │
│                    HTTP 入口 + WS 连接管理                                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                        │
                ┌───────────────────────┼───────────────────────┐
                │                       │                       │
          gRPC ▼                       │                   gRPC ▼
┌─────────────────────┐   ┌─────────────┴────────────┐  ┌─────────────────────┐
│   User Service      │   │   Market Data Service    │  │   Order Service    │
│     50001           │   │        50006             │  │      50003          │
│                     │   │  (共享 Prediction       │  │                     │
│  - 注册/登录         │   │   Market 数据库)        │  │  - 创建订单         │
│  - KYC/2FA          │   │                         │  │  - 取消订单         │
│  - 会话管理          │   │  - 行情查询             │  │  - 订单查询         │
└─────────────────────┘   └─────────────┬────────────┘  └──────────┬──────────┘
                                        │                             │
                                        │                    ┌──────────▼──────────┐
                                        │                    │  Matching Engine    │
                                        │                    │     50009           │
                                        │                    │                    │
                                        │                    │  ┌────────────────┐ │
┌─────────────────────┐   ┌─────────────┴────────────┐  │  │  内存 CLOB     │ │
│ Prediction Market  │   │   Market Data Service    │◀─┼──│  + WAL 持久化  │ │
│    Service          │   │   (共享数据库)            │  │  │                │ │
│     50010          │   └───────────────────────────┘  │  └────────────────┘ │
│                     │                                 │                     │
│  - 市场管理         │   ┌───────────────────────────┐  │  ┌────────────────▼──┐
│  - 选项管理         │   │      Kafka               │  │  │      Kafka        │
│  - 结算派彩         │   │    事件总线              │  │  └──────────┬───────┘
└─────────────────────┘   └───────────┬────────────┘             │
                                        │                        │
                    ┌───────────────────┼──────────────────────────┼────────────┐
                    │                   │                          │            │
                    ▼                   ▼                          ▼            ▼
            ┌───────────────┐   ┌───────────────┐   ┌───────────────┐  ┌───────────────┐
            │ ws-market-    │   │ ws-order      │   │ ws-prediction │  │   Position    │
            │ data         │   │               │   │               │  │   Service      │
            │  50016       │   │  50017        │   │   50018        │  │   50005        │
            └───────────────┘   └───────────────┘   └───────────────┘  └───────────────┘
                                        │                          │            │
                                        │                          ▼            │
                                        │                 ┌───────────────┐      │
                                        │                 │   Account     │      │
                                        │                 │   Service     │      │
                                        │                 └───────┬───────┘      │
                                        │                         │              │
                                        │                         ▼              │
                                        │                 ┌───────────────┐      │
                                        │                 │    Ledger     │      │
                                        │                 │   Service     │      │
                                        │                 │   50011       │      │
                                        │                 └───────────────┘      │
                                        │                                            │
                                        ▼                                            │
                              ┌───────────────────────┐                              │
                              │    PostgreSQL         │                              │
                              │  (各服务独立数据库)    │                              │
                              └───────────────────────┘                              │
```

### 2.6 鉴权与限流设计

#### common/auth - 鉴权组件

统一鉴权接口，支持多种鉴权方式：

```
crates/common/auth/
├── lib.rs
├── jwt.rs           # JWT Token 验证
├── api_key.rs       # API Key 验证
├── context.rs       # AuthContext (用户信息提取)
├── error.rs         # AuthError 错误类型
└── traits.rs        # AuthService trait
```

**核心 Trait**：

```rust
pub trait AuthService: Send + Sync {
    fn validate_token(&self, token: &str) -> Result<AuthContext, AuthError>;
    fn validate_api_key(&self, key: &str) -> Result<AuthContext, AuthError>;
}

pub struct AuthContext {
    pub user_id: i64,
    pub token_type: TokenType,  // JWT / API_KEY / WS_TOKEN
    pub extra: HashMap<String, String>,
}
```

**鉴权流程**：

```
API Gateway:
  请求 → [JWT/API Key] → AuthService::validate → AuthContext → Handler

ws-order:
  连接/首消息 → [Token] → AuthService::validate → AuthContext → 存储 Context
```

#### common/rate_limiter - 限流组件

支持多种限流算法和存储后端：

```
crates/common/rate_limiter/
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

**核心 Trait**：

```rust
pub trait RateLimiter: Send + Sync {
    fn check(&self, key: &str) -> Result<(), RateLimitError>;
    fn get_limit(&self, key: &str) -> RateLimit;
}
```

**限流维度**：

| 服务 | Key 格式 | 限制 |
|------|----------|------|
| API Gateway | `api:{user_id}:{endpoint}` | 100/minute |
| ws-order | `ws:{user_id}:order` | 10/second |
| ws-market-data | `ws:{ip}:connect` | 5/minute |

---

## 三、核心模块设计

### 3.1 预测市场模型

预测市场与传统金融市场不同，每个 Market (事件) 有多个 Outcome (选项)，用户交易的是每个选项的**概率价格** (0-1 之间)。

```
PredictionMarket (预测市场)
├── question: 事件问题 (如 "A队能夺冠吗?")
├── category: 分类 (如 "体育/政治/金融")
├── start_time / end_time: 交易时间窗口
├── status: open / resolved / cancelled
└── outcomes: []MarketOutcome

MarketOutcome (市场选项)
├── name: 选项名称 (如 "A队夺冠")
├── price: 当前价格 (0-1, 概率)
├── volume: 累计成交量
└── probability: 计算概率
```

### 3.2 CLOB 订单簿设计

每个 Market + Outcome 对应一个独立的订单簿：

```
Market 1: "2024年世界杯冠军"
├── Outcome 1: "巴西"  →  订单簿 A
├── Outcome 2: "阿根廷" → 订单簿 B
└── Outcome 3: "法国"  →  订单簿 C
```

订单簿结构：
- **Bids (买盘)**: 按价格降序排列
- **Asks (卖盘)**: 按价格升序排列
- 每档价格有多个订单，按时间优先

### 3.3 订单类型

| 类型 | 说明 |
|------|------|
| **Limit** | 限价单，挂在订单簿等待成交 |
| **Market** | 市价单，立即以对手方最优价成交 |
| **IOC** | 即时成交否则取消，未成交部分撤单 |
| **FOK** | 全部成交否则取消，必须完全成交否则全部撤单 |
| **PostOnly** | 只挂单，如果会立即成交则拒绝 |

### 3.4 Matching Engine 持久化设计

撮合引擎采用 **内存撮合 + WAL + 异步 DB** 方案：

```
┌─────────────────────────────────────────────────────────────────┐
│                   Matching Engine 持久化架构                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  内存撮合（高性能）                                                │
│       │                                                          │
│       ├──▶ WAL (实时追加) ────────────── 崩溃后可恢复             │
│       │         │                                                │
│       │         └── 每笔成交实时写入                               │
│       │                                                            │
│       ├──▶ Redis (可选缓存) ─────────── 加速恢复                 │
│       │                                                            │
│       └──▶ PostgreSQL (异步批量) ───────── 最终持久化             │
│                   │                                                │
│                   └── 批量写入（100ms 或 100 笔成交）             │
│                                                                  │
│  定期快照                                                         │
│       │                                                          │
│       └── 每 N 笔成交 或 每秒 ──▶ 订单簿完整状态                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**持久化内容**：

| 数据 | 持久化位置 | 频率 | 说明 |
|------|-----------|------|------|
| 订单簿状态 | WAL + 快照 | 实时 + 定期 | 可完整恢复 |
| 成交记录 | PostgreSQL | 异步批量 | trades 表 |
| 订单状态 | PostgreSQL | 异步批量 | orders 表 |
| 市场统计 | PostgreSQL | 异步批量 | market_stats 表 |

**恢复流程**：

```
Matching Engine 启动：
1. 加载最新快照 → 重建内存订单簿
2. 重放快照后的 WAL → 恢复到最新状态
3. 从 DB 补充未完成的成交记录
4. 继续撮合
```

### 3.5 WebSocket 服务设计

#### ws-market-data (50016)

| 订阅主题 | 数据类型 | 说明 |
|----------|----------|------|
| `kline:{market_id}:{interval}` | K线数据 | 1m/5m/15m/1h/4h/1d |
| `trades:{market_id}` | 实时成交 | 成交记录 |
| `depth:{market_id}` | 订单簿深度 | 买卖盘档位 |
| `ticker:{market_id}` | 24h 统计 | 价格/成交量变化 |

#### ws-order (50017)

| 订阅主题 | 数据类型 | 说明 |
|----------|----------|------|
| `orders:{user_id}` | 订单状态变更 | 用户私有，只能订阅自己的 |

#### ws-prediction (50018)

| 订阅主题 | 数据类型 | 说明 |
|----------|----------|------|
| `market:{market_id}:status` | 市场状态变更 | 结算/关闭通知 |

---

## 四、数据模型

### 4.1 核心表结构

#### 4.1.1 用户模块 (User Service)

**users** - 用户表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| username | VARCHAR(50) | 用户名 (唯一) |
| email | VARCHAR(255) | 邮箱 (唯一) |
| password_hash | VARCHAR(255) | 密码哈希 |
| kyc_status | VARCHAR(20) | KYC 状态: none/submitting/pending/verified/rejected |
| two_factor_enabled | BOOLEAN | 是否启用 2FA |
| status | VARCHAR(20) | 账户状态: active/frozen/closed |
| created_at | TIMESTAMP | 创建时间 |
| updated_at | TIMESTAMP | 更新时间 |

**user_sessions** - 会话表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| user_id | BIGINT | 关联用户 |
| token | TEXT | JWT Token (唯一) |
| refresh_token | TEXT | 刷新 Token |
| expires_at | TIMESTAMP | 过期时间 |
| login_method | VARCHAR(20) | 登录方式: email/wallet |

**wallet_addresses** - 钱包地址表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| user_id | BIGINT | 关联用户 |
| wallet_address | VARCHAR(100) | 钱包地址 |
| wallet_type | VARCHAR(20) | 钱包类型: eth/btc/tron |
| chain_type | VARCHAR(20) | 链类型 |
| is_primary | BOOLEAN | 是否主地址 |

#### 4.1.2 预测市场模块 (Prediction Market Service)

**prediction_markets** - 预测市场表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| question | TEXT | 事件问题 |
| description | TEXT | 描述 |
| category | VARCHAR(50) | 分类 |
| image_url | TEXT | 图片URL |
| start_time | BIGINT | 开始时间 (毫秒) |
| end_time | BIGINT | 结束时间 (毫秒) |
| status | VARCHAR(20) | 状态: open/resolved/cancelled |
| resolved_outcome_id | BIGINT | 结算选项ID |
| resolved_at | BIGINT | 结算时间 |
| total_volume | TEXT | 总成交量 (Decimal) |
| created_at | BIGINT | 创建时间 |
| updated_at | BIGINT | 更新时间 |

**market_outcomes** - 市场选项表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| market_id | BIGINT | 关联市场 |
| name | VARCHAR(100) | 选项名称 |
| description | TEXT | 描述 |
| image_url | TEXT | 图片URL |
| price | TEXT | 当前价格 (0-1 Decimal) |
| volume | TEXT | 累计成交量 |
| probability | TEXT | 计算概率 |
| created_at | BIGINT | 创建时间 |
| updated_at | BIGINT | 更新时间 |

**user_positions** - 用户持仓表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| user_id | BIGINT | 用户ID |
| market_id | BIGINT | 市场ID |
| outcome_id | BIGINT | 选项ID |
| quantity | TEXT | 持仓数量 |
| avg_price | TEXT | 平均价格 |
| created_at | BIGINT | 创建时间 |
| updated_at | BIGINT | 更新时间 |
| UNIQUE(user_id, market_id, outcome_id) | | 联合唯一 |

**market_trades** - 成交记录表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| market_id | BIGINT | 市场ID |
| outcome_id | BIGINT | 选项ID |
| user_id | BIGINT | 用户ID |
| side | VARCHAR(10) | buy/sell |
| price | TEXT | 成交价格 |
| quantity | TEXT | 成交数量 |
| amount | TEXT | 成交金额 |
| fee | TEXT | 手续费 |
| created_at | BIGINT | 成交时间 |

**market_resolutions** - 结算记录表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| market_id | BIGINT | 市场ID (唯一) |
| outcome_id | BIGINT | 获胜选项ID |
| total_payout | TEXT | 总派彩 |
| winning_quantity | TEXT | 获胜总量 |
| payout_ratio | TEXT | 派彩比例 |
| resolved_at | BIGINT | 结算时间 |

#### 4.1.3 订单模块 (Order Service)

**orders** - 订单表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT | 订单ID (时间序列号) |
| user_id | BIGINT | 用户ID |
| market_id | BIGINT | 市场ID |
| outcome_id | BIGINT | 选项ID |
| side | VARCHAR(10) | buy/sell |
| order_type | VARCHAR(20) | 订单类型 |
| price | TEXT | 价格 |
| quantity | TEXT | 数量 |
| filled_quantity | TEXT | 已成交数量 |
| filled_amount | TEXT | 已成交金额 |
| status | VARCHAR(20) | 状态 |
| client_order_id | TEXT | 客户端订单ID |
| created_at | BIGINT | 创建时间 |
| updated_at | BIGINT | 更新时间 |

**索引**:
- `PRIMARY KEY (id)`
- `INDEX idx_user_id (user_id)`
- `INDEX idx_market_id (market_id)`
- `INDEX idx_status (status)`
- `INDEX idx_created_at (created_at)`

**order_events** - 订单事件表 (不可变记录)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| order_id | TEXT | 订单ID |
| event_type | VARCHAR(30) | 事件类型 |
| old_status | VARCHAR(20) | 变更前状态 |
| new_status | VARCHAR(20) | 变更后状态 |
| filled_quantity | TEXT | 成交数量 (成交事件) |
| filled_amount | TEXT | 成交金额 |
| price | TEXT | 成交价格 |
| reason | TEXT | 变更原因 |
| created_at | BIGINT | 事件时间 |

**事件类型**:
| 事件 | 说明 |
|------|------|
| `created` | 订单创建 |
| `submitted` | 提交到撮合引擎 |
| `partially_filled` | 部分成交 |
| `filled` | 完全成交 |
| `cancelled` | 用户取消 |
| `rejected` | 拒单 (价格不在区间、风控拒绝等) |

---

## 五、服务接口设计

### 5.1 User Service (50001)

```protobuf
service UserService {
    // 认证
    rpc Register(RegisterRequest) returns (RegisterResponse);
    rpc Login(LoginRequest) returns (LoginResponse);
    rpc Logout(LogoutRequest) returns (LogoutResponse);
    rpc RefreshToken(RefreshTokenRequest) returns (RefreshTokenResponse);

    // 钱包登录
    rpc WalletLogin(WalletLoginRequest) returns (WalletLoginResponse);
    rpc WalletBind(WalletBindRequest) returns (WalletBindResponse);

    // 用户信息
    rpc GetUser(GetUserRequest) returns (GetUserResponse);
    rpc UpdateUser(UpdateUserRequest) returns (UpdateUserResponse);

    // 安全
    rpc Enable2FA(Enable2FARequest) returns (Enable2FAResponse);
    rpc Verify2FA(Verify2FARequest) returns (Verify2FAResponse);

    // KYC
    rpc SubmitKYC(SubmitKYCRequest) returns (SubmitKYCResponse);
    rpc GetKYCStatus(GetKYCStatusRequest) returns (GetKYCStatusResponse);
}
```

### 5.2 Prediction Market Service (50010)

```protobuf
service PredictionMarketService {
    // 市场管理
    rpc CreateMarket(CreateMarketRequest) returns (CreateMarketResponse);
    rpc UpdateMarket(UpdateMarketRequest) returns (UpdateMarketResponse);
    rpc CloseMarket(CloseMarketRequest) returns (CloseMarketResponse);
    rpc GetMarket(GetMarketRequest) returns (GetMarketResponse);
    rpc ListMarkets(ListMarketsRequest) returns (ListMarketsResponse);

    // 选项管理
    rpc AddOutcome(AddOutcomeRequest) returns (AddOutcomeResponse);
    rpc GetOutcomes(GetOutcomesRequest) returns (GetOutcomesResponse);

    // 结算
    rpc ResolveMarket(ResolveMarketRequest) returns (ResolveMarketResponse);
    rpc CalculatePayout(CalculatePayoutRequest) returns (CalculatePayoutResponse);

    // 用户持仓
    rpc GetUserPositions(GetUserPositionsRequest) returns (GetUserPositionsResponse);
}
```

### 5.3 Order Service (50003)

```protobuf
service OrderService {
    // 订单操作
    rpc CreateOrder(CreateOrderRequest) returns (CreateOrderResponse);
    rpc CancelOrder(CancelOrderRequest) returns (CancelOrderResponse);
    rpc GetOrder(GetOrderRequest) returns (GetOrderResponse);
    rpc GetUserOrders(GetUserOrdersRequest) returns (GetUserOrdersResponse);

    // 批量操作
    rpc GetOrdersByMarket(GetOrdersByMarketRequest) returns (GetOrdersByMarketResponse);

    // 订单状态更新（Matching Engine 调用）
    rpc UpdateOrderStatus(UpdateOrderStatusRequest) returns (UpdateOrderStatusResponse);
}
```

### 5.4 Market Data Service (50006)

```protobuf
service MarketDataService {
    // 市场查询
    rpc GetMarkets(GetMarketsRequest) returns (GetMarketsResponse);
    rpc GetMarketDetail(GetMarketDetailRequest) returns (GetMarketDetailResponse);

    // 价格查询
    rpc GetOutcomePrices(GetOutcomePricesRequest) returns (GetOutcomePricesResponse);

    // 订单簿
    rpc GetOrderBook(GetOrderBookRequest) returns (GetOrderBookResponse);

    // K线
    rpc GetKlines(GetKlinesRequest) returns (GetKlinesResponse);

    // 成交记录
    rpc GetRecentTrades(GetRecentTradesRequest) returns (GetRecentTradesResponse);

    // 24h 统计
    rpc Get24hStats(Get24hStatsRequest) returns (Get24hStatsResponse);
}
```

### 5.5 Matching Engine (50009)

撮合引擎为内存服务，通过 gRPC 与其他服务通信，通过 Kafka 发布事件：

```protobuf
service MatchingEngineService {
    // 订单处理
    rpc SubmitOrder(SubmitOrderRequest) returns (SubmitOrderResponse);
    rpc CancelOrder(CancelOrderRequest) returns (CancelOrderResponse);

    // 订单簿查询
    rpc GetOrderBook(GetOrderBookRequest) returns (GetOrderBookResponse);
    rpc GetDepth(GetDepthRequest) returns (GetDepthResponse);

    // 统计
    rpc GetMarketStats(GetMarketStatsRequest) returns (GetMarketStatsResponse);

    // 市场管理 (由 Prediction Market Service 调用)
    rpc CreateMarket(CreateMarketRequest) returns (CreateMarketResponse);
    rpc CloseMarket(CloseMarketRequest) returns (CloseMarketResponse);
}
```

**Kafka 事件发布**:

- `order_events`: 订单状态变更事件
- `trade_executed`: 成交事件
- `market_data`: 行情更新事件

---

## 六、服务间通信

### 6.1 通信矩阵

| 源服务 | 目标服务 | 通信内容 | 方式 |
|--------|----------|----------|------|
| API Gateway | User Service | 认证、用户信息 | gRPC |
| API Gateway | Order Service | 下单、撤单、查询 | gRPC |
| API Gateway | Market Data Service | 行情、深度、K线 | gRPC |
| API Gateway | Prediction Market Service | 市场列表、结算 | gRPC |
| Order Service | Matching Engine | 提交订单 | gRPC |
| Order Service | Matching Engine | 取消订单 | gRPC |
| Matching Engine | Kafka | 发布订单事件 | Kafka |
| Matching Engine | Kafka | 发布成交事件 | Kafka |
| Kafka Consumer | Order Service | 消费订单事件 | Kafka |
| Kafka Consumer | Trade Service | 消费成交事件 | Kafka |
| Kafka Consumer | Position Service | 消费持仓更新 | Kafka |
| Kafka Consumer | Account Service | 消费余额更新 | Kafka |
| Prediction Market Service | Matching Engine | 创建市场、初始化订单簿 | gRPC |
| Prediction Market Service | Kafka | 发布市场事件 | Kafka |

### 6.2 交易流程 (事件驱动)

```
1. 用户登录 -> User Service
         ↓
2. 查询市场列表 -> Prediction Market Service
         ↓
3. 查询行情/深度 -> Market Data Service
         ↓
4. 下单 -> Order Service (创建订单，记录事件)
         ↓
5. Order Service -> Matching Engine (gRPC 提交订单)
         ↓
6. Matching Engine 撮合 -> 发布 Kafka 事件
         │
         ├── order_filled 事件 ──▶ Order Service (更新状态 + 事件)
         │
         ├── trade_executed 事件 ──▶ Trade Service (写入成交记录 + 事件)
         │                                  │
         │                                  ▼
         │                           持仓/余额更新
         │
         └── market_data 事件 ──▶ Market Data Service (推送行情)
         ↓
7. 外部订阅者订阅事件 (WebSocket、审计、报表)
```

### 6.3 Kafka 事件系统

#### 6.3.1 Topic 设计

| Topic | 说明 | Key | 分区策略 |
|-------|------|-----|----------|
| `order_events` | 订单状态变更事件 | order_id | market_id |
| `trade_executed` | 成交事件 | order_id | market_id |
| `position_updates` | 持仓更新事件 | user_id | user_id |
| `balance_updates` | 余额更新事件 | user_id | user_id |
| `market_events` | 市场事件 (创建/结算/关闭) | market_id | market_id |

#### 6.3.2 事件格式

**OrderEvent (订单事件)**
```json
{
    "event_id": "evt_260422153045000001",
    "order_id": "260422153045010123456789",
    "event_type": "filled",
    "old_status": "partially_filled",
    "new_status": "filled",
    "filled_quantity": "100",
    "filled_amount": "50.5",
    "price": "0.505",
    "reason": null,
    "timestamp": 1776809545000
}
```

**TradeExecutedEvent (成交事件)**
```json
{
    "event_id": "tde_260422153045000001",
    "trade_id": "26042215304500012301",
    "order_id": "260422153045010123456789",
    "counter_order_id": "260422153045010223456789",
    "market_id": 1,
    "outcome_id": 2,
    "maker_user_id": 100,
    "taker_user_id": 101,
    "side": "buy",
    "price": "0.505",
    "quantity": "100",
    "amount": "50.5",
    "maker_fee": "0.0505",
    "taker_fee": "0.101",
    "fee_token": "USDT",
    "timestamp": 1776809545000
}
```

#### 6.3.3 事件消费者

```
Kafka 事件流
     │
     ├──▶ Order Service
     │    消费: order_events
     │    处理: 更新订单状态 + 记录事件
     │
     ├──▶ Trade Service  [NEW]
     │    消费: trade_executed
     │    处理: 写入成交记录 + 生成成交事件
     │
     ├──▶ Position Service
     │    消费: trade_executed
     │    处理: 更新用户持仓
     │
     ├──▶ Account Service
     │    消费: trade_executed
     │    处理: 更新账户余额 (扣除手续费)
     │
     ├──▶ Market Data Service
     │    消费: trade_executed
     │    处理: 更新最新价格 + 成交记录
     │
     └──▶ 外部订阅者 (可选)
          消费: order_events, trade_executed
          处理: WebSocket 推送、审计、报表
```

#### 6.3.4 消息事务保证

使用 Kafka 事务 API 保证 Exactly-Once：

```rust
// Matching Engine 内部
async fn execute_trade(&self, trade: Trade) -> Result<(), Error> {
    let producer = self.kafka_producer.transaction();
    
    // 在同一事务中发送多个事件
    producer.send_in_transaction(
        ("order_filled", order_event),
        ("trade_executed", trade_event),
        ("position_updates", position_event),
    ).await?;
    
    producer.commit().await?;
}
```

**幂等消费**：
- 每个事件包含 `event_id`，消费者记录已处理的事件 ID
- 重复消费时检查 ID 存在则跳过

### 6.4 跨服务事务 (最终一致)

撮合成交涉及多服务更新，采用事件驱动最终一致：

```
Matching Engine 成交
       │
       ▼ 发布 Kafka 事件 (原子操作)
  ┌────────────────────────────────────────┐
  │  order_filled, trade_executed 事件     │
  └────────────────────────────────────────┘
       │
       ├─────────────────┬──────────────────┤
       ▼                 ▼                  ▼
  Order Service      Trade Service     Position Service
  (更新订单状态)      (写入成交记录)     (更新持仓)
       │                 │                  │
       └─────────────────┴──────────────────┘
                         │
                         ▼
                  定期对账检查
```

**对账机制**：
- 每日定时比对 Order Service 订单状态与 Trade Service 成交记录
- 发现不一致时告警并自动修复

---

## 七、数据库规划

### 7.1 数据库部署

| 服务 | SQLite (测试) | PostgreSQL (生产) | 备注 |
|------|:-------------:|:-----------------:|------|
| User Service | ✅ | ✅ | 独立 |
| Wallet Service | ✅ | ✅ | 独立 |
| Order Service | ✅ | ✅ | 独立 |
| Risk Service | ✅ | ✅ | 独立 |
| Position Service | ✅ | ✅ | 独立 |
| Market Data Service | ✅ | ✅ | **共享 Prediction Market DB** |
| Admin Service | ✅ | ✅ | 独立 |
| Clearing Service | ✅ | ✅ | 独立 |
| Matching Engine | ❌ | 内存 + WAL | 无 DB |
| Prediction Market Service | ✅ | ✅ | **主数据库** |
| Ledger Service | ✅ | ✅ | 独立 |
| Trade Service | ✅ | ✅ | 独立 |
| Account Service | ✅ | ✅ | 独立 |
| Reconciliation Service | ✅ | ✅ | 独立 |

### 7.2 数据库共享关系

```
Prediction Market Service (主数据库)
    │
    ├──▶ prediction_markets 表
    ├──▶ market_outcomes 表
    └──▶ market_resolutions 表
              │
              └──▶ Market Data Service (共享读取)

独立服务各自有独立数据库：
- User Service: users, user_sessions, wallet_addresses
- Order Service: orders, order_events
- Position Service: user_positions
- Trade Service: trades, trade_events
- Account Service: accounts, balances
- Ledger Service: ledger_entries
- Clearing Service: clearing_records
- Reconciliation Service: reconciliation_tasks, reconciliation_diffs
```

### 7.3 Kafka Topic 设计

| Topic | 生产者 | 消费者 | Key | 说明 |
|-------|--------|--------|-----|------|
| `order_events` | Matching Engine | Order Service | order_id | 订单状态变更事件 |
| `trade_executed` | Matching Engine | Trade, Position, Account, Market Data | order_id | 成交事件 |
| `position_updates` | Position Service | Account, Ledger | user_id | 持仓变更 |
| `balance_updates` | Account Service | Ledger | user_id | 余额变更 |
| `market_events` | Prediction Market Service | Matching Engine, ws-prediction | market_id | 市场事件 |
| `kline_updates` | Market Data Service | ws-market-data | market_id | K线更新 |

### 7.4 事件 Schema

**OrderEvent**
```json
{
    "event_id": "evt_260422153045000001",
    "order_id": "260422153045010123456789",
    "event_type": "filled",
    "old_status": "partially_filled",
    "new_status": "filled",
    "filled_quantity": "100",
    "filled_amount": "50.5",
    "price": "0.505",
    "timestamp": 1776809545000
}
```

**TradeExecutedEvent**
```json
{
    "event_id": "tde_260422153045000001",
    "trade_id": "26042215304500012301",
    "order_id": "260422153045010123456789",
    "market_id": 1,
    "outcome_id": 2,
    "maker_user_id": 100,
    "taker_user_id": 101,
    "side": "buy",
    "price": "0.505",
    "quantity": "100",
    "amount": "50.5",
    "maker_fee": "0.0505",
    "taker_fee": "0.101",
    "timestamp": 1776809545000
}
```

---

## 八、已实现服务详情

### 8.1 User Service ✅

**Proto**: `crates/user-service/src/pb/user.proto`

功能:
- 邮箱/用户名注册登录
- 钱包登录 (MetaMask 等)
- 2FA 安全验证
- KYC 提交
- 会话管理

### 7.2 Prediction Market Service ✅

**Proto**: `crates/prediction-market-service/src/pb/prediction_market.proto`

功能:
- 创建预测市场 (带选项)
- 更新/关闭市场
- 获取市场详情和列表
- 添加选项
- 结算市场 (设定获胜选项)
- 计算用户派彩

### 7.3 Market Data Service ✅

**Proto**: `crates/market-data-service/src/pb/market_data.proto`

功能:
- 市场列表查询 (分页、过滤、排序)
- 市场详情
- 选项价格
- 订单簿深度
- K线数据
- 成交记录
- 24h 统计

### 7.4 Order Service ✅

**Proto**: `crates/order-service/src/pb/order_service.proto`

功能:
- 创建订单 (限价/市价/IOC/FOK/PostOnly)
- 取消订单
- 查询订单
- 用户订单列表 (分页)
- 市场订单列表
- 订单状态更新 (Matching Engine 调用)

### 7.5 Matching Engine ✅

**代码**: `crates/matching-engine/src/`

功能:
- 内存 CLOB 订单簿
- 多订单类型支持 (Limit/Market/IOC/FOK/PostOnly)
- 撮合逻辑
- 深度查询
- Kafka 事件发布

### 7.6 Trade Service (规划)

职责: 成交记录管理

**数据模型**:
- `trades` - 成交记录表
- `trade_events` - 成交事件表

**表结构**:
| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| trade_id | TEXT | 成交ID (时间序列) |
| order_id | TEXT | 订单ID |
| counter_order_id | TEXT | 对手订单ID |
| market_id | BIGINT | 市场ID |
| outcome_id | BIGINT | 选项ID |
| maker_user_id | BIGINT | Maker 用户 |
| taker_user_id | BIGINT | Taker 用户 |
| side | VARCHAR(10) | taker side |
| price | TEXT | 成交价格 |
| quantity | TEXT | 成交数量 |
| amount | TEXT | 成交金额 |
| maker_fee | TEXT | Maker 手续费 |
| taker_fee | TEXT | Taker 手续费 |
| fee_token | VARCHAR(20) | 手续费币种 |
| created_at | BIGINT | 成交时间 |

**接口设计**:
```protobuf
service TradeService {
    // 成交查询
    rpc GetTrade(GetTradeRequest) returns (GetTradeResponse);
    rpc GetUserTrades(GetUserTradesRequest) returns (GetUserTradesResponse);
    rpc GetMarketTrades(GetMarketTradesRequest) returns (GetMarketTradesResponse);

    // 成交统计
    rpc GetTradeStats(GetTradeStatsRequest) returns (GetTradeStatsResponse);
}
```

---

## 九、订单服务详细设计

### 8.1 订单号生成算法

采用时间序列订单号，支持推断时间和分片：

```
格式: {时间6位}{市场2位}{用户2位}{序列6位}
      YYMMDD   MM       UU       SSSSSS

例: 260422153045010123456789
    │││││││││││││││││││└─ 序列号 (0-999999)
    ││││││││││││││││││└── 用户ID后2位 (01)
    │││││││││││││││││└─── 市场ID后2位 (01)
    ││││││││││││││││└──── 22年
    │││││││││││││││└───── 04月
    ││││││││││││││└────── 22日
    │││││││││││││└─────── 15时
    ││││││││││││└──────── 30分
    │││││││││││└───────── 45秒

总长度: 26位数字 (无符号64位整数可存)
```

**优点**:
- 有序，适合数据库索引范围查询
- 可从订单号推断大致时间
- 同一市场、用户订单号接近，方便分区
- 26位数字在 64 位整数范围内

### 8.2 订单状态流转

```
                    ┌──────────────┐
                    │              │
                    ▼              │
              ┌──────────┐        │
              │ pending  │────────┼──────┐
              └──────────┘        │      │
                   │              │      │ 提交失败
                   │ 提交         │      ▼
                   ▼              │ ┌────────────┐
              ┌───────────┐       │ │ rejected   │
              │ submitted │───────┼─┤ (拒单)     │
              └───────────┘       │ └────────────┘
                   │              │
         ┌─────────┴─────────┐    │
         │                   │    │
         ▼                   ▼    │
  ┌──────────────┐   ┌──────────────┤
  │ partially_   │   │ cancelled    │
  │ filled       │   │ (用户取消)   │
  └──────┬───────┘   └──────────────┘
         │
         │ 完全成交
         ▼
  ┌──────────────┐
  │ filled       │
  │ (完全成交)   │
  └──────────────┘
```

### 8.3 订单事件记录

每次订单状态变更都会记录事件：

```rust
#[derive(Debug, Clone)]
pub struct OrderEvent {
    pub id: i64,
    pub order_id: String,
    pub event_type: String,      // created, submitted, filled, cancelled...
    pub old_status: Option<String>,
    pub new_status: String,
    pub filled_quantity: Option<Decimal>,
    pub filled_amount: Option<Decimal>,
    pub price: Option<Decimal>,
    pub reason: Option<String>,
    pub created_at: i64,
}
```

**事件用途**:
1. **可追溯**: 完整还原订单生命周期
2. **对账**: 与成交记录交叉验证
3. **异步通知**: 消费者订阅做后续处理
4. **问题排查**: 快速定位异常

### 8.4 与 Matching Engine 通信

```
┌─────────────────────────────────────────────────────────────────┐
│                    订单处理流程                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Order Service                  Matching Engine                │
│       │                                │                        │
│       │  1. CreateOrder (gRPC)         │                        │
│       │───────────────────────────────▶│                        │
│       │                                │                        │
│       │  2. 返回订单结果                │                        │
│       │◀───────────────────────────────│                        │
│       │                                │                        │
│       │  3. 发布 order_created 事件     │                        │
│       │       (到 Kafka)               │                        │
│       │                                │                        │
│       │                                │ 4. 撮合                 │
│       │                                │    (内存操作)           │
│       │                                │                        │
│       │  5. 发布 order_filled 事件      │                        │
│       │       (到 Kafka)               │                        │
│       │                                │                        │
│       │  6. 发布 trade_executed 事件    │                        │
│       │       (到 Kafka)               │                        │
│       │                                │                        │
└─────────────────────────────────────────────────────────────────┘

各服务消费 Kafka 事件:
- Order Service: 更新订单状态 + 记录 order_events
- Trade Service: 写入 trades + 记录 trade_events
- Position Service: 更新持仓
- Account Service: 更新余额
- Market Data Service: 更新行情
```

---

## 十、开发状态总结

| 模块 | Proto 定义 | Server 实现 | Tests | 文档 |
|------|-----------|-------------|-------|------|
| User Service | ✅ | ✅ | - | - |
| Prediction Market Service | ✅ | ✅ | ✅ | - |
| Market Data Service | ✅ | ✅ | ✅ | - |
| Order Service | ✅ | ✅ | - | - |
| Trade Service | - | - | - | - |
| Reconciliation Service | - | - | - | - |
| Matching Engine | - | ✅ (Core) | ✅ | - |
| API Gateway | - | - | - | - |
| Wallet Service | - | - | - | - |
| Position Service | - | - | - | - |
| Risk Service | - | - | - | - |

---

## 十一、下一步工作

1. **Order Service**: 完善编译，加入订单事件表设计
2. **Trade Service**: 实现成交记录服务
3. **Reconciliation Service**: 实现对账服务
4. **Matching Engine**: 集成 Kafka 事件发布
5. **API Gateway**: 实现 HTTP -> gRPC 转换
5. **集成测试**: 端到端交易流程测试

---

## 附录 A: 服务设计文档

详细服务设计文档见 `docs/services/` 目录：

| 服务 | 文档 | 状态 |
|------|------|------|
| User Service | `USER_SERVICE.md` | ✅ 已完成 |
| Account Service | `ACCOUNT_SERVICE.md` | ✅ 已完成 |
| Order Service | `ORDER_SERVICE.md` | ✅ 已完成 |
| Prediction Market Service | `PREDICTION_MARKET_SERVICE.md` | ✅ 已完成 |
| Matching Engine | `MATCHING_ENGINE.md` | ✅ 已完成 |
| Position Service | `POSITION_SERVICE.md` | ✅ 已完成 |
| Clearing Service | `CLEARING_SERVICE.md` | ✅ 已完成 |
| Market Data Service | `MARKET_DATA_SERVICE.md` | ✅ 已完成 |
| Trade Service | `TRADE_SERVICE.md` | ✅ 已完成 |
| Ledger Service | `LEDGER_SERVICE.md` | ✅ 已完成 |
| Risk Service | `RISK_SERVICE.md` | ✅ 已完成 |
| Wallet Service | `WALLET_SERVICE.md` | ✅ 已完成 |
| Reconciliation Service | `RECONCILIATION_SERVICE.md` | ✅ 已完成 |
| ws-market-data | `WS_MARKET_DATA.md` | ✅ 已完成 |
| ws-order | `WS_ORDER.md` | ✅ 已完成 |
| ws-prediction | `WS_PREDICTION.md` | ✅ 已完成 |
| API Gateway | `API_GATEWAY.md` | ✅ 已完成 |
| Admin Service | `ADMIN_SERVICE.md` | ✅ 已完成 |

---

## 附录 B: SQL 表结构汇总

### A.1 Order Service 表

```sql
-- 订单表
CREATE TABLE orders (
    id TEXT PRIMARY KEY,                    -- 时间序列订单号
    user_id BIGINT NOT NULL,
    market_id BIGINT NOT NULL,
    outcome_id BIGINT NOT NULL,
    side VARCHAR(10) NOT NULL,              -- buy, sell
    order_type VARCHAR(20) NOT NULL,        -- limit, market, ioc, fok, post_only
    price TEXT NOT NULL,
    quantity TEXT NOT NULL,
    filled_quantity TEXT NOT NULL DEFAULT '0',
    filled_amount TEXT NOT NULL DEFAULT '0',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    client_order_id TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_market_id ON orders(market_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);

-- 订单事件表
CREATE TABLE order_events (
    id BIGSERIAL PRIMARY KEY,
    order_id TEXT NOT NULL,
    event_type VARCHAR(30) NOT NULL,
    old_status VARCHAR(20),
    new_status VARCHAR(20) NOT NULL,
    filled_quantity TEXT,
    filled_amount TEXT,
    price TEXT,
    reason TEXT,
    created_at BIGINT NOT NULL
);

CREATE INDEX idx_order_events_order_id ON order_events(order_id);
CREATE INDEX idx_order_events_created_at ON order_events(created_at);
```

### A.2 Trade Service 表

```sql
-- 成交记录表
CREATE TABLE trades (
    id BIGSERIAL PRIMARY KEY,
    trade_id TEXT NOT NULL UNIQUE,
    order_id TEXT NOT NULL,
    counter_order_id TEXT,
    market_id BIGINT NOT NULL,
    outcome_id BIGINT NOT NULL,
    maker_user_id BIGINT NOT NULL,
    taker_user_id BIGINT NOT NULL,
    side VARCHAR(10) NOT NULL,
    price TEXT NOT NULL,
    quantity TEXT NOT NULL,
    amount TEXT NOT NULL,
    maker_fee TEXT NOT NULL,
    taker_fee TEXT NOT NULL,
    fee_token VARCHAR(20),
    created_at BIGINT NOT NULL
);

CREATE INDEX idx_trades_market_id ON trades(market_id);
CREATE INDEX idx_trades_maker_user_id ON trades(maker_user_id);
CREATE INDEX idx_trades_taker_user_id ON trades(taker_user_id);
CREATE INDEX idx_trades_created_at ON trades(created_at);

-- 成交事件表
CREATE TABLE trade_events (
    id BIGSERIAL PRIMARY KEY,
    trade_id TEXT NOT NULL,
    event_type VARCHAR(30) NOT NULL,
    old_status VARCHAR(20),
    new_status VARCHAR(20) NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX idx_trade_events_trade_id ON trade_events(trade_id);
CREATE INDEX idx_trade_events_created_at ON trade_events(created_at);
```

### 7.7 Reconciliation Service (规划)

职责: 数据一致性对账

**对账类型**:

| 类型 | 频率 | 内容 |
|------|------|------|
| 实时对账 | 事件级别 | 每次事件处理后验证 |
| 定时对账 | 每分钟 | 扫描未处理事件 |
| 日终对账 | 每天 | 全量数据一致性检查 |
| 手动对账 | 按需 | 特定订单/用户 |

**对账内容**:

1. **订单-成交对账**
   - 订单状态为 filled/partially_filled，必须有对应 trade 记录
   - filled_quantity/filled_amount 与 trade 交叉验证

2. **持仓对账**
   - 用户持仓 = 历史买入 - 历史卖出
   - 与 trade 记录交叉验证

3. **余额对账**
   - 账户余额 = 初始 + 入金 - 出金 + 手续费补贴
   - 与持仓市值对比

4. **手续费对账**
   - sum(maker_fee + taker_fee) = 手续费总收入

**接口设计**:

```protobuf
service ReconciliationService {
    // 触发对账
    rpc RunReconciliation(RunReconciliationRequest) returns (RunReconciliationResponse);

    // 查询对账任务
    rpc GetTask(GetTaskRequest) returns (GetTaskResponse);
    rpc ListTasks(ListTasksRequest) returns (ListTasksResponse);

    // 查询差异
    rpc GetDiffs(GetDiffsRequest) returns (GetDiffsResponse);

    // 修复差异
    rpc FixDiff(FixDiffRequest) returns (FixDiffResponse);

    // 健康检查
    rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
}
```

**表结构**:

```sql
-- 对账任务记录
CREATE TABLE reconciliation_tasks (
    id BIGSERIAL PRIMARY KEY,
    task_type VARCHAR(30) NOT NULL,
    status VARCHAR(20) NOT NULL,
    start_time BIGINT NOT NULL,
    end_time BIGINT,
    total_count INTEGER DEFAULT 0,
    consistent_count INTEGER DEFAULT 0,
    inconsistent_count INTEGER DEFAULT 0,
    missing_count INTEGER DEFAULT 0,
    created_at BIGINT NOT NULL
);

-- 对账差异记录
CREATE TABLE reconciliation_diffs (
    id BIGSERIAL PRIMARY KEY,
    task_id BIGINT NOT NULL,
    diff_type VARCHAR(30) NOT NULL,
    entity_type VARCHAR(20),
    entity_id TEXT,
    expected_value TEXT,
    actual_value TEXT,
    description TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    resolved_at BIGINT,
    resolution TEXT,
    created_at BIGINT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES reconciliation_tasks(id)
);

CREATE INDEX idx_diffs_task_id ON reconciliation_diffs(task_id);
CREATE INDEX idx_diffs_resolved ON reconciliation_diffs(resolved);
```

---

## 附录 B: Kafka 事件 Schema

### B.1 OrderEvent

```protobuf
message OrderEvent {
    string event_id = 1;
    string order_id = 2;
    string event_type = 3;
    string old_status = 4;
    string new_status = 5;
    string filled_quantity = 6;
    string filled_amount = 7;
    string price = 8;
    string reason = 9;
    int64 timestamp = 10;
}
```

### B.2 TradeExecutedEvent

```protobuf
message TradeExecutedEvent {
    string event_id = 1;
    string trade_id = 2;
    string order_id = 3;
    string counter_order_id = 4;
    int64 market_id = 5;
    int64 outcome_id = 6;
    int64 maker_user_id = 7;
    int64 taker_user_id = 8;
    string side = 9;
    string price = 10;
    string quantity = 11;
    string amount = 12;
    string maker_fee = 13;
    string taker_fee = 14;
    string fee_token = 15;
    int64 timestamp = 16;
}
```