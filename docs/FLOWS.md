# 预测市场交易所 - 业务流程文档

## 目录

1. [业务流程总览](#1-业务流程总览)
2. [用户注册登录](#2-用户注册登录)
3. [充值流程](#3-充值流程)
4. [下单/挂单流程](#4-下单挂单流程)
5. [取现流程](#5-取现流程)
6. [WebSocket 实时数据](#6-websocket-实时数据)
7. [预测市场业务流](#7-预测市场业务流)

---

## 1. 业务流程总览

### 1.1 系统架构

```
┌──────────────────────────────────────────────────────────────────────┐
│                         前端 (React)                                  │
│   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│   │  登录页    │ │  充值页    │ │  交易页    │ │  钱包页    │   │
│   └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘   │
└──────────┼───────────────┼───────────────┼───────────────┼──────────┘
           │               │               │               │
           └───────────────┴──────┬────────┴───────────────┘
                                  │ HTTP/REST
                                  ▼
┌──────────────────────────────────────────────────────────────────────┐
│                      API Gateway (:8080)                             │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │                    路由 + 中间件                              │   │
│   │   • JWT 认证中间件   • 限流中间件   • CORS 中间件            │   │
│   └─────────────────────────────────────────────────────────────┘   │
│           │              │              │              │              │
│           ▼              ▼              ▼              ▼              │
│   ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐       │
│   │ User Svc  │  │Portfolio  │  │ Order Svc │  │ Market    │       │
│   │ :50001    │  │ :50003    │  │ :50004    │  │ :50006    │       │
│   └─────┬─────┘  └─────┬─────┘  └─────┬─────┘  └───────────┘       │
└─────────┼──────────────┼──────────────┼─────────────────────────────┘
          │              │              │
          │    gRPC      │    gRPC      │    Kafka
          ▼              ▼              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Kafka (事件总线)                              │
└─────────────────────────────────────────────────────────────────────┘
          │              │
          ▼              ▼
┌─────────────────┐  ┌─────────────────┐
│ Matching Engine │  │ ws-market-data  │
│    :50007      │  │    :50016       │
└─────────────────┘  └─────────────────┘
```

### 1.2 服务端口映射

| 服务 | 端口 | 协议 | 说明 |
|------|------|------|------|
| API Gateway | 8080 | HTTP/REST | 统一入口 |
| User Service | 50001 | gRPC | 用户管理 |
| Portfolio Service | 50003 | gRPC | 账户+持仓+清算 |
| Order Service | 50004 | gRPC | 订单管理 |
| Market Data Service | 50006 | gRPC | 行情数据 |
| Matching Engine | 50007 | gRPC | 撮合引擎 |
| Auth Service | 50009 | gRPC | 鉴权服务 |
| ws-market-data | 50016 | WebSocket | 行情推送 |

---

## 2. 用户注册登录

### 2.1 注册流程

```
用户              前端                 API Gateway           User Service
 │                  │                       │                      │
 │  输入注册信息    │                       │                      │
 │  (邮箱/密码)    │                       │                      │
 │────────────────►│                       │                      │
 │                  │                       │                      │
 │                  │   POST /api/v1/users/register                │
 │                  │   {                   │                      │
 │                  │     email,            │                      │
 │                  │     password,         │                      │
 │                  │     username          │                      │
 │                  │   }                   │                      │
 │                  │──────────────────────►│                      │
 │                  │                       │   gRPC CreateUser    │
 │                  │                       │─────────────────────►│
 │                  │                       │                      │
 │                  │                       │◄─────────────────────│
 │                  │                       │   user_id            │
 │                  │                       │                      │
 │                  │◄──────────────────────│                      │
 │                  │   { success: true }  │                      │
 │                  │                      │                      │
 │  注册成功        │                       │                      │
 │◄────────────────│                       │                      │
```

### 2.2 登录流程

```
用户              前端                 API Gateway           Auth Service          User Service
 │                  │                       │                      │                    │
 │  输入登录信息    │                       │                      │                    │
 │  (邮箱/密码)    │                       │                      │                    │
 │────────────────►│                       │                      │                    │
 │                  │                       │                      │                    │
 │                  │   POST /api/v1/users/login                   │                    │
 │                  │   {                   │                      │                    │
 │                  │     email,            │                      │                    │
 │                  │     password          │                      │                    │
 │                  │   }                   │                      │                    │
 │                  │──────────────────────►│                      │                    │
 │                  │                       │                      │                    │
 │                  │                       │   gRPC Login         │                    │
 │                  │                       │─────────────────────────────────────────►│
 │                  │                       │                      │                    │
 │                  │                       │◄─────────────────────────────────────────│
 │                  │                       │   session_id + token   │                │
 │                  │                       │                      │                    │
 │                  │                       │   gRPC ValidateToken  │                    │
 │                  │                       │──────────────────────►│                    │
 │                  │                       │                      │                    │
 │                  │                       │◄──────────────────────│                    │
 │                  │                       │   AuthContext         │                    │
 │                  │                       │                      │                    │
 │                  │◄──────────────────────│                      │                    │
 │                  │   {                   │                      │                    │
 │                  │     token,            │                      │                    │
 │                  │     user_id           │                      │                    │
 │                  │   }                   │                      │                    │
 │                  │                      │                      │                    │
 │  登录成功        │                       │                      │                    │
 │  (保存token)    │                       │                      │                    │
 ◄─────────────────────────────────────────────────────────────────────────────────────│
```

### 2.3 HTTP 接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| POST | `/api/v1/users/register` | 否 | 用户注册 |
| POST | `/api/v1/users/login` | 否 | 用户登录 |
| GET | `/api/v1/users/me` | JWT | 获取当前用户信息 |
| PUT | `/api/v1/users/me` | JWT | 更新用户信息 |

### 2.4 请求/响应示例

**注册请求:**
```json
POST /api/v1/users/register
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "username": "john_doe"
}
```

**注册响应:**
```json
{
  "success": true,
  "user_id": "usr_abc123",
  "message": "Registration successful"
}
```

**登录请求:**
```json
POST /api/v1/users/login
{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

**登录响应:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

---

## 3. 充值流程

### 3.1 充值流程 (链上充值)

```
用户              前端              API Gateway       Wallet Service      Portfolio Service
 │                  │                   │                  │                    │
 │  选择充值资产    │                   │                  │                    │
 │  (USDT/TRX)     │                   │                  │                    │
 │────────────────►│                   │                  │                    │
 │                  │                   │                  │                    │
 │                  │ GET /api/v1/wallet/deposit/address?asset=USDT           │
 │                  │──────────────────►│                  │                    │
 │                  │                   │                  │                    │
 │                  │                   │ gRPC GetDepositAddress                │
 │                  │                   │─────────────────────────────────────►│
 │                  │                   │                  │                    │
 │                  │                   │◄─────────────────────────────────────│
 │                  │                   │   deposit_address   │                │
 │                  │                   │                  │                    │
 │                  │◄─────────────────│                  │                    │
 │                  │ { address: "0x..."}│                  │                    │
 │                  │                  │                  │                    │
 │  显示充值地址    │                   │                  │                    │
 ◄─────────────────────────────────────────────────────────────────────────────│
 │                  │                   │                  │                    │
 │  向地址转账      │                   │                  │                    │
 │  (链上确认)     │                   │                  │                    │
 │─────────────────│                   │                  │                    │
 │                  │                   │                  │                    │
 │                  │                   │   区块链 Webhook  │                    │
 │                  │                   │◄──────────────────│                    │
 │                  │                   │                  │                    │
 │                  │                   │ gRPC ConfirmDeposit                  │
 │                  │                   │─────────────────────────────────────►│
 │                  │                   │                  │                    │
 │                  │                   │◄─────────────────────────────────────│
 │                  │                   │                  │                    │
 │                  │  WebSocket 通知   │                  │                    │
 │◄─────────────────│◄──────────────────│                  │                    │
 │  充值到账        │                   │                  │                    │
```

### 3.2 HTTP 接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/wallet/deposit/address` | JWT | 获取充值地址 |
| POST | `/api/v1/wallet/deposit/confirm` | Webhook | 充值确认回调 |
| GET | `/api/v1/wallet/history` | JWT | 获取钱包历史 |

### 3.3 请求/响应示例

**获取充值地址:**
```
GET /api/v1/wallet/deposit/address?asset=USDT
Authorization: Bearer <token>
```

**响应:**
```json
{
  "address": "TRC20:TMN8X2UJhbZMwF1J3x2Kj3jP1jM2J3X2U",
  "asset": "USDT",
  "network": "TRC20",
  "memo": "12345678"
}
```

---

## 4. 下单挂单流程

### 4.1 下单流程 (市价单)

```
用户              前端              API Gateway        Risk Service      Portfolio Service    Order Service    Matching Engine
 │                  │                   │                  │                   │                   │                │
 │  选择市场        │                   │                  │                   │                   │                │
 │  输入数量       │                   │                  │                   │                   │                │
 │  选择 YES/NO   │                   │                  │                   │                   │                │
 │────────────────►│                   │                  │                   │                   │                │
 │                  │                   │                  │                   │                   │                │
 │                  │ POST /api/v1/orders                                │                   │                │
 │                  │ {                   │                  │                   │                   │                │
 │                  │   market_id: 1,     │                  │                   │                   │                │
 │                  │   outcome_id: 1,    │                  │                   │                   │                │
 │                  │   side: "YES",      │                  │                   │                   │                │
 │                  │   order_type: "market",                 │                   │                   │                │
 │                  │   quantity: 100     │                  │                   │                   │                │
 │                  │ }                   │                  │                   │                   │                │
 │                  │────────────────────►│                  │                   │                   │                │
 │                  │                   │                  │                   │                   │                │
 │                  │                   │ gRPC CheckRisk   │                   │                   │                │
 │                  │                   │──────────────────────────────────────────────────────────►│                │
 │                  │                   │                  │                   │                   │                │
 │                  │                   │◄──────────────────────────────────────────────────────────│                │
 │                  │                   │   risk_ok         │                   │                   │                │
 │                  │                   │                  │                   │                   │                │
 │                  │                   │ gRPC Freeze       │                   │                   │                │
 │                  │                   │───────────────────────────────────────────────────────────►│                │
 │                  │                   │                  │                   │                   │                │
 │                  │                   │◄───────────────────────────────────────────────────────────│                │
 │                  │                   │                  │                   │                   │                │
 │                  │                   │ gRPC CreateOrder │                   │                   │                │
 │                  │                   │─────────────────────────────────────────────────────────────────────►│                │
 │                  │                   │                  │                   │                   │                │
 │                  │                   │◄─────────────────────────────────────────────────────────────────────│                │
 │                  │                   │                  │                   │                   │                │
 │                  │                   │   Kafka: order.commands                  │                   │                │
 │                  │                   │───────────────────────────────────────────────────────────────────────►│                │
 │                  │                   │                  │                   │                   │                │
 │                  │                   │                  │                   │                   │  撮合处理       │
 │                  │                   │                  │                   │                   │                │
 │                  │                   │                  │                   │      Kafka: match.events ◄────────────│
 │                  │                   │                  │                   │◄──────────────────────────────────────│
 │                  │                   │                  │                   │                               │
 │                  │                   │                  │                   │  gRPC SettleTrade                   │
 │                  │                   │                  │                   │─────────────────────────────►│        │
 │                  │                   │                  │                   │                               │
 │                  │◄──────────────────────────────────────────────────────────────────────────────────────────│
 │                  │   {                │                  │                   │                               │
 │                  │     order_id,     │                  │                   │                               │
 │                  │     status: "filled"                 │                   │                               │
 │                  │   }               │                  │                   │                               │
 │                  │                  │                  │                   │                               │
 ◄──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────│
```

### 4.2 挂单流程 (限价单)

```
用户              前端              API Gateway        Order Service     Matching Engine
 │                  │                   │                  │                │
 │  选择市场        │                   │                  │                │
 │  输入价格       │                   │                  │                │
 │  输入数量       │                   │                  │                │
 │  选择 YES/NO   │                   │                  │                │
 │────────────────►│                   │                  │                │
 │                  │                   │                  │                │
 │                  │ POST /api/v1/orders                                │                │
 │                  │ {                   │                  │                │
 │                  │   market_id: 1,     │                  │                │
 │                  │   outcome_id: 1,    │                  │                │
 │                  │   side: "YES",      │                  │                │
 │                  │   order_type: "limit",                │                │
 │                  │   price: 0.65,      │                  │                │
 │                  │   quantity: 100     │                  │                │
 │                  │ }                   │                  │                │
 │                  │────────────────────►│                  │                │
 │                  │                   │                  │                │
 │                  │                   │ gRPC PlaceOrder │                │
 │                  │                   │────────────────────────────────►│
 │                  │                   │                  │                │
 │                  │                   │◄────────────────────────────────│
 │                  │                   │   order_id (挂单中)            │
 │                  │                   │                  │                │
 │                  │                   │  Kafka: order.commands          │
 │                  │                   │────────────────────────────────►│
 │                  │                   │                  │                │
 │                  │◄──────────────────│                  │                │
 │                  │   {               │                  │                │
 │                  │     order_id,     │                  │                │
 │                  │     status: "pending"                │                │
 │                  │   }               │                  │                │
 │                  │                  │                  │                │
 ◄────────────────────────────────────────────────────────────────────────│
 │                  │                   │                  │                │
 │  订单挂在订单簿   │                   │                  │                │
 │  等待撮合        │                   │                  │                │
```

### 4.3 撤单流程

```
用户              前端              API Gateway        Order Service     Matching Engine
 │                  │                   │                  │                │
 │  选择要撤的订单   │                   │                  │                │
 │────────────────►│                   │                  │                │
 │                  │                   │                  │                │
 │                  │ DELETE /api/v1/orders/{order_id}     │                │
 │                  │────────────────────►│                  │                │
 │                  │                   │                  │                │
 │                  │                   │ gRPC CancelOrder │                │
 │                  │                   │────────────────────────────────►│
 │                  │                   │                  │                │
 │                  │                   │◄────────────────────────────────│
 │                  │                   │   success + unfreeze            │
 │                  │                   │                  │                │
 │                  │                   │  Kafka: order.commands          │
 │                  │                   │────────────────────────────────►│
 │                  │                   │                  │                │
 │                  │◄──────────────────│                  │                │
 │                  │   {               │                  │                │
 │                  │     success       │                  │                │
 │                  │   }               │                  │                │
 │                  │                  │                  │                │
 ◄────────────────────────────────────────────────────────────────────────│
```

### 4.4 HTTP 接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| POST | `/api/v1/orders` | JWT | 创建订单 |
| GET | `/api/v1/orders` | JWT | 获取订单列表 |
| GET | `/api/v1/orders/{order_id}` | JWT | 获取订单详情 |
| DELETE | `/api/v1/orders/{order_id}` | JWT | 取消订单 |
| GET | `/api/v1/accounts/balance` | JWT | 获取账户余额 |
| GET | `/api/v1/positions` | JWT | 获取持仓 |

### 4.5 请求/响应示例

**下单请求:**
```json
POST /api/v1/orders
Authorization: Bearer <token>
{
  "market_id": 1,
  "outcome_id": 1,
  "side": "YES",
  "order_type": "limit",
  "price": "0.65",
  "quantity": "100"
}
```

**下单响应:**
```json
{
  "order_id": "ord_xyz789",
  "status": "pending",
  "side": "YES",
  "order_type": "limit",
  "price": "0.65",
  "quantity": "100",
  "filled_quantity": "0",
  "created_at": "2024-01-15T10:30:00Z"
}
```

---

## 5. 取现流程

### 5.1 取现流程

```
用户              前端              API Gateway       Portfolio Service    Wallet Service
 │                  │                   │                  │                    │
 │  选择资产        │                   │                  │                    │
 │  输入金额       │                   │                  │                    │
 │  输入地址       │                   │                  │                    │
 │────────────────►│                   │                  │                    │
 │                  │                   │                  │                    │
 │                  │ POST /api/v1/wallet/withdraw                       │                    │
 │                  │ {                   │                  │                    │
 │                  │   asset: "USDT",   │                  │                    │
 │                  │   amount: "500",   │                  │                    │
 │                  │   address: "0x..." │                  │                    │
 │                  │ }                   │                  │                    │
 │                  │───────────────────►│                  │                    │
 │                  │                   │                  │                    │
 │                  │                   │ gRPC Withdraw    │                    │
 │                  │                   │────────────────────────────────────►│
 │                  │                   │                  │                    │
 │                  │                   │◄────────────────────────────────────│
 │                  │                   │   withdrawal_id   │                │
 │                  │                   │                  │                    │
 │                  │                   │ gRPC Debit       │                    │
 │                  │                   │────────────────────────────────────►│
 │                  │                   │                  │                    │
 │                  │                   │◄────────────────────────────────────│
 │                  │                   │                  │                    │
 │                  │◄──────────────────│                  │                    │
 │                  │   {                │                  │                    │
 │                  │     withdrawal_id, │                  │                    │
 │                  │     status: "pending"                │                    │
 │                  │   }               │                  │                    │
 │                  │                  │                  │                    │
 ◄─────────────────────────────────────────────────────────────────────────────│
 │                  │                   │                  │                    │
 │                  │                   │   人工审核/链上转账                   │
 │                  │                   │◄──────────────────│                    │
 │                  │                   │                  │                    │
 │                  │                   │ gRPC ConfirmWithdraw                 │
 │                  │                   │────────────────────────────────────►│
 │                  │                   │                  │                    │
 │                  │   WebSocket 通知  │                  │                    │
 │◄─────────────────│◄──────────────────│                  │                    │
 │  取现完成        │                   │                  │                    │
```

### 5.2 HTTP 接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| POST | `/api/v1/wallet/withdraw` | JWT | 申请取现 |
| GET | `/api/v1/wallet/withdraw/{id}` | JWT | 查询取现状态 |
| GET | `/api/v1/wallet/history` | JWT | 获取钱包历史 |

### 5.3 请求/响应示例

**取现请求:**
```json
POST /api/v1/wallet/withdraw
Authorization: Bearer <token>
{
  "asset": "USDT",
  "amount": "500",
  "address": "TRC20:TMN8X2UJhbZMwF1J3x2Kj3jP1jM2J3X2U"
}
```

**取现响应:**
```json
{
  "withdrawal_id": "wd_abc123",
  "asset": "USDT",
  "amount": "500",
  "fee": "1",
  "net_amount": "499",
  "status": "pending",
  "created_at": "2024-01-15T10:30:00Z"
}
```

---

## 6. WebSocket 实时数据

### 6.1 连接流程

```
用户              前端              ws-market-data
 │                  │                   │
 │  连接 WebSocket  │                   │
 │────────────────►│                   │
 │                  │                   │
 │                  │ GET /ws/market-data              │
 │                  │ Upgrade: websocket                 │
 │                  │──────────────────────────────────►│
 │                  │                   │
 │                  │◄──────────────────────────────────│
 │                  │   Connection established           │
 │                  │                   │
 │                  │                   │
 │  订阅市场        │                   │
 │  subscribe(1)   │                   │
 │────────────────►│                   │
 │                  │                   │
 │                  │   Kafka: market.events             │
 │                  │◄──────────────────────────────────│
 │                  │                   │
 │                  │                   │
 │  接收行情数据    │                   │
 ◄─────────────────│◄──────────────────│
```

### 6.2 订阅消息格式

**订阅请求:**
```json
{
  "action": "subscribe",
  "channel": "orderbook",
  "market_id": 1,
  "outcome_id": 1
}
```

**订阅响应:**
```json
{
  "action": "subscribed",
  "channel": "orderbook",
  "market_id": 1,
  "outcome_id": 1
}
```

**数据推送:**
```json
{
  "channel": "orderbook",
  "market_id": 1,
  "outcome_id": 1,
  "data": {
    "asks": [["0.66", "100"], ["0.67", "200"]],
    "bids": [["0.64", "150"], ["0.63", "300"]]
  },
  "timestamp": 1705312200000
}
```

### 6.3 支持的 Channel

| Channel | 说明 | 数据内容 |
|---------|------|---------|
| `orderbook` | 订单簿 | 买方/卖方深度 |
| `trade` | 成交 | 最新成交记录 |
| `ticker` | 行情 | 24h 统计 |
| `kline_{interval}` | K线 | OHLCV 数据 |

---

## 附录: 错误码

### 认证错误

| 错误码 | 说明 |
|--------|------|
| `AUTH_INVALID_TOKEN` | Token 无效 |
| `AUTH_TOKEN_EXPIRED` | Token 已过期 |
| `AUTH_USER_NOT_FOUND` | 用户不存在 |
| `AUTH_INVALID_PASSWORD` | 密码错误 |

### 交易错误

| 错误码 | 说明 |
|--------|------|
| `INSUFFICIENT_BALANCE` | 余额不足 |
| `INSUFFICIENT_POSITION` | 持仓不足 |
| `ORDER_NOT_FOUND` | 订单不存在 |
| `ORDER_CANNOT_CANCEL` | 订单无法取消 |
| `MARKET_CLOSED` | 市场已关闭 |

### 钱包错误

| 错误码 | 说明 |
|--------|------|
| `INVALID_ADDRESS` | 地址无效 |
| `MIN_WITHDRAWAL` | 低于最小取现金额 |
| `MAX_WITHDRAWAL` | 超过最大取现金额 |
| `PENDING_WITHDRAWAL` | 存在待处理取现 |

---

## 附录: 数据库表

### user_sessions (会话表)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | VARCHAR(36) | 会话ID |
| user_id | VARCHAR(36) | 用户ID |
| token_hash | VARCHAR(64) | Token 哈希 |
| refresh_token_hash | VARCHAR(64) | 刷新Token 哈希 |
| device_id | VARCHAR(64) | 设备ID |
| ip_address | VARCHAR(45) | IP地址 |
| expires_at | TIMESTAMP | 过期时间 |
| created_at | TIMESTAMP | 创建时间 |

### api_keys (API Key表)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | VARCHAR(36) | Key ID |
| user_id | VARCHAR(36) | 用户ID |
| key_hash | VARCHAR(64) | Key 哈希 |
| secret_hash | VARCHAR(64) | Secret 哈希 |
| name | VARCHAR(100) | Key 名称 |
| permissions | JSON | 权限列表 |
| is_active | BOOLEAN | 是否激活 |
| last_used_at | TIMESTAMP | 最后使用时间 |
| expires_at | TIMESTAMP | 过期时间 |
| created_at | TIMESTAMP | 创建时间 |

### deposits (充值记录)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | VARCHAR(36) | 充值ID |
| user_id | VARCHAR(36) | 用户ID |
| asset | VARCHAR(10) | 资产类型 |
| amount | DECIMAL | 金额 |
| address | VARCHAR(255) | 充值地址 |
| tx_hash | VARCHAR(255) | 交易哈希 |
| confirmations | INT | 确认数 |
| status | VARCHAR(20) | 状态 |
| created_at | TIMESTAMP | 创建时间 |
| confirmed_at | TIMESTAMP | 确认时间 |

### withdrawals (取现记录)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | VARCHAR(36) | 取现ID |
| user_id | VARCHAR(36) | 用户ID |
| asset | VARCHAR(10) | 资产类型 |
| amount | DECIMAL | 金额 |
| fee | DECIMAL | 手续费 |
| net_amount | DECIMAL | 到账金额 |
| address | VARCHAR(255) | 目标地址 |
| tx_hash | VARCHAR(255) | 交易哈希 |
| status | VARCHAR(20) | 状态 |
| reviewed_at | TIMESTAMP | 审核时间 |
| created_at | TIMESTAMP | 创建时间 |

---

## 7. 预测市场业务流

### 7.1 市场生命周期

预测市场从创建到结算的完整生命周期：

```
创建市场 -> 开放交易 -> 关闭市场 -> 结算市场 -> 赔付分发
```

#### 市场状态机

```
┌─────────┐    创建     ┌─────────┐    关闭    ┌─────────┐   结算   ┌───────────┐
│  Draft  │────────────▶│  Open   │───────────▶│ Closed  │────────▶│ Resolved  │
└─────────┘             └─────────┘            └─────────┘         └───────────┘
                              │                                       │
                              │ 取消                                  │ 取消
                              ▼                                       ▼
                        ┌───────────┐                           ┌───────────┐
                        │ Cancelled │                           │ Cancelled │
                        └───────────┘                           └───────────┘
```

| 状态 | 说明 | 允许操作 |
|------|------|---------|
| `open` | 开放交易 | 下单、撤单、添加选项 |
| `closed` | 停止交易 | 等待结算，不可下单 |
| `resolved` | 已结算 | 计算赔付、分发资金 |
| `cancelled` | 已取消 | 退回冻结资金 |

### 7.2 创建市场流程

```
管理员/运营              Prediction Market Service
    │                              │
    │  创建市场                    │
    │  (问题/选项/时间)            │
    │─────────────────────────────▶│
    │                              │
    │                              │  写入数据库
    │                              │  prediction_markets
    │                              │  market_outcomes
    │                              │
    │◄─────────────────────────────│
    │   { market_id, outcomes }    │
```

**HTTP 接口:**

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| POST | `/api/v1/markets` | Admin | 创建市场 |

**请求示例:**
```json
POST /api/v1/markets
Authorization: Bearer <admin_token>
{
  "question": "Will BTC exceed $100k by end of 2024?",
  "description": "Predict whether Bitcoin will surpass $100,000",
  "category": "crypto",
  "image_url": "https://...",
  "start_time": 1704067200000,
  "end_time": 1735689600000,
  "outcomes": [
    { "name": "Yes", "description": "BTC > $100k" },
    { "name": "No", "description": "BTC <= $100k" }
  ]
}
```

### 7.3 市场查询流程

```
用户/前端              API Gateway         Prediction Market Service
    │                       │                              │
    │  查市场列表            │                              │
    │──────────────────────▶│                              │
    │                       │  gRPC ListMarkets            │
    │                       │─────────────────────────────▶│
    │                       │                              │
    │                       │◄─────────────────────────────│
    │                       │   markets[]                  │
    │◄──────────────────────│                              │
    │   { markets, total }  │                              │
```

**HTTP 接口:**

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/markets` | 否 | 市场列表 |
| GET | `/api/v1/markets/{market_id}` | 否 | 市场详情 |
| GET | `/api/v1/markets/{market_id}/outcomes` | 否 | 选项列表 |
| GET | `/api/v1/markets/{market_id}/price` | 否 | 当前价格 |
| GET | `/api/v1/markets/{market_id}/depth` | 否 | 订单簿深度 |

### 7.4 关闭市场流程

当市场到达结束时间或需要提前停止交易时：

```
管理员/系统            Prediction Market Service
    │                              │
    │  关闭市场                    │
    │  (market_id)                 │
    │─────────────────────────────▶│
    │                              │
    │                              │  更新状态: open -> closed
    │                              │
    │◄─────────────────────────────│
    │   { success }                │
    │                              │
    │                              │  Kafka: market.closed
    │                              │──────────▶ Matching Engine
    │                              │            (停止接受新订单)
```

### 7.5 结算市场流程

当事件结果确定后，管理员结算市场：

```
管理员                Prediction Market Service      Portfolio Service
    │                              │                              │
    │  结算市场                    │                              │
    │  (winning_outcome_id)        │                              │
    │─────────────────────────────▶│                              │
    │                              │                              │
    │                              │  更新状态: closed -> resolved│
    │                              │                              │
    │                              │  gRPC CalculatePayout        │
    │                              │─────────────────────────────▶│
    │                              │                              │
    │                              │◄─────────────────────────────│
    │                              │   payouts[]                  │
    │                              │                              │
    │                              │  Kafka: market.resolved      │
    │                              │──────────▶ ws-market-data    │
    │                              │            (推送结果)         │
    │◄─────────────────────────────│                              │
    │   { success, resolution }    │                              │
```

**结算规则（二元预测市场）:**
- 获胜选项持有者：每份获得 1 USDC
- 失败选项持有者：每份获得 0 USDC
- 赔付 = 持仓数量 × 1 (如果持有获胜选项)

**HTTP 接口:**

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| POST | `/api/v1/markets/{market_id}/resolve` | Admin | 结算市场 |
| GET | `/api/v1/markets/{market_id}/payout` | JWT | 查询赔付 |

### 7.6 用户持仓查询

```
用户/前端              API Gateway         Prediction Market Service
    │                       │                              │
    │  查我的持仓            │                              │
    │──────────────────────▶│                              │
    │                       │  gRPC GetUserPositions       │
    │                       │  (user_id, market_id?)       │
    │                       │─────────────────────────────▶│
    │                       │                              │
    │                       │◄─────────────────────────────│
    │                       │   positions[]                │
    │◄──────────────────────│                              │
    │   { positions }       │                              │
```

**HTTP 接口:**

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/positions` | JWT | 用户所有持仓 |
| GET | `/api/v1/positions?market_id=1` | JWT | 某市场持仓 |

**响应示例:**
```json
{
  "positions": [
    {
      "id": 1,
      "market_id": 1,
      "outcome_id": 1,
      "quantity": "100",
      "avg_price": "0.65",
      "created_at": 1705312200000
    }
  ]
}
```

### 7.7 gRPC 接口清单

| 接口 | 服务 | 说明 |
|------|------|------|
| CreateMarket | PredictionMarketService | 创建市场 |
| UpdateMarket | PredictionMarketService | 更新市场信息 |
| CloseMarket | PredictionMarketService | 关闭市场 |
| GetMarket | PredictionMarketService | 获取市场详情 |
| ListMarkets | PredictionMarketService | 市场列表 |
| AddOutcome | PredictionMarketService | 添加选项 |
| GetOutcomes | PredictionMarketService | 获取选项 |
| GetMarketPrice | PredictionMarketService | 获取价格 |
| GetMarketDepth | PredictionMarketService | 获取深度 |
| ResolveMarket | PredictionMarketService | 结算市场 |
| CalculatePayout | PredictionMarketService | 计算赔付 |
| GetUserPositions | PredictionMarketService | 用户持仓 |
