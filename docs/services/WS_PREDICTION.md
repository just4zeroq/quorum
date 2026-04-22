# ws-prediction - 市场事件 WebSocket 服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50018 |
| 协议 | WebSocket |
| 数据库 | 无 |
| 依赖 | Prediction Market Service (Kafka 消费) |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 市场事件推送 | 市场状态变更实时推送 |

## 2. 订阅主题

| 主题 | 数据类型 | 说明 |
|------|----------|------|
| market:{market_id}:status | MarketStatus | 市场状态变更 |

## 3. 消息格式

```json
// MarketStatus
{
  "type": "market_status",
  "market_id": 1,
  "data": {
    "status": "resolved",
    "winning_outcome_id": 2,
    "resolved_at": 1776809545000
  }
}
```

## 4. Kafka 消费

| Topic | 数据源 |
|-------|--------|
| market_events | Prediction Market Service |

## 5. 配置

```yaml
service:
  port: 50018
kafka:
  brokers:
    - "localhost:9092"
```

## 6. 目录结构

```
crates/ws-prediction/
├── Cargo.toml, config/, src/
```
