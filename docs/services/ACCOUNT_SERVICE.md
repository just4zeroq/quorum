# Account Service - 账户服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50019 (内部服务，无公网) |
| 协议 | gRPC (内部调用) |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | 无 (被其他服务调用) |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 余额管理 | Available/Frozen/Locked 余额管理 |
| 冻结/解冻 | 下单冻结、成交/取消解冻 |
| 充值/提现 | 账户充值、提现扣账 |
| 内部划转 | 用户间资金划转 |
| 风控联动 | 风控锁定/解锁账户余额 |
| 结算派彩 | 预测市场结算时结果代币兑换为基础资产 |

> **预测市场资产模型**: 账户支持两类资产：
> - **基础资产**: USDT (用于下单购买)
> - **结果代币**: 格式为 `{market_id}_{outcome}` (如 `12345_yes`, `12345_no`)，代表用户在某个市场某个结果上的持仓
>
> 交易流程：买入时 USDT 被冻结/扣减 -> 成交后增加结果代币余额；结算时获胜结果代币兑换回 USDT。

### 1.1.2 功能列表

```
Account Service
├── 余额管理
│   ├── GetBalance - 获取余额
│   └── GetBalances - 获取所有余额
├── 冻结/解冻
│   ├── Freeze - 冻结余额 (下单)
│   ├── Unfreeze - 解冻余额 (取消/成交)
│   ├── FreezeAndDeduct - 冻结并扣减 (成交)
│   └── ⚠️ CheckAndFreeze - 原子检查并冻结 (推荐)
├── 充值/提现
│   ├── Deposit - 充值
│   └── Withdraw - 提现
├── 划转
│   └── Transfer - 内部转账
└── 风控
    ├── CheckBalance - 检查余额是否足够
    └── CheckFrozen - 检查冻结是否足够
```

> ⚠️ **重要**: 强烈建议使用 `CheckAndFreeze` 代替 `CheckBalance` + `Freeze` 两步操作，以避免并发情况下的余额检查通过但冻结失败问题。

## 2. 数据模型

### 2.1 Domain Model

```rust
// domain/src/account/model/mod.rs

/// 账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub user_id: i64,
    pub asset: String,           // 资产标识: "USDT" (基础资产) 或 "{market_id}_{outcome}" (结果代币，如 "12345_yes")
    pub available: Decimal,   // 可用余额
    pub frozen: Decimal,       // 冻结余额 (下单冻结)
    pub locked: Decimal,       // 锁定余额 (风控锁定，不可交易)
    pub created_at: i64,
    pub updated_at: i64,
}

/// 余额操作记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceOperation {
    pub id: i64,
    pub account_id: i64,
    pub user_id: i64,
    pub asset: String,
    pub operation_type: BalanceOperationType,
    pub amount: Decimal,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
    pub frozen_before: Decimal,
    pub frozen_after: Decimal,
    pub reason: String,
    pub ref_id: String,           // 关联ID: order_id, trade_id, etc.
    pub created_at: i64,
}

/// 余额操作类型
pub enum BalanceOperationType {
    Deposit,          // 充值
    Withdraw,         // 提现
    Freeze,           // 冻结 (下单)
    Unfreeze,         // 解冻 (撤单)
    Deduct,           // 扣减 (成交消耗)
    TransferIn,       // 转入
    TransferOut,      // 转出
    Fee,              // 手续费
    Lock,             // 风控锁定
    Unlock,           // 风控解锁
    Settlement,       // 结算派彩 (结果代币 -> 基础资产)
}
```

### 2.2 数据库表

**accounts 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| user_id | BIGINT | NOT NULL | 用户ID |
| asset | VARCHAR(32) | NOT NULL | 资产标识: "USDT" 或 "{market_id}_{outcome}" |
| available | TEXT | NOT NULL DEFAULT '0' | 可用余额 |
| frozen | TEXT | NOT NULL DEFAULT '0' | 冻结余额 |
| locked | TEXT | NOT NULL DEFAULT '0' | 锁定余额 |
| created_at | BIGINT | NOT NULL | 创建时间 |
| updated_at | BIGINT | NOT NULL | 更新时间 |

**索引**:
- `PRIMARY KEY (id)`
- `UNIQUE INDEX idx_user_asset (user_id, asset)`
- `INDEX idx_user_id (user_id)`

**balance_operations 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| account_id | BIGINT | NOT NULL | 账户ID |
| user_id | BIGINT | NOT NULL | 用户ID |
| asset | VARCHAR(32) | NOT NULL | 资产标识: "USDT" 或 "{market_id}_{outcome}" |
| operation_type | VARCHAR(20) | NOT NULL | 操作类型 |
| amount | TEXT | NOT NULL | 操作金额 |
| balance_before | TEXT | NOT NULL | 操作前余额 |
| balance_after | TEXT | NOT NULL | 操作后余额 |
| frozen_before | TEXT | NOT NULL | 操作前冻结 |
| frozen_after | TEXT | NOT NULL | 操作后冻结 |
| reason | VARCHAR(255) | | 原因 |
| ref_id | VARCHAR(64) | | 关联ID |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `INDEX idx_account_id (account_id)`
- `INDEX idx_user_id (user_id)`
- `INDEX idx_ref_id (ref_id)`
- `INDEX idx_created_at (created_at)`

## 3. Proto 接口定义

### 3.1 服务定义

```protobuf
syntax = "proto3";

package account;

service AccountService {
    // 余额查询
    rpc GetBalance(GetBalanceRequest) returns (GetBalanceResponse);
    rpc GetBalances(GetBalancesRequest) returns (GetBalancesResponse);

    // 冻结/解冻 (Order Service 调用)
    rpc Freeze(FreezeRequest) returns (FreezeResponse);
    rpc Unfreeze(UnfreezeRequest) returns (UnfreezeResponse);
    rpc FreezeAndDeduct(FreezeAndDeductRequest) returns (FreezeAndDeductResponse);
    rpc CheckAndFreeze(CheckAndFreezeRequest) returns (CheckAndFreezeResponse);  // ⚠️ 推荐使用

    // 充值/提现 (Wallet Service 调用)
    rpc Deposit(DepositRequest) returns (DepositResponse);
    rpc Withdraw(WithdrawRequest) returns (WithdrawResponse);

    // 划转 (内部服务调用)
    rpc Transfer(TransferRequest) returns (TransferResponse);

    // 风控锁定/解锁 (Risk Service 调用)
    rpc Lock(LockRequest) returns (LockResponse);
    rpc Unlock(UnlockRequest) returns (UnlockResponse);

    // 风控查询
    rpc CheckBalance(CheckBalanceRequest) returns (CheckBalanceResponse);
    rpc CheckFrozen(CheckFrozenRequest) returns (CheckFrozenResponse);

    // 结算派彩 (Clearing Service 调用)
    rpc Settle(SettleRequest) returns (SettleResponse);

    // 批量操作 (用于对账)
    rpc BatchGetBalances(BatchGetBalancesRequest) returns (BatchGetBalancesResponse);
}
```

### 3.2 消息定义

```protobuf
// ==================== 余额查询 ====================

message GetBalanceRequest {
    int64 user_id = 1;
    string asset = 2;
}

message GetBalanceResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    int64 user_id = 4;
    string asset = 5;
    string available = 6;
    string frozen = 7;
    string locked = 8;
}

message GetBalancesRequest {
    int64 user_id = 1;
}

message GetBalancesResponse {
    repeated Balance balances = 1;
}

message Balance {
    int64 account_id = 1;
    string asset = 2;
    string available = 3;
    string frozen = 4;
    string locked = 5;
}

// ==================== 冻结/解冻 ====================

message FreezeRequest {
    int64 user_id = 1;
    string asset = 2;
    string amount = 3;
    string order_id = 4;
    string reason = 5;
}

message FreezeResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    string available_before = 4;
    string available_after = 5;
    string frozen_before = 6;
    string frozen_after = 7;
}

message UnfreezeRequest {
    int64 user_id = 1;
    string asset = 2;
    string amount = 3;
    string order_id = 4;
    string reason = 5;
}

message UnfreezeResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    string available_before = 4;
    string available_after = 5;
    string frozen_before = 6;
    string frozen_after = 7;
}

message FreezeAndDeductRequest {
    int64 user_id = 1;
    string asset = 2;              // 基础资产 (USDT)
    string freeze_amount = 3;      // 从 frozen 中扣减的金额
    string deduct_amount = 4;      // 实际扣减金额 (可能含手续费差额)
    string order_id = 5;
    string reason = 6;
    string outcome_asset = 7;      // 结果代币标识 (如 "12345_yes")，为空则不增加结果代币
    string outcome_amount = 8;     // 增加的结果代币数量
}

message FreezeAndDeductResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    string available_before = 4;
    string available_after = 5;
    string frozen_before = 6;
    string frozen_after = 7;
}

// ==================== 原子操作 (推荐) ====================

message CheckAndFreezeRequest {
    int64 user_id = 1;
    string asset = 2;
    string amount = 3;
    string order_id = 4;
    string reason = 5;
}

message CheckAndFreezeResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    string available_before = 4;
    string available_after = 5;
    string frozen_before = 6;
    string frozen_after = 7;
}

// ==================== 充值/提现 ====================

message DepositRequest {
    int64 user_id = 1;
    string asset = 2;
    string amount = 3;
    string tx_id = 4;
    string reason = 5;
}

message DepositResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    string available_before = 4;
    string available_after = 5;
}

message WithdrawRequest {
    int64 user_id = 1;
    string asset = 2;
    string amount = 3;
    string withdraw_id = 4;
    string reason = 5;
}

message WithdrawResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    string available_before = 4;
    string available_after = 5;
}

// ==================== 划转 ====================

message TransferRequest {
    int64 from_user_id = 1;
    int64 to_user_id = 2;
    string asset = 3;
    string amount = 4;
    string transfer_id = 5;
    string reason = 6;
}

message TransferResponse {
    bool success = 1;
    string message = 2;
    string from_available_before = 3;
    string from_available_after = 4;
    string to_available_before = 5;
    string to_available_after = 6;
}

// ==================== 风控 ====================

message CheckBalanceRequest {
    int64 user_id = 1;
    string asset = 2;
    string amount = 3;
}

message CheckBalanceResponse {
    bool sufficient = 1;
    string available = 2;
    string required = 3;
}

message CheckFrozenRequest {
    int64 user_id = 1;
    string asset = 2;
    string order_id = 3;
}

message CheckFrozenResponse {
    bool sufficient = 1;
    string frozen = 2;
    string required = 3;
}

// ==================== 风控锁定/解锁 ====================

message LockRequest {
    int64 user_id = 1;
    string asset = 2;
    string amount = 3;           // 锁定金额，为空则锁定全部
    string reason = 4;           // 锁定原因
}

message LockResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    string available_before = 4;
    string available_after = 5;
    string locked_before = 6;
    string locked_after = 7;
}

message UnlockRequest {
    int64 user_id = 1;
    string asset = 2;
    string amount = 3;           // 解锁金额，为空则解锁全部
    string reason = 4;           // 解锁原因
}

message UnlockResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    string available_before = 4;
    string available_after = 5;
    string locked_before = 6;
    string locked_after = 7;
}

// ==================== 结算派彩 ====================

message SettleRequest {
    int64 user_id = 1;
    string outcome_asset = 2;    // 结算的结果代币 (如 "12345_yes")
    string outcome_amount = 3;   // 消耗的结果代币数量
    string base_asset = 4;       // 兑换的基础资产 (USDT)
    string base_amount = 5;      // 兑换获得的基础资产数量
    int64 market_id = 6;
    string reason = 7;
}

message SettleResponse {
    bool success = 1;
    string message = 2;
    string outcome_available_after = 3;  // 结果代币剩余余额
    string base_available_after = 4;     // 基础资产增加后余额
}

// ==================== 批量操作 ====================

message BatchGetBalancesRequest {
    repeated int64 user_ids = 1;
    repeated string assets = 2;
}

message BatchGetBalancesResponse {
    repeated Balance balances = 1;
}
```

## 4. Kafka 事件

### 4.1 生产的事件

| 事件 | Topic | 说明 |
|------|-------|------|
| Deposited | balance_updates | 充值成功 |
| Withdrawn | balance_updates | 提现成功 |
| Frozen | balance_updates | 冻结成功 |
| Unfrozen | balance_updates | 解冻成功 |
| Deducted | balance_updates | 成交扣减成功 |
| Transferred | balance_updates | 划转成功 |
| Locked | balance_updates | 风控锁定 |
| Unlocked | balance_updates | 风控解锁 |
| Settled | balance_updates | 结算派彩 |

### 4.2 事件 Schema

```rust
// domain/src/account/event/mod.rs

pub enum AccountEvent {
    Deposited {
        user_id: i64,
        asset: String,
        amount: String,
        balance_after: String,
        tx_id: String,
    },
    Withdrawn {
        user_id: i64,
        asset: String,
        amount: String,
        balance_after: String,
        withdraw_id: String,
    },
    Frozen {
        user_id: i64,
        asset: String,
        amount: String,
        available_after: String,
        frozen_after: String,
        order_id: String,
    },
    Unfrozen {
        user_id: i64,
        asset: String,
        amount: String,
        available_after: String,
        frozen_after: String,
        order_id: String,
    },
    Deducted {
        user_id: i64,
        asset: String,
        amount: String,
        frozen_after: String,
        order_id: String,
    },
    Transferred {
        from_user_id: i64,
        to_user_id: i64,
        asset: String,
        amount: String,
        transfer_id: String,
    },
    Locked {
        user_id: i64,
        asset: String,
        amount: String,
        locked_after: String,
        reason: String,
    },
    Unlocked {
        user_id: i64,
        asset: String,
        amount: String,
        locked_after: String,
        reason: String,
    },
    Settled {
        user_id: i64,
        outcome_asset: String,    // 结算的结果代币 (如 "12345_yes")
        outcome_amount: String,   // 消耗的结果代币数量
        base_asset: String,       // 兑换的基础资产 (USDT)
        base_amount: String,      // 兑换获得的基础资产数量
        market_id: i64,
    },
}
```

## 5. 服务间通信

### 5.1 gRPC 调用 (被调用)

| 调用方 | 接口 | 说明 |
|--------|------|------|
| Order Service | Freeze | 下单时冻结余额 |
| Order Service | Unfreeze | 撤单时解冻余额 |
| Order Service | FreezeAndDeduct | 成交时扣减冻结并增加结果代币 |
| Order Service | CheckBalance | 下单前检查余额 |
| Clearing Service | Freeze | 结算时冻结 |
| Clearing Service | Unfreeze | 结算时解冻 |
| Clearing Service | Settle | 结算派彩：结果代币兑换基础资产 |
| Wallet Service | Deposit | 充值入账 |
| Wallet Service | Withdraw | 提现扣账 |
| Position Service | CheckBalance | 检查余额 |
| Risk Service | Lock | 风控锁定余额 |
| Risk Service | Unlock | 风控解锁余额 |

### 5.2 Kafka 消费

无（Account Service 不消费任何事件，纯同步 gRPC 调用）

## 6. 配置

```yaml
# crates/account-service/config/account_service.yaml
service:
  host: "0.0.0.0"
  port: 50019  # 内部服务，无公网端口

database:
  driver: "sqlite"
  url: "sqlite:./data/accounts.db"
  max_connections: 20

# 支持的资产
assets:
  # 基础资产 (用于下单购买)
  base_assets:
    - USDT
  # 结果代币 (格式: {market_id}_{outcome})，由系统自动创建，无需预配置
  # 例如: 12345_yes, 12345_no

# 风控配置
risk:
  # 单笔最大充值
  max_deposit: "1000000"
  # 单笔最大提现
  max_withdraw: "100000"
  # 冻结比例限制 (可选)
  freeze_ratio_limit: "1.0"
```

## 7. 错误码

| 错误码 | 说明 |
|--------|------|
| ACCOUNT_NOT_FOUND | 账户不存在 |
| ASSET_NOT_SUPPORTED | 资产不支持 |
| INSUFFICIENT_BALANCE | 可用余额不足 |
| INSUFFICIENT_FROZEN | 冻结余额不足 |
| INSUFFICIENT_LOCKED | 锁定余额不足 (解锁时) |
| AMOUNT_INVALID | 金额无效 |
| AMOUNT_TOO_SMALL | 金额太小 |
| AMOUNT_TOO_LARGE | 金额太大 |
| USER_FROZEN | 用户已冻结 |
| ACCOUNT_LOCKED | 账户被风控锁定 |
| OUTCOME_ASSET_INVALID | 结果代币标识无效 |
| SETTLEMENT_FAILED | 结算派彩失败 |
| DUPLICATE_OPERATION | 重复操作 |
| OPERATION_TIMEOUT | 操作超时 |

## 8. 核心逻辑

### 8.1 冻结流程 (下单)

```rust
async fn freeze(&self, req: FreezeRequest) -> Result<FreezeResponse, Error> {
    // 1. 检查余额是否足够
    let balance = self.get_balance(req.user_id, &req.asset).await?;
    if balance.available < req.amount {
        return Err(Error::InsufficientBalance);
    }

    // 2. 原子操作：扣减 available，增加 frozen
    let op = BalanceOperation::new(
        req.user_id,
        &req.asset,
        BalanceOperationType::Freeze,
        req.amount,
        balance.available,
        balance.available - req.amount,
        balance.frozen,
        balance.frozen + req.amount,
        &req.order_id,
    );

    // 3. 记录操作
    self.record_operation(&op).await?;

    // 4. 发布事件
    self.publish_event(AccountEvent::Frozen {
        user_id: req.user_id,
        asset: req.asset,
        amount: req.amount,
        order_id: req.order_id,
    }).await?;

    Ok(response)
}
```

### ⚠️ 8.2 原子操作：检查并冻结 (推荐)

> 强烈推荐使用此方法代替 `CheckBalance` + `Freeze` 两步操作！

```rust
/// 原子操作：检查余额并冻结
/// 避免两步操作之间的时间窗口导致余额检查通过但冻结失败
async fn check_and_freeze(&self, req: CheckAndFreezeRequest) -> Result<CheckAndFreezeResponse, Error> {
    // 在事务中执行：检查余额 -> 冻结
    self.transaction(|conn| async move {
        // 1. FOR UPDATE 锁定账户行，防止并发
        let balance = self.get_balance_for_update(conn, req.user_id, &req.asset).await?;

        // 2. 检查余额是否足够
        if balance.available < req.amount {
            return Err(Error::InsufficientBalance);
        }

        // 3. 原子操作：扣减 available，增加 frozen
        let new_available = balance.available - req.amount;
        let new_frozen = balance.frozen + req.amount;

        // 4. 更新账户
        self.update_balance(conn, req.user_id, &req.asset, new_available, new_frozen).await?;

        // 5. 记录操作
        let op = BalanceOperation::new(
            req.user_id,
            &req.asset,
            BalanceOperationType::Freeze,
            req.amount,
            balance.available,
            new_available,
            balance.frozen,
            new_frozen,
            &req.order_id,
        );
        self.record_operation(conn, &op).await?;

        Ok(response)
    }).await
}
```

### 8.3 冻结并扣减流程 (成交)

> 预测市场成交场景：买家 USDT 从 frozen 中扣减（消耗掉），同时增加买家对应结果代币余额。
> 例如：买入 yes_outcome，USDT frozen 扣减，`{market_id}_yes` available 增加。

```rust
async fn freeze_and_deduct(&self, req: FreezeAndDeductRequest) -> Result<FreezeAndDeductResponse, Error> {
    // 在事务中执行，防止并发问题
    self.transaction(|conn| async move {
        // 1. FOR UPDATE 锁定账户行
        let balance = self.get_balance_for_update(conn, req.user_id, &req.asset).await?;

        // 2. 检查冻结余额是否足够
        if balance.frozen < req.freeze_amount {
            return Err(Error::InsufficientFrozen);
        }

        // 3. 从 frozen 中扣减 (成交消耗，资金不返回 available)
        let new_frozen = balance.frozen - req.freeze_amount;
        let new_available = balance.available; // available 不变

        // 4. 更新账户余额
        self.update_balance(conn, req.user_id, &req.asset, new_available, new_frozen).await?;

        // 5. 如果指定了结果代币，增加买方的结果代币余额
        if !req.outcome_asset.is_empty() {
            let outcome_balance = self.get_or_create_balance(conn, req.user_id, &req.outcome_asset).await?;
            let new_outcome_available = outcome_balance.available + req.outcome_amount;
            self.update_available(conn, req.user_id, &req.outcome_asset, new_outcome_available).await?;
        }

        // 6. 记录操作
        let op = BalanceOperation::new(
            req.user_id,
            &req.asset,
            BalanceOperationType::Deduct,
            req.freeze_amount,
            balance.available,
            new_available,
            balance.frozen,
            new_frozen,
            &req.order_id,
        );
        self.record_operation(conn, &op).await?;

        Ok(response)
    }).await
}
```

## 9. 目录结构

```
crates/account-service/
├── Cargo.toml
├── build.rs
├── config/
│   └── account_service.yaml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── config.rs
    ├── server.rs
    ├── error.rs
    ├── pb.rs
    ├── pb/
    │   ├── account.proto
    │   └── account.rs
    ├── repository/
    │   ├── mod.rs
    │   ├── account_repo.rs
    │   └── operation_repo.rs
    ├── services/
    │   ├── mod.rs
    │   └── account_service_impl.rs
    └── utils/
        └── mod.rs
```
