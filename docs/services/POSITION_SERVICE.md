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
| ⚠️ 盈亏计算 | 仅记录持仓，实时盈亏需外部提供价格 |
| 平仓 | 平仓后清空持仓 |

### 1.1.2 功能列表

```
Position Service
├── 持仓查询
│   ├── GetPosition - 查询单笔持仓
│   ├── GetUserPositions - 查询用户所有持仓
│   └── GetMarketPositions - 查询市场所有持仓
├── 持仓更新
│   ├── UpdatePosition - 更新持仓 (成交后)
│   └── ClosePosition - 平仓
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
    rpc GetPosition(GetPositionRequest) returns (GetPositionResponse);
    rpc GetUserPositions(GetUserPositionsRequest) returns (GetUserPositionsResponse);
    rpc UpdatePosition(UpdatePositionRequest) returns (UpdatePositionResponse);
}

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
    string quantity = 5;
    string avg_price = 6;
    string unrealized_pnl = 7;
    string current_price = 8;
    int64 updated_at = 9;
}

message GetUserPositionsRequest {
    int64 user_id = 1;
    int64 market_id = 2;
}

message GetUserPositionsResponse {
    repeated PositionSummary positions = 1;
}

message PositionSummary {
    int64 market_id = 1;
    int64 outcome_id = 2;
    string quantity = 3;
    string avg_price = 4;
    string unrealized_pnl = 5;
}

message UpdatePositionRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    int64 outcome_id = 3;
    string quantity_change = 4;
    string new_avg_price = 5;
    string trade_id = 6;
}

message UpdatePositionResponse {
    bool success = 1;
    string message = 2;
}
```

## 4. 服务间通信

### 4.1 gRPC 调用

| 被调用方 | 接口 | 场景 |
|----------|------|------|
| Market Data Service (50006) | GetMarketPrice | 获取当前价格计算盈亏 |
| Trade Service (50013) | GetTrades | 查询成交记录 |

### 4.2 gRPC 被调用

| 调用方 | 接口 | 场景 |
|----------|------|------|
| Clearing Service (50008) | GetUserPositions | 结算时获取用户持仓 |
| API Gateway | GetPosition | 用户查询持仓 |

### 4.3 Kafka 消费

| Topic | 处理 |
|-------|------|
| trade_executed | 更新持仓 |

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
└─────────────────┬───────────────────────┘
                  │
                  ▼
返回
```

### 5.2 盈亏计算公式

```
未实现盈亏 (Unrealized PnL) = 当前价值 - 持仓成本

Case 1: Buy (做多)
  成本 = quantity * avg_price
  当前价值 = quantity * current_price
  PnL = 当前价值 - 成本

Case 2: Sell (做空) - 预测市场场景
  成本 = quantity * (1 - avg_price)  // 买入花了 (1-价格)
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
```

## 6. 目录结构

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
