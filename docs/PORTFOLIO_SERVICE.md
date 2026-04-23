# Portfolio Service 设计文档

## 概述

Portfolio Service 是合并后的服务，整合了 Account、Position、Clearing、Ledger 四个服务的职责。

**合并自：**
- Account Service (账户余额管理)
- Position Service (持仓管理)
- Clearing Service (结算清算)
- Ledger Service (账本流水)

---

## 服务职责

| 模块 | 职责 |
|------|------|
| Account | 余额冻结/解冻/扣减/增加 |
| Position | 开仓/平仓/盈亏计算 |
| Clearing | Taker/Maker 手续费/资金划转/市场结算 |
| Ledger | 不可变流水记录 |

---

## 数据模型

### Account

```rust
struct Account {
    id: String,
    user_id: String,
    asset: String,           // "USDC", "BTC"
    account_type: AccountType, // Spot, Futures
    available: Decimal,       // 可用余额
    frozen: Decimal,         // 冻结余额
}
```

### Position

```rust
struct Position {
    id: String,
    user_id: String,
    market_id: u64,
    outcome_id: u64,
    side: PositionSide,      // Long(买YES) / Short(买NO)
    size: Decimal,           // 持仓数量
    entry_price: Decimal,    // 开仓价格
}
```

### Settlement

```rust
struct Settlement {
    id: String,
    trade_id: String,
    amount: Decimal,
    fee: Decimal,
    payout: Decimal,         // 派彩
    status: SettlementStatus,
}
```

### LedgerEntry

```rust
struct LedgerEntry {
    id: String,
    ledger_type: LedgerType,  // Deposit/Withdraw/Trade/Settle
    asset: String,
    amount: Decimal,
    balance_after: Decimal,
    reference_id: String,   // 关联订单/成交ID
}
```

---

## 资金流向

```
用户下单
    ↓
Risk Service 检查
    ↓
Order Service 创建订单
    ↓
Matching Engine 撮合
    ↓
Portfolio Service:
  ├── Account.freeze(冻结保证金)
  ├── Position.open(记录持仓)
  ├── Clearing.settle(计算盈亏)
  ├── Account.debit/credit(资金划转)
  └── Ledger.record(记录流水)
```

---

## API

```protobuf
service PortfolioService {
    // 账户
    rpc GetBalance(GetBalanceRequest) returns (GetBalanceResponse);
    rpc Freeze(FreezeRequest) returns (FreezeResponse);
    rpc Unfreeze(UnfreezeRequest) returns (UnfreezeResponse);

    // 持仓
    rpc GetPositions(GetPositionsRequest) returns (GetPositionsResponse);
    rpc GetPosition(GetPositionRequest) returns (GetPositionResponse);

    // 清算
    rpc SettleTrade(SettleTradeRequest) returns (SettleTradeResponse);

    // 账本
    rpc GetLedger(GetLedgerRequest) returns (GetLedgerResponse);
}
```

---

## Kafka 消费

| 主题 | 说明 |
|------|------|
| `match.events` | 撮合成交事件 |
| `account.freeze` | 冻结请求 |
| `account.unfreeze` | 解冻请求 |

---

## 错误码

| 错误码 | 说明 |
|--------|------|
| `INSUFFICIENT_BALANCE` | 余额不足 |
| `INSUFFICIENT_POSITION` | 持仓不足 |
| `ACCOUNT_NOT_FOUND` | 账户不存在 |
| `POSITION_NOT_FOUND` | 持仓不存在 |
