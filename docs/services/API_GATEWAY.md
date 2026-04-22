# API Gateway - API 网关

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 8080 (HTTP), 8081 (WS) |
| 协议 | HTTP + WebSocket |
| 数据库 | 无 |
| 依赖 | 所有 gRPC 服务 |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 路由转发 | HTTP -> gRPC 转换 |
| 鉴权 | Token 验证 |
| 限流 | 请求限流 |
| WS 管理 | WebSocket 连接管理 |

## 2. HTTP 路由

### 2.1 用户服务

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/v1/auth/register | 注册 |
| POST | /api/v1/auth/login | 登录 |
| POST | /api/v1/auth/logout | 登出 |
| GET | /api/v1/users/me | 获取用户信息 |

### 2.2 订单服务

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/v1/orders | 创建订单 |
| DELETE | /api/v1/orders/{order_id} | 取消订单 |
| GET | /api/v1/orders/{order_id} | 查询订单 |
| GET | /api/v1/orders | 用户订单列表 |

### 2.3 预测市场服务

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/markets | 市场列表 |
| GET | /api/v1/markets/{id} | 市场详情 |
| GET | /api/v1/markets/{id}/outcomes | 市场结果选项 |
| GET | /api/v1/markets/{id}/klines | K线 |
| GET | /api/v1/markets/{id}/depth | 深度 |
| GET | /api/v1/markets/{id}/trades | 成交记录 |
| GET | /api/v1/markets/{id}/ticker | 24h行情统计 |
| GET | /api/v1/markets/categories | 市场分类列表 |

### 2.4 钱包与账户服务

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/account/balances | 获取所有资产余额 |
| GET | /api/v1/account/balances/{asset} | 获取单个资产余额 |
| POST | /api/v1/wallet/deposit-address | 获取充值地址 |
| POST | /api/v1/wallet/withdraw | 申请提现 |
| GET | /api/v1/wallet/deposits | 充值记录 |
| GET | /api/v1/wallet/withdrawals | 提现记录 |

### 2.5 管理后台

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/v1/admin/users | 用户列表 |
| GET | /api/v1/admin/users/{id} | 用户详情 |
| POST | /api/v1/admin/users/{id}/freeze | 冻结用户 |
| POST | /api/v1/admin/users/{id}/unfreeze | 解冻用户 |
| GET | /api/v1/admin/kyc | KYC申请列表 |
| POST | /api/v1/admin/kyc/{id}/review | 审核KYC |
| GET | /api/v1/admin/withdrawals | 待审核提现 |
| POST | /api/v1/admin/withdrawals/{id}/approve | 批准提现 |
| POST | /api/v1/admin/withdrawals/{id}/reject | 拒绝提现 |
| POST | /api/v1/admin/markets | 创建市场 |
| PUT | /api/v1/admin/markets/{id} | 更新市场 |
| POST | /api/v1/admin/markets/{id}/close | 关闭市场 |
| POST | /api/v1/admin/markets/{id}/resolve | 结算市场 |
| GET | /api/v1/admin/config/fee | 手续费配置 |
| PUT | /api/v1/admin/config/fee | 设置手续费 |
| GET | /api/v1/admin/config/risk | 风控配置 |
| PUT | /api/v1/admin/config/risk | 设置风控 |
| GET | /api/v1/admin/stats | 系统统计 |

### 2.6 WebSocket

| 路径 | 认证 | 说明 |
|------|------|------|
| /ws/market | 无需认证 | 公开行情推送 (K线/深度/成交/ticker) |
| /ws/order | 必须认证 | 用户私有订单状态变更，需携带 Token |
| /ws/prediction | 无需认证 | 市场事件推送 (市场关闭/结算等) |

> **WebSocket 认证方式**: 认证型 WS 连接时需在 URL 中携带 token 参数，如 `/ws/order?token={jwt_token}`，或连接后发送认证消息。未认证的 /ws/order 连接将被拒绝。

## 3. 认证

| 方式 | 说明 |
|------|------|
| Bearer Token | Header Authorization: Bearer {token} |
| API Key | Header X-API-Key: {key} |

## 4. 限流

| 级别 | 限制 |
|------|------|
| IP | 100/minute |
| User | 1000/minute |
| Endpoint | 100/minute |

## 5. 配置

```yaml
service:
  http_port: 8080
  ws_port: 8081

grpc:
  user_service: localhost:50001
  wallet_service: localhost:50002
  order_service: localhost:50003
  risk_service: localhost:50004
  position_service: localhost:50005
  market_data_service: localhost:50006
  admin_service: localhost:50007
  clearing_service: localhost:50008
  prediction_market_service: localhost:50010
  account_service: localhost:50019

jwt:
  secret: "${JWT_SECRET}"

rate_limiter:
  ip_limit: 100
  user_limit: 1000
```

## 6. 目录结构

```
crates/api-gateway/
├── Cargo.toml, config/, src/
```
