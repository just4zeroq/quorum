# Order Service - 订单服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50003 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | Account Service (50019), Matching Engine (50009), User Service (50001) |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 订单创建 | 创建订单、参数校验、生成订单号 |
| 订单取消 | 用户取消订单 |
| 订单查询 | 查询订单详情、订单列表 |
| 冻结联动 | 创建订单时调用 Account Service 冻结余额 |
| 事件记录 | 记录订单状态变更事件 |
| 成交回报 | 接收 Matching Engine 成交回报 |

### 1.1.2 功能列表

```
Order Service
├── 订单操作
│   ├── CreateOrder - 创建订单
│   ├── CancelOrder - 取消订单
│   ├── GetOrder - 查询订单
│   └── GetUserOrders - 查询用户订单列表
├── 订单市场查询
│   ├── GetMarketOrders - 查询市场订单列表
│   └── GetOrderBookSnapshot - 获取订单簿快照
├── 订单事件
│   ├── GetOrderEvents - 查询订单事件
│   └── GetRecentEvents - 查询最近事件
└── 内部服务
    ├── UpdateOrderStatus - 更新订单状态 (Matching Engine 调用)
    └── SyncOrderFromMarket - 从市场同步订单 (市场关闭时)
```

## 2. 数据模型

### 2.1 Domain Model

已在 `crates/domain/src/order/model/mod.rs` 定义：

```rust
/// 订单状态
pub enum OrderStatus {
    Pending,        // 待提交
    Submitted,      // 已提交
    PartiallyFilled,// 部分成交
    Filled,         // 完全成交
    Cancelled,      // 已取消
    Rejected,       // 已拒绝
}

/// 订单方向
pub enum OrderSide {
    Buy,
    Sell,
}

/// 订单类型
pub enum OrderType {
    Limit,      // 限价单
    Market,     // 市价单
    IOC,        // 即时成交否则取消
    FOK,        // 全部成交否则取消
    PostOnly,   // 只挂单
}

/// 订单
pub struct Order {
    pub id: String,              // 订单号 (时间序列)
    pub user_id: i64,
    pub market_id: i64,
    pub outcome_id: i64,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub filled_amount: Decimal,
    pub status: OrderStatus,
    pub client_order_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 订单查询
pub struct OrderQuery {
    pub user_id: Option<i64>,
    pub market_id: Option<i64>,
    pub outcome_id: Option<i64>,
    pub status: Option<OrderStatus>,
    pub side: Option<OrderSide>,
    pub page: i32,
    pub page_size: i32,
}

/// 订单事件记录
pub struct OrderEventRecord {
    pub id: i64,
    pub order_id: String,
    pub event_type: String,
    pub old_status: Option<OrderStatus>,
    pub new_status: OrderStatus,
    pub filled_quantity: Option<Decimal>,
    pub filled_amount: Option<Decimal>,
    pub price: Option<Decimal>,
    pub reason: Option<String>,
    pub created_at: i64,
}
```

### 2.2 Domain Event

```rust
// domain/src/order/event/mod.rs

pub enum OrderEvent {
    Created {
        order_id: String,
        user_id: i64,
        market_id: i64,
        outcome_id: i64,
    },
    Submitted {
        order_id: String,
    },
    PartiallyFilled {
        order_id: String,
        filled_quantity: String,
        filled_amount: String,
        price: String,
    },
    Filled {
        order_id: String,
        filled_quantity: String,
        filled_amount: String,
    },
    Cancelled {
        order_id: String,
        reason: Option<String>,
    },
    Rejected {
        order_id: String,
        reason: String,
    },
}
```

### 2.3 数据库表

**orders 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | TEXT | PRIMARY KEY | 订单ID (时间序列号) |
| user_id | BIGINT | NOT NULL | 用户ID |
| market_id | BIGINT | NOT NULL | 市场ID |
| outcome_id | BIGINT | NOT NULL | 选项ID |
| side | VARCHAR(10) | NOT NULL | buy/sell |
| order_type | VARCHAR(20) | NOT NULL | limit/market/ioc/fok/post_only |
| price | TEXT | NOT NULL | 价格 |
| quantity | TEXT | NOT NULL | 数量 |
| filled_quantity | TEXT | NOT NULL DEFAULT '0' | 已成交数量 |
| filled_amount | TEXT | NOT NULL DEFAULT '0' | 已成交金额 |
| status | VARCHAR(20) | NOT NULL DEFAULT 'pending' | 状态 |
| client_order_id | TEXT | | 客户端订单ID |
| created_at | BIGINT | NOT NULL | 创建时间 |
| updated_at | BIGINT | NOT NULL | 更新时间 |

**索引**:
- `PRIMARY KEY (id)`
- `INDEX idx_user_id (user_id)`
- `INDEX idx_market_id (market_id)`
- `INDEX idx_status (status)`
- `INDEX idx_created_at (created_at)`

**order_events 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| order_id | TEXT | NOT NULL | 订单ID |
| event_type | VARCHAR(30) | NOT NULL | 事件类型 |
| old_status | VARCHAR(20) | | 变更前状态 |
| new_status | VARCHAR(20) | NOT NULL | 变更后状态 |
| filled_quantity | TEXT | | 成交数量 |
| filled_amount | TEXT | | 成交金额 |
| price | TEXT | | 成交价格 |
| reason | TEXT | | 变更原因 |
| created_at | BIGINT | NOT NULL | 事件时间 |

**索引**:
- `INDEX idx_order_id (order_id)`
- `INDEX idx_created_at (created_at)`

## 3. Proto 接口定义

### 3.1 服务定义

```protobuf
syntax = "proto3";

package order;

service OrderService {
    // 订单操作
    rpc CreateOrder(CreateOrderRequest) returns (CreateOrderResponse);
    rpc CancelOrder(CancelOrderRequest) returns (CancelOrderResponse);
    rpc GetOrder(GetOrderRequest) returns (GetOrderResponse);
    rpc GetUserOrders(GetUserOrdersRequest) returns (GetUserOrdersResponse);

    // 市场订单
    rpc GetMarketOrders(GetMarketOrdersRequest) returns (GetMarketOrdersResponse);
    rpc GetOrderBookSnapshot(GetOrderBookSnapshotRequest) returns (GetOrderBookSnapshotResponse);

    // 订单事件
    rpc GetOrderEvents(GetOrderEventsRequest) returns (GetOrderEventsResponse);
    rpc GetRecentEvents(GetRecentEventsRequest) returns (GetRecentEventsResponse);

    // 内部服务
    rpc UpdateOrderStatus(UpdateOrderStatusRequest) returns (UpdateOrderStatusResponse);
    rpc SyncOrderFromMarket(SyncOrderFromMarketRequest) returns (SyncOrderFromMarketResponse);
}
```

### 3.2 消息定义

```protobuf
// ==================== 订单操作 ====================

message CreateOrderRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    int64 outcome_id = 3;
    string outcome_asset = 4;  // 结果代币标识 (如 "12345_yes")
    string side = 5;           // "buy" or "sell"
    string order_type = 6;     // "limit" / "market" / "ioc" / "fok" / "post_only"
    string price = 7;
    string quantity = 8;
    string client_order_id = 9; // 可选，客户端自定义ID
}

message CreateOrderResponse {
    bool success = 1;
    string message = 2;
    string order_id = 3;
    int64 created_at = 4;
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
    int64 market_id = 3;
    int64 outcome_id = 4;
    string side = 5;
    string order_type = 6;
    string price = 7;
    string quantity = 8;
    string filled_quantity = 9;
    string filled_amount = 10;
    string status = 11;
    string client_order_id = 12;
    int64 created_at = 13;
    int64 updated_at = 14;
}

message GetUserOrdersRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    string status = 3;
    int32 page = 4;
    int32 page_size = 5;
}

message GetUserOrdersResponse {
    repeated OrderSummary orders = 1;
    int64 total = 2;
    int32 page = 3;
    int32 page_size = 4;
}

message OrderSummary {
    string order_id = 1;
    int64 market_id = 2;
    int64 outcome_id = 3;
    string side = 4;
    string order_type = 5;
    string price = 6;
    string quantity = 7;
    string filled_quantity = 8;
    string status = 9;
    int64 created_at = 10;
}

// ==================== 市场订单 ====================

message GetMarketOrdersRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    string side = 3;
    int32 limit = 4;
}

message GetMarketOrdersResponse {
    repeated OrderSummary orders = 1;
}

message GetOrderBookSnapshotRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    int32 depth = 3;
}

message GetOrderBookSnapshotResponse {
    repeated OrderLevel bids = 1;
    repeated OrderLevel asks = 2;
    int64 timestamp = 3;
}

message OrderLevel {
    string price = 1;
    string quantity = 2;
    int32 orders = 3;
}

// ==================== 订单事件 ====================

message GetOrderEventsRequest {
    string order_id = 1;
}

message GetOrderEventsResponse {
    repeated OrderEventSummary events = 1;
}

message OrderEventSummary {
    int64 id = 1;
    string event_type = 2;
    string old_status = 3;
    string new_status = 4;
    string filled_quantity = 5;
    string filled_amount = 6;
    string price = 7;
    string reason = 8;
    int64 created_at = 9;
}

message GetRecentEventsRequest {
    int64 limit = 1;
}

message GetRecentEventsResponse {
    repeated OrderEventSummary events = 1;
}

// ==================== 内部服务 ====================

message UpdateOrderStatusRequest {
    string order_id = 1;
    string status = 2;
    string filled_quantity = 3;
    string filled_amount = 4;
    string price = 5;
    string reason = 6;
    string outcome_asset = 7;      // 结果代币标识 (FreezeAndDeduct 需要)
    string outcome_amount = 8;     // 结果代币数量 (成交后增加)
}

message UpdateOrderStatusResponse {
    bool success = 1;
    string message = 2;
}

message SyncOrderFromMarketRequest {
    int64 market_id = 1;
    string reason = 2;             // 同步原因 (如 "market_closed")
}

message SyncOrderFromMarketResponse {
    bool success = 1;
    string message = 2;
    int32 cancelled_orders = 3;    // 被取消的未成交订单数
    int32 synced_orders = 4;       // 同步的订单数
}
```

## 4. Kafka 事件

### 4.1 生产的事件

| 事件 | Topic | 说明 |
|------|-------|------|
| OrderCreated | order_events | 订单创建 |
| OrderSubmitted | order_events | 订单提交 |
| OrderPartiallyFilled | order_events | 部分成交 |
| OrderFilled | order_events | 完全成交 |
| OrderCancelled | order_events | 订单取消 |
| OrderRejected | order_events | 订单拒绝 |

### 4.2 事件 Schema

```rust
pub enum OrderEvent {
    Created {
        order_id: String,
        user_id: i64,
        market_id: i64,
        outcome_id: i64,
    },
    Submitted {
        order_id: String,
    },
    PartiallyFilled {
        order_id: String,
        filled_quantity: String,
        filled_amount: String,
        price: String,
    },
    Filled {
        order_id: String,
        filled_quantity: String,
        filled_amount: String,
    },
    Cancelled {
        order_id: String,
        reason: Option<String>,
    },
    Rejected {
        order_id: String,
        reason: String,
    },
}
```

## 5. 服务间通信

### 5.1 gRPC 调用

| 被调用方 | 接口 | 场景 |
|----------|------|------|
| Account Service (50019) | Freeze | 下单时冻结余额 |
| Account Service (50019) | Unfreeze | 撤单时解冻余额 |
| Account Service (50019) | CheckBalance | 下单前检查余额 |
| Matching Engine (50009) | SubmitOrder | 提交订单 |
| Matching Engine (50009) | CancelOrder | 取消订单 |
| User Service (50001) | GetUserById | 获取用户信息 |
| Risk Service (50004) | CheckOrder | 下单前风控检查 |

### 5.2 gRPC 被调用

| 调用方 | 接口 | 场景 |
|----------|------|------|
| Matching Engine | UpdateOrderStatus | 成交回报 |
| API Gateway | CreateOrder | 用户下单 |
| API Gateway | CancelOrder | 用户撤单 |
| API Gateway | GetOrder | 查询订单 |

### 5.3 Kafka 消费

| Topic | 处理 | 说明 |
|-------|------|------|
| order_events | 无 | Order Service 不消费自己发布的事件 |

## 6. 核心流程

### 6.1 创建订单流程

```
CreateOrder 请求
    │
    ▼
┌─────────────────────────────────────────┐
│ 1. 参数校验                              │
│    - price > 0, quantity > 0             │
│    - side in {buy, sell}                 │
│    - order_type valid                    │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 2. 生成订单号                            │
│    - utils::id::generate_order_id()      │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 3. 调用 Risk Service 风控检查            │
│    - CheckOrder (同步)                   │
│    - 风控拒绝 → 返回失败                  │
│    ⚠️ 必须集成，否则无法下单              │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 4. 调用 Account Service 原子冻结余额      │
│    - CheckAndFreeze (同步，原子操作)     │
│    - 买入: 冻结 USDT                    │
│    - 卖出: 冻结结果代币                  │
│    - 余额不足 → 返回失败                  │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 5. 保存订单到数据库                       │
│    - status = Pending                   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 6. 记录订单事件                          │
│    - event_type = "created"             │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 7. 发布 Kafka 事件                       │
│    - OrderCreated                       │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 8. 调用 Matching Engine 提交订单          │
│    - SubmitOrder (异步)                 │
│    - 提交成功 → 后续由 Matching Engine    │
│      通过 UpdateOrderStatus 更新状态       │
└─────────────────┬───────────────────────┘
                  │
                  ▼
返回 CreateOrderResponse
```

### 6.2 取消订单流程

```
CancelOrder 请求
    │
    ▼
┌─────────────────────────────────────────┐
│ 1. 查询订单                              │
│    - order_id → Order                   │
│    - 订单不存在 → 返回失败               │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 2. 验证权限                              │
│    - user_id 匹配                       │
│    - 不匹配 → 返回无权限                 │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 3. 检查状态                              │
│    - status in {Pending, Submitted,     │
│      PartiallyFilled}                   │
│    - 不在可取消状态 → 返回失败           │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 4. 调用 Matching Engine 取消订单          │
│    - CancelOrder                        │
│    - 如果有冻结金额需要解冻               │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 5. 调用 Account Service 解冻余额          │
│    - Unfreeze (如果有冻结)               │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 6. 更新订单状态                          │
│    - status = Cancelled                 │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 7. 记录订单事件                          │
│    - event_type = "cancelled"           │
└─────────────────┬───────────────────────┘
                  │
                  ▼
返回 CancelOrderResponse
```

### 6.3 成交回报流程 (UpdateOrderStatus)

```
UpdateOrderStatus 请求 (Matching Engine 调用)
    │
    ▼
┌─────────────────────────────────────────┐
│ 1. 查询订单                              │
│    - order_id → Order                   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 2. 更新订单                              │
│    - filled_quantity                    │
│    - filled_amount                      │
│    - status = new_status                 │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 3. 如果完全成交                          │
│    - 调用 Account Service 扣减冻结        │
│    - FreezeAndDeduct                    │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 4. 如果部分成交且订单完成                 │
│    - 解冻剩余金额                        │
│    - Unfreeze                           │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 5. 记录订单事件                          │
│    - event_type = filled/partially_filled│
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 6. 发布 Kafka 事件                       │
│    - OrderFilled / OrderPartiallyFilled  │
└─────────────────┬───────────────────────┘
                  │
                  ▼
返回 UpdateOrderStatusResponse
```

## 7. 配置

```yaml
# crates/order-service/config/order_service.yaml
service:
  host: "0.0.0.0"
  port: 50003

database:
  driver: "sqlite"
  url: "sqlite:./data/orders.db"
  max_connections: 20

matching_engine:
  addr: "localhost:50009"
  timeout_ms: 5000

account_service:
  addr: "localhost:50019"
  timeout_ms: 3000

user_service:
  addr: "localhost:50001"
  timeout_ms: 3000

kafka:
  brokers:
    - "localhost:9092"
  topics:
    order_events: "order_events"

order:
  # 订单有效期 (毫秒)，0 = 永不过期
  default_ttl_ms: 0
  # 最大未成交订单数 per user
  max_pending_orders_per_user: 100
```

## 8. 错误码

| 错误码 | 说明 |
|--------|------|
| ORDER_NOT_FOUND | 订单不存在 |
| ORDER_ALREADY_EXISTS | 订单已存在 |
| ORDER_CANNOT_CANCEL | 订单不可取消 |
| ORDER_CANNOT_MODIFY | 订单不可修改 |
| INSUFFICIENT_BALANCE | 余额不足 |
| MARKET_NOT_FOUND | 市场不存在 |
| OUTCOME_NOT_FOUND | 选项不存在 |
| PRICE_INVALID | 价格无效 |
| QUANTITY_INVALID | 数量无效 |
| USER_NOT_FOUND | 用户不存在 |
| DUPLICATE_CLIENT_ORDER_ID | 客户端订单ID重复 |
| ORDER_LIMIT_EXCEEDED | 订单数超限 |

## 9. 目录结构

```
crates/order-service/
├── Cargo.toml
├── build.rs
├── config/
│   └── order_service.yaml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── config.rs
    ├── server.rs
    ├── error.rs
    ├── pb.rs
    ├── pb/
    │   ├── order.proto
    │   └── order.rs
    ├── repository/
    │   ├── mod.rs
    │   ├── order_repo.rs
    │   └── event_repo.rs
    ├── services/
    │   ├── mod.rs
    │   └── order_service_impl.rs
    ├── clients/
    │   ├── mod.rs
    │   ├── matching_engine_client.rs
    │   ├── account_service_client.rs
    │   ├── user_service_client.rs
    │   └── risk_service_client.rs
    └── utils/
        └── mod.rs
```
