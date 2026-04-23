# WebSocket Market Data Service 设计文档

## 概述

ws-market-data 是一个独立的 WebSocket 服务，负责实时推送行情数据（订单簿、K线、成交、24h统计）给客户端。

**服务端口**: 50016

---

## 功能特性

- [x] 订单簿实时推送
- [x] K线实时推送
- [x] 成交实时推送
- [x] 24h Ticker 推送
- [x] 多市场同时订阅
- [x] 心跳保活

---

## 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                    ws-market-data Service                    │
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐   │
│  │  WebSocket  │    │   Kafka     │    │   Market    │   │
│  │   Client    │◄───│  Consumer   │◄───│    Data     │   │
│  │  Manager    │    │ (orderbook, │    │   Service   │   │
│  │             │    │  kline,     │    │             │   │
│  │             │    │  trade)     │    │             │   │
│  └─────────────┘    └─────────────┘    └─────────────┘   │
│         │                                       ▲           │
│         │                                       │           │
│         ▼                                       │           │
│  ┌─────────────┐                                │           │
│  │  Session    │                                │           │
│  │   Store     │                                │           │
│  │ (MarketID  │                                │           │
│  │  -> Vec<   │                                │           │
│  │   Client>) │                                │           │
│  └─────────────┘                                │           │
└─────────────────────────────────────────────────────────────┘
```

---

## Kafka 主题

| 主题名 | 消息类型 | 说明 |
|--------|----------|------|
| `market.orderbook` | OrderBookUpdate | 订单簿更新 |
| `market.kline` | KlineUpdate | K线更新 |
| `market.trade` | TradeUpdate | 成交推送 |
| `market.ticker` | TickerUpdate | 24h Ticker |

---

## 消息格式

### 1. 订单簿更新 (OrderBookUpdate)

```json
{
    "type": "orderbook",
    "market_id": 1,
    "timestamp": 1234567890,
    "data": {
        "yes_bids": [
            {"price": "0.65", "quantity": "1000", "orders": 5},
            {"price": "0.64", "quantity": "2000", "orders": 10}
        ],
        "yes_asks": [
            {"price": "0.66", "quantity": "1500", "orders": 8},
            {"price": "0.67", "quantity": "3000", "orders": 15}
        ],
        "no_bids": [...],
        "no_asks": [...]
    }
}
```

### 2. K线更新 (KlineUpdate)

```json
{
    "type": "kline",
    "market_id": 1,
    "interval": "1m",
    "data": {
        "timestamp": 1234567890,
        "open": "0.60",
        "high": "0.68",
        "low": "0.58",
        "close": "0.65",
        "volume": "50000",
        "quote_volume": "32500"
    }
}
```

### 3. 成交推送 (TradeUpdate)

```json
{
    "type": "trade",
    "market_id": 1,
    "data": {
        "id": 12345,
        "outcome_id": 1,
        "side": "buy",
        "price": "0.65",
        "quantity": "100",
        "amount": "65.00",
        "fee": "0.01",
        "timestamp": 1234567890
    }
}
```

### 4. 24h Ticker 更新 (TickerUpdate)

```json
{
    "type": "ticker",
    "market_id": 1,
    "data": {
        "volume_24h": "500000",
        "amount_24h": "250000",
        "high_24h": "0.70",
        "low_24h": "0.55",
        "price_change": "0.05",
        "price_change_percent": "8.33%",
        "last_price": "0.65",
        "trade_count_24h": 1500,
        "timestamp": 1234567890
    }
}
```

---

## WebSocket 协议

### 连接

```
ws://localhost:50016/ws
```

### 客户端订阅消息

```json
// 订阅单个市场
{
    "action": "subscribe",
    "channel": "orderbook",
    "market_id": 1
}

// 订阅多个市场
{
    "action": "subscribe",
    "channel": "kline",
    "market_ids": [1, 2, 3],
    "params": {
        "interval": "1m"
    }
}

// 取消订阅
{
    "action": "unsubscribe",
    "channel": "orderbook",
    "market_id": 1
}
```

### 支持的 Channel

| Channel | 说明 | 参数 |
|---------|------|------|
| `orderbook` | 订单簿推送 | market_id, depth |
| `kline` | K线推送 | market_ids, interval |
| `trade` | 成交推送 | market_ids |
| `ticker` | Ticker推送 | market_ids |

### 心跳

服务端定期发送心跳：

```json
{
    "type": "pong",
    "timestamp": 1234567890
}
```

客户端应发送：

```json
{
    "type": "ping"
}
```

---

## 实现细节

### Session 管理

```rust
struct ClientSession {
    id: Uuid,
    sender: WebSocketSender,
    subscriptions: HashMap<String, HashSet<u64>>, // channel -> market_ids
    subscribed_at: DateTime<Utc>,
}

struct SessionManager {
    sessions: RwLock<HashMap<Uuid, ClientSession>>,
}
```

### 消息路由

1. Kafka Consumer 接收消息
2. 解析消息类型
3. 根据 market_id 查找订阅者
4. 广播到所有订阅者

### 消息过滤

- 订单簿：只推送订阅的市场
- K线：根据 interval 过滤
- 成交：推送订阅市场的最新成交
- Ticker：推送订阅市场的24h统计

---

## 错误处理

### 客户端错误

| 错误码 | 说明 |
|--------|------|
| 1000 | 正常关闭 |
| 1001 | 服务端重启 |
| 1002 | 协议错误 |
| 1003 | 不支持的数据类型 |
| 1010 | 拒绝连接（满载） |

### 消息错误

```json
{
    "type": "error",
    "code": 400,
    "message": "Invalid subscription",
    "details": "Unknown channel: unknown"
}
```

---

## 性能优化

1. **消息批量处理**
   - 累积一定数量的消息后批量发送
   - 减少网络往返

2. **增量更新**
   - 订单簿推送增量变化，而非全量
   - 减少带宽占用

3. **连接池**
   - 使用连接池管理 Kafka Consumer
   - 复用连接

4. **Goroutine**
   - 每个连接一个 Goroutine
   - Tokio 异步处理

---

## 配置

```yaml
server:
  host: "0.0.0.0"
  port: 50016
  max_connections: 10000

kafka:
  brokers:
    - "localhost:9092"
  consumer_group: "ws-market-data"
  topics:
    - "market.orderbook"
    - "market.kline"
    - "market.trade"
    - "market.ticker"

websocket:
  ping_interval_secs: 30
  ping_timeout_secs: 10
  max_message_size: 1024 * 1024  # 1MB

cache:
  orderbook_ttl_secs: 60
  ticker_ttl_secs: 5
```

---

## 监控指标

| 指标 | 说明 |
|------|------|
| `ws_connections` | 当前连接数 |
| `ws_messages_sent` | 发送消息数 |
| `ws_messages_received` | 接收消息数 |
| `ws_subscribe_total` | 订阅总数 |
| `ws_unsubscribe_total` | 取消订阅总数 |
| `kafka_lag` | Kafka 消费延迟 |
