# Prediction Market Service - 预测市场服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50010 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL (主数据库) |
| 依赖 | Matching Engine (50009), Clearing Service (50008) |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 市场管理 | 创建、更新、关闭预测市场 |
| 选项管理 | 添加、更新市场选项 |
| 市场结算 | 设定获胜选项、计算派彩 |
| 市场查询 | 市场列表、市场详情 |
| 初始化撮合 | 创建市场时初始化 Matching Engine 订单簿 |

### 1.1.2 功能列表

```
Prediction Market Service
├── 市场管理
│   ├── CreateMarket - 创建市场
│   ├── UpdateMarket - 更新市场
│   ├── CloseMarket - 关闭市场
│   ├── GetMarket - 查询市场
│   └── ListMarkets - 市场列表
├── 选项管理
│   ├── AddOutcome - 添加选项
│   ├── UpdateOutcome - 更新选项
│   └── GetOutcomes - 获取选项列表
├── 结算
│   ├── ResolveMarket - 结算市场
│   ├── CalculatePayout - 计算派彩
│   └── GetResolutions - 结算记录
└── 内部服务
    └── CreateMarketOnEngine - 在撮合引擎创建市场
```

## 2. 数据模型

### 2.1 Domain Model

已在 `crates/domain/src/prediction_market/model/mod.rs` 定义：

```rust
/// 市场状态
pub enum MarketStatus {
    Open,
    Resolved,
    Cancelled,
}

/// 预测市场
pub struct PredictionMarket {
    pub id: i64,
    pub question: String,
    pub description: Option<String>,
    pub category: String,
    pub image_url: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub status: MarketStatus,
    pub resolved_outcome_id: Option<i64>,
    pub resolved_at: Option<i64>,
    pub total_volume: Decimal,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 市场选项
pub struct MarketOutcome {
    pub id: i64,
    pub market_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub price: Decimal,
    pub volume: Decimal,
    pub probability: Decimal,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 结算记录
pub struct Resolution {
    pub id: i64,
    pub market_id: i64,
    pub outcome_id: i64,
    pub total_payout: Decimal,
    pub winning_quantity: Decimal,
    pub payout_ratio: Decimal,
    pub resolved_at: i64,
}
```

### 2.2 Domain Event

```rust
// domain/src/prediction_market/event/mod.rs

pub enum PredictionMarketEvent {
    MarketCreated {
        market_id: i64,
        question: String,
    },
    MarketClosed {
        market_id: i64,
    },
    MarketResolved {
        market_id: i64,
        outcome_id: i64,
    },
    OutcomeAdded {
        market_id: i64,
        outcome_id: i64,
        name: String,
    },
}
```

### 2.3 数据库表

**prediction_markets 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | 市场ID |
| question | TEXT | NOT NULL | 事件问题 |
| description | TEXT | | 描述 |
| category | VARCHAR(50) | NOT NULL | 分类 |
| image_url | TEXT | | 图片URL |
| start_time | BIGINT | NOT NULL | 开始时间 |
| end_time | BIGINT | NOT NULL | 结束时间 |
| status | VARCHAR(20) | NOT NULL DEFAULT 'open' | 状态 |
| resolved_outcome_id | BIGINT | | 结算选项ID |
| resolved_at | BIGINT | | 结算时间 |
| total_volume | TEXT | NOT NULL DEFAULT '0' | 总成交量 |
| created_at | BIGINT | NOT NULL | 创建时间 |
| updated_at | BIGINT | NOT NULL | 更新时间 |

**索引**:
- `PRIMARY KEY (id)`
- `INDEX idx_category (category)`
- `INDEX idx_status (status)`
- `INDEX idx_end_time (end_time)`

**market_outcomes 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | 选项ID |
| market_id | BIGINT | NOT NULL, FK | 市场ID |
| name | VARCHAR(100) | NOT NULL | 选项名称 |
| description | TEXT | | 描述 |
| image_url | TEXT | | 图片URL |
| price | TEXT | NOT NULL DEFAULT '0.5' | 当前价格 |
| volume | TEXT | NOT NULL DEFAULT '0' | 成交量 |
| probability | TEXT | NOT NULL DEFAULT '0' | 计算概率 |
| created_at | BIGINT | NOT NULL | 创建时间 |
| updated_at | BIGINT | NOT NULL | 更新时间 |

**索引**:
- `PRIMARY KEY (id)`
- `INDEX idx_market_id (market_id)`

**market_resolutions 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| market_id | BIGINT | UNIQUE, NOT NULL | 市场ID |
| outcome_id | BIGINT | NOT NULL | 获胜选项ID |
| total_payout | TEXT | NOT NULL | 总派彩 |
| winning_quantity | TEXT | NOT NULL | 获胜总量 |
| payout_ratio | TEXT | NOT NULL | 派彩比例 |
| resolved_at | BIGINT | NOT NULL | 结算时间 |

**索引**:
- `PRIMARY KEY (id)`
- `UNIQUE INDEX idx_market_id (market_id)`

## 3. Proto 接口定义

### 3.1 服务定义

```protobuf
syntax = "proto3";

package prediction_market;

service PredictionMarketService {
    // 市场管理
    rpc CreateMarket(CreateMarketRequest) returns (CreateMarketResponse);
    rpc UpdateMarket(UpdateMarketRequest) returns (UpdateMarketResponse);
    rpc CloseMarket(CloseMarketRequest) returns (CloseMarketResponse);
    rpc GetMarket(GetMarketRequest) returns (GetMarketResponse);
    rpc ListMarkets(ListMarketsRequest) returns (ListMarketsResponse);

    // 选项管理
    rpc AddOutcome(AddOutcomeRequest) returns (AddOutcomeResponse);
    rpc UpdateOutcome(UpdateOutcomeRequest) returns (UpdateOutcomeResponse);
    rpc GetOutcomes(GetOutcomesRequest) returns (GetOutcomesResponse);

    // 结算
    rpc ResolveMarket(ResolveMarketRequest) returns (ResolveMarketResponse);
    rpc CalculatePayout(CalculatePayoutRequest) returns (CalculatePayoutResponse);
    rpc GetResolutions(GetResolutionsRequest) returns (GetResolutionsResponse);
}
```

### 3.2 消息定义

```protobuf
// ==================== 市场管理 ====================

message CreateMarketRequest {
    string question = 1;
    string description = 2;
    string category = 3;
    string image_url = 4;
    int64 start_time = 5;
    int64 end_time = 6;
    repeated CreateOutcomeRequest outcomes = 7;
}

message CreateMarketResponse {
    bool success = 1;
    string message = 2;
    int64 market_id = 3;
}

message UpdateMarketRequest {
    int64 market_id = 1;
    string question = 2;
    string description = 3;
    string image_url = 4;
}

message UpdateMarketResponse {
    bool success = 1;
    string message = 2;
    int64 market_id = 3;
}

message CloseMarketRequest {
    int64 market_id = 1;
}

message CloseMarketResponse {
    bool success = 1;
    string message = 2;
}

message GetMarketRequest {
    int64 market_id = 1;
}

message GetMarketResponse {
    int64 market_id = 1;
    string question = 2;
    string description = 3;
    string category = 4;
    string image_url = 5;
    int64 start_time = 6;
    int64 end_time = 7;
    string status = 8;
    int64 resolved_outcome_id = 9;
    int64 resolved_at = 10;
    string total_volume = 11;
    repeated OutcomeSummary outcomes = 12;
    int64 created_at = 13;
    int64 updated_at = 14;
}

message ListMarketsRequest {
    string category = 1;
    string status = 2;
    int32 page = 3;
    int32 page_size = 4;
}

message ListMarketsResponse {
    repeated MarketSummary markets = 1;
    int64 total = 2;
    int32 page = 3;
    int32 page_size = 4;
}

message MarketSummary {
    int64 market_id = 1;
    string question = 2;
    string category = 3;
    string status = 4;
    int64 end_time = 5;
    string total_volume = 6;
}

// ==================== 选项管理 ====================

message CreateOutcomeRequest {
    string name = 1;
    string description = 2;
    string image_url = 3;
}

message AddOutcomeRequest {
    int64 market_id = 1;
    string name = 2;
    string description = 3;
    string image_url = 4;
}

message AddOutcomeResponse {
    bool success = 1;
    string message = 2;
    int64 outcome_id = 3;
}

message UpdateOutcomeRequest {
    int64 outcome_id = 1;
    string name = 2;
    string description = 3;
    string image_url = 4;
}

message UpdateOutcomeResponse {
    bool success = 1;
    string message = 2;
}

message GetOutcomesRequest {
    int64 market_id = 1;
}

message GetOutcomesResponse {
    repeated OutcomeSummary outcomes = 1;
}

message OutcomeSummary {
    int64 outcome_id = 1;
    int64 market_id = 2;
    string name = 3;
    string description = 4;
    string price = 5;
    string volume = 6;
    string probability = 7;
}

// ==================== 结算 ====================

message ResolveMarketRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
}

message ResolveMarketResponse {
    bool success = 1;
    string message = 2;
    int64 market_id = 3;
    int64 outcome_id = 4;
    string total_payout = 5;
    string payout_ratio = 6;
}

message CalculatePayoutRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    int64 user_id = 3;
}

message CalculatePayoutResponse {
    int64 user_id = 1;
    int64 market_id = 2;
    int64 outcome_id = 3;
    string quantity = 4;
    string avg_price = 5;
    string payout = 6;
}

message GetResolutionsRequest {
    int64 market_id = 1;
}

message GetResolutionsResponse {
    int64 market_id = 1;
    int64 outcome_id = 2;
    string total_payout = 3;
    string winning_quantity = 4;
    string payout_ratio = 5;
    int64 resolved_at = 6;
}
```

## 4. Kafka 事件

### 4.1 生产的事件

| 事件 | Topic | 说明 |
|------|-------|------|
| MarketCreated | market_events | 市场创建 |
| MarketClosed | market_events | 市场关闭 |
| MarketResolved | market_events | 市场结算 |
| OutcomeAdded | market_events | 选项添加 |

### 4.2 事件 Schema

```rust
pub enum PredictionMarketEvent {
    MarketCreated {
        market_id: i64,
        question: String,
    },
    MarketClosed {
        market_id: i64,
    },
    MarketResolved {
        market_id: i64,
        outcome_id: i64,
    },
    OutcomeAdded {
        market_id: i64,
        outcome_id: i64,
        name: String,
    },
}
```

## 5. 服务间通信

### 5.1 gRPC 调用

| 被调用方 | 接口 | 场景 |
|----------|------|------|
| Matching Engine (50009) | CreateMarket | 创建市场时初始化订单簿 |
| Matching Engine (50009) | CloseMarket | 关闭市场 |
| Clearing Service (50008) | SettleMarket | 结算时执行派彩 |

### 5.2 gRPC 被调用

| 调用方 | 接口 | 场景 |
|----------|------|------|
| API Gateway | CreateMarket | 创建市场 |
| API Gateway | GetMarket | 查询市场 |
| API Gateway | ListMarkets | 市场列表 |
| Market Data Service | GetMarket | 行情数据同步 |

### 5.3 Kafka 消费

无

## 6. 核心流程

### 6.1 创建市场流程

```
CreateMarket 请求
    │
    ▼
┌─────────────────────────────────────────┐
│ 1. 参数校验                              │
│    - question 不为空                     │
│    - end_time > start_time              │
│    - outcomes 至少2个                    │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 2. 保存市场到数据库                       │
│    - status = Open                      │
│    - 生成 market_id                      │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 3. 保存选项到数据库                       │
│    - 每个 outcome 生成 outcome_id         │
│    - 初始价格 = 0.5                     │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 4. 调用 Matching Engine 创建订单簿         │
│    - CreateMarket                       │
│    - 传入 market_id 和 outcomes          │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 5. 发布 Kafka 事件                       │
│    - MarketCreated                     │
└─────────────────┬───────────────────────┘
                  │
                  ▼
返回 CreateMarketResponse
```

### 6.2 结算市场流程

```
ResolveMarket 请求
    │
    ▼
┌─────────────────────────────────────────┐
│ 1. 查询市场                              │
│    - market_id → PredictionMarket        │
│    - 状态必须为 Open                    │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 2. 验证选项                              │
│    - outcome_id 属于该市场               │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 3. 计算派彩 (⚠️ 修正计算逻辑)             │
│    - 统计所有选项的 volume               │
│    - 正确逻辑:                           │
│      输家资金 = sum(输家持仓 * 买入价格)  │
│      赢家总收益 = 输家资金 * (1 - 手续费) │
│      每用户派彩 = 赢家持仓 / 赢家总持仓 * │
│                   赢家总收益              │
│    ❌ 错误逻辑: payout_ratio = 1/(1+sum) │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 4. 调用 Clearing Service 执行派彩        │
│    - SettleMarket (同步)                │
│    ⚠️ 必须调用，否则资金不落地            │
│    - 传递: market_id, winning_outcome_id │
│    - Clearing Service 完成实际资金分配   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 5. 更新市场状态                          │
│    - status = Resolved                  │
│    - resolved_outcome_id                │
│    - resolved_at = now                  │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 6. 保存结算记录                          │
│    - market_resolutions 表              │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 7. 调用 Matching Engine 关闭订单簿         │
│    - CloseMarket                       │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 8. 发布 Kafka 事件                       │
│    - MarketResolved                   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
返回 ResolveMarketResponse
```

## 7. 配置

```yaml
# crates/prediction-market-service/config/prediction_market_service.yaml
service:
  host: "0.0.0.0"
  port: 50010

database:
  driver: "sqlite"
  url: "sqlite:./data/prediction_markets.db"
  max_connections: 20

matching_engine:
  addr: "localhost:50009"
  timeout_ms: 5000

kafka:
  brokers:
    - "localhost:9092"
  topics:
    market_events: "market_events"

market:
  # 市场有效期 (毫秒)
  default_duration_ms: 2592000000  # 30天
  # 最小选项数
  min_outcomes: 2
  # 最大选项数
  max_outcomes: 10
```

## 8. 错误码

| 错误码 | 说明 |
|--------|------|
| MARKET_NOT_FOUND | 市场不存在 |
| MARKET_ALREADY_EXISTS | 市场已存在 |
| MARKET_NOT_OPEN | 市场未开放 |
| MARKET_ALREADY_RESOLVED | 市场已结算 |
| MARKET_ALREADY_CLOSED | 市场已关闭 |
| OUTCOME_NOT_FOUND | 选项不存在 |
| OUTCOME_ALREADY_EXISTS | 选项已存在 |
| INVALID_TIME_RANGE | 无效时间范围 |
| TOO_FEW_OUTCOMES | 选项数不足 |
| TOO_MANY_OUTCOMES | 选项数过多 |
| RESOLUTION_FAILED | 结算失败 |

## 9. 目录结构

```
crates/prediction-market-service/
├── Cargo.toml
├── build.rs
├── config/
│   └── prediction_market_service.yaml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── config.rs
    ├── server.rs
    ├── error.rs
    ├── pb.rs
    ├── pb/
    │   ├── prediction_market.proto
    │   └── prediction_market.rs
    ├── repository/
    │   ├── mod.rs
    │   ├── market_repo.rs
    │   ├── outcome_repo.rs
    │   └── resolution_repo.rs
    ├── services/
    │   ├── mod.rs
    │   └── prediction_market_service_impl.rs
    ├── clients/
    │   ├── mod.rs
    │   └── matching_engine_client.rs
    └── utils/
        └── mod.rs
```
