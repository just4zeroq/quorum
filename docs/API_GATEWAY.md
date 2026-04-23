# API Gateway 接口文档

## 概述

API Gateway 是系统的统一入口，提供 HTTP REST API 和 WebSocket 接口。

**Base URL**: `http://localhost:8080`

**认证方式**: JWT Bearer Token

```
Authorization: Bearer <token>
```

---

## 目录

1. [认证接口](#1-认证接口)
2. [用户接口](#2-用户接口)
3. [市场接口](#3-市场接口)
4. [订单接口](#4-订单接口)
5. [账户接口](#5-账户接口)
6. [钱包接口](#6-钱包接口)
7. [WebSocket](#7-websocket)

---

## 1. 认证接口

### 1.1 注册

```
POST /api/v1/auth/register
```

**请求体**:
```json
{
    "username": "string",
    "email": "user@example.com",
    "password": "string",
    "invite_code": "string (可选)"
}
```

**响应**:
```json
{
    "success": true,
    "user_id": "uuid",
    "token": "jwt_token",
    "message": "注册成功"
}
```

---

### 1.2 登录

```
POST /api/v1/auth/login
```

**请求体**:
```json
{
    "email": "user@example.com",
    "password": "string",
    "code_2fa": "string (可选)"
}
```

**响应**:
```json
{
    "success": true,
    "user_id": "uuid",
    "token": "jwt_token",
    "refresh_token": "string",
    "expires_at": 1234567890,
    "user": { ... }
}
```

---

### 1.3 登出

```
POST /api/v1/auth/logout
```

**请求头**: `Authorization: Bearer <token>`

**响应**:
```json
{
    "success": true
}
```

---

### 1.4 刷新Token

```
POST /api/v1/auth/refresh
```

**请求体**:
```json
{
    "refresh_token": "string"
}
```

**响应**:
```json
{
    "token": "new_jwt_token",
    "refresh_token": "new_refresh_token",
    "expires_at": 1234567890
}
```

---

### 1.5 钱包登录

```
POST /api/v1/auth/wallet/login
```

**请求体**:
```json
{
    "wallet_address": "0x...",
    "wallet_type": "evm|solana|tron",
    "signature": "string",
    "timestamp": 1234567890
}
```

**响应**:
```json
{
    "success": true,
    "user_id": "uuid",
    "token": "jwt_token",
    "is_new_user": true
}
```

---

### 1.6 获取钱包Nonce

```
GET /api/v1/auth/wallet/nonce?wallet_address=0x...
```

**响应**:
```json
{
    "nonce": "random_string",
    "expires_at": 1234567890
}
```

---

## 2. 用户接口

### 2.1 获取当前用户信息

```
GET /api/v1/users/me
```

**请求头**: `Authorization: Bearer <token>`

**响应**:
```json
{
    "id": "uuid",
    "username": "string",
    "email": "user@example.com",
    "phone": "string",
    "status": "active|frozen|closed",
    "kyc_status": "none|pending|verified|rejected",
    "two_factor_enabled": true,
    "created_at": "2024-01-01T00:00:00Z"
}
```

---

### 2.2 更新用户信息

```
PUT /api/v1/users/me
```

**请求头**: `Authorization: Bearer <token>`

**请求体**:
```json
{
    "username": "string (可选)",
    "email": "string (可选)"
}
```

---

### 2.3 获取用户风险等级

```
GET /api/v1/users/me/risk
```

**响应**:
```json
{
    "risk_level": 1,
    "kyc_level": 2,
    "frozen": false,
    "frozen_reason": null
}
```

---

## 3. 市场接口

### 3.1 获取市场列表

```
GET /api/v1/markets
```

**查询参数**:
| 参数 | 类型 | 说明 |
|------|------|------|
| category | string | 分类过滤 |
| status | string | open/resolved/cancelled |
| sort_by | string | created_at/end_time/volume |
| descending | boolean | 降序 (默认true) |
| page | int | 页码 (默认1) |
| page_size | int | 每页数量 (默认20) |

**响应**:
```json
{
    "markets": [
        {
            "id": 1,
            "question": "Bitcoin will exceed $100k by 2025?",
            "category": "crypto",
            "end_time": 1735689600,
            "status": "open",
            "total_volume": "1000000",
            "outcomes": [
                {"id": 1, "name": "Yes", "price": "0.65", "volume": "600000", "probability": "65%"},
                {"id": 2, "name": "No", "price": "0.35", "volume": "400000", "probability": "35%"}
            ],
            "created_at": "2024-01-01T00:00:00Z"
        }
    ],
    "total": 100,
    "page": 1,
    "page_size": 20
}
```

---

### 3.2 获取市场详情

```
GET /api/v1/markets/{market_id}
```

**响应**:
```json
{
    "id": 1,
    "question": "Bitcoin will exceed $100k by 2025?",
    "description": "...",
    "category": "crypto",
    "image_url": "https://...",
    "start_time": 1704067200,
    "end_time": 1735689600,
    "status": "open",
    "total_volume": "1000000",
    "resolved_outcome_id": null,
    "outcomes": [
        {
            "id": 1,
            "name": "Yes",
            "description": "...",
            "price": "0.65",
            "volume": "600000",
            "probability": "65%"
        }
    ]
}
```

---

### 3.3 获取市场深度

```
GET /api/v1/markets/{market_id}/orderbook
```

**查询参数**:
| 参数 | 类型 | 说明 |
|------|------|------|
| depth | int | 深度档位数 (默认10) |

**响应**:
```json
{
    "market_id": 1,
    "bids": [
        {"price": "0.65", "quantity": "1000", "orders": 5},
        {"price": "0.64", "quantity": "2000", "orders": 10}
    ],
    "asks": [
        {"price": "0.66", "quantity": "1500", "orders": 8},
        {"price": "0.67", "quantity": "3000", "orders": 15}
    ],
    "timestamp": 1234567890
}
```

---

### 3.4 获取K线数据

```
GET /api/v1/markets/{market_id}/klines
```

**查询参数**:
| 参数 | 类型 | 说明 |
|------|------|------|
| interval | string | 1m/5m/15m/1h/4h/1d |
| start_time | int64 | 开始时间戳 |
| end_time | int64 | 结束时间戳 |
| limit | int | 数量限制 (默认100) |

**响应**:
```json
{
    "market_id": 1,
    "interval": "1h",
    "klines": [
        {
            "timestamp": 1234567890,
            "open": "0.60",
            "high": "0.68",
            "low": "0.58",
            "close": "0.65",
            "volume": "50000",
            "quote_volume": "32500"
        }
    ]
}
```

---

### 3.5 获取24h统计

```
GET /api/v1/markets/{market_id}/stats
```

**响应**:
```json
{
    "market_id": 1,
    "volume_24h": "500000",
    "amount_24h": "250000",
    "high_24h": "0.70",
    "low_24h": "0.55",
    "price_change": "0.05",
    "price_change_percent": "8.33%",
    "trade_count_24h": 1500,
    "timestamp": 1234567890
}
```

---

### 3.6 获取最近成交

```
GET /api/v1/markets/{market_id}/trades
```

**查询参数**:
| 参数 | 类型 | 说明 |
|------|------|------|
| limit | int | 数量限制 (默认50) |

**响应**:
```json
{
    "trades": [
        {
            "id": 1,
            "market_id": 1,
            "outcome_id": 1,
            "user_id": "uuid",
            "side": "buy",
            "price": "0.65",
            "quantity": "100",
            "amount": "65.00",
            "fee": "0.01",
            "timestamp": 1234567890
        }
    ],
    "has_more": true
}
```

---

## 4. 订单接口

### 4.1 创建订单

```
POST /api/v1/orders
```

**请求头**: `Authorization: Bearer <token>`

**请求体**:
```json
{
    "market_id": 1,
    "outcome_id": 1,
    "side": "buy|sell",
    "order_type": "limit|market|ioc|fok|post_only",
    "price": "0.65",
    "quantity": "100"
}
```

**响应**:
```json
{
    "success": true,
    "order_id": "uuid",
    "order": {
        "id": "uuid",
        "market_id": 1,
        "outcome_id": 1,
        "side": "buy",
        "order_type": "limit",
        "price": "0.65",
        "quantity": "100",
        "filled_quantity": "0",
        "filled_amount": "0",
        "status": "submitted",
        "created_at": 1234567890
    }
}
```

---

### 4.2 取消订单

```
DELETE /api/v1/orders/{order_id}
```

**请求头**: `Authorization: Bearer <token>`

**响应**:
```json
{
    "success": true,
    "message": "Order cancelled"
}
```

---

### 4.3 获取订单详情

```
GET /api/v1/orders/{order_id}
```

**请求头**: `Authorization: Bearer <token>`

**响应**:
```json
{
    "id": "uuid",
    "market_id": 1,
    "outcome_id": 1,
    "side": "buy",
    "order_type": "limit",
    "price": "0.65",
    "quantity": "100",
    "filled_quantity": "50",
    "filled_amount": "32.50",
    "status": "partially_filled",
    "created_at": 1234567890,
    "updated_at": 1234567900
}
```

---

### 4.4 获取用户订单列表

```
GET /api/v1/orders
```

**请求头**: `Authorization: Bearer <token>`

**查询参数**:
| 参数 | 类型 | 说明 |
|------|------|------|
| market_id | int | 市场ID过滤 |
| status | string | submitted/partially_filled/filled/cancelled/rejected |
| page | int | 页码 |
| page_size | int | 每页数量 |

**响应**:
```json
{
    "orders": [...],
    "total": 50,
    "page": 1,
    "page_size": 20
}
```

---

## 5. 账户接口

### 5.1 获取账户余额

```
GET /api/v1/accounts/balance
```

**请求头**: `Authorization: Bearer <token>`

**响应**:
```json
{
    "accounts": [
        {
            "asset": "USDC",
            "available": "1000.00",
            "frozen": "100.00",
            "equity": "1100.00"
        }
    ]
}
```

---

### 5.2 账户间划转

```
POST /api/v1/accounts/transfer
```

**请求头**: `Authorization: Bearer <token>`

**请求体**:
```json
{
    "from_account": "spot|futures",
    "to_account": "spot|futures",
    "asset": "USDC",
    "amount": "100.00"
}
```

**响应**:
```json
{
    "success": true,
    "message": "Transfer successful",
    "new_balance": "..."
}
```

---

## 6. 钱包接口

### 6.1 获取充值地址

```
GET /api/v1/wallet/deposit/address
```

**请求头**: `Authorization: Bearer <token>`

**查询参数**:
| 参数 | 类型 | 说明 |
|------|------|------|
| asset | string | 资产类型 (如 BTC, ETH) |
| network | string | 链名称 (如 eth, bsc) |

**响应**:
```json
{
    "address": "0x1234567890abcdef",
    "asset": "USDC",
    "network": "eth",
    "tag": "12345678 (可选)"
}
```

---

### 6.2 提现

```
POST /api/v1/wallet/withdraw
```

**请求头**: `Authorization: Bearer <token>`

**请求体**:
```json
{
    "asset": "USDC",
    "amount": "100.00",
    "address": "0x...",
    "network": "eth",
    "fee": "1.00"
}
```

**响应**:
```json
{
    "success": true,
    "withdrawal_id": "uuid",
    "status": "pending",
    "message": "Withdrawal submitted"
}
```

---

### 6.3 获取钱包历史

```
GET /api/v1/wallet/history
```

**请求头**: `Authorization: Bearer <token>`

**查询参数**:
| 参数 | 类型 | 说明 |
|------|------|------|
| asset | string | 资产类型 |
| type | string | deposit/withdraw/transfer |
| start_time | int64 | 开始时间 |
| end_time | int64 | 结束时间 |
| page | int | 页码 |
| page_size | int | 每页数量 |

**响应**:
```json
{
    "deposits": [
        {
            "id": "uuid",
            "asset": "USDC",
            "amount": "100.00",
            "address": "0x...",
            "tx_hash": "0x...",
            "status": "confirmed",
            "timestamp": 1234567890
        }
    ],
    "withdrawals": [...],
    "total": 100,
    "page": 1,
    "page_size": 20
}
```

---

## 7. WebSocket

### 7.1 行情WebSocket

**连接地址**: `ws://localhost:8080/ws/market`

**订阅消息**:
```json
{
    "action": "subscribe",
    "channel": "orderbook|kline|trade|ticker",
    "market_id": 1,
    "params": {
        "depth": 10,
        "interval": "1m"
    }
}
```

**推送消息**:

#### 订单簿更新
```json
{
    "channel": "orderbook",
    "market_id": 1,
    "data": {
        "bids": [...],
        "asks": [...],
        "timestamp": 1234567890
    }
}
```

#### K线更新
```json
{
    "channel": "kline",
    "market_id": 1,
    "interval": "1m",
    "data": {
        "timestamp": 1234567890,
        "open": "0.60",
        "high": "0.68",
        "low": "0.58",
        "close": "0.65",
        "volume": "50000"
    }
}
```

#### 成交推送
```json
{
    "channel": "trade",
    "market_id": 1,
    "data": {
        "id": 1,
        "side": "buy",
        "price": "0.65",
        "quantity": "100",
        "timestamp": 1234567890
    }
}
```

---

### 7.2 订单WebSocket

**连接地址**: `ws://localhost:8080/ws/order`

**订阅消息**:
```json
{
    "action": "subscribe",
    "token": "jwt_token"
}
```

**推送消息**:

#### 订单状态更新
```json
{
    "channel": "order",
    "data": {
        "order_id": "uuid",
        "status": "filled",
        "filled_quantity": "100",
        "filled_amount": "65.00",
        "timestamp": 1234567890
    }
}
```

---

## 错误码

| 错误码 | 说明 |
|--------|------|
| 400 | 请求参数错误 |
| 401 | 未授权 / Token无效 |
| 403 | 权限不足 |
| 404 | 资源不存在 |
| 429 | 请求过于频繁 |
| 500 | 服务器内部错误 |

**错误响应格式**:
```json
{
    "error": {
        "code": 400,
        "message": "Invalid parameter",
        "details": {
            "field": "price",
            "reason": "must be positive"
        }
    }
}
```

---

## 限流规则

| 接口 | 限制 |
|------|------|
| 读取接口 | 100次/分钟 |
| 写入接口 | 30次/分钟 |
| 订单接口 | 60次/分钟 |
| WebSocket | 10连接/用户 |
