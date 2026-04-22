# Trade Service - 成交记录服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50013 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | 无 |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 成交记录 | 成交历史查询 |
| 成交统计 | 成交统计 |

## 2. 数据模型

### 2.1 数据库表

**trades 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| trade_id | VARCHAR(64) | UNIQUE, NOT NULL | 成交ID |
| order_id | VARCHAR(64) | NOT NULL | 订单ID |
| counter_order_id | VARCHAR(64) | | 对手订单ID |
| market_id | BIGINT | NOT NULL | 市场ID |
| outcome_id | BIGINT | NOT NULL | 选项ID |
| outcome_asset | VARCHAR(32) | NOT NULL | 结果代币标识 |
| maker_user_id | BIGINT | NOT NULL | Maker用户 |
| taker_user_id | BIGINT | NOT NULL | Taker用户 |
| side | VARCHAR(10) | NOT NULL | Taker方向 |
| price | TEXT | NOT NULL | 成交价格 |
| quantity | TEXT | NOT NULL | 成交数量 |
| amount | TEXT | NOT NULL | 成交金额 |
| maker_fee | TEXT | NOT NULL | Maker手续费 |
| taker_fee | TEXT | NOT NULL | Taker手续费 |
| fee_token | VARCHAR(20) | | 手续费币种 |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `PRIMARY KEY (id)`
- `UNIQUE INDEX idx_trade_id (trade_id)`
- `INDEX idx_market_outcome (market_id, outcome_id)`
- `INDEX idx_maker_user (maker_user_id)`
- `INDEX idx_taker_user (taker_user_id)`
- `INDEX idx_created_at (created_at)`

## 3. Proto 接口

```protobuf
syntax = "proto3";

package trade;

service TradeService {
    rpc GetTrade(GetTradeRequest) returns (GetTradeResponse);
    rpc GetUserTrades(GetUserTradesRequest) returns (GetUserTradesResponse);
    rpc GetMarketTrades(GetMarketTradesRequest) returns (GetMarketTradesResponse);
}

message GetTradeRequest {
    string trade_id = 1;
}

message GetTradeResponse {
    string trade_id = 1;
    string order_id = 2;
    int64 market_id = 3;
    int64 outcome_id = 4;
    string outcome_asset = 5;       // 结果代币标识
    int64 maker_user_id = 6;
    int64 taker_user_id = 7;
    string side = 8;
    string price = 9;
    string quantity = 10;
    string amount = 11;
    string maker_fee = 12;
    string taker_fee = 13;
    int64 timestamp = 14;
}

message GetUserTradesRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    int32 page = 3;
    int32 page_size = 4;
}

message GetUserTradesResponse {
    repeated TradeSummary trades = 1;
    int64 total = 2;
}

message GetMarketTradesRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    int32 limit = 3;
}

message GetMarketTradesResponse {
    repeated TradeSummary trades = 1;
}

message TradeSummary {
    string trade_id = 1;
    string side = 2;
    string price = 3;
    string quantity = 4;
    int64 timestamp = 5;
}
```

## 4. Kafka 消费

| Topic | 处理 |
|-------|------|
| trade_executed | 保存成交记录 |

## 5. 配置

```yaml
service:
  port: 50013
database:
  driver: "sqlite"
  url: "sqlite:./data/trades.db"
kafka:
  brokers:
    - "localhost:9092"
  topics:
    trade_executed: "trade_executed"
```

## 6. 目录结构

```
crates/trade-service/
├── Cargo.toml, build.rs, config/, src/
```
