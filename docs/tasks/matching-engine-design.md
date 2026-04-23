# Matching Engine 详细设计文档

> 基于 `matching-core` 高性能撮合引擎，适配 quorum 预测市场

---

## 0. 预测市场订单簿特性

### 0.1 市场类型

预测市场分为 **二元市场** 和 **多元市场**：

| 类型 | 示例 | 选项数 | 特点 |
|------|------|--------|------|
| 二元市场 | "明天是否下雨" | 2 (Yes/No) | 价格 0-100 |
| 多元市场 | "世界杯冠军" | N (A/B/C...) | 每个选项独立订单簿 |

### 0.2 数据结构模型

```
PredictionMarket (预测市场)
├── market_id: 100
├── name: "2024 世界杯冠军"
├── base_asset: "USDT"
├── base_precision: 6
├── status: Open / Closed / Resolved
└── outcomes: Vec<Outcome>
    ├── Outcome A (Team A Wins)
    │   ├── outcome_id: 1
    │   ├── outcome_asset: "100_A"
    │   ├── name: "Team A Wins"
    │   └── order_book: OrderBook
    ├── Outcome B (Team B Wins)
    │   ├── outcome_id: 2
    │   ├── outcome_asset: "100_B"
    │   └── order_book: OrderBook
    └── Outcome C (Team C Wins)
        ├── outcome_id: 3
        ├── outcome_asset: "100_C"
        └── order_book: OrderBook
```

### 0.3 订单模型

| 订单方向 | 买入 | 卖出 |
|---------|------|------|
| 冻结 | USDT (bid_price × size) | 结果代币 (size) |
| 获得 | 结果代币 | USDT (立即到账) |
| 结算时 | 如果获胜：结果代币 → USDT (1:1) | 如果获胜：获得 USDT |

### 0.4 价格含义

预测市场价格表示**概率/赔率**：

| 模型 | 价格范围 | 示例 |
|------|----------|------|
| 概率模型 | 0-100 或 0-1 | 0.65 = 65% 概率 |
| 赔率模型 | 1.0+ | 3.0 = 下注1赔3 |

**Quorum 采用概率模型**：价格范围 0-100（精度6：0.000001-100.000000）

### 0.5 结算流程

```
市场结算 (ResolveMarket):
1. 确定获胜选项 W
2. 对每个选项 O:
   - 如果 O == W (赢家):
     - 用户结果代币 O → USDT (按 1:1 兑换)
   - 如果 O != W (输家):
     - 结果代币 O → 归零
3. 清空所有订单簿
```

---

## 1. 现状分析

### 1.1 当前 quorum 撮合引擎

| 项目 | 当前状态 |
|------|----------|
| 协议 | 无 gRPC Server |
| 存储 | 内存撮合，无持久化 |
| 订单类型 | Limit/Market/IOC/FOK/PostOnly |
| 订单簿 | BTreeMap + VecDeque |
| 精度 | rust_decimal::Decimal |
| 事件发布 | 无 Kafka |
| 高级订单 | 不支持 |

### 1.2 matching-core 特性

| 特性 | 说明 |
|------|------|
| 高性能 | 数百万 TPS，微秒级延迟 |
| 订单类型 | GTC/IOC/FOK/PostOnly/Stop/Iceberg/GTD/Day |
| 交易品种 | 现货/期货/永续/期权 |
| 内存优化 | SOA 布局、对象池、SmallVec |
| 序列化 | rkyv 零拷贝 WAL |
| 架构 | Disruptor 无锁环形缓冲区、分片 |
| 持久化 | WAL + 快照 |

---

## 2. 移植目标

### 2.1 Phase 1 - 核心撮合 (MVP)

**目标**: 实现基本撮合功能，支持 quorum 预测市场

| 功能 | 优先级 | 说明 |
|------|--------|------|
| gRPC Server | P0 | 端口 50009 |
| 基础订单簿 | P0 | AdvancedOrderBook 核心 |
| Limit/Market/IOC/FOK/PostOnly | P0 | 预测市场需要 |
| 结果代币支持 | P0 | `{market_id}_{outcome}` |
| 内存持久化 | P1 | 快照 |
| Kafka 事件 | P1 | order_events, trade_executed |

### 2.2 Phase 2 - 高级功能

| 功能 | 说明 |
|------|------|
| Stop Order | 止损单 |
| Iceberg Order | 冰山单 |
| WAL 持久化 | rkyv 零拷贝 |
| 分片架构 | 多市场分片 |

---

## 3. 架构设计

### 3.1 目录结构

```
crates/matching-engine/
├── Cargo.toml
├── build.rs
├── config/
│   └── matching_engine.yaml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── config.rs
    ├── error.rs
    ├── pb.rs                    # Proto 生成代码
    ├── pb/
    │   ├── matching_engine.proto
    │   └── market.proto          # 市场管理 Proto
    ├── market/                   # 市场管理 (新增)
    │   ├── mod.rs
    │   ├── market.rs
    │   └── outcome.rs
    ├── engine/                  # 核心撮合
    │   ├── mod.rs
    │   ├── orderbook/
    │   │   ├── mod.rs
    │   │   ├── order_book.rs    # 基础订单簿
    │   │   └── advanced.rs      # 高级订单簿
    │   └── settlement.rs         # 结算逻辑
    ├── services/
    │   ├── mod.rs
    │   ├── matching_engine_impl.rs
    │   └── market_service_impl.rs  # 市场管理服务
    └── kafka/
        ├── mod.rs
        └── producer.rs
```

### 3.2 核心类型映射

#### 3.2.1 类型差异

| matching-core | quorum | 说明 |
|----------------|--------|------|
| `UserId = u64` | `user_id: i64` | 用户 ID |
| `OrderId = u64` | `order_id: String` | 订单 ID |
| `SymbolId = i32` | `market_id: i64` | 市场 ID |
| `Price = i64` | `Decimal` | 价格 (整数最小单位) |
| `Size = i64` | `Decimal` | 数量 (整数最小单位) |
| `OrderAction::Ask/Bid` | `Side::Buy/Sell` | 买卖方向 |
| `SymbolType` | - | 预测市场只用 CurrencyExchangePair |

#### 3.2.2 订单类型映射

| matching-core | quorum | 说明 |
|----------------|--------|------|
| `OrderType::Gtc` | `OrderType::Limit` | GTC = 限价单 |
| `OrderType::Ioc` | `OrderType::IOC` | IOC |
| `OrderType::Fok` | `OrderType::FOK` | FOK |
| `OrderType::PostOnly` | `OrderType::PostOnly` | PostOnly |
| `OrderType::StopLimit` | - | 暂不支持 |
| `OrderType::Iceberg` | - | 暂不支持 |
| `OrderType::Gtd(ts)` | - | 暂不支持 |

#### 3.2.3 新增类型

```rust
/// 预测市场特定类型

/// 结果代币标识
/// 格式: "{market_id}_{outcome}"
/// 例: "12345_yes", "12345_no"
pub type OutcomeAsset = String;

/// 市场规格 (预测市场版)
#[derive(Debug, Clone)]
pub struct MarketSpecification {
    pub market_id: i64,
    pub base_currency: i32,      // 基础资产 (USDT = 0)
    pub quote_currency: i32,     // 计价资产 (USDT = 0)
    pub base_scale_k: i64,       // 基础资产精度
    pub quote_scale_k: i64,      // 计价资产精度
    pub taker_fee: i64,         // taker 手续费 (最小单位)
    pub maker_fee: i64,          // maker 手续费 (最小单位)
}

/// 订单命令 (适配 quorum)
#[derive(Debug, Clone)]
pub struct QuorumOrderCommand {
    pub command: OrderCommandType,
    pub result_code: CommandResultCode,

    pub uid: UserId,             // 用户 ID
    pub order_id: OrderId,      // 订单 ID
    pub market_id: i64,         // 市场 ID
    pub outcome_asset: String,   // 结果代币 "12345_yes"

    pub price: Price,            // 价格 (最小单位)
    pub size: Size,              // 数量 (最小单位)
    pub action: OrderAction,    // Ask/Bid
    pub order_type: OrderType,   // Gtc/Ioc/Fok/PostOnly

    pub timestamp: i64,
    pub events_group: u64,

    // 撮合事件
    pub matcher_events: Vec<QuorumTradeEvent>,
}

/// 撮合成交事件
#[derive(Debug, Clone)]
pub struct QuorumTradeEvent {
    pub event_type: MatcherEventType,
    pub size: Size,              // 成交数量
    pub price: Price,           // 成交价格
    pub matched_order_id: OrderId,
    pub matched_order_uid: UserId,
    pub outcome_asset: String,   // 结果代币
    pub base_amount: i64,       // 基础资产金额
}
```

---

## 4. 详细设计

### 4.1 订单簿设计

#### 4.1.1 AdvancedOrderBook (核心)

```rust
/// 高级订单簿 - 预测市场版
pub struct QuorumOrderBook {
    /// 市场规格
    market_spec: MarketSpecification,

    /// 结果代币标识
    outcome_asset: String,

    /// 卖盘 (价格 -> 档位)
    ask_buckets: BTreeMap<Price, AdvancedBucket>,
    /// 买盘 (价格 -> 档位)
    bid_buckets: BTreeMap<Price, AdvancedBucket>,

    /// 订单索引: order_id -> (price, action)
    order_map: AHashMap<OrderId, (Price, OrderAction)>,

    /// 最新成交价
    last_trade_price: Option<Price>,

    /// 最优价格缓存
    best_ask_price: Option<Price>,
    best_bid_price: Option<Price>,
}

impl QuorumOrderBook {
    /// 下单
    pub fn new_order(&mut self, cmd: &mut QuorumOrderCommand) -> CommandResultCode;

    /// 取消订单
    pub fn cancel_order(&mut self, cmd: &mut QuorumOrderCommand) -> CommandResultCode;

    /// 移动订单
    pub fn move_order(&mut self, cmd: &mut QuorumOrderCommand) -> CommandResultCode;

    /// 减少订单
    pub fn reduce_order(&mut self, cmd: &mut QuorumOrderCommand) -> CommandResultCode;
}
```

#### 4.1.2 档位设计

```rust
/// 价格档位
struct AdvancedBucket {
    price: Price,
    orders: SmallVec<[AdvancedOrder; 8]>,  // 同价订单队列
    total_volume: Size,      // 总真实挂单量
    visible_volume: Size,    // 显示挂单量 (冰山单)
}

impl AdvancedBucket {
    /// 添加订单
    fn add(&mut self, order: AdvancedOrder);

    /// 移除订单
    fn remove(&mut self, order_id: OrderId) -> Option<AdvancedOrder>;

    /// 撮合
    fn match_order(&mut self, taker_size: Size, taker_uid: UserId, current_time: i64)
        -> (Size, SmallVec<[MatcherTradeEvent; 4]>);
}

/// 订单
struct AdvancedOrder {
    order_id: OrderId,
    uid: UserId,
    price: Price,
    size: Size,
    filled: Size,
    action: OrderAction,
    order_type: OrderType,
    reserve_price: Price,     // 买方保留价格
    timestamp: i64,
}
```

### 4.2 撮合流程

#### 4.2.1 下单流程

```
1. 接收 QuorumOrderCommand
2. 验证订单 (重复检查、订单类型验证)
3. PostOnly 检查 (如果会立即成交则拒绝)
4. Stop 订单处理 (暂存到 stop_orders 池)
5. 尝试撮合 (try_match)
   - 遍历对手方订单簿
   - 价格匹配则成交
   - 生成 MatcherTradeEvent
6. 更新最新成交价
7. 处理触发止损单
8. IOC/FOK 不挂单
9. GTC/PostOnly 挂单
10. 返回结果
```

#### 4.2.2 成交流程

```
买单 Ask 匹配:
1. 获取卖盘 (ask_buckets) 价格 <= 买单价格
2. 遍历价格档位
3. 从最低价开始成交
4. 生成成交事件

卖单 Bid 匹配:
1. 获取买盘 (bid_buckets) 价格 >= 卖单价格
2. 遍历价格档位
3. 从最高价开始成交
4. 生成成交事件
```

### 4.3 预测市场适配

#### 4.3.1 结果代币支持

```rust
/// 预测市场每个选项有独立的订单簿
struct PredictionMarket {
    market_id: i64,
    base_currency: Currency,      // USDT
    outcomes: Vec<Outcome>,    // 所有选项
}

struct Outcome {
    outcome_id: i64,
    outcome_asset: String,      // "12345_yes"
    order_book: QuorumOrderBook,
}
```

#### 4.3.2 订单结构

```rust
/// 买单 (买入结果代币)
QuorumOrderCommand {
    action: OrderAction::Bid,   // 买入
    price: 500000,             // 价格 0.5 USDT (精度6)
    size: 1000,                // 数量 1000 个
    outcome_asset: "12345_yes", // 结果代币
}

/// 卖单 (卖出结果代币)
QuorumOrderCommand {
    action: OrderAction::Ask,  // 卖出
    price: 500000,
    size: 500,
    outcome_asset: "12345_yes",
}
```

#### 4.3.3 多元市场处理

```rust
/// 订单路由到对应选项的订单簿
impl MatchingEngine {
    fn submit_order(&self, cmd: &mut OrderCommand) -> CommandResultCode {
        let book = self.order_books.get(&cmd.outcome_asset)?;
        book.write().place_order(cmd)
    }
}
```

### 4.4 结算逻辑

#### 4.4.1 结算触发

```
结算由 Prediction Market Service 调用 ResolveMarket 触发:

1. Prediction Market Service 调用 MarketService.ResolveMarket
2. MarketService.Market 更新状态为 Resolved
3. MarketService 发布 MarketResolved 事件到 Kafka
4. Clearing Service 消费事件，调用 BatchSettle
5. Clearing Service 更新用户余额
```

#### 4.4.2 结算流程

```rust
/// 批量结算
async fn batch_settle(&self, req: BatchSettleRequest) -> Result<BatchSettleResponse> {
    let market = self.markets.get(&req.market_id)?;
    let winning_outcome = req.winning_outcome;

    let mut total_payout = 0i64;
    let mut settled = 0;

    for outcome in &market.outcomes {
        let book = self.get_orderbook(&outcome.outcome_asset)?;

        if outcome.outcome_asset.ends_with(&format!("_{}", winning_outcome)) {
            // 赢家：结果代币 → USDT (按 1:1 兑换)
            for (order_id, record) in book.get_all_orders() {
                let payout = record.filled * record.price / SCALE; // 价格已经是概率
                total_payout += payout;
                settled += 1;
            }
        } else {
            // 输家：结果代币归零
            book.clear()?;
        }
    }

    Ok(BatchSettleResponse {
        success: true,
        settled_positions: settled,
        total_payout,
    })
}
```

#### 4.4.3 结算示例

```
世界杯冠军市场:
- 选项 A (Team A Wins): 买单 1000 个 @ 0.35 USDT
- 选项 B (Team B Wins): 买单 500 个 @ 0.45 USDT
- 选项 C (Team C Wins): 买单 200 个 @ 0.20 USDT

最终 A 队获胜:

结算后:
- A 持有者: 1000 × 1.0 = 1000 USDT (赢得 1000 - 350 = 650 USDT)
- B 持有者: 500 × 0 = 0 USDT (损失 225 USDT)
- C 持有者: 200 × 0 = 0 USDT (损失 40 USDT)
```

### 4.5 Market Service 实现

#### 4.5.1 服务结构

```rust
/// 市场服务实现
pub struct MarketServiceImpl {
    /// 市场: market_id -> PredictionMarket
    markets: RwLock<HashMap<MarketId, PredictionMarket>>,
    /// 订单簿: outcome_asset -> Arc<RwLock<OrderBook>>
    order_books: RwLock<HashMap<String, Arc<RwLock<OrderBook>>>>,
}
```

#### 4.5.2 核心方法

```rust
impl MarketServiceImpl {
    /// 创建市场
    pub async fn create_market(&self, req: CreateMarketRequest) -> Result<...>;

    /// 添加选项到市场
    pub async fn add_outcome(&self, market_id: MarketId, outcome: OutcomeSpec) -> Result<...>;

    /// 关闭市场 (停止交易)
    pub async fn close_market(&self, req: CloseMarketRequest) -> Result<...>;

    /// 结算市场
    pub async fn resolve_market(&self, req: ResolveMarketRequest) -> Result<...>;

    /// 批量结算 (由 Clearing Service 调用)
    pub async fn batch_settle(&self, req: BatchSettleRequest) -> Result<...>;
}

#### 4.5.3 市场生命周期</parameter>


```
┌─────────────┐
│   Pending  │  创建市场，添加选项
└──────┬──────┘
       │ add_outcome()
       ▼
┌─────────────┐
│    Open    │◄─ 接受交易
└──────┬──────┘
       │ close_market()
       ▼
┌─────────────┐
│   Closed   │  停止交易
└──────┬─────┘
       │ resolve_market()
       ▼
┌─────────────┐
│  Resolved  │  结算完成
└─────────────┘
```

### 4.6 多元市场约束机制

#### 4.6.1 约束层面总览

| 层面 | 约束类型 | 实现位置 |
|------|----------|--------|
| 撮合引擎 | **无约束** | 独立订单簿 |
| 概率校验 | 软约束 (可选) | Market Service |
| 保证金风控 | **硬约束** | Risk Service |
| 结算分配 | **关联约束** | Clearing Service |

#### 4.6.2 撮合层面（无约束）

每个选项的订单簿**完全独立**，撮合引擎不维护跨选项约束：

```
Market 100 (世界杯冠军)
├── 100_A → OrderBook ← 独立撮合，无跨选项约束
├── 100_B → OrderBook ← 独立撮合
└── 100_C → OrderBook ← 独立撮合
```

订单路由完全基于 `outcome_asset`，撮合逻辑在单个订单簿内完成。

#### 4.6.3 概率层面（软约束）

市场可以校验所有选项价格之和，但**不强制执行**：

```rust
/// 可选的概率校验 (Market Service 层)
fn validate_probability_sum(market: &PredictionMarket) -> bool {
    let total: f64 = market.outcomes.iter()
        .map(|o| o.last_price / SCALE)
        .sum();

    // 允许小幅度偏离 (如 95%-105% 市场校正误差)
    (95.0..=105.0).contains(&total)
}
```

#### 4.6.4 保证金层面（硬约束）

用户在整个市场的总投注有上限，防止过度投机：

```rust
/// 市场级别的保证金约束
pub struct MarketStakeLimit {
    pub market_id: MarketId,
    /// 用户在市场的最大总投注
    pub max_total_stake: i64,
    /// 单选项最大投注
    pub max_per_outcome: i64,
    /// 单笔最大投注
    pub max_single_stake: i64,
}

/// Risk Service 检查
impl MarketRiskService {
    fn check_stake_limit(&self, uid: UserId, cmd: &OrderCommand) -> Result<()> {
        // 1. 计算用户在市场的当前总投注
        let current_stake = self.get_total_stake(uid, cmd.market_id)?;

        // 2. 计算新订单所需保证金
        let new_stake = cmd.size * cmd.price / SCALE;

        // 3. 检查单笔限制
        if new_stake > self.limit.max_single_stake {
            return Err(RiskError::SingleStakeExceeded);
        }

        // 4. 检查市场总限额
        if current_stake + new_stake > self.limit.max_total_stake {
            return Err(RiskError::MarketStakeExceeded);
        }

        Ok(())
    }
}
```

#### 4.6.5 结算层面（资金守恒）

结算时，输家的保证金分配给赢家，资金守恒：

```rust
/// 结算时的资金流
struct SettlementAccounting {
    market_id: MarketId,
    winning_outcome: String,
}

impl SettlementAccounting {
    /// 计算派彩
    fn calculate_payouts(&self) -> Vec<Payout> {
        let market = self.get_market(self.market_id);
        let winner = &market.outcomes[&self.winning_outcome];

        // 1. 汇总输家保证金池
        let loser_pool: i64 = market.outcomes.iter()
            .filter(|o| o.outcome_id != self.winning_outcome)
            .map(|o| o.total_stake)
            .sum();

        // 2. 计算赢家总持仓
        let winner_holders: Vec<Position> = self.get_winner_positions(self.market_id, &self.winning_outcome);

        // 3. 按比例分配 (简化模型: 1:1 兑换)
        let payouts: Vec<Payout> = winner_holders.iter().map(|h| Payout {
            user_id: h.user_id,
            outcome_asset: h.outcome_asset.clone(),
            payout: h.filled * SCALE,  // 1:1 兑换
        }).collect();

        payouts
    }
}
```

#### 4.6.6 约束决策表

| 决策点 | 选择 | 原因 |
|--------|------|------|
| 撮合引擎约束 | **无** | 性能考虑，简化设计 |
| 概率校验 | **可选** | 前端/风控层处理 |
| 市场投注限额 | **需要** | 防止投机风险 |
| 结算资金守恒 | **需要** | 财务正确性 |

#### 4.6.7 Proto 扩展（风控）

```protobuf
// Risk Service Market Limit Proto
message MarketStakeLimit {
    int64 market_id = 1;
    int64 max_total_stake = 2;     // 市场总限额
    int64 max_per_outcome = 3;    // 单选项限额
    int64 max_single_stake = 4;   // 单笔限额
}

message CheckMarketStakeRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    string outcome_asset = 3;
    int64 new_stake = 4;           // 新订单所需保证金
}

message CheckMarketStakeResponse {
    bool allowed = 1;
    string reason = 2;
    int64 current_stake = 3;
    int64 limit = 4;
}
```

### 4.4 gRPC 服务设计

#### 4.4.1 服务列表

| 服务 | 端口 | 说明 |
|------|------|------|
| MatchingEngineService | 50009 | 订单撮合 |
| MarketService | 50010 | 市场管理 |

#### 4.4.2 MatchingEngineService Proto

```protobuf
package matching;

service MatchingEngineService {
    // 订单操作
    rpc SubmitOrder(SubmitOrderRequest) returns (SubmitOrderResponse);
    rpc CancelOrder(CancelOrderRequest) returns (CancelOrderResponse);
    rpc GetOrder(GetOrderRequest) returns (GetOrderResponse);

    // 订单簿查询
    rpc GetOrderBook(GetOrderBookRequest) returns (GetOrderBookResponse);
    rpc GetDepth(GetDepthRequest) returns (GetDepthResponse);

    // 统计
    rpc GetMarketStats(GetMarketStatsRequest) returns (GetMarketStatsResponse);
}
```

#### 4.4.3 MarketService Proto (市场管理)

```protobuf
package market;

service MarketService {
    // 市场管理
    rpc CreateMarket(CreateMarketRequest) returns (CreateMarketResponse);
    rpc GetMarket(GetMarketRequest) returns (GetMarketResponse);
    rpc ListMarkets(ListMarketsRequest) returns (ListMarketsResponse);

    // 选项管理
    rpc AddOutcome(AddOutcomeRequest) returns (AddOutcomeResponse);

    // 市场状态
    rpc CloseMarket(CloseMarketRequest) returns (CloseMarketResponse);
    rpc ResolveMarket(ResolveMarketRequest) returns (ResolveMarketResponse);

    // 批量结算 (由 Clearing Service 调用)
    rpc BatchSettle(BatchSettleRequest) returns (BatchSettleResponse);
}

// ========== 市场管理 ==========

message CreateMarketRequest {
    int64 market_id = 1;
    string name = 2;
    string description = 3;
    string base_asset = 4;        // "USDT"
    int32 base_precision = 5;      // 6
    repeated OutcomeSpec outcomes = 6;  // 选项列表
}

message OutcomeSpec {
    string outcome_id = 1;           // "A", "B", "C"
    string outcome_asset = 2;        // "100_A", "100_B", "100_C"
    string name = 3;               // "Team A Wins"
}

message CreateMarketResponse {
    bool success = 1;
    string message = 2;
    int64 market_id = 3;
}

message GetMarketRequest {
    int64 market_id = 1;
}

message GetMarketResponse {
    int64 market_id = 1;
    string name = 2;
    string description = 3;
    string base_asset = 4;
    int32 base_precision = 5;
    string status = 6;              // "open", "closed", "resolved"
    repeated OutcomeInfo outcomes = 7;
    string winning_outcome = 8;
    int64 created_at = 9;
    int64 closed_at = 10;
    int64 resolved_at = 11;
}

message OutcomeInfo {
    string outcome_id = 1;
    string outcome_asset = 2;
    string name = 3;
    string last_price = 4;
    string total_volume = 5;
}

message ListMarketsRequest {
    string status_filter = 1;     // "open", "closed", "resolved"
    int32 offset = 2;
    int32 limit = 3;
}

message ListMarketsResponse {
    repeated MarketSummary markets = 1;
    int32 total = 2;
}

message MarketSummary {
    int64 market_id = 1;
    string name = 2;
    string status = 3;
    int32 outcome_count = 4;
}

// ========== 市场状态 ==========

message CloseMarketRequest {
    int64 market_id = 1;
    string reason = 2;
}

message CloseMarketResponse {
    bool success = 1;
    string message = 2;
    int32 cancelled_orders = 3;
}

message ResolveMarketRequest {
    int64 market_id = 1;
    string winning_outcome = 2;     // "A" 或 "100_A"
    string reason = 3;
}

message ResolveMarketResponse {
    bool success = 1;
    string message = 2;
    int64 market_id = 3;
    string winning_outcome = 4;
}

// ========== 批量结算 ==========

message BatchSettleRequest {
    int64 market_id = 1;
    string winning_outcome = 2;
}

message BatchSettleResponse {
    bool success = 1;
    string message = 2;
    int32 settled_positions = 3;
    int64 total_payout = 4;        // 总派彩金额
}
```

#### 4.4.4 订单消息定义

```protobuf
message SubmitOrderRequest {
    string order_id = 1;
    int64 user_id = 2;
    int64 market_id = 3;
    string outcome_asset = 4;      // "100_A"
    string side = 5;               // "buy" or "sell"
    string order_type = 6;         // "limit" / "market" / "ioc" / "fok" / "post_only"
    string price = 7;             // 概率价格 (0-100000000)
    string quantity = 8;          // 数量
    int64 timestamp = 9;
}

message SubmitOrderResponse {
    bool success = 1;
    string message = 2;
    string order_id = 3;
    string status = 4;            // "submitted", "filled", "cancelled"
    string filled_quantity = 5;
    repeated TradeInfo trades = 6;
}

message TradeInfo {
    string trade_id = 1;
    string order_id = 2;
    string counter_order_id = 3;
    string side = 4;
    string price = 5;
    string quantity = 6;
    string outcome_asset = 7;
    string base_amount = 8;          // USDT 金额
    int64 timestamp = 9;
}

message CancelOrderRequest {
    string order_id = 1;
    int64 user_id = 2;
    int64 market_id = 3;
    string outcome_asset = 4;
}

message CancelOrderResponse {
    bool success = 1;
    string message = 2;
}

message GetOrderBookRequest {
    int64 market_id = 1;
    string outcome_asset = 2;
    int32 limit = 3;
}

message GetOrderBookResponse {
    repeated OrderBookLevel bids = 1;
    repeated OrderBookLevel asks = 2;
    int64 timestamp = 3;
}

message OrderBookLevel {
    string price = 1;
    string quantity = 2;
    int32 orders = 3;
}

### 4.5 持久化设计

#### 4.5.1 快照

```rust
/// 订单簿快照
#[derive(Serialize, Deserialize)]
struct OrderBookSnapshot {
    market_id: i64,
    outcome_asset: String,
    timestamp: i64,
    ask_buckets: Vec<BucketSnapshot>,
    bid_buckets: Vec<BucketSnapshot>,
    last_trade_price: Option<Price>,
}

#[derive(Serialize, Deserialize)]
struct BucketSnapshot {
    price: Price,
    orders: Vec<OrderSnapshot>,
}
```

#### 4.5.2 WAL (可选 Phase 2)

```rust
/// WAL 条目
#[derive(Archive, Serialize, Deserialize)]
struct WALEntry {
    sequence: u64,
    timestamp: i64,
    command: OrderCommand,
}
```

---

## 5. 移植步骤

### Phase 1: 基础撮合

| 步骤 | 任务 | 文件 |
|------|------|------|
| 5.1 | 创建 `api/` 模块 | `src/api/types.rs`, `commands.rs`, `events.rs` |
| 5.2 | 创建 `engine/orderbook/advanced.rs` | 核心订单簿 |
| 5.3 | 创建 `engine/processors/matching_engine.rs` | 撮合处理器 |
| 5.4 | 创建 `pb/matching_engine.proto` | Proto 定义 |
| 5.5 | 创建 `services/matching_engine_impl.rs` | gRPC 服务实现 |
| 5.6 | 创建 `config.rs`, `server.rs` | 配置和启动 |
| 5.7 | 集成 Kafka 生产者 | `kafka/producer.rs` |
| 5.8 | 单元测试 | - |

### Phase 2: 市场管理 + 结算

| 步骤 | 任务 |
|------|------|
| 6.1 | MarketService 实现 | 市场创建/关闭/结算 |
| 6.2 | 多元选项支持 | 动态添加市场选项 |
| 6.3 | 批量结算 | BatchSettle 实现 |
| 6.4 | 结算事件发布 | Kafka 事件 |
| 6.5 | 市场投注限额 | MarketStakeLimit 实现 |

### Phase 3: 高级功能

| 步骤 | 任务 |
|------|------|
| 7.1 | Stop Order 支持 |
| 7.2 | WAL 持久化 (rkyv) |
| 7.3 | 快照恢复 |
| 7.4 | 分片架构 |

---

## 6. 关键设计决策

### 6.1 价格/数量精度

- **matching-core**: i64 整数，最小单位由 `scale_k` 决定
- **quorum**: i64 整数，概率模型价格范围 0-100000000 (精度6)

```rust
// 概率价格 (0-100)
Price = 65000000   // = 0.65 = 65% 概率

// 转换
fn price_to_probability(price: i64) -> f64 {
    price as f64 / 1_000_000.0
}

fn probability_to_price(p: f64) -> i64 {
    (p * 1_000_000.0) as i64
}
```

### 6.2 订单 ID

- **matching-core**: `OrderId = u64`
- **quorum**: 使用 `order_id` 字符串 (前端生成 UUID)

转换: 在 API 层做字符串 <-> u64 映射

### 6.3 结果代币订单簿

每个预测市场选项 (`{market_id}_{outcome}`) 有独立的订单簿:

```
Market 12345 (世界杯冠军)
├── 12345_A  -> OrderBook  (A队)
├── 12345_B  -> OrderBook  (B队)
└── 12345_C  -> OrderBook  (C队)
```

### 6.4 多元市场路由

订单根据 `outcome_asset` 路由到对应选项的订单簿:

```rust
fn route_order(cmd: &OrderCommand) -> &OrderBook {
    // outcome_asset = "100_A" 直接路由到 100_A 订单簿
    order_books.get(&cmd.outcome_asset)
}
```

### 6.5 结算决策

| 决策点 | 选择 | 原因 |
|--------|------|------|
| 结算触发者 | Prediction Market Service | 业务逻辑在该服务 |
| 结算执行者 | Clearing Service | 余额操作在 Account Service |
| 派彩计算 | 按概率 1:1 兑换 | 简化模型 |

### 6.6 Kafka 事件映射

| 事件 | Kafka Topic | 说明 |
|------|-------------|------|
| `OrderSubmitted` | `order_events` | 订单提交 |
| `OrderFilled` | `order_events` | 订单成交 |
| `OrderCancelled` | `order_events` | 订单取消 |
| `TradeExecuted` | `trade_executed` | 成交执行 |
| `MarketResolved` | `market_events` | 市场结算 |

---

## 7. 风险和注意事项

### 7.1 内存管理

- matching-core 使用对象池和预分配
- 需要确保订单不会无限增长
- 需要实现订单过期机制

### 7.2 并发安全

- Disruptor 模式适用于高并发
- 当前 quorum 撮合引擎使用 `parking_lot::RwLock`
- 建议保持简单：单线程撮合 + 多线程网络

### 7.3 精度处理

- 避免使用 `rust_decimal::Decimal` 进行撮合 (性能问题)
- 使用 i64 整数存储价格/数量
- 在 API 层做精度转换

---

## 8. 附录

### 8.1 依赖变更

```toml
# 新增依赖
rkyv = "0.7"           # 零拷贝序列化
ahash = "0.8"          # 快速哈希
smallvec = "1.11"      # 小向量优化
slab = "0.4"           # 对象池

# 保留依赖
parking_lot = "0.12"  # 锁
serde = { version = "1.0", features = ["derive"] }
```

### 8.2 参考文档

- matching-core README: `/home/ubuntu/code/matching-core/README.md`
- matching-core 高级订单簿: `/home/ubuntu/code/matching-core/src/core/orderbook/advanced.rs`
- 当前 quorum 撮合引擎: `/home/ubuntu/code/quorum/crates/matching-engine/`

---

## 9. Polymarket 最佳实践与系统更新

### 9.1 Polymarket 核心架构

Polymarket 是当前最成熟的预测市场实现，其关键架构：

| 维度 | Polymarket | Quorum (当前) |
|------|------------|---------------|
| 撮合模式 | CLOB + Off-chain 撮合 | gRPC 服务 + In-memory |
| 结算 | On-chain (Polygon) | Off-chain (可选) |
| 代币标准 | ERC1155 | 内部余额系统 |
| 清算 | UMA Oracle | 手动/外部 |
| 费用 | 动态费率 (分类) | 固定费率 |

### 9.2 核心差异分析

#### A. 费用体系
Polymarket 采用分类动态费率：

| 分类 | Taker 费率 | Maker 返利 |
|------|-----------|-----------|
| Crypto | 7.2% | 20% |
| Sports | 3.0% | 25% |
| Finance/Politics | 4.0% | 25% |
| Geopolitical | **0%** | N/A |

**Quorum 需更新**: 引入 MarketCategory 枚举和动态费率计算。

#### B. 事件容器抽象
Polymarket 通过 Event 容器管理多市场：

```rust
// Polymarket 模型
Event {
    event_id: String,
    event_type: Single | Multi,
    markets: Vec<Market>,  // 多市场互斥
}
```

**Quorum 需更新**: 增加 Event 容器支持多市场关联。

#### C. Negative Risk 机制
Polymarket 通过 **Neg Risk** 实现多结果事件的持仓关联：

```
Neg Risk 转换:
"某结果的 No 份额" → "其他所有结果的 Yes 份额"

示例: 2024 美国总统大选
- 持有 "Other" 的 1 份 No
- 可转换为 "Trump" + "Harris" 的各 1 份 Yes
```

| 组件 | 说明 |
|------|------|
| Neg Risk Adapter | 转换合约 |
| negRisk 字段 | 标识市场是否支持 |

**Quorum 建议**: 在 Clearing Service 层实现 No→Yes 转换，不在撮合引擎处理。

#### D. 订单簿增强
Polymarket 订单簿提供更丰富的市场统计：

```rust
// 需增强的字段
MarketStats {
    bid_depth: i64,      // 买盘深度
    ask_depth: i64,      // 卖盘深度
    spread: Price,       // 价差
    spread_percent: f64, // 价差百分比
}
```

### 9.3 系统更新建议

#### 高优先级

| 更新项 | 当前设计 | 建议更新 | 原因 |
|--------|----------|----------|------|
| 费用体系 | `taker_fee: i64` 固定 | `FeeConfig { category, taker_rate, maker_rebate }` | 匹配 Polymarket 分类费率 |
| 市场统计 | 基础 | 增加 bid_depth, ask_depth, spread | 提供更丰富的行情数据 |
| Event 容器 | 无 | `Event { id, event_type, markets }` | 支持多市场关联 |

#### 中优先级

| 更新项 | 说明 |
|--------|------|
| EIP-712 订单签名 | 增强安全性，支持批量订单 |
| 做市商激励 | MakerRebate 计划 |
| 动态费率引擎 | 根据市场类型调整费率 |

### 9.4 架构优化建议

| 当前设计 | 优化方向 | 说明 |
|----------|----------|------|
| 单一 gRPC 服务 | CLOB 分离 | 撮合服务 + 清算服务分离 |
| 内存存储 | Redis 集群 | 支持分布式部署 |
| 固定费率 | 动态费率 | 根据市场分类和流动性调整 |

### 9.5 参考资料

- [Polymarket Docs](https://docs.polymarket.com)
- [Polymarket Research](docs/tasks/polymarket-research.md)
- [UMA Optimistic Oracle](https://docs.umaproject.org/)
- [Gnosis Conditional Token Framework](https://docs.gnosis.io/conditionaltokens/)
