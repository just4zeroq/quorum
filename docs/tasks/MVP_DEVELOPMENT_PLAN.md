# 预测市场 MVP 开发计划

> 基于项目 `quorum` 当前实现状态，制定分阶段开发计划
> 目标：完成预测市场最小可用产品 (MVP)，实现闭环交易

## 当前实现状态

| 服务 | Proto | Server | DB | 状态 |
|------|:-----:|:------:|:--:|------|
| domain | N/A | N/A | N/A | ✅ 完整 |
| common (db/cache/queue/utils/auth) | N/A | N/A | N/A | ✅ 完整 |
| user-service | ✅ | ✅ gRPC | ✅ | ✅ 完整 |
| prediction-market-service | ✅ | ✅ gRPC | ✅ | ✅ 完整 |
| market-data-service | ✅ | ✅ gRPC | ✅ | ✅ 完整 |
| order-service | ✅ | ✅ gRPC | ✅ | 🟡 完整但缺测试 |
| matching-engine | ❌ | ✅ Core | N/A | 🟡 核心完成，缺 Proto/Server |
| account-service | ❌ | 🟡 Axum | ❌ 内存 | 🔴 需重构为 gRPC + DB |
| api-gateway | ❌ | 🟡 Salvo | ❌ Mock | 🔴 需重构，接入 gRPC 客户端 |
| position-service | ❌ | ❌ | ❌ 内存 | 🔴 仅有核心逻辑 |
| clearing-service | ❌ | ❌ | ❌ 内存 | 🔴 仅有核心逻辑 |
| ledger-service | ❌ | ❌ | ❌ 内存 | 🔴 仅有核心逻辑 |
| risk-service | ❌ | ❌ | ❌ 内存 | 🔴 仅有核心逻辑 |
| wallet-service | ❌ | ❌ | ❌ 内存 | 🔴 仅有核心逻辑 |
| trade-service | ❌ | ❌ | ❌ 不存在 | 🔴 未创建 |
| ws-* | ❌ | ❌ | ❌ 不存在 | 🔴 未创建 |

## MVP 最小闭环

```
用户注册/登录 → 查看市场/行情 → 下单 → 撮合 → 成交清算 → 持仓/余额更新
                                                    ↓
                                              市场结算 → 派彩
```

## 开发阶段

---

### Phase 1: 核心交易链路 (3-4 周)

> 目标：实现下单 → 撮合 → 成交 → 账户余额更新的完整链路

#### 1.1 Account Service 重构 (优先级最高)

当前状态：Axum HTTP + 内存存储，无 gRPC
目标状态：Tonic gRPC + SQLite/PostgreSQL + 结果代币支持

**任务清单：**
- [ ] 创建 `account.proto` (参考 docs/services/ACCOUNT_SERVICE.md)
  - GetBalance, GetBalances
  - Freeze, Unfreeze, FreezeAndDeduct, CheckAndFreeze
  - Deposit, Withdraw, Transfer
  - Lock, Unlock (风控)
  - Settle (结算派彩)
  - BatchGetBalances
- [ ] 创建 `build.rs` 配置 tonic-build
- [ ] 实现数据库层：accounts 表 + balance_operations 表
  - 支持 `asset` 字段为 USDT 或 `{market_id}_{outcome}` 格式
  - 实现 `CheckAndFreeze` 原子操作 (SELECT FOR UPDATE)
- [ ] 实现 gRPC 服务端
  - 余额查询、冻结/解冻、充值/提现、划转、锁定/解锁、结算
- [ ] 实现余额变更 Kafka 事件发布
- [ ] 编写单元测试
- [ ] 迁移现有内存逻辑到 DB 持久化

**验收标准：** gRPC 服务可启动，CheckAndFreeze 原子操作正确，结果代币余额可管理

#### 1.2 Matching Engine gRPC Server

当前状态：核心撮合逻辑完成，无 gRPC 接口
目标状态：完整 gRPC 服务 + Kafka 事件发布

**任务清单：**
- [ ] 创建 `matching_engine.proto`
  - SubmitOrder, CancelOrder, GetOrder
  - GetOrderBook, GetDepth, GetTrades
  - CreateMarket, CloseMarket
  - GetMarketStats
- [ ] 创建 `build.rs`
- [ ] 实现 gRPC Server 包装现有撮合核心
- [ ] 实现 Kafka 事件发布
  - OrderSubmitted, OrderFilled, OrderPartiallyFilled, OrderCancelled → `order_events`
  - TradeExecuted (含 outcome_asset) → `trade_executed`
  - OrderBookUpdated → `order_book_updates`
- [ ] 集成 WAL 持久化 + 快照恢复
- [ ] 编写集成测试

**验收标准：** gRPC 服务可启动，订单可提交/取消/撮合，Kafka 事件正确发布

#### 1.3 Order Service 集成对接

当前状态：gRPC Server 完整，但未对接 Account/Matching
目标状态：完整下单链路

**任务清单：**
- [ ] 添加 Account Service gRPC 客户端
  - 创建 `CheckAndFreeze` 调用（买入冻结 USDT，卖出冻结结果代币）
  - 创建 `Unfreeze` 调用（撤单解冻）
  - 创建 `FreezeAndDeduct` 调用（成交扣减）
- [ ] 添加 Matching Engine gRPC 客户端
  - SubmitOrder, CancelOrder 调用
- [ ] 添加 Risk Service gRPC 客户端（先 stub，Phase 2 实现风控）
- [ ] 完善 CreateOrder 流程
  - 参数校验 → 风控检查 → CheckAndFreeze → 保存订单 → SubmitOrder
- [ ] 完善 CancelOrder 流程
  - 权限验证 → CancelOrder → Unfreeze → 更新状态
- [ ] 完善 UpdateOrderStatus (成交回报)
  - FreezeAndDeduct → 更新订单状态 → 发布 Kafka 事件
- [ ] 添加 outcome_asset 字段支持
- [ ] 编写集成测试

**验收标准：** 下单 → 冻结余额 → 提交撮合 → 成交 → 扣减冻结 → 更新订单状态

#### 1.4 Position Service gRPC 化

当前状态：内存逻辑，无 gRPC
目标状态：Tonic gRPC + DB 持久化

**任务清单：**
- [ ] 创建 `position.proto`
  - GetPosition, GetUserPositions, GetMarketPositions
  - UpdatePosition, SettlePosition
  - CalculateUnrealizedPnL, GetPositionWithPnL
- [ ] 创建 `build.rs`
- [ ] 实现数据库层：user_positions 表
  - 含 outcome_asset 字段
  - UNIQUE(user_id, market_id, outcome_id)
- [ ] 实现 gRPC 服务端
- [ ] 实现 Kafka 消费
  - `trade_executed` → 更新持仓 (买入增加/卖出减少)
  - `settlement_events` → 结算清零
- [ ] 编写单元测试

**验收标准：** 成交后持仓自动更新，可按市场查询所有用户持仓

#### 1.5 Clearing Service gRPC 化

当前状态：内存逻辑，无 gRPC
目标状态：Tonic gRPC + DB 持久化 + 预测市场清算

**任务清单：**
- [ ] 创建 `clearing.proto`
  - ClearTrade, BatchClearTrades
  - SettleMarket, CalculatePayout, GetPayoutDetails
  - GetFeeConfig, SetFeeConfig, CalculateFee
  - GetClearingRecord
- [ ] 创建 `build.rs`
- [ ] 实现数据库层：clearing_records, market_settlements, user_payouts, fee_configs
- [ ] 实现成交清算 (ClearTrade)
  - 买方：Account.FreezeAndDeduct (USDT 扣减) + 结果代币增加
  - 卖方：Account.FreezeAndDeduct (结果代币扣减) + USDT 增加
  - Position.UpdatePosition (双方持仓变更)
- [ ] 实现市场结算 (SettleMarket)
  - Position.GetMarketPositions → 获取所有用户持仓
  - 计算赢家派彩 + 输家清零
  - Account.Settle → 赢家结果代币兑 USDT，输家清零
  - Position.SettlePosition → 持仓清零
- [ ] 实现 Kafka 消费/发布
  - 消费 `trade_executed` → 清算
  - 消费 `market_events` → 触发结算
  - 发布 `TradeCleared` → `trade_events`
  - 发布 `MarketSettled` → `settlement_events`
- [ ] 编写集成测试

**验收标准：** 成交后清算正确，市场结算派彩正确，结果代币清零正确

---

### Phase 2: 市场管理 + 风控 + 行情闭环 (2-3 周)

> 目标：完善市场全生命周期，风控保护，行情数据实时更新

#### 2.1 Prediction Market Service 增强

当前状态：基础 CRUD 完整
目标状态：对接 Matching Engine + Clearing Service + 结果代币

**任务清单：**
- [ ] 添加 Matching Engine gRPC 客户端
  - CreateMarket (含 outcome_asset)
  - CloseMarket
- [ ] 添加 Clearing Service gRPC 客户端
  - SettleMarket
- [ ] 完善创建市场流程
  - 保存市场/选项 → 生成 outcome_asset → CreateMarket → 发布 Kafka
- [ ] 完善结算市场流程
  - 验证 → SettleMarket(Clearing) → 更新状态 → CloseMarket(Matching) → 发布 Kafka
- [ ] 添加 outcome_asset 到 MarketOutcome 模型
- [ ] 添加 ResolveMarketRequest.reason + resolver_id
- [ ] 编写结算集成测试

**验收标准：** 创建市场自动在撮合引擎创建订单簿，结算自动触发派彩和关闭

#### 2.2 Market Data Service Kafka 集成

当前状态：gRPC Server 完整，但无 Kafka 消费
目标状态：实时行情数据更新

**任务清单：**
- [ ] 添加 Kafka 消费
  - `trade_executed` → 更新成交记录、K线、24h统计
  - `order_book_updates` → 更新订单簿深度缓存
  - `market_events` → 更新市场状态
- [ ] 添加 GetOrderBookDepth RPC
- [ ] 添加 GetCategories RPC
- [ ] 更新 Proto 消息增加 outcome_asset 字段
- [ ] 编写集成测试

**验收标准：** 下单/成交后行情数据自动更新

#### 2.3 Risk Service gRPC 化

当前状态：内存逻辑，无 gRPC
目标状态：Tonic gRPC + DB + 简化风控规则

**任务清单：**
- [ ] 创建 `risk.proto`
  - CheckOrder, CheckOrderBatch, GetOrderQuota
  - CheckWithdraw, GetWithdrawQuota
  - CheckUserStatus
  - GetRiskConfig, UpdateRiskConfig
- [ ] 创建 `build.rs`
- [ ] 实现数据库层：风控配置表 + 用户日限额表
- [ ] 实现简化风控规则
  - 单笔金额限制、日限额、用户状态检查
  - ⚠️ MVP 不实现 FreezeUser (由 Admin Service 直接调 User Service)
- [ ] 实现 gRPC 服务端
- [ ] 编写单元测试

**验收标准：** Order Service 下单前调用 CheckOrder，超限订单被拒绝

#### 2.4 Trade Service 实现

当前状态：不存在
目标状态：Tonic gRPC + DB + Kafka 消费

**任务清单：**
- [ ] 创建 crate 骨架 (Cargo.toml, build.rs, config/)
- [ ] 创建 `trade.proto`
  - GetTrade, GetUserTrades, GetMarketTrades
- [ ] 实现数据库层：trades 表 (含 outcome_asset)
- [ ] 实现 Kafka 消费
  - `trade_executed` → 保存成交记录
- [ ] 实现 gRPC 服务端
- [ ] 编写单元测试

**验收标准：** 成交记录自动持久化，可按用户/市场查询

#### 2.5 Ledger Service gRPC 化

当前状态：内存逻辑，无 gRPC
目标状态：Tonic gRPC + DB + Kafka 消费

**任务清单：**
- [ ] 创建 `ledger.proto`
  - GetEntries, GetBalanceSummary, VerifyBalance
- [ ] 创建 `build.rs`
- [ ] 实现数据库层：ledger_entries 表
  - 含 counterpart_entry_id (复式记账)
  - biz_type 枚举含 settlement_win/settlement_lose
- [ ] 实现 Kafka 消费
  - `balance_updates` → 记录账本流水
  - `settlement_events` → 记录结算流水
- [ ] 实现 gRPC 服务端
- [ ] 编写单元测试

**验收标准：** 余额变更自动记录流水，支持复式记账验证

---

### Phase 3: API Gateway + 用户入口 (2 周)

> 目标：用户可通过 HTTP/WS 完成所有操作

#### 3.1 API Gateway 重构

当前状态：Salvo + Mock 处理器
目标状态：Salvo + gRPC 客户端转发

**任务清单：**
- [ ] 添加所有服务 gRPC 客户端
- [ ] 实现用户认证路由 (/api/v1/auth/*)
  - 注册、登录、登出、刷新 Token
- [ ] 实现预测市场路由 (/api/v1/markets/*)
  - 市场列表、详情、选项、K线、深度、成交
- [ ] 实现订单路由 (/api/v1/orders/*)
  - 创建、取消、查询
- [ ] 实现账户/钱包路由 (/api/v1/account/*, /api/v1/wallet/*)
  - 余额查询、充值地址、提现
- [ ] 实现管理后台路由 (/api/v1/admin/*)
- [ ] 实现 JWT 认证中间件
- [ ] 实现限流中间件
- [ ] 编写集成测试

**验收标准：** 所有 HTTP 接口可正确转发到 gRPC 服务

#### 3.2 WebSocket 服务实现

**ws-market-data (行情推送)：**
- [ ] 创建 crate 骨架
- [ ] Kafka 消费 → WS 推送
- [ ] 支持订阅：kline/trades/depth/ticker (按 market_id + outcome_id)
- [ ] 实现连接管理

**ws-order (订单推送)：**
- [ ] 创建 crate 骨架
- [ ] Kafka 消费 `order_events` → WS 推送
- [ ] 必须认证：只推送用户自己的订单变更
- [ ] 实现连接管理

**ws-prediction (市场事件推送)：**
- [ ] 创建 crate 骨架
- [ ] Kafka 消费 `market_events` + `settlement_events` → WS 推送
- [ ] 支持订阅：市场状态变更 + 结算派彩结果
- [ ] 实现连接管理

**验收标准：** WS 连接可订阅行情/订单/市场事件，数据实时推送

---

### Phase 4: 钱包 + 管理后台 (2 周)

> 目标：完整充值提现流程，管理后台可管理市场

#### 4.1 Wallet Service gRPC 化

当前状态：内存逻辑，无 gRPC
目标状态：Tonic gRPC + DB + 链上监听 (简化版)

**任务清单：**
- [ ] 创建 `wallet.proto` (参考 docs/services/WALLET_SERVICE.md)
- [ ] 创建 `build.rs`
- [ ] 实现数据库层：deposit_addresses, withdraw_records, whitelist_addresses, payment_passwords
- [ ] 实现 gRPC 服务端
  - 充值地址管理
  - 充值确认 → Account.Deposit
  - 提现流程 → Risk.CheckWithdraw → Account.Withdraw
  - 地址白名单
  - 支付密码
- [ ] MVP 简化：链上充值监听用定时轮询代替实时监听
- [ ] 编写单元测试

**验收标准：** 充值/提现流程完整，余额正确更新

#### 4.2 Admin Service 实现

当前状态：不存在
目标状态：Tonic gRPC + DB

**任务清单：**
- [ ] 创建 crate 骨架
- [ ] 创建 `admin.proto`
- [ ] 实现用户管理 (调用 User Service + Account Service.Lock/Unlock)
- [ ] 实现市场管理 (调用 Prediction Market Service)
- [ ] 实现提现审核
- [ ] 实现系统配置管理
- [ ] 实现审计日志 (audit_logs 表)
- [ ] 实现统计报表
- [ ] 编写单元测试

**验收标准：** 管理员可冻结用户、创建/结算市场、审核提现

---

### Phase 5: 对账 + 稳定性 + 上线 (1-2 周)

> 目标：数据一致性保障，端到端测试，可上线

#### 5.1 Reconciliation Service

- [ ] 创建 crate + proto + DB
- [ ] 实现订单对账、余额对账、持仓对账、结算对账
- [ ] 定时任务调度

#### 5.2 端到端测试

- [ ] 用户注册 → 登录 → 获取余额 → 查看市场 → 下单 → 成交 → 持仓更新
- [ ] 市场结算 → 派彩 → 余额更新 → 持仓清零
- [ ] 撤单 → 余额解冻
- [ ] 并发下单 → 余额正确性
- [ ] 对账验证

#### 5.3 部署配置

- [ ] Docker Compose (所有服务 + PostgreSQL + Redis + Kafka)
- [ ] 健康检查接口
- [ ] 优雅关闭

---

## 依赖关系图

```
Phase 1 (核心交易链路):
  1.1 Account Service ←── 无依赖，优先
  1.2 Matching Engine  ←── 无依赖，可并行
  1.3 Order Service    ←── 依赖 1.1 + 1.2
  1.4 Position Service ←── 依赖 1.1 (DB schema)
  1.5 Clearing Service ←── 依赖 1.1 + 1.4

Phase 2 (市场+风控+行情):
  2.1 PM Service 增强  ←── 依赖 1.2 + 1.5
  2.2 Market Data      ←── 依赖 1.2 (Kafka)
  2.3 Risk Service     ←── 无硬依赖，可并行
  2.4 Trade Service    ←── 依赖 1.2 (Kafka)
  2.5 Ledger Service   ←── 依赖 1.1 (Kafka)

Phase 3 (用户入口):
  3.1 API Gateway      ←── 依赖 Phase 1 + 2
  3.2 WebSocket 服务   ←── 依赖 Phase 2 (Kafka)

Phase 4 (钱包+管理):
  4.1 Wallet Service   ←── 依赖 1.1 + 2.3
  4.2 Admin Service    ←── 依赖 Phase 1 + 2

Phase 5 (对账+上线):
  5.x                  ←── 依赖 Phase 1-4
```

## 每个服务开发模板

所有服务遵循统一模式：

```
1. 创建/更新 Proto 文件 (src/pb/{service}.proto)
2. 创建 build.rs (tonic-build 编译配置)
3. 实现 config.rs (配置加载)
4. 实现 repository/ (数据库 CRUD)
5. 实现 services/ (gRPC 服务实现)
6. 实现 main.rs (服务启动 + gRPC Server 注册)
7. 添加 Kafka 生产者/消费者 (如需要)
8. 编写单元测试 + 集成测试
```

## MVP 裁剪说明

以下功能在 MVP 中**简化或推迟**：

| 功能 | MVP 策略 |
|------|----------|
| 链上充值监听 | 定时轮询代替实时监听 |
| 离线签名 | MVP 使用热钱包 |
| KYC 审核 | 仅状态管理，不做人工审核流程 |
| 提现白名单 | MVP 默认关闭 |
| 支付密码 | MVP 可选 |
| 对账服务 | Phase 5 实现 |
| Admin 审计日志 | Phase 4 实现 |
| WebSocket | Phase 3 实现，MVP 可先用轮询 |
| 2FA | MVP 可选 |

## 关键技术决策

1. **Account Service 必须最先完成** — 所有其他服务依赖余额操作
2. **CheckAndFreeze 必须用事务 + 行锁** — 防止并发余额问题
3. **结果代币格式统一为 `{market_id}_{outcome}`** — 全系统一致
4. **成交清算由 Clearing Service 消费 Kafka 异步执行** — 解耦撮合与清算
5. **市场结算由 PM Service 调用 Clearing Service 同步执行** — 结算必须确保完成
6. **所有 gRPC 服务使用 tonic，API Gateway 使用 salvo** — 保持一致
