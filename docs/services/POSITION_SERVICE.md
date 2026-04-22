# Position Service - 持仓服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50005 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | Trade Service (50013), Account Service (50019), Market Data Service (50006) |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 持仓管理 | 用户持仓查询 |
| 持仓更新 | 成交后更新持仓 |
| 盈亏计算 | 仅记录持仓，实时盈亏需外部提供价格 |
| 平仓 | 平仓后清空持仓 |
| 结算清零 | 市场结算时清零失败选项持仓 |

### 1.1.2 功能列表

```
Position Service
├── 持仓查询
│   ├── GetPosition - 查询单笔持仓
│   ├── GetUserPositions - 查询用户所有持仓
│   └── GetMarketPositions - 查询市场所有持仓 (结算用)
├── 持仓更新
│   ├── UpdatePosition - 更新持仓 (成交后)
│   └── SettlePosition - 结算持仓 (结算后清零)
├── 盈亏计算
│   ├── CalculateUnrealizedPnL - 计算未实现盈亏
│   │   ⚠️ 需要外部传入 current_price
│   └── GetPositionWithPnL - 获取持仓含盈亏
└── 持仓统计
    ├── GetTotalVolume - 获取总成交量
    └── GetUserTotalVolume - 获取用户总成交量
```

> ⚠️ **注意**: Position Service 只记录持仓数据（quantity, avg_price），不存储实时价格。计算未实现盈亏时需要调用方传入当前价格（current_price）。

## 2. 数据模型

### 2.1 数据库表

**user_positions 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| user_id | BIGINT | NOT NULL | 用户ID |
| market_id | BIGINT | NOT NULL | 市场ID |
| outcome_id | BIGINT | NOT NULL | 选项ID |
| outcome_asset | VARCHAR(32) | NOT NULL | 结果代币标识 (如 "12345_yes") |
| quantity | TEXT | NOT NULL DEFAULT '0' | 持仓数量 |
| avg_price | TEXT | NOT NULL DEFAULT '0' | 平均价格 |
| created_at | BIGINT | NOT NULL | 创建时间 |
| updated_at | BIGINT | NOT NULL | 更新时间 |

**索引**:
- `PRIMARY KEY (id)`
- `UNIQUE INDEX idx_user_market_outcome (user_id, market_id, outcome_id)`
- `INDEX idx_user_id (user_id)`
- `INDEX idx_market_id (market_id)`

## 3. Proto 接口

```protobuf
syntax = "proto3";

package position;

service PositionService {
    // 持仓查询
    rpc GetPosition(GetPositionRequest) returns (GetPositionResponse);
    rpc GetUserPositions(GetUserPositionsRequest) returns (GetUserPositionsResponse);
    rpc GetMarketPositions(GetMarketPositionsRequest) returns (GetMarketPositionsResponse);

    // 持仓更新
    rpc UpdatePosition(UpdatePositionRequest) returns (UpdatePositionResponse);
    rpc SettlePosition(SettlePositionRequest) returns (SettlePositionResponse);

    // 盈亏计算
    rpc CalculateUnrealizedPnL(CalculateUnrealizedPnLRequest) returns (CalculateUnrealizedPnLResponse);
    rpc GetPositionWithPnL(GetPositionWithPnLRequest) returns (GetPositionWithPnLResponse);

    // 统计
    rpc GetTotalVolume(GetTotalVolumeRequest) returns (GetTotalVolumeResponse);
    rpc GetUserTotalVolume(GetUserTotalVolumeRequest) returns (GetUserTotalVolumeResponse);
}

// ==================== 持仓查询 ====================

message GetPositionRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    int64 outcome_id = 3;
}

message GetPositionResponse {
    int64 position_id = 1;
    int64 user_id = 2;
    int64 market_id = 3;
    int64 outcome_id = 4;
    string outcome_asset = 5;       // 结果代币标识
    string quantity = 6;
    string avg_price = 7;
    int64 updated_at = 8;
}

message GetUserPositionsRequest {
    int64 user_id = 1;
    int64 market_id = 2;            // 可选，按市场过滤
}

message GetUserPositionsResponse {
    repeated PositionSummary positions = 1;
}

message PositionSummary {
    int64 market_id = 1;
    int64 outcome_id = 2;
    string outcome_asset = 3;       // 结果代币标识
    string quantity = 4;
    string avg_price = 5;
}

message GetMarketPositionsRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;           // 可选，按选项过滤
    int32 page = 3;
    int32 page_size = 4;
}

message GetMarketPositionsResponse {
    repeated MarketPositionSummary positions = 1;
    int64 total = 2;
}

message MarketPositionSummary {
    int64 user_id = 1;
    int64 outcome_id = 2;
    string outcome_asset = 3;
    string quantity = 4;
    string avg_price = 5;
}

// ==================== 持仓更新 ====================

message UpdatePositionRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    int64 outcome_id = 3;
    string outcome_asset = 4;       // 结果代币标识
    string quantity_change = 5;     // 正数=增加, 负数=减少
    string new_avg_price = 6;
    string trade_id = 7;
}

message UpdatePositionResponse {
    bool success = 1;
    string message = 2;
}

message SettlePositionRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    int64 outcome_id = 3;
    string outcome_asset = 4;       // 结果代币标识
    bool is_winner = 5;             // 是否获胜选项
    string settled_quantity = 6;    // 结算后清零的数量
    string settlement_id = 7;
}

message SettlePositionResponse {
    bool success = 1;
    string message = 2;
    string quantity_before = 3;
    string quantity_after = 4;      // 应为 "0"
}

// ==================== 盈亏计算 ====================

message CalculateUnrealizedPnLRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    int64 outcome_id = 3;
    string current_price = 4;       // 外部传入当前价格
}

message CalculateUnrealizedPnLResponse {
    string quantity = 1;
    string avg_price = 2;
    string current_price = 3;
    string cost = 4;                // 持仓成本 = quantity * avg_price
    string current_value = 5;       // 当前价值 = quantity * current_price
    string unrealized_pnl = 6;      // 未实现盈亏 = current_value - cost
}

message GetPositionWithPnLRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    int64 outcome_id = 3;
    string current_price = 4;
}

message GetPositionWithPnLResponse {
    int64 market_id = 1;
    int64 outcome_id = 2;
    string outcome_asset = 3;
    string quantity = 4;
    string avg_price = 5;
    string current_price = 6;
    string unrealized_pnl = 7;
}

// ==================== 统计 ====================

message GetTotalVolumeRequest {
    int64 market_id = 1;            // 可选
}

message GetTotalVolumeResponse {
    string total_volume = 1;
    int64 total_trades = 2;
}

message GetUserTotalVolumeRequest {
    int64 user_id = 1;
}

message GetUserTotalVolumeResponse {
    int64 user_id = 1;
    string total_volume = 2;
    int64 total_trades = 3;
}
```

## 4. 服务间通信

### 4.1 gRPC 调用

| 被调用方 | 接口 | 场景 |
|----------|------|------|
| Market Data Service (50006) | GetOutcomePrices | 获取当前价格计算盈亏 |

### 4.2 gRPC 被调用

| 调用方 | 接口 | 场景 |
|----------|------|------|
| Clearing Service (50008) | GetMarketPositions | 结算时获取市场所有用户持仓 |
| Clearing Service (50008) | SettlePosition | 结算时清零持仓 |
| API Gateway | GetPosition | 用户查询持仓 |

### 4.3 Kafka 消费

| Topic | 处理 |
|-------|------|
| trade_executed | 更新持仓 |
| settlement_events | 结算后清零失败选项持仓 |

## 5. 核心流程

### 5.1 持仓更新流程

```
Matching Engine 成交
    │
    ▼ 发布 Kafka: trade_executed
┌─────────────────────────────────────────┐
│ Position Service 消费 trade_executed    │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 1. 查询当前持仓                          │
│    - user_id, market_id, outcome_id     │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 2. 计算新持仓                            │
│    - 新数量 = 旧数量 ± 成交数量          │
│    - 新均价 = 加权平均                   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 3. 保存持仓                             │
│    - 插入或更新                         │
└─────────────────────────────────────────┘
```

### 5.2 盈亏计算公式

```
未实现盈亏 (Unrealized PnL) = 当前价值 - 持仓成本

Case 1: Buy (做多)
  成本 = quantity * avg_price
  当前价值 = quantity * current_price
  PnL = 当前价值 - 成本

Case 2: Sell (做空) - 预测市场场景
  成本 = quantity * (1 - avg_price)
  当前价值 = quantity * (1 - current_price)
  PnL = 当前价值 - 成本
```

## 6. 配置

```yaml
service:
  port: 50005
database:
  driver: "sqlite"
  url: "sqlite:./data/positions.db"
kafka:
  brokers:
    - "localhost:9092"
  topics:
    trade_executed: "trade_executed"
    settlement_events: "settlement_events"
```

## 7. 目录结构

```
crates/position-service/
├── Cargo.toml
├── build.rs
├── config/
│   └── position_service.yaml
└── src/
    ├── lib.rs, main.rs, server.rs, error.rs
    ├── pb.rs, pb/
    ├── repository/, services/, clients/
```
