# Matching Engine - 撮合引擎

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50009 |
| 协议 | gRPC |
| 数据库 | 无 (内存撮合) |
| 依赖 | 无 |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 内存订单簿 | 维护 CLOB 订单簿 |
| 撮合 | 按 Price-Time 优先级撮合 |
| 订单类型 | 支持 Limit/Market/IOC/FOK/PostOnly |
| 持久化 | WAL + 快照 (内存 + 异步 DB) |
| 事件发布 | 发布成交/订单事件到 Kafka |

### 1.1.2 功能列表

```
Matching Engine
├── 订单管理
│   ├── SubmitOrder - 提交订单
│   ├── CancelOrder - 取消订单
│   └── GetOrder - 查询订单
├── 订单簿查询
│   ├── GetOrderBook - 订单簿查询
│   ├── GetDepth - 深度查询
│   └── GetTrades - 成交查询
├── 市场管理
│   ├── CreateMarket - 创建市场订单簿
│   └── CloseMarket - 关闭市场
└── 统计
    └── GetMarketStats - 市场统计
```

## 2. 数据模型

### 2.1 内存结构

```rust
// 订单簿
pub struct OrderBook {
    pub market_id: i64,
    pub outcome_id: i64,
    pub bids: BTreeMap<Price, PriceLevel>,  // 买盘 Price -> PriceLevel
    pub asks: BTreeMap<Price, PriceLevel>,  // 卖盘
    pub trades: Vec<Trade>,                    // 成交记录
}

// 价格档位
pub struct PriceLevel {
    pub price: Price,
    pub orders: Vec<Order>,                    // 同价订单队列
    pub total_quantity: Quantity,
    pub order_count: usize,
}

// 订单
pub struct Order {
    pub id: String,
    pub user_id: i64,
    pub market_id: i64,
    pub outcome_id: i64,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
    pub remaining: Quantity,
    pub filled: Quantity,
    pub filled_amount: Amount,
    pub status: OrderStatus,
    pub created_at: i64,
}
```

## 3. Proto 接口定义

### 3.1 服务定义

```protobuf
syntax = "proto3";

package matching;

service MatchingEngine {
    // 订单操作
    rpc SubmitOrder(SubmitOrderRequest) returns (SubmitOrderResponse);
    rpc CancelOrder(CancelOrderRequest) returns (CancelOrderResponse);
    rpc GetOrder(GetOrderRequest) returns (GetOrderResponse);

    // 订单簿查询
    rpc GetOrderBook(GetOrderBookRequest) returns (GetOrderBookResponse);
    rpc GetDepth(GetDepthRequest) returns (GetDepthResponse);
    rpc GetTrades(GetTradesRequest) returns (GetTradesResponse);

    // 市场管理
    rpc CreateMarket(CreateMarketRequest) returns (CreateMarketResponse);
    rpc CloseMarket(CloseMarketRequest) returns (CloseMarketResponse);

    // 统计
    rpc GetMarketStats(GetMarketStatsRequest) returns (GetMarketStatsResponse);
}
```

### 3.2 消息定义

```protobuf
message SubmitOrderRequest {
    string order_id = 1;
    int64 user_id = 2;
    int64 market_id = 3;
    int64 outcome_id = 4;
    string side = 5;          // "buy" or "sell"
    string order_type = 6;    // "limit" / "market" / "ioc" / "fok" / "post_only"
    string price = 7;
    string quantity = 8;
    int64 timestamp = 9;
}

message SubmitOrderResponse {
    bool success = 1;
    string message = 2;
    string order_id = 3;
    string status = 4;
    string filled_quantity = 5;
    string filled_amount = 6;
    repeated TradeSummary trades = 7;
}

message CancelOrderRequest {
    string order_id = 1;
    int64 user_id = 2;
}

message CancelOrderResponse {
    bool success = 1;
    string message = 2;
    string order_id = 3;
}

message GetOrderRequest {
    string order_id = 1;
}

message GetOrderResponse {
    string order_id = 1;
    int64 user_id = 2;
    string side = 3;
    string order_type = 4;
    string price = 5;
    string quantity = 6;
    string remaining = 7;
    string filled = 8;
    string status = 9;
}

message GetOrderBookRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    int32 limit = 3;
}

message GetOrderBookResponse {
    repeated OrderBookLevel bids = 1;
    repeated OrderBookLevel asks = 2;
    int64 timestamp = 3;
}

message OrderBookLevel {
    string price = 1;
    string quantity = 2;
    int32 orders = 3;
}

message GetDepthRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    int32 depth = 3;
}

message GetDepthResponse {
    repeated PriceLevel bids = 1;
    repeated PriceLevel asks = 2;
    string spread = 3;
    string spread_percent = 4;
}

message PriceLevel {
    string price = 1;
    string quantity = 2;
    int32 orders = 3;
}

message GetTradesRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    int32 limit = 3;
}

message GetTradesResponse {
    repeated TradeSummary trades = 1;
}

message TradeSummary {
    string trade_id = 1;
    string order_id = 2;
    string counter_order_id = 3;
    string side = 4;
    string price = 5;
    string quantity = 6;
    string amount = 7;
    int64 timestamp = 8;
}

message CreateMarketRequest {
    int64 market_id = 1;
    repeated OutcomeInfo outcomes = 2;
}

message OutcomeInfo {
    int64 outcome_id = 1;
    string outcome_asset = 2;        // 结果代币标识 (如 "12345_yes")
    string name = 3;
}

message CreateMarketResponse {
    bool success = 1;
    string message = 2;
}

message CloseMarketRequest {
    int64 market_id = 1;
}

message CloseMarketResponse {
    bool success = 1;
    string message = 2;
    int32 cancelled_orders = 3;       // 被取消的未成交订单数
}

message GetMarketStatsRequest {
    int64 market_id = 1;
}

message GetMarketStatsResponse {
    int64 market_id = 1;
    string last_price = 2;
    string total_volume = 3;
    string total_trades = 4;
}
```

## 4. Kafka 事件

### 4.1 生产的事件

| 事件 | Topic | 说明 |
|------|-------|------|
| OrderSubmitted | order_events | 订单提交 |
| OrderFilled | order_events | 订单成交 |
| OrderPartiallyFilled | order_events | 部分成交 |
| OrderCancelled | order_events | 订单取消 |
| TradeExecuted | trade_executed | 成交执行 |
| OrderBookUpdated | order_book_updates | 订单簿变更 (深度推送) |

### 4.2 事件 Schema

```rust
// OrderEvent
pub struct OrderSubmittedEvent {
    pub event_id: String,
    pub order_id: String,
    pub market_id: i64,
    pub outcome_id: i64,
    pub side: String,
    pub price: String,
    pub quantity: String,
    pub timestamp: i64,
}

pub struct OrderFilledEvent {
    pub event_id: String,
    pub order_id: String,
    pub filled_quantity: String,
    pub filled_amount: String,
    pub timestamp: i64,
}

// TradeEvent
pub struct TradeExecutedEvent {
    pub event_id: String,
    pub trade_id: String,
    pub order_id: String,
    pub counter_order_id: String,
    pub market_id: i64,
    pub outcome_id: i64,
    pub outcome_asset: String,        // 结果代币标识 (如 "12345_yes")
    pub maker_user_id: i64,
    pub taker_user_id: i64,
    pub side: String,
    pub price: String,
    pub quantity: String,
    pub amount: String,
    pub maker_fee: String,
    pub taker_fee: String,
    pub timestamp: i64,
}
```

## 5. 持久化设计

### 5.1 持久化架构

```
┌─────────────────────────────────────────────────────┐
│                 Matching Engine                      │
│                                                 │
│  ┌─────────────┐                                │
│  │  内存订单簿  │ ◄── 实时撮合                    │
│  └──────┬──────┘                                │
│         │                                        │
│         ├──────────────────┐                      │
│         │                  │                      │
│         ▼                  ▼                      │
│  ┌─────────────┐   ┌─────────────┐              │
│  │    WAL     │   │   Redis    │ (可选)         │
│  │  实时追加   │   │   快照缓存  │              │
│  └──────┬──────┘   └─────────────┘              │
│         │                                        │
│         ▼                                        │
│  ┌─────────────┐                                │
│  │ PostgreSQL │  异步批量写入                    │
│  │  (orders) │                                │
│  └─────────────┘                                │
│                                                 │
│  ┌─────────────┐                                │
│  │   快照文件  │  定期保存                       │
│  └─────────────┘                                │
└─────────────────────────────────────────────────────┘
```

### 5.2 恢复流程

```
启动时：
1. 加载最新快照 → 重建内存订单簿
2. 重放 WAL → 恢复到最新状态
3. 继续撮合
```

## 6. 配置

```yaml
# crates/matching-engine/config/matching_engine.yaml
service:
  host: "0.0.0.0"
  port: 50009

kafka:
  brokers:
    - "localhost:9092"
  topics:
    order_events: "order_events"
    trade_executed: "trade_executed"

persistence:
  # WAL 启用
  wal_enabled: true
  # 快照间隔 (笔数)
  snapshot_interval: 1000
  # 异步 DB 写入间隔 (ms)
  db_flush_interval: 100
  # 数据目录
  data_dir: "./data/matching_engine"

matching:
  # 最大订单簿深度
  max_book_depth: 100
  # 价格精度 (小数位)
  price_precision: 8
  # 数量精度
  quantity_precision: 8
```

## 7. 目录结构

```
crates/matching-engine/
├── Cargo.toml
├── build.rs
├── config/
│   └── matching_engine.yaml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── config.rs
    ├── server.rs
    ├── error.rs
    ├── pb.rs
    ├── pb/
    │   ├── matching_engine.proto
    │   └── matching_engine.rs
    ├── engine/
    │   ├── mod.rs
    │   ├── orderbook.rs
    │   ├── matching.rs
    │   └── order.rs
    ├── persistence/
    │   ├── mod.rs
    │   ├── wal.rs
    │   ├── snapshot.rs
    │   └── db_writer.rs
    ├── kafka/
    │   ├── mod.rs
    │   └── producer.rs
    └── services/
        ├── mod.rs
        └── matching_engine_impl.rs
```
