# Portfolio Service

合并自: Account Service + Position Service + Clearing Service + Ledger Service

## 职责

- 账户余额管理 (冻结/解冻/扣减/增加)
- 持仓管理 (开仓/平仓/盈亏)
- 结算清算 (Taker/Maker手续费/资金划转)
- 账本流水 (不可变记录)

## 端口

待定

## 模块

- `account.rs` - 账户余额
- `position.rs` - 持仓管理
- `clearing.rs` - 结算清算
- `ledger.rs` - 账本流水

## 数据模型

```rust
Account { id, user_id, asset, available, frozen }
Position { id, user_id, market_id, outcome_id, side, size, entry_price }
Settlement { id, trade_id, amount, fee, payout, status }
LedgerEntry { id, ledger_type, asset, amount, balance_after, reference_id }
```

## Kafka

消费: `match.events`
