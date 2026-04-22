# ws-order - 订单 WebSocket 服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50017 |
| 协议 | WebSocket |
| 数据库 | 无 |
| 依赖 | Order Service (Kafka 消费) |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 订单状态推送 | 用户订单状态变更实时推送 |

## 2. 订阅主题

| 主题 | 数据类型 | 说明 |
|------|----------|------|
| orders:{user_id} | OrderUpdate | 用户订单状态变更 |

## 3. 消息格式

```json
// OrderUpdate
{
  "type": "order_update",
  "user_id": 1001,
  "data": {
    "order_id": "o123",
    "status": "filled",
    "side": "buy",
    "price": "0.52",
    "quantity": "100",
    "filled_quantity": "100",
    "filled_amount": "52.0",
    "updated_at": 1776809545000
  }
}
```

## 4. Kafka 消费

| Topic | 数据源 |
|-------|--------|
| order_events | Order Service |

## 5. 配置

```yaml
service:
  port: 50017
kafka:
  brokers:
    - "localhost:9092"
```

## 6. 目录结构

```
crates/ws-order/
├── Cargo.toml, config/, src/
```
