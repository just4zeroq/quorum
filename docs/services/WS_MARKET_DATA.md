# ws-market-data - 行情 WebSocket 服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50016 |
| 协议 | WebSocket |
| 数据库 | 无 |
| 依赖 | Market Data Service (Kafka 消费) |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| K线推送 | 实时K线数据 |
| 成交推送 | 实时成交记录 |
| 深度推送 | 订单簿深度 |
| Ticker推送 | 24h统计 |

## 2. 订阅主题

| 主题 | 数据类型 | 说明 |
|------|----------|------|
| kline:{market_id}:{interval} | Kline | K线数据 |
| trades:{market_id} | Trade | 实时成交 |
| depth:{market_id} | OrderBook | 订单簿深度 |
| ticker:{market_id} | Ticker | 24h统计 |

## 3. 消息格式

```json
// Kline
{
  "type": "kline",
  "market_id": 1,
  "interval": "1m",
  "data": {
    "timestamp": 1776809545000,
    "open": "0.5",
    "high": "0.55",
    "low": "0.48",
    "close": "0.52",
    "volume": "10000"
  }
}

// Trade
{
  "type": "trade",
  "market_id": 1,
  "data": {
    "trade_id": "t123",
    "side": "buy",
    "price": "0.52",
    "quantity": "100",
    "timestamp": 1776809545000
  }
}

// Depth
{
  "type": "depth",
  "market_id": 1,
  "data": {
    "bids": [{"price": "0.51", "quantity": "100"}],
    "asks": [{"price": "0.53", "quantity": "200"}]
  }
}

// Ticker
{
  "type": "ticker",
  "market_id": 1,
  "data": {
    "last_price": "0.52",
    "volume_24h": "1000000",
    "price_change": "0.02"
  }
}
```

## 4. Kafka 消费

| Topic | 数据源 |
|-------|--------|
| kline_updates | Market Data Service |
| trade_executed | Matching Engine |
| market_events | Prediction Market Service |

## 5. 配置

```yaml
service:
  port: 50016
kafka:
  brokers:
    - "localhost:9092"
```

## 6. 目录结构

```
crates/ws-market-data/
├── Cargo.toml, config/, src/
```
