# Polymarket 调研报告

## 1. Polymarket 概述

Polymarket 是一个基于 Polygon 的预测市场协议，采用混合去中心化模式：
- **Off-chain 撮合**: CLOB (中央限价订单簿) 进行订单匹配
- **On-chain 结算**: 在 Polygon 上进行代币结算
- **非托管交易**: 用户资产始终在控制之中

### 核心数据
- 2024 年交易量超过 10 亿美元
- 基于 UMA Optimistic Oracle 进行去中心化清算
- 支持 USDC 稳定币交易

---

## 2. 市场结构

### 2.1 Market (市场)
Market 是可交易的基本单元，代表一个二元问题 (Yes/No)。

```
关键标识:
- Condition ID: CTF 合约标识
- Question ID: 用于清算的哈希问题
- Token ID: ERC1155 代币标识，用于 CLOB 交易
```

### 2.2 Event (事件)
Event 是市场容器，聚合一个或多个相关市场：

| 类型 | 说明 | 示例 |
|------|------|------|
| Single-market | 单一市场 | "Bitcoin 2024 年会超过 $100k?" |
| Multi-market | 多市场互斥 | "2024 总统大选" → Trump/Biden/Other |

### 2.3 代币模型
- 使用 **ERC1155** 标准
- 每笔交易生成 **成对的代币** (Yes + No)
- 每个正确的代币可兑换 **$1.00**

```
交易流程:
1. Split: $1 → 1 Yes + 1 No
2. Trade: 在订单簿上交易 Yes/No
3. Merge: 等量的 Yes + No → $1
4. Redeem: 获胜代币 → $1
```

---

## 3. 订单簿与定价

### 3.1 CLOB 混合模式
```
Off-chain 撮合 + On-chain 结算
- 运营商维护订单簿进行撮合
- 交易在 Polygon 上原子结算
- EIP-712 签名确保订单真实性
```

### 3.2 价格机制
- 价格范围: **$0.00 - $1.00**
- 直接代表市场概率
- 显示价格为 **买卖价差中点** 或 **最新成交价**

### 3.3 订单类型
| 类型 | 说明 |
|------|------|
| Market Order | 即时成交，按最优价格 |
| Limit Order | 指定价格或更优价格 |
| Post-Only | 只挂单不主动成交 |

- 支持**部分成交**和**取消**

### 3.4 流动性机制
- **做市商奖励计划**: 按比例返还手续费
- **Tick Size**: 根据价格区间调整

---

## 4. 费用结构

### 4.1 费率计算
```
fee = C × feeRate × p × (1 - p)

C = 成交数量
p = 成交价格
```

### 4.2 分类费率

| 分类 | Taker 费率 | Maker 返利比例 |
|------|-----------|---------------|
| Crypto | 7.2% | 20% |
| Sports | 3.0% | 25% |
| Finance/Politics/Tech | 4.0% | 25% |
| Econ/Culture/Weather | 5.0% | 25% |
| Geopolitical | **0%** | N/A |

### 4.3 特点
- 只对 Taker 收费，Maker 获得返利
- 费用在成交时计算
- Geopolitical 市场免手续费

---

## 5. 清算机制

### 5.1 UMA Optimistic Oracle
```
提案 → 挑战期 (2小时) → 无争议清算 (~2小时)
                        ↘ 有争议 → 投票 (4-6天)
```

### 5.2 流程
1. 任何人可提案结果 + 保证金
2. 2 小时挑战期
3. 无争议 → 自动清算
4. 有争议 → 升级至 UMA 代币持有者投票

---

## 6. Negative Risk (负风险) 机制

Polymarket 针对多结果事件实现了 **Negative Risk** 机制：

### 核心原理
在标准多结果事件中，每个市场独立 - 看空某结果需购买该结果的 No 代币。

**Neg Risk 创新**: "任何市场的 **No 份额**可以转换为**其他所有市场的各 1 份 Yes 份额**"

```
2024 美国总统大选 (Neg Risk 事件):
├── Trump 市场: Yes @ 0.52
├── Harris 市场: Yes @ 0.47
└── Other 市场: Yes @ 0.01

持有 "Other" 的 1 份 No
    ↓ 转换
获得 Trump Yes + Harris Yes 各 1 份
```

### 技术实现

| 组件 | 说明 |
|------|------|
| Neg Risk Adapter | 转换合约 |
| Neg Risk CTF Exchange | 交易合约 |
| negRisk 字段 | Gamma API 标识 |

### 资金效率提升

| 操作 | 标准模式 | Neg Risk 模式 |
|------|----------|---------------|
| 看空 Trump | 买入 Trump No | 买入其他结果 No |
| 成本 | 1 - 0.52 = 0.48 | 0.01 (Other) |
| 效果 | 单独对冲 | 同时做多 Harris |

### 对 Quorum 的启发

当前 Quorum 设计中每个 `outcome_asset` 有独立订单簿，**撮合引擎无跨选项约束**。

但 **Neg Risk 机制** 提供了另一种关联模式：

```rust
// 建议新增: NegRiskEvent 容器
pub struct NegRiskEvent {
    event_id: i64,
    outcomes: Vec<OutcomeAsset>,  // 关联的选项
    neg_risk_enabled: bool,
}

// 转换操作 (在账户层实现)
fn convert_no_to_yes(
    &self,
    user_id: i64,
    from_outcome: &str,  // "100_Other"
    to_outcomes: Vec<&str>,  // ["100_Trump", "100_Harris"]
) -> Result<()>;
```

**实现建议**: 在 **Clearing Service** 或 **Account Service** 层实现转换逻辑，不在撮合引擎层处理。

---

## 7. 与 Quorum 设计对比

### 7.1 架构对比

| 维度 | Polymarket | Quorum (当前设计) |
|------|------------|-------------------|
| 撮合模式 | CLOB + Off-chain | gRPC 服务 + In-memory |
| 结算 | On-chain (Polygon) | Off-chain (可选) |
| 代币标准 | ERC1155 | 内部余额系统 |
| 清算 | UMA Oracle | 手动/外部 |
| 费用 | 动态费率 | 固定费率 |

### 7.2 多市场约束对比

**Polymarket**:
- 多市场通过 Event 容器关联
- 互斥性通过代币兑换机制隐式保证
- Neg Risk 实现跨市场持仓关联

**Quorum (设计)**:
- MarketStakeLimit 显式约束
- 独立的 OrderBook per outcome_asset
- 支持概率约束、保证金约束

---

## 8. 系统更新建议

### 8.1 高优先级更新

#### A. 费用体系重构
```rust
// 当前设计缺少动态费率机制
pub struct FeeConfig {
    taker_fee_rate: f64,
    maker_rebate_rate: f64,
    category: MarketCategory,
}

// 建议按市场分类配置费率
```

#### B. 事件容器抽象
```rust
// 新增 Event 容器
pub struct Event {
    event_id: String,
    event_type: EventType,  // Single, Multi
    markets: Vec<MarketId>,
    resolved_outcome: Option<String>,
}
```

#### C. 订单簿深度增强
```rust
// 增强市场统计
pub struct MarketStats {
    total_trades: u64,
    total_volume: i64,
    last_price: Option<Price>,
    bid_depth: i64,      // 新增
    ask_depth: i64,      // 新增
    spread: Price,       // 新增
}
```

### 8.2 中优先级更新

#### D. EIP-712 订单签名 (可选)
- 离线签名增强安全性
- 支持批量订单

#### E. 做市商激励
```rust
pub struct MakerReward {
    user_id: u64,
    market_id: i64,
    rebate_amount: i64,
    period: i64,
}
```

### 8.3 架构优化建议

| 当前设计 | 优化方向 |
|----------|----------|
| 单一 gRPC 服务 | 考虑 CLOB 分离 (撮合服务 + 清算服务) |
| 内存存储 | 引入 Redis 集群支持分布式部署 |
| 固定费率 | 动态费率引擎 |

---

## 9. 结论

Polymarket 展示了预测市场的成熟形态：

1. **混合架构** 是最佳实践: Off-chain 撮合 + On-chain 结算
2. **ERC1155 代币模型** 提供了标准化的头寸表示
3. **动态费率** 激励流动性提供者
4. **去中心化清算** 确保公平性

### Quorum 需要重点关注

1. ✅ **费用体系**: 需要支持分类费率和 Maker 返利
2. ✅ **Event 抽象**: 支持多市场容器
3. ✅ **统计增强**: 深度、价差等指标
4. ⚠️ **区块链集成**: 考虑支持 Polygon/Base 等 L2

### 参考资料

- [Polymarket Docs](https://docs.polymarket.com)
- [Polymarket API](https://docs.polymarket.com/api)
- [Gnosis Conditional Token Framework](https://docs.gnosis.io/conditionaltokens/)
- [UMA Protocol](https://docs.umaproject.org/)
