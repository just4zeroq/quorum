# Market Data Service - 行情服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50006 |
| 协议 | gRPC |
| 数据库 | 共享 Prediction Market DB |
| 依赖 | Prediction Market Service (50010) |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 行情查询 | 市场/选项/价格查询 |
| K线数据 | K线生成与查询 |
| 成交记录 | 历史成交查询 |
| 24h 统计 | 24小时统计 |
| 订单簿/深度 | 买卖盘深度数据 |
| 市场分类 | 分类列表查询 |

## 2. 数据模型

### 2.1 数据库表

**market_klines 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| market_id | BIGINT | NOT NULL | 市场ID |
| outcome_id | BIGINT | NOT NULL | 选项ID |
| interval | VARCHAR(10) | NOT NULL | 周期 |
| open | TEXT | NOT NULL | 开盘价 |
| high | TEXT | NOT NULL | 最高价 |
| low | TEXT | NOT NULL | 最低价 |
| close | TEXT | NOT NULL | 收盘价 |
| volume | TEXT | NOT NULL | 成交量 |
| quote_volume | TEXT | NOT NULL | 成交额 |
| timestamp | BIGINT | NOT NULL | 时间戳 |

**索引**:
- `PRIMARY KEY (id)`
- `UNIQUE INDEX idx_market_outcome_interval_ts (market_id, outcome_id, interval, timestamp)`
- `INDEX idx_timestamp (timestamp)`

**market_trades 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| market_id | BIGINT | NOT NULL | 市场ID |
| outcome_id | BIGINT | NOT NULL | 选项ID |
| user_id | BIGINT | NOT NULL | 用户ID |
| side | VARCHAR(10) | NOT NULL | 方向 |
| price | TEXT | NOT NULL | 成交价格 |
| quantity | TEXT | NOT NULL | 成交数量 |
| amount | TEXT | NOT NULL | 成交金额 |
| fee | TEXT | NOT NULL | 手续费 |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `PRIMARY KEY (id)`
- `INDEX idx_market_outcome (market_id, outcome_id)`
- `INDEX idx_created_at (created_at)`

## 3. Proto 接口

```protobuf
syntax = "proto3";

package market_data;

service MarketDataService {
    // 市场查询
    rpc GetMarkets(GetMarketsRequest) returns (GetMarketsResponse);
    rpc GetMarketDetail(GetMarketDetailRequest) returns (GetMarketDetailResponse);
    rpc GetCategories(GetCategoriesRequest) returns (GetCategoriesResponse);

    // 选项价格
    rpc GetOutcomePrices(GetOutcomePricesRequest) returns (GetOutcomePricesResponse);

    // K线
    rpc GetKlines(GetKlinesRequest) returns (GetKlinesResponse);

    // 成交记录
    rpc GetRecentTrades(GetRecentTradesRequest) returns (GetRecentTradesResponse);

    // 订单簿深度
    rpc GetOrderBookDepth(GetOrderBookDepthRequest) returns (GetOrderBookDepthResponse);

    // 24h统计
    rpc Get24hStats(Get24hStatsRequest) returns (Get24hStatsResponse);
}

// ==================== 市场查询 ====================

message GetMarketsRequest {
    string category = 1;
    string status = 2;
    int32 page = 3;
    int32 page_size = 4;
}

message GetMarketsResponse {
    repeated MarketSummary markets = 1;
    int64 total = 2;
}

message MarketSummary {
    int64 market_id = 1;
    string question = 2;
    string category = 3;
    string status = 4;
    string total_volume = 5;
    int32 outcome_count = 6;          // 结果选项数量
    int64 end_time = 7;
}

message GetMarketDetailRequest {
    int64 market_id = 1;
}

message GetMarketDetailResponse {
    int64 market_id = 1;
    string question = 2;
    string description = 3;
    string category = 4;
    string status = 5;
    string total_volume = 6;
    repeated OutcomeDetail outcomes = 7;
}

message OutcomeDetail {
    int64 outcome_id = 1;
    string outcome_asset = 2;         // 结果代币标识 (如 "12345_yes")
    string name = 3;
    string price = 4;
    string volume = 5;
    string probability = 6;
}

message GetCategoriesRequest {}

message GetCategoriesResponse {
    repeated CategoryInfo categories = 1;
}

message CategoryInfo {
    string name = 1;
    int64 market_count = 2;
}

// ==================== 选项价格 ====================

message GetOutcomePricesRequest {
    int64 market_id = 1;
}

message GetOutcomePricesResponse {
    repeated OutcomePrice prices = 1;
}

message OutcomePrice {
    int64 outcome_id = 1;
    string outcome_asset = 2;         // 结果代币标识
    string price = 3;
    string volume = 4;
}

// ==================== K线 ====================

message GetKlinesRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    string interval = 3;
    int64 start_time = 4;
    int64 end_time = 5;
    int32 limit = 6;
}

message GetKlinesResponse {
    repeated Kline klines = 1;
}

message Kline {
    int64 timestamp = 1;
    string open = 2;
    string high = 3;
    string low = 4;
    string close = 5;
    string volume = 6;
    string quote_volume = 7;
}

// ==================== 成交记录 ====================

message GetRecentTradesRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    int32 limit = 3;
}

message GetRecentTradesResponse {
    repeated TradeSummary trades = 1;
}

message TradeSummary {
    int64 trade_id = 1;
    string side = 2;
    string price = 3;
    string quantity = 4;
    string amount = 5;
    int64 timestamp = 6;
}

// ==================== 订单簿深度 ====================

message GetOrderBookDepthRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    int32 limit = 3;                  // 每档深度条数，默认 20
}

message GetOrderBookDepthResponse {
    repeated PriceLevel bids = 1;     // 买盘 (价格从高到低)
    repeated PriceLevel asks = 2;     // 卖盘 (价格从低到高)
}

message PriceLevel {
    string price = 1;
    string quantity = 2;
    int32 order_count = 3;            // 该价位订单数
}

// ==================== 24h统计 ====================

message Get24hStatsRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;             // 可选，指定结果选项
}

message Get24hStatsResponse {
    int64 market_id = 1;
    int64 outcome_id = 2;
    string volume_24h = 3;
    string amount_24h = 4;
    string high_24h = 5;
    string low_24h = 6;
    string price_change = 7;
    string price_change_percent = 8;
}
```

## 4. Kafka 消费

| Topic | 处理 |
|-------|------|
| trade_executed | 更新成交记录、K线、24h统计 |
| order_book_updates | 更新订单簿深度缓存 |
| market_events | 更新市场状态 |

## 5. 配置

```yaml
service:
  port: 50006
database:
  driver: "sqlite"
  url: "sqlite:./data/market_data.db"  # 共享 Prediction Market DB
kafka:
  brokers:
    - "localhost:9092"
  topics:
    trade_executed: "trade_executed"
    order_book_updates: "order_book_updates"
    market_events: "market_events"
```

## 6. 目录结构

```
crates/market-data-service/
├── Cargo.toml
├── build.rs
├── config/
│   └── market_data_service.yaml
└── src/
    ├── lib.rs, main.rs, server.rs, error.rs
    ├── pb.rs, pb/
    ├── repository/, services/, clients/
```
