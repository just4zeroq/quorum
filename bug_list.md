# Bug / Issue List

## P0 — 已修复

| # | Issue | Fix | Status |
|---|-------|-----|--------|
| 1 | Portfolio.Freeze 未在下单流程调用 | API Gateway create_order 新增 step 2: 调用 Portfolio.Freeze 冻结资金 | ✅ |
| 2 | Kafka 消息格式不匹配（Order Service 发 Decimal 字符串，Matching Engine 期望 i64/u64） | queue_producer.rs 统一为 i64 price/u64 id 格式 | ✅ |
| 3 | 撮合后 Portfolio.SettleTrade 未调用 | queue_consumer.rs 新增 call_settle_trade，Trade 事件处理后调用 | ✅ |
| 4 | 订单 ID 格式：DB 存 "ord_xxx"，Matching Engine 发 u64，导致 update_status 查不到 | queue_consumer.rs 统一加 `ord_` 前缀 | ✅ |
| 5 | 用户 ID 格式：Portfolio 期望 "usr_xxx"，Matching Engine 发 u64 | queue_consumer.rs call_settle_trade 加 `usr_` 前缀 | ✅ |
| 6 | Matching Engine 启动后无 symbol 注册，PlaceOrder 返回 InvalidSymbol | server.rs run_sync_worker 新增懒注册（首次 PlaceOrder 自动创建 spec） | ✅ |
| 7 | PRICE_SCALE / OUTCOME_MULTIPLIER 在三处重复定义 | 抽取到 utils::constants | ✅ |

## P1 — 待修复

| # | Issue | 影响 | 优先级 |
|---|-------|------|--------|
| 8 | ws-prediction 订阅 "market_events" 和 "settlement_events" topic，但无服务发布 | WebSocket 行情/结算无推送 | P1 |
| 9 | PredictionMarketService.ResolveMarket 结算后不发送 settlement_events | 结算后通知缺失 | P1 |

## P2 — 待修复

| # | Issue | 影响 | 优先级 |
|---|-------|------|--------|
| 10 | `extract_numeric_id` 在 queue_producer.rs 中为模块私有，且用哈希兜底可能导致碰撞 | 理论上存在 ID 冲突风险 | P2 |
| 11 | api-gateway handlers.rs 中 Portfolio.Freeze 成功后如果 Order Service 调用失败，Unfreeze 缺少重试/补偿机制 | 极端情况下资金可能被冻结未解冻 | P2 |
