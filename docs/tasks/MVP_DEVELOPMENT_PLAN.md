# MVP 开发计划（业务流程驱动版）

> 版本: v2.0
> 日期: 2026-04-24
> 核心思想：**纵向穿透业务流程**，每个 Sprint 交付一个可验证的端到端业务闭环

---

## 1. 为什么采用业务流程驱动

### 1.1 传统 Phase 方案的问题

| 问题 | 说明 |
|------|------|
| 横向切面过宽 | Phase 1 做完 Gateway 所有接口，但下游服务很多未实现，无法验证 |
| 长期无闭环 | 做了 3 周还无法完成一次完整交易 |
| 返工风险高 | 先假设下游接口，实际开发时 proto/模型变更，Gateway 需重写 |
| 业务视角缺失 | 按服务拆分容易忽略跨服务数据一致性 |

### 1.2 业务流程驱动的优势

```
传统 Phase:  Gateway → Portfolio → Risk → Order → Matching → WebSocket
             (3周无验证) (再做3周) ...

业务流程:    Flow 1 注册登录(1天) → Flow 2 看行情(1天) → Flow 3 下单闭环(2周)
             每一天都有可 curl 验证的端到端链路
```

**核心原则：**
1. **纵向穿透**：一个 Flow 涉及的所有服务，只实现这个 Flow 需要的最小子集
2. **每轮可验证**：每个 Flow 完成后，能用 `curl` 或浏览器跑通完整链路
3. **服务随流生长**：服务不是一次性设计完整，而是在不同 Flow 中逐步增强

---

## 2. 业务流程总览

### 2.1 MVP 最小闭环

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           MVP 最小业务闭环                                   │
└─────────────────────────────────────────────────────────────────────────────┘

  [Flow 1]        [Flow 2]         [Flow 3]          [Flow 4]         [Flow 5]
  用户鉴权   →   查看行情   →    下单交易   →     查看持仓    →    充值提现
     │              │                │                 │                │
     ▼              ▼                ▼                 ▼                ▼
  注册/登录     市场列表        下单/冻结         账户余额        充值地址
  JWT Token     订单簿深度      撮合/成交         持仓列表        提现申请
                K线/Ticker      清算/解冻         交易流水

  [Flow 6]        [Flow 7]
  市场结算   →   实时推送
     │              │
     ▼              ▼
  管理员结算      WebSocket
  用户派彩        行情/订单状态
```

### 2.2 服务参与矩阵

| 服务 | Flow 1 | Flow 2 | Flow 3 | Flow 4 | Flow 5 | Flow 6 | Flow 7 |
|------|:------:|:------:|:------:|:------:|:------:|:------:|:------:|
| API Gateway | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| User Service | ✅ | - | - | - | - | - | - |
| Auth Service | ✅ | - | - | - | - | - | - |
| Prediction Market Service | - | ✅ | - | - | - | ✅ | - |
| Market Data Service | - | ✅ | - | - | - | - | - |
| Order Service | - | - | ✅ | - | - | - | - |
| Risk Service | - | - | ✅ | - | - | - | - |
| Portfolio Service | - | - | ✅ | ✅ | ✅ | ✅ | - |
| Matching Engine | - | - | ✅ | - | - | - | - |
| Wallet Service | - | - | - | - | ✅ | - | - |
| ws-market-data | - | - | - | - | - | - | ✅ |
| ws-order | - | - | - | - | - | - | ✅ |

> ✅ = 该 Flow 必须此服务参与

---

## 3. Flow 详细设计

---

### Flow 1: 用户注册/登录/鉴权（认证闭环）

#### 3.1.1 目标
用户能注册账号，登录后获得 JWT Token，并用 Token 访问受保护接口。

#### 3.1.2 流程时序

```
用户/前端        API Gateway        User Service        Auth Service
   │                 │                  │                   │
   │  注册           │                  │                   │
   │────────────────►│                  │                   │
   │                 │  gRPC CreateUser │                   │
   │                 │─────────────────►│                   │
   │                 │◄─────────────────│                   │
   │◄────────────────│                  │                   │
   │                 │                  │                   │
   │  登录           │                  │                   │
   │────────────────►│                  │                   │
   │                 │  gRPC Login      │                   │
   │                 │───────────────────────────────────────│
   │                 │  (验证密码获取 user_id)                │
   │                 │◄───────────────────────────────────────│
   │                 │                  │                   │
   │                 │  gRPC Login      │                   │
   │                 │  (user_id → JWT) │                   │
   │                 │──────────────────────────────────────►│
   │                 │◄──────────────────────────────────────│
   │◄────────────────│  {access_token, refresh_token}        │
   │                 │                  │                   │
   │  访问受保护接口  │                  │                   │
   │────────────────►│                  │                   │
   │  Authorization  │                  │                   │
   │  Bearer <token> │                  │                   │
   │                 │  gRPC ValidateToken                   │
   │                 │──────────────────────────────────────►│
   │                 │◄──────────────────────────────────────│
   │                 │  {valid, user_id}                     │
   │◄────────────────│                  │                   │
```

#### 3.1.3 服务功能清单

| 服务 | 当前状态 | Flow 1 需要的能力 | 工作量 |
|------|---------|------------------|--------|
| User Service | ✅ Server 完整 | 已有：CreateUser, Login(验证密码) | 0 |
| Auth Service | ✅ Server 完整 | 已有：Login(签发JWT), ValidateToken, RefreshToken, Logout | 0 |
| API Gateway | 🟡 部分 | **升级：** login 改调 Auth Service；**新增：** logout, refresh_token；**升级：** middleware auth 改调 Auth Service ValidateToken | 1天 |

#### 3.1.4 验收标准
- [ ] `POST /api/v1/users/register` 成功注册
- [ ] `POST /api/v1/users/login` 返回 access_token + refresh_token
- [ ] `POST /api/v1/users/refresh` 用 refresh_token 换取新 access_token
- [ ] `POST /api/v1/users/logout` 使 token 失效
- [ ] 带有效 Token 访问 `GET /api/v1/users/me` 成功
- [ ] 带无效 Token 访问受保护接口返回 401

---

### Flow 2: 查看市场与行情（数据展示闭环）

#### 3.2.1 目标
用户无需登录即可浏览预测市场列表、查看市场详情、订单簿深度、K线和成交记录。

#### 3.2.2 流程时序

```
用户/前端        API Gateway        Prediction Market Svc    Market Data Service
   │                 │                      │                      │
   │  市场列表       │                      │                      │
   │────────────────►│                      │                      │
   │                 │  gRPC ListMarkets    │                      │
   │                 │─────────────────────►│                      │
   │                 │◄─────────────────────│                      │
   │◄────────────────│                      │                      │
   │                 │                      │                      │
   │  市场详情       │                      │                      │
   │────────────────►│                      │                      │
   │                 │  gRPC GetMarket      │                      │
   │                 │─────────────────────►│                      │
   │                 │◄─────────────────────│                      │
   │◄────────────────│                      │                      │
   │                 │                      │                      │
   │  订单簿深度     │                      │                      │
   │────────────────►│                      │                      │
   │                 │  gRPC GetOrderBook   │                      │
   │                 │─────────────────────────────────────────────►│
   │                 │◄─────────────────────────────────────────────│
   │◄────────────────│                      │                      │
   │                 │                      │                      │
   │  K线数据        │                      │                      │
   │────────────────►│                      │                      │
   │                 │  gRPC GetKlines      │                      │
   │                 │─────────────────────────────────────────────►│
   │                 │◄─────────────────────────────────────────────│
   │◄────────────────│                      │                      │
```

#### 3.2.3 服务功能清单

| 服务 | 当前状态 | Flow 2 需要的能力 | 工作量 |
|------|---------|------------------|--------|
| Prediction Market Service | ✅ Server 完整 | 已有：ListMarkets, GetMarket, GetOutcomes, GetMarketPrice, GetMarketDepth, GetUserPositions | 0 |
| Market Data Service | ✅ Server 完整 | 已有：GetOrderBook, Get24hStats, GetKlines, GetRecentTrades | 0 |
| API Gateway | 🟡 部分 | **已有：** list_markets, get_market, get_market_outcomes, get_market_price, get_depth, get_ticker, get_kline, get_recent_trades | 0 |

#### 3.2.4 验收标准
- [ ] `GET /api/v1/markets` 返回市场列表（无需认证）
- [ ] `GET /api/v1/markets/:id` 返回市场详情+选项
- [ ] `GET /api/v1/markets/:id/price` 返回各选项当前价格
- [ ] `GET /api/v1/market/depth?market_id=1` 返回订单簿
- [ ] `GET /api/v1/market/ticker?market_id=1` 返回24h统计
- [ ] `GET /api/v1/market/kline?market_id=1&interval=1h` 返回K线
- [ ] `GET /api/v1/market/trades?market_id=1` 返回近期成交

---

### Flow 3: 下单交易闭环（核心交易链路）

#### 3.3.1 目标
用户能下限价/市价单，订单经过风控检查、冻结保证金、进入撮合引擎、成交后更新持仓和账户。

#### 3.3.2 流程时序

```
用户      Gateway     Order Svc    Risk Svc    Portfolio Svc    Kafka    Matching Engine
 │           │            │            │              │            │            │
 │ 下单     │            │            │              │            │            │
 │─────────►│            │            │              │            │            │
 │          │            │            │              │            │            │
 │          │  gRPC      │            │              │            │            │
 │          │ CreateOrder│            │              │            │            │
 │          │───────────►│            │              │            │            │
 │          │            │            │              │            │            │
 │          │            │ gRPC       │              │            │            │
 │          │            │ CheckOrder │              │            │            │
 │          │            │───────────►│              │            │            │
 │          │            │◄───────────│              │            │            │
 │          │            │  pass/fail │              │            │            │
 │          │            │            │              │            │            │
 │          │            │ gRPC       │              │            │            │
 │          │            │ Freeze     │              │            │            │
 │          │            │───────────────────────────►│            │            │
 │          │            │◄───────────────────────────│            │            │
 │          │            │  冻结成功   │              │            │            │
 │          │            │            │              │            │            │
 │          │            │ 保存订单    │              │            │            │
 │          │            │ 写入DB      │              │            │            │
 │          │            │            │              │            │            │
 │          │◄───────────│  order_id  │              │            │            │
 │◄─────────│            │  status=pending            │            │            │
 │          │            │            │              │            │            │
 │          │            │  Kafka     │              │            │            │
 │          │            │ order.commands            │            │            │
 │          │            │────────────────────────────────────────►│            │
 │          │            │            │              │            │            │
 │          │            │            │              │            │ 撮合处理    │
 │          │            │            │              │            │            │
 │          │            │            │              │            │◄───────────│
 │          │            │            │              │            │ match.events│
 │          │            │            │              │            │            │
 │          │            │            │              │◄───────────│            │
 │          │            │            │  gRPC        │            │            │
 │          │            │            │ SettleTrade  │            │            │
 │          │            │            │─────────────►│            │            │
 │          │            │            │              │ 更新持仓    │            │
 │          │            │            │              │ 解冻/扣减   │            │
 │          │            │            │◄─────────────│            │            │
 │          │            │            │              │            │            │
 │          │            │ gRPC       │              │            │            │
 │          │            │ UpdateOrder│              │            │            │
 │          │            │ 更新状态    │              │            │            │
 │          │            │ filled     │              │            │            │
```

#### 3.3.3 服务功能清单

| 服务 | 当前状态 | Flow 3 需要的能力 | 工作量 |
|------|---------|------------------|--------|
| Order Service | ✅ gRPC Server | **升级：** CreateOrder 集成 Risk.CheckOrder → Portfolio.Freeze → 写DB → 发 Kafka | 3天 |
| Risk Service | 🔴 Lib only | **新建：** CheckOrder gRPC（单笔限额/日限额/用户状态） | 2天 |
| Portfolio Service | 🔴 Lib only | **新建：** Freeze/Unfreeze/SettleTrade gRPC + DB | 5天 |
| Matching Engine | 🟡 Core 完成 | **升级：** 消费 Kafka order.commands → 撮合 → 发 match.events | 3天 |
| API Gateway | ✅ | 已有 create_order, cancel_order, get_order, get_orders | 0 |

#### 3.3.4 关键设计决策

**预测市场价格转换（方案1: 单一 YES 订单簿）**

```
用户下单: "买 YES @ 0.65" → 内部: YES 簿 Bid 0.65
用户下单: "卖 NO  @ 0.35" → 内部: YES 簿 Bid 0.65 (SCALE - 0.35)
用户下单: "买 NO  @ 0.35" → 内部: YES 簿 Ask 0.65
用户下单: "卖 YES @ 0.65" → 内部: YES 簿 Ask 0.65
```

**冻结规则**
- 限价买：冻结 `价格 × 数量` USDC
- 限价卖：冻结对应 outcome 的持仓数量
- 市价单：按盘口最坏价格预估冻结

#### 3.3.5 验收标准
- [ ] `POST /api/v1/orders` 下单后返回订单号，状态 pending
- [ ] Risk Service 超限订单被拒绝（如单笔 > 1000 USDC）
- [ ] Portfolio Service 正确冻结保证金
- [ ] Matching Engine 消费到 Kafka 订单并完成撮合
- [ ] Portfolio Service 消费 match.events 完成清算
- [ ] `GET /api/v1/orders/:id` 订单状态变为 filled
- [ ] `GET /api/v1/accounts/balance` 余额正确更新
- [ ] `GET /api/v1/positions` 持仓正确更新

---

### Flow 4: 账户与持仓查询（资产查看闭环）

#### 3.4.1 目标
用户能查看账户余额、持仓列表、交易流水。

#### 3.4.2 流程时序

```
用户/前端        API Gateway        Portfolio Service
   │                 │                      │
   │  查余额         │                      │
   │────────────────►│                      │
   │                 │  gRPC GetBalance     │
   │                 │─────────────────────►│
   │                 │◄─────────────────────│
   │◄────────────────│                      │
   │                 │                      │
   │  查持仓         │                      │
   │────────────────►│                      │
   │                 │  gRPC GetPositions   │
   │                 │─────────────────────►│
   │                 │◄─────────────────────│
   │◄────────────────│                      │
   │                 │                      │
   │  查流水         │                      │
   │────────────────►│                      │
   │                 │  gRPC GetLedger      │
   │                 │─────────────────────►│
   │                 │◄─────────────────────│
   │◄────────────────│                      │
```

#### 3.4.3 服务功能清单

| 服务 | 当前状态 | Flow 4 需要的能力 | 工作量 |
|------|---------|------------------|--------|
| Portfolio Service | 🔴 Lib only | **新建：** GetBalance, GetPositions, GetLedger gRPC + DB | 3天 |
| API Gateway | 🟡 部分 | **升级：** get_balance, get_positions 从 mock 改为调 Portfolio Service | 0.5天 |

#### 3.4.4 验收标准
- [ ] `GET /api/v1/accounts/balance` 返回 available / frozen / equity
- [ ] `GET /api/v1/positions` 返回各市场持仓（quantity, avg_price）
- [ ] `GET /api/v1/accounts/ledger` 返回流水（deposit, withdraw, trade, fee）

---

### Flow 5: 充值与提现（资金闭环）

#### 3.5.1 目标
用户能获取充值地址、链上充值到账、申请提现。

#### 3.5.2 流程时序

```
用户      Gateway     Wallet Svc     Portfolio Svc     区块链/Webhook
 │           │            │                │                │
 │ 充值     │            │                │                │
 │─────────►│            │                │                │
 │          │ gRPC       │                │                │
 │          │ GetDepositAddress          │                │
 │          │───────────►│                │                │
 │          │◄───────────│                │                │
 │◄─────────│ address    │                │                │
 │          │            │                │                │
 │  转账    │            │                │                │
 │────────────────────────────────────────────────────────►│
 │          │            │                │                │
 │          │            │                │  Webhook       │
 │          │            │◄────────────────────────────────│
 │          │            │  ConfirmDeposit│                │
 │          │            │  user_id       │                │
 │          │            │  amount        │                │
 │          │            │                │                │
 │          │            │ gRPC Credit    │                │
 │          │            │───────────────────────────────►│
 │          │            │                │                │
 │          │   WebSocket/Push 通知       │                │
 │◄─────────│◄────────────────────────────────────────────│
 │          │            │                │                │
 │ 提现     │            │                │                │
 │─────────►│            │                │                │
 │          │ gRPC       │                │                │
 │          │ Withdraw   │                │                │
 │          │───────────►│                │                │
 │          │            │ gRPC Debit     │                │
 │          │            │───────────────────────────────►│
 │          │            │                │                │
 │          │◄───────────│ withdrawal_id  │                │
 │◄─────────│ status=pending             │                │
```

#### 3.5.3 服务功能清单

| 服务 | 当前状态 | Flow 5 需要的能力 | 工作量 |
|------|---------|------------------|--------|
| Wallet Service | 🔴 Lib only | **新建：** GetDepositAddress, ConfirmDeposit, Withdraw, GetWithdrawStatus gRPC + DB | 4天 |
| Portfolio Service | 🟡 Flow 3/4 新建 | **升级：** Credit（充值入账）, Debit（提现扣减） | 1天 |
| API Gateway | 🟡 部分 | **升级：** get_deposit_address, withdraw, get_wallet_history 从 mock 改为调 Wallet Service | 0.5天 |

#### 3.5.4 MVP 裁剪
- 链上充值监听：用定时轮询代替实时 Webhook
- 提现审核：MVP 自动通过（无人工审核）
- 冷钱包/离线签名：MVP 使用热钱包

#### 3.5.5 验收标准
- [ ] `GET /api/v1/wallet/deposit/address?asset=USDT` 返回地址
- [ ] 模拟充值后余额增加
- [ ] `POST /api/v1/wallet/withdraw` 申请提现，余额扣减
- [ ] `GET /api/v1/wallet/history` 显示充值/提现记录

---

### Flow 6: 市场结算与派彩（预测市场特色闭环）

#### 3.6.1 目标
管理员能结算市场，系统向获胜选项持有者派发赔付。

#### 3.6.2 流程时序

```
管理员      API Gateway     Prediction Market Svc    Portfolio Svc
   │            │                    │                    │
   │ 结算市场   │                    │                    │
   │───────────►│                    │                    │
   │            │ gRPC ResolveMarket │                    │
   │            │ (winning_outcome)  │                    │
   │            │───────────────────►│                    │
   │            │                    │                    │
   │            │                    │ 更新状态: resolved │
   │            │                    │                    │
   │            │                    │ gRPC CalculatePayout│
   │            │                    │───────────────────►│
   │            │                    │                    │
   │            │                    │◄───────────────────│
   │            │                    │ payouts[]           │
   │            │                    │                    │
   │            │                    │ 逐个 gRPC Credit    │
   │            │                    │───────────────────►│
   │            │                    │                    │
   │◄───────────│ {success}         │                    │
   │            │                    │                    │
   │            │   用户查询余额增加  │                    │
```

#### 3.6.3 服务功能清单

| 服务 | 当前状态 | Flow 6 需要的能力 | 工作量 |
|------|---------|------------------|--------|
| Prediction Market Service | ✅ | 已有：ResolveMarket, CalculatePayout, GetUserPositions | 0 |
| Portfolio Service | 🟡 Flow 3/4 新建 | **升级：** Credit（派彩入账） | 0.5天 |
| API Gateway | 🟡 | **新增：** `POST /api/v1/admin/markets/:id/resolve`（Admin 认证） | 0.5天 |

#### 3.6.4 结算规则
- 二元市场：获胜选项 1 USDC/份，失败选项 0 USDC/份
- 多选项市场：仅获胜选项获得赔付

#### 3.6.5 验收标准
- [ ] `POST /api/v1/admin/markets/1/resolve` 管理员结算市场
- [ ] 市场状态变为 resolved
- [ ] 持有获胜选项的用户余额增加（持仓数量 × 1 USDC）
- [ ] 持有失败选项的用户持仓清零

---

### Flow 7: WebSocket 实时推送（体验增强闭环）

#### 3.7.1 目标
用户能在页面上实时看到订单簿变化、成交记录、自己的订单状态更新。

#### 3.7.2 流程时序

```
用户/前端      ws-market-data      Kafka      Matching Engine    Order Service
   │               │                │                │                │
   │ 连接 WS       │                │                │                │
   │──────────────►│                │                │                │
   │               │                │                │                │
   │  subscribe    │                │                │                │
   │  market_id=1  │                │                │                │
   │──────────────►│                │                │                │
   │               │                │                │                │
   │               │◄───────────────│ match.events   │                │
   │◄──────────────│  orderbook     │                │                │
   │               │  trade         │                │                │
   │               │  ticker        │                │                │
   │               │                │                │                │
   │               │◄───────────────│ order_events   │                │
   │               │                │                │                │
   │◄──────────────│ (用户自己的)    │                │                │
   │               │  order_update  │                │                │
```

#### 3.7.3 服务功能清单

| 服务 | 当前状态 | Flow 7 需要的能力 | 工作量 |
|------|---------|------------------|--------|
| ws-market-data | ✅ 已创建目录 | **新建：** Kafka Consumer（match.events → 推送 orderbook/trade/ticker/kline） | 2天 |
| ws-order | - | **新建：** Kafka Consumer（order_events → 按 user_id 推送订单状态） | 2天 |
| Matching Engine | 🟡 | **升级：** 撮合后发送 match.events 到 Kafka | 1天 |
| Order Service | 🟡 | **升级：** 订单状态变更时发送 order_events 到 Kafka | 1天 |

#### 3.7.4 验收标准
- [ ] WebSocket 连接 `ws://gateway/ws/market-data`
- [ ] 订阅 orderbook 后，下单时能实时看到深度变化
- [ ] 订阅 trades 后，成交时实时看到记录
- [ ] 订阅 orders 后，只能收到自己的订单状态更新

---

## 4. Sprint 排期

### 4.1 时间线

```
Week 1        Week 2        Week 3        Week 4        Week 5        Week 6
  │             │             │             │             │             │
  ▼             ▼             ▼             ▼             ▼             ▼
┌─────┐      ┌─────┐      ┌─────────┐   ┌─────┐      ┌─────┐      ┌─────┐
│Flow1│  →   │Flow2│  →   │  Flow3  │ → │Flow4│  →   │Flow5│  →   │Flow6│
│1天  │      │1天  │      │  2周    │   │3天  │      │1周  │      │2天  │
└─────┘      └─────┘      └─────────┘   └─────┘      └─────┘      └─────┘
                                                              → Flow7(3天)
```

### 4.2 详细 Sprint 计划

| Sprint | 周期 | 目标 Flow | 交付物 | 可验证点 |
|--------|------|----------|--------|----------|
| Sprint 0 | Day 1 | Flow 1 | Gateway login → Auth Service, logout, refresh, middleware 调 ValidateToken | `curl` 注册→登录→访问受保护接口 |
| Sprint 1 | Day 2-3 | Flow 2 | 确认 Gateway 行情接口全部可通 | `curl` 查市场/深度/K线/成交 |
| Sprint 2 | Week 2 | Flow 3 上半 | Risk Service Server, Portfolio Service Server (Freeze/SettleTrade), Order Service 集成 Risk+Portfolio | 下单返回订单号，风控拦截超限单 |
| Sprint 3 | Week 3 | Flow 3 下半 | Matching Engine Kafka 对接, Portfolio 消费 match.events | 下单→撮合→持仓更新，完整链路 |
| Sprint 4 | Week 4 前半 | Flow 4 | Portfolio GetBalance/GetPositions/GetLedger | 余额/持仓/流水查询正确 |
| Sprint 5 | Week 4 后半-5 | Flow 5 | Wallet Service Server, Portfolio Credit/Debit | 充值→余额增加，提现→余额扣减 |
| Sprint 6 | Week 6 前半 | Flow 6 | Admin resolve 接口, 派彩 Credit | 结算后获胜者余额增加 |
| Sprint 7 | Week 6 后半 | Flow 7 | ws-market-data + ws-order | 页面实时看到订单簿和订单状态 |

### 4.3 风险缓冲

| 风险 | 缓解措施 |
|------|----------|
| Matching Engine Kafka 对接复杂 | Week 3 预留 2 天 buffer，可先用手动测试数据验证 Portfolio 清算 |
| Portfolio 数据一致性 | 每个 Sprint 结束后跑对账脚本（订单总额 = 持仓变化 + 手续费） |
| gRPC 接口变更 | Flow 内一次性对齐 proto，不跨 Flow 改接口 |

---

## 5. 服务实现追踪矩阵

### 5.1 按服务汇总

| 服务 | Flow 1 | Flow 2 | Flow 3 | Flow 4 | Flow 5 | Flow 6 | Flow 7 | 总工作量 |
|------|:------:|:------:|:------:|:------:|:------:|:------:|:------:|:--------:|
| **API Gateway** | 升级 1d | 0 | 0 | 升级 0.5d | 升级 0.5d | 新增 0.5d | 0 | **2.5天** |
| **User Service** | 0 | - | - | - | - | - | - | **0** |
| **Auth Service** | 0 | - | - | - | - | - | - | **0** |
| **Prediction Market Service** | - | 0 | - | - | - | 0 | - | **0** |
| **Market Data Service** | - | 0 | - | - | - | - | - | **0** |
| **Order Service** | - | - | 升级 3d | - | - | - | 升级 1d | **4天** |
| **Risk Service** | - | - | 新建 2d | - | - | - | - | **2天** |
| **Portfolio Service** | - | - | 新建 5d | 新建 3d | 升级 1d | 升级 0.5d | - | **9.5天** |
| **Matching Engine** | - | - | 升级 3d | - | - | - | - | **3天** |
| **Wallet Service** | - | - | - | - | 新建 4d | - | - | **4天** |
| **ws-market-data** | - | - | - | - | - | - | 新建 2d | **2天** |
| **ws-order** | - | - | - | - | - | - | 新建 2d | **2天** |

> 总工作量约 29 人天 ≈ 6 周（1人全职）

### 5.2 Proto 新增/变更清单

| Flow | 服务 | Proto 变更 |
|------|------|-----------|
| Flow 1 | Auth Service | 无（已有完整接口） |
| Flow 3 | Risk Service | **新增** `risk_service.proto`: CheckOrder, CheckOrderBatch, CheckWithdraw, GetRiskConfig |
| Flow 3 | Portfolio Service | **新增** `portfolio_service.proto`: GetBalance, Freeze, Unfreeze, Debit, Credit, SettleTrade, GetPositions, GetPosition, GetLedger |
| Flow 5 | Wallet Service | **新增** `wallet_service.proto`: GetDepositAddress, ConfirmDeposit, Withdraw, GetWithdrawStatus |
| Flow 7 | - | Kafka topic: `match.events`, `order.events` |

---

## 6. 附录

### 6.1 每个 Flow 的启动检查清单

**启动 Flow 3 前必须完成：**
- [ ] Flow 1 通过：能登录并拿到有效 Token
- [ ] Flow 2 通过：能看到市场列表（至少有一个开放市场）
- [ ] Kafka 本地可启动（`docker-compose up kafka`）
- [ ] PostgreSQL 已初始化 Portfolio 相关表

**启动 Flow 5 前必须完成：**
- [ ] Flow 3 通过：能完成一次完整下单→撮合→清算
- [ ] Flow 4 通过：能查询余额和持仓

### 6.2 数据库表清单（按 Flow）

| Flow | 新增表 | 所属服务 |
|------|--------|----------|
| Flow 3 | `accounts`, `positions`, `settlements`, `risk_configs`, `user_limits` | Portfolio, Risk |
| Flow 4 | `ledger_entries` | Portfolio |
| Flow 5 | `deposits`, `withdrawals`, `addresses` | Wallet |

### 6.3 与旧版 Phase 计划对照

| 旧 Phase | 对应新版 Flow | 改进点 |
|----------|--------------|--------|
| Phase 1: Gateway 接入 | Flow 1 + Flow 2 | 按业务聚合，而非按服务聚合 |
| Phase 2: Portfolio | Flow 3 + Flow 4 | Portfolio 随交易链路一起生长 |
| Phase 3: Risk | Flow 3 | Risk 不是独立上线，而是下单链路的一部分 |
| Phase 4: 下单链路 | Flow 3 | 核心不变，但前置依赖更清晰 |
| Phase 5: WebSocket | Flow 7 | 后置到所有链路完成后 |
| Phase 6: Wallet | Flow 5 | 在交易闭环之后，资金需求更明确 |
| Phase 7: Admin | Flow 6 | 结算作为独立 Flow，可提前验证 |
