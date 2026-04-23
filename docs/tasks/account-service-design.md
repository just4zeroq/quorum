# Account Service 详细设计文档

## 1. 现状分析

### 1.1 当前实现

当前 `crates/account-service` 是一个基于 Axum HTTP 的内存原型：

| 项目 | 当前 | 目标 |
|------|------|------|
| 协议 | Axum HTTP (端口 8083) | Tonic gRPC (端口 50019) |
| 存储 | 内存 HashMap + parking_lot::RwLock | SQLite/PostgreSQL (common/db) |
| 资产模型 | AccountType(Funding/Spot/Futures) | 预测市场双层资产: USDT + 结果代币 |
| 余额字段 | available/frozen/equity (Decimal) | available/frozen/locked (i64 整数) |
| 金额精度 | Decimal 字符串 | i64 整数存储 + 精度存于账户记录 |
| 持久化 | 无，重启丢失 | 数据库持久化 + 操作记录 |
| 事件发布 | 无 | Kafka (balance_updates) |
| Proto | 无 | account.proto |
| DB 组件 | 无 | 依赖 common/db (DBPool/DBManager/Config) |

### 1.2 核心改动

1. **从 Axum HTTP 迁移到 Tonic gRPC** — 与其他服务保持一致
2. **使用 common/db 组件** — DBPool/DBManager/Config/MergedConfig，支持 SQLite/PostgreSQL 双模式
3. **金额使用 i64 整数存储** — 精度随账户记录持久化，创建时写入
4. **适配预测市场资产模型** — 支持结果代币 (`{market_id}_{outcome}`)
5. **补全所有文档定义的 RPC** — 参照修改后的 ACCOUNT_SERVICE.md

---

## 2. 目标架构

### 2.1 目录结构

```
crates/account-service/
├── Cargo.toml
├── build.rs                          # Proto 编译
├── config/
│   └── account_service.yaml          # 服务配置 (含 DB 配置 + 资产精度)
└── src/
    ├── lib.rs                        # 模块导出
    ├── main.rs                       # 入口：加载配置 -> DBManager::init -> 启动gRPC
    ├── config.rs                     # 配置加载 (包含 db::Config 合并)
    ├── server.rs                     # gRPC Server 启动 + 建表
    ├── error.rs                      # 统一错误类型 (Error -> tonic::Status)
    ├── models.rs                     # 领域模型 (Account, BalanceOperation, AssetPrecision)
    ├── precision.rs                  # 资产精度管理 (i64 <-> 人类可读金额转换)
    ├── pb.rs                         # Proto 生成代码引入
    ├── pb/
    │   └── account.proto             # Proto 定义
    ├── repository/
    │   ├── mod.rs
    │   ├── account_repo.rs           # 账户 CRUD
    │   └── operation_repo.rs         # 操作记录 CRUD
    └── services/
        ├── mod.rs
        └── account_service_impl.rs   # gRPC 服务实现
```

### 2.2 依赖 (Cargo.toml)

```toml
[package]
name = "account-service"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
common = { path = "../common" }
db = { path = "../common/db", features = ["sqlite"] }
domain = { path = "../domain" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }

# gRPC
tonic = { workspace = true }
prost = { workspace = true }
tonic-reflection = { workspace = true }

# Config
serde_yaml = { workspace = true }

[build-dependencies]
tonic-build = { workspace = true }
```

> **关键变化**: 移除 `rust_decimal` 依赖 (金额用 i64)、移除直接 `sqlx` 依赖 (通过 `db` crate 使用)、添加 `db = { path = "../common/db", features = ["sqlite"] }`

---

## 3. 资产精度设计

### 3.1 核心概念

所有金额在数据库中以 **i64 整数** 存储，表示"最小单位"的数量。**精度在创建账户记录时写入数据库**，不同结果代币可以有不同精度。

| 资产 | 精度 (小数位) | 最小单位 | 示例 |
|------|:----------:|---------|------|
| USDT | 6 | 10^-6 USDT (微USDT) | 1.5 USDT → 存储为 1500000 |
| 结果代币 (outcome) | 创建时指定 | 由市场决定 | 100 个 (precision=4) → 存储为 1000000 |

**设计原则**: 精度属于账户数据的固有属性，随账户记录一起持久化。原因：
- 不同市场的结果代币可能需要不同精度
- 市场创建时确定精度，后续不变
- 读取账户时可直接获得精度，无需额外查询或依赖配置

### 3.2 配置 (account_service.yaml)

```yaml
service:
  name: "account-service"
  host: "0.0.0.0"
  port: 50019

database:
  db_type: "sqlite"
  file_path: "./data/account_service.db"
  max_connections: 10
  min_connections: 1

assets:
  base_precision: 6        # 基础资产 (USDT) 默认精度
  outcome_precision: 4     # 结果代币默认精度 (创建账户时作为默认值，可被调用方覆盖)
```

### 3.3 精度模块 (precision.rs)

```rust
/// 资产精度工具函数 (无状态，精度从账户记录读取)
pub struct AssetPrecision;

impl AssetPrecision {
    /// 基础资产默认精度 (从配置读取)
    pub fn base_precision() -> u8 {
        // 从配置或常量获取，USDT = 6
    }

    /// 结果代币默认精度 (从配置读取，创建账户时使用)
    pub fn outcome_default_precision() -> u8 {
        // 从配置或常量获取，默认 4
    }

    /// 存储整数 → 人类可读金额 (根据账户记录的精度)
    /// 例: to_human(1500000, 6) → "1.500000"
    /// 例: to_human(1000000, 4) → "100.0000"
    pub fn to_human(stored: i64, precision: u8) -> String {
        let divisor = 10i64.pow(precision as u32);
        let int_part = stored / divisor;
        let frac_part = stored % divisor;
        format!("{}.{:0>width$}", int_part, frac_part.abs(), width = precision as usize)
    }

    /// 检查金额是否有效 (非零正整数)
    pub fn is_valid_amount(amount: i64) -> bool {
        amount > 0
    }
}
```

### 3.4 精度流转流程

```
创建账户 (get_or_create):
  1. 基础资产 (USDT): precision 从配置读取 → 写入 accounts.precision = 6
  2. 结果代币: precision 由调用方传入，未传则用配置默认值 → 写入 accounts.precision

读取账户:
  1. SELECT ... FROM accounts WHERE user_id=? AND asset=?
  2. 返回时附带 precision 字段
  3. Proto 响应中包含 precision，调用方据此转换为人类可读金额

示例:
  创建 USDT 账户: INSERT (..., precision=6)
  创建 12345_yes 账户: INSERT (..., precision=4)
  创建 67890_no 账户: INSERT (..., precision=2)  — 不同市场可不同精度
```

### 3.5 金额流转示例

```
用户下单买 1.5 USDT 的 Yes 结果代币:

1. Proto 请求: amount = 1500000 (int64, USDT 最小单位)
2. 数据库存储: available -= 1500000, frozen += 1500000
3. 响应返回: available = 8500000 (int64), precision = 6
4. API Gateway 展示: 8.500000 USDT (通过 precision=6 转换)

成交清算 FreezeAndDeduct:
1. 扣减 USDT 冻结: frozen -= 1500000
2. 增加结果代币: outcome_token available += 1000000 (precision=4)
   (价格 0.52, quantity=100, amount=52 USDT)
   - USDT frozen -= 52000000 (52 USDT * 10^6)
   - outcome available += 1000000 (100 个 * 10^4, precision=4)
```

---

## 4. 数据库设计

### 4.1 accounts 表

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | ID |
| user_id | INTEGER | NOT NULL | 用户ID |
| asset | TEXT | NOT NULL | 资产标识: "USDT" 或 "{market_id}_{outcome}" |
| precision | INTEGER | NOT NULL | 资产精度 (小数位数)，创建时写入 |
| available | INTEGER | NOT NULL DEFAULT 0 | 可用余额 (最小单位整数) |
| frozen | INTEGER | NOT NULL DEFAULT 0 | 冻结余额 (下单) |
| locked | INTEGER | NOT NULL DEFAULT 0 | 锁定余额 (风控) |
| created_at | INTEGER | NOT NULL | 创建时间 (unix_ms) |
| updated_at | INTEGER | NOT NULL | 更新时间 (unix_ms) |

**索引**:
- `UNIQUE INDEX idx_user_asset (user_id, asset)`
- `INDEX idx_user_id (user_id)`

### 4.2 balance_operations 表

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | ID |
| account_id | INTEGER | NOT NULL | 账户ID |
| user_id | INTEGER | NOT NULL | 用户ID |
| asset | TEXT | NOT NULL | 资产标识 |
| operation_type | TEXT | NOT NULL | 操作类型 |
| amount | INTEGER | NOT NULL | 操作金额 (最小单位整数) |
| balance_before | INTEGER | NOT NULL | 操作前可用余额 |
| balance_after | INTEGER | NOT NULL | 操作后可用余额 |
| frozen_before | INTEGER | NOT NULL | 操作前冻结 |
| frozen_after | INTEGER | NOT NULL | 操作后冻结 |
| reason | TEXT | | 原因 |
| ref_id | TEXT | | 关联ID (order_id, trade_id 等) |
| created_at | INTEGER | NOT NULL | 创建时间 |

**索引**:
- `INDEX idx_account_id (account_id)`
- `INDEX idx_user_id (user_id)`
- `INDEX idx_ref_id (ref_id)`
- `INDEX idx_created_at (created_at)`

### 4.3 初始化 SQL (server.rs::init_tables)

> 注意: 由于 common/db 的 `DBPool::create_tables()` 内建的是 user-service 的表，Account Service 需要自行建表。通过 `DBPool::sqlite_pool()` / `DBPool::pg_pool()` 获取底层 pool 执行建表。

```sql
-- SQLite
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    asset TEXT NOT NULL,
    precision INTEGER NOT NULL,
    available INTEGER NOT NULL DEFAULT 0,
    frozen INTEGER NOT NULL DEFAULT 0,
    locked INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_asset ON accounts(user_id, asset);
CREATE INDEX IF NOT EXISTS idx_user_id ON accounts(user_id);

CREATE TABLE IF NOT EXISTS balance_operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    asset TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    amount INTEGER NOT NULL,
    balance_before INTEGER NOT NULL,
    balance_after INTEGER NOT NULL,
    frozen_before INTEGER NOT NULL,
    frozen_after INTEGER NOT NULL,
    reason TEXT,
    ref_id TEXT,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_op_account_id ON balance_operations(account_id);
CREATE INDEX IF NOT EXISTS idx_op_user_id ON balance_operations(user_id);
CREATE INDEX IF NOT EXISTS idx_op_ref_id ON balance_operations(ref_id);
CREATE INDEX IF NOT EXISTS idx_op_created_at ON balance_operations(created_at);
```

```sql
-- PostgreSQL
CREATE TABLE IF NOT EXISTS accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    asset VARCHAR(50) NOT NULL,
    precision INTEGER NOT NULL,
    available BIGINT NOT NULL DEFAULT 0,
    frozen BIGINT NOT NULL DEFAULT 0,
    locked BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_asset ON accounts(user_id, asset);
CREATE INDEX IF NOT EXISTS idx_user_id ON accounts(user_id);

CREATE TABLE IF NOT EXISTS balance_operations (
    id BIGSERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    asset VARCHAR(50) NOT NULL,
    operation_type VARCHAR(30) NOT NULL,
    amount BIGINT NOT NULL,
    balance_before BIGINT NOT NULL,
    balance_after BIGINT NOT NULL,
    frozen_before BIGINT NOT NULL,
    frozen_after BIGINT NOT NULL,
    reason TEXT,
    ref_id TEXT,
    created_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_op_account_id ON balance_operations(account_id);
CREATE INDEX IF NOT EXISTS idx_op_user_id ON balance_operations(user_id);
CREATE INDEX IF NOT EXISTS idx_op_ref_id ON balance_operations(ref_id);
CREATE INDEX IF NOT EXISTS idx_op_created_at ON balance_operations(created_at);
```

---

## 5. Proto 定义 (account.proto)

> **关键变化**: 所有金额字段从 `string` 改为 `int64`，表示最小单位整数

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
    rpc CheckAndFreeze(CheckAndFreezeRequest) returns (CheckAndFreezeResponse);

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

// ==================== 余额查询 ====================

message GetBalanceRequest {
    int64 user_id = 1;
    string asset = 2;
    int32 precision = 3;     // 创建账户时使用的精度 (仅首次创建有效，已存在则忽略)
}

message GetBalanceResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    int64 user_id = 4;
    string asset = 5;
    int64 available = 6;    // 最小单位整数
    int64 frozen = 7;       // 最小单位整数
    int64 locked = 8;       // 最小单位整数
    int32 precision = 9;    // 资产精度 (小数位数)
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
    int64 available = 3;    // 最小单位整数
    int64 frozen = 4;       // 最小单位整数
    int64 locked = 5;       // 最小单位整数
    int32 precision = 6;    // 资产精度 (小数位数)
}

// ==================== 冻结/解冻 ====================

message FreezeRequest {
    int64 user_id = 1;
    string asset = 2;
    int64 amount = 3;       // 最小单位整数
    string order_id = 4;
    string reason = 5;
    int32 precision = 6;    // 创建账户时的精度 (仅首次创建有效)
}

message FreezeResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    int64 available_before = 4; // 最小单位整数
    int64 available_after = 5;
    int64 frozen_before = 6;
    int64 frozen_after = 7;
}

message UnfreezeRequest {
    int64 user_id = 1;
    string asset = 2;
    int64 amount = 3;       // 最小单位整数
    string order_id = 4;
    string reason = 5;
}

message UnfreezeResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    int64 available_before = 4;
    int64 available_after = 5;
    int64 frozen_before = 6;
    int64 frozen_after = 7;
}

message FreezeAndDeductRequest {
    int64 user_id = 1;
    string asset = 2;           // 基础资产 (USDT)
    int64 freeze_amount = 3;    // 从冻结扣减的金额 (最小单位)
    int64 deduct_amount = 4;    // 实际扣减金额 (最小单位, 可能因部分成交而不同)
    string order_id = 5;
    string reason = 6;
    string outcome_asset = 7;   // 结果代币标识 ("{market_id}_{outcome}")
    int64 outcome_amount = 8;   // 结果代币增加数量 (最小单位)
    int32 outcome_precision = 9; // 结果代币精度 (创建账户时使用)
}

message FreezeAndDeductResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    int64 available_before = 4;
    int64 available_after = 5;
    int64 frozen_before = 6;
    int64 frozen_after = 7;
}

message CheckAndFreezeRequest {
    int64 user_id = 1;
    string asset = 2;
    int64 amount = 3;       // 最小单位整数
    string order_id = 4;
    string reason = 5;
    int32 precision = 6;    // 创建账户时的精度 (仅首次创建有效)
}

message CheckAndFreezeResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    int64 available_before = 4;
    int64 available_after = 5;
    int64 frozen_before = 6;
    int64 frozen_after = 7;
}

// ==================== 充值/提现 ====================

message DepositRequest {
    int64 user_id = 1;
    string asset = 2;
    int64 amount = 3;       // 最小单位整数
    string tx_id = 4;
    string reason = 5;
}

message DepositResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    int64 available_before = 4;
    int64 available_after = 5;
}

message WithdrawRequest {
    int64 user_id = 1;
    string asset = 2;
    int64 amount = 3;       // 最小单位整数
    string withdraw_id = 4;
    string reason = 5;
}

message WithdrawResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    int64 available_before = 4;
    int64 available_after = 5;
}

// ==================== 划转 ====================

message TransferRequest {
    int64 from_user_id = 1;
    int64 to_user_id = 2;
    string asset = 3;
    int64 amount = 4;       // 最小单位整数
    string transfer_id = 5;
    string reason = 6;
}

message TransferResponse {
    bool success = 1;
    string message = 2;
    int64 from_available_before = 3;
    int64 from_available_after = 4;
    int64 to_available_before = 5;
    int64 to_available_after = 6;
}

// ==================== 风控锁定/解锁 ====================

message LockRequest {
    int64 user_id = 1;
    string asset = 2;
    int64 amount = 3;       // 最小单位整数
    string reason = 4;
}

message LockResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    int64 available_before = 4;
    int64 available_after = 5;
    int64 locked_before = 6;
    int64 locked_after = 7;
}

message UnlockRequest {
    int64 user_id = 1;
    string asset = 2;
    int64 amount = 3;       // 最小单位整数
    string reason = 4;
}

message UnlockResponse {
    bool success = 1;
    string message = 2;
    int64 account_id = 3;
    int64 available_before = 4;
    int64 available_after = 5;
    int64 locked_before = 6;
    int64 locked_after = 7;
}

// ==================== 风控查询 ====================

message CheckBalanceRequest {
    int64 user_id = 1;
    string asset = 2;
    int64 amount = 3;       // 最小单位整数
}

message CheckBalanceResponse {
    bool sufficient = 1;
    int64 available = 2;    // 最小单位整数
    int64 required = 3;     // 最小单位整数
}

message CheckFrozenRequest {
    int64 user_id = 1;
    string asset = 2;
    string order_id = 3;
}

message CheckFrozenResponse {
    bool sufficient = 1;
    int64 frozen = 2;       // 最小单位整数
    int64 required = 3;     // 最小单位整数
}

// ==================== 结算派彩 ====================

message SettleRequest {
    int64 user_id = 1;
    string outcome_asset = 2;   // 结果代币标识
    int64 outcome_amount = 3;   // 消耗的结果代币数量 (最小单位)
    string base_asset = 4;      // 基础资产标识 (USDT)
    int64 base_amount = 5;      // 派彩基础资产数量 (最小单位)
    int64 market_id = 6;
    string reason = 7;
}

message SettleResponse {
    bool success = 1;
    string message = 2;
    int64 outcome_available_after = 3;  // 结算后结果代币可用
    int64 base_available_after = 4;     // 结算后基础资产可用
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

---

## 6. 核心模型 (models.rs)

```rust
use serde::{Deserialize, Serialize};

/// 账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub user_id: i64,
    pub asset: String,           // "USDT" 或 "{market_id}_{outcome}"
    pub available: i64,          // 最小单位整数
    pub frozen: i64,             // 最小单位整数
    pub locked: i64,             // 最小单位整数
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
    pub amount: i64,             // 最小单位整数
    pub balance_before: i64,     // 操作前可用
    pub balance_after: i64,      // 操作后可用
    pub frozen_before: i64,      // 操作前冻结
    pub frozen_after: i64,       // 操作后冻结
    pub reason: Option<String>,
    pub ref_id: Option<String>,
    pub created_at: i64,
}

/// 余额操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BalanceOperationType {
    Deposit,
    Withdraw,
    Freeze,
    Unfreeze,
    Deduct,
    TransferIn,
    TransferOut,
    Fee,
    Lock,
    Unlock,
    Settlement,
}

impl BalanceOperationType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Deposit => "deposit",
            Self::Withdraw => "withdraw",
            Self::Freeze => "freeze",
            Self::Unfreeze => "unfreeze",
            Self::Deduct => "deduct",
            Self::TransferIn => "transfer_in",
            Self::TransferOut => "transfer_out",
            Self::Fee => "fee",
            Self::Lock => "lock",
            Self::Unlock => "unlock",
            Self::Settlement => "settlement",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "deposit" => Some(Self::Deposit),
            "withdraw" => Some(Self::Withdraw),
            "freeze" => Some(Self::Freeze),
            "unfreeze" => Some(Self::Unfreeze),
            "deduct" => Some(Self::Deduct),
            "transfer_in" => Some(Self::TransferIn),
            "transfer_out" => Some(Self::TransferOut),
            "fee" => Some(Self::Fee),
            "lock" => Some(Self::Lock),
            "unlock" => Some(Self::Unlock),
            "settlement" => Some(Self::Settlement),
            _ => None,
        }
    }
}
```

---

## 7. 配置设计 (config.rs)

### 7.1 配置结构

```rust
use serde::Deserialize;
use std::fs;
use std::path::Path;
use db::Config as DBConfig;

/// 服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub service: ServiceConfig,
    pub database: DBConfig,          // 直接使用 common/db 的 Config
    pub assets: AssetsConfig,        // 资产精度配置
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetsConfig {
    pub base: BaseAssetConfig,
    pub outcome: OutcomeAssetConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BaseAssetConfig {
    pub name: String,          // "USDT"
    pub precision: u8,         // 6
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutcomeAssetConfig {
    pub precision: u8,         // 4
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_default() -> Self {
        Self::load("config/account_service.yaml").unwrap_or_else(|_| {
            Config {
                service: ServiceConfig {
                    name: "account-service".to_string(),
                    host: "0.0.0.0".to_string(),
                    port: 50019,
                },
                database: DBConfig {
                    db_type: Some("sqlite".to_string()),
                    file_path: Some("./data/account_service.db".to_string()),
                    max_connections: Some(10),
                    min_connections: Some(1),
                    ..DBConfig::default()
                },
                assets: AssetsConfig {
                    base: BaseAssetConfig {
                        name: "USDT".to_string(),
                        precision: 6,
                    },
                    outcome: OutcomeAssetConfig {
                        precision: 4,
                    },
                },
            }
        })
    }

    /// 获取合并后的数据库配置
    pub fn merged_db_config(&self) -> db::MergedConfig {
        self.database.merge()
    }
}
```

---

## 8. Server 启动设计 (server.rs)

```rust
use db::{DBManager, DBPool};
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic_reflection::server::Builder;

use crate::config::Config;
use crate::precision::AssetPrecision;
use crate::services::AccountServiceImpl;
use crate::pb::account_service_server::AccountServiceServer;

pub struct AccountServer {
    config: Config,
}

impl AccountServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // 使用 common/db: 通过 DBManager 初始化
        let merged_config = self.config.merged_db_config();
        let db_manager = DBManager::new(merged_config);
        db_manager.init().await?;

        let pool = db_manager.get_pool()
            .await
            .ok_or("Failed to get DB pool")?;

        // 自行建表 (不使用 DBPool::create_tables，因为那是 user-service 的表)
        Self::init_tables(&pool).await?;

        // 创建精度管理器
        let asset_precision = AssetPrecision::new(&self.config.assets);

        // 创建服务
        let account_service = AccountServiceImpl::new(pool, asset_precision);
        let addr = format!("{}:{}", self.config.service.host, self.config.service.port)
            .parse::<SocketAddr>()?;

        tracing::info!("Starting Account Service on {}", addr);

        // 添加反射服务
        let reflection_service = Builder::configure()
            .register_encoded_file_descriptor_set(include_bytes!("pb/account_service.desc"))
            .build_v1()?;

        // 构建 gRPC 服务器
        Server::builder()
            .add_service(reflection_service)
            .add_service(AccountServiceServer::new(account_service))
            .serve(addr)
            .await?;

        Ok(())
    }

    /// 自行建表 — 根据 DBPool 类型选择 SQLite 或 PostgreSQL DDL
    async fn init_tables(pool: &DBPool) -> Result<(), sqlx::Error> {
        match pool {
            DBPool::Sqlite(pool) => {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS accounts (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        user_id INTEGER NOT NULL,
                        asset TEXT NOT NULL,
                        precision INTEGER NOT NULL,
                        available INTEGER NOT NULL DEFAULT 0,
                        frozen INTEGER NOT NULL DEFAULT 0,
                        locked INTEGER NOT NULL DEFAULT 0,
                        created_at INTEGER NOT NULL,
                        updated_at INTEGER NOT NULL
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_user_asset ON accounts(user_id, asset)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_id ON accounts(user_id)")
                    .execute(pool)
                    .await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS balance_operations (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        account_id INTEGER NOT NULL,
                        user_id INTEGER NOT NULL,
                        asset TEXT NOT NULL,
                        operation_type TEXT NOT NULL,
                        amount INTEGER NOT NULL,
                        balance_before INTEGER NOT NULL,
                        balance_after INTEGER NOT NULL,
                        frozen_before INTEGER NOT NULL,
                        frozen_after INTEGER NOT NULL,
                        reason TEXT,
                        ref_id TEXT,
                        created_at INTEGER NOT NULL
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_account_id ON balance_operations(account_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_user_id ON balance_operations(user_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_ref_id ON balance_operations(ref_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_created_at ON balance_operations(created_at)")
                    .execute(pool)
                    .await?;
            }
            DBPool::Postgres(pool) => {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS accounts (
                        id BIGSERIAL PRIMARY KEY,
                        user_id BIGINT NOT NULL,
                        asset VARCHAR(50) NOT NULL,
                        precision INTEGER NOT NULL,
                        available BIGINT NOT NULL DEFAULT 0,
                        frozen BIGINT NOT NULL DEFAULT 0,
                        locked BIGINT NOT NULL DEFAULT 0,
                        created_at BIGINT NOT NULL,
                        updated_at BIGINT NOT NULL
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_user_asset ON accounts(user_id, asset)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_id ON accounts(user_id)")
                    .execute(pool)
                    .await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS balance_operations (
                        id BIGSERIAL PRIMARY KEY,
                        account_id BIGINT NOT NULL,
                        user_id BIGINT NOT NULL,
                        asset VARCHAR(50) NOT NULL,
                        operation_type VARCHAR(30) NOT NULL,
                        amount BIGINT NOT NULL,
                        balance_before BIGINT NOT NULL,
                        balance_after BIGINT NOT NULL,
                        frozen_before BIGINT NOT NULL,
                        frozen_after BIGINT NOT NULL,
                        reason TEXT,
                        ref_id TEXT,
                        created_at BIGINT NOT NULL
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_account_id ON balance_operations(account_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_user_id ON balance_operations(user_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_ref_id ON balance_operations(ref_id)")
                    .execute(pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_op_created_at ON balance_operations(created_at)")
                    .execute(pool)
                    .await?;
            }
        }

        tracing::info!("Account tables initialized");
        Ok(())
    }
}
```

---

## 9. 服务实现设计 (services/account_service_impl.rs)

```rust
pub struct AccountServiceImpl {
    pool: DBPool,                       // common/db 的 DBPool
    asset_precision: AssetPrecision,    // 资产精度管理器
}

impl AccountServiceImpl {
    pub fn new(pool: DBPool, asset_precision: AssetPrecision) -> Self {
        Self { pool, asset_precision }
    }
}
```

---

## 10. Repository 层设计

### 10.1 AccountRepository

```rust
use db::DBPool;

pub struct AccountRepository;

impl AccountRepository {
    /// 获取或创建账户 (无则自动创建，余额为0)
    /// 根据 DBPool 类型操作 SQLite 或 PostgreSQL
    pub async fn get_or_create(pool: &DBPool, user_id: i64, asset: &str) -> Result<Account>;

    /// 获取账户 (必须存在)
    pub async fn get(pool: &DBPool, user_id: i64, asset: &str) -> Result<Option<Account>>;

    /// 获取用户所有账户
    pub async fn get_by_user(pool: &DBPool, user_id: i64) -> Result<Vec<Account>>;

    /// 更新余额 (在事务内调用)
    /// SQLite: tx: &mut sqlx::SqliteConnection
    /// PostgreSQL: tx: &mut sqlx::PgConnection
    /// 使用 DBPool 枚举分发
    pub async fn update_balance(
        pool: &DBPool,
        account_id: i64,
        available: i64,
        frozen: i64,
        locked: i64,
    ) -> Result<()>;

    /// 批量获取余额
    pub async fn batch_get(pool: &DBPool, user_ids: &[i64], assets: &[String]) -> Result<Vec<Account>>;
}
```

### 10.2 OperationRepository

```rust
pub struct OperationRepository;

impl OperationRepository {
    /// 记录操作 (在事务内调用)
    pub async fn record(
        pool: &DBPool,
        op: &BalanceOperation,
    ) -> Result<i64>;
}
```

---

## 11. 核心服务逻辑设计

### 11.1 CheckAndFreeze (原子操作，最关键)

```
BEGIN IMMEDIATE TRANSACTION

1. SELECT * FROM accounts WHERE user_id=? AND asset=?
   -- 无记录则 INSERT (available=0, frozen=0, locked=0)

2. IF available < amount THEN ROLLBACK, return INSUFFICIENT_BALANCE

3. UPDATE accounts SET available=available-amount, frozen=frozen+amount, updated_at=now
   WHERE id=?

4. INSERT INTO balance_operations (...)

COMMIT
```

### 11.2 FreezeAndDeduct (成交清算)

```
BEGIN IMMEDIATE TRANSACTION

1. SELECT * FROM accounts WHERE user_id=? AND asset=? (基础资产)
   -- 确认 frozen >= freeze_amount

2. UPDATE accounts SET frozen=frozen-freeze_amount WHERE id=? (基础资产扣减冻结)

3. IF outcome_asset 不为空:
   SELECT * FROM accounts WHERE user_id=? AND asset=outcome_asset (结果代币)
   -- 无则 INSERT
   UPDATE accounts SET available=available+outcome_amount WHERE id=? (结果代币增加)

4. INSERT INTO balance_operations (基础资产操作 + 结果代币操作)

COMMIT
```

### 11.3 Settle (结算派彩)

```
BEGIN IMMEDIATE TRANSACTION

1. 消耗结果代币:
   SELECT * FROM accounts WHERE user_id=? AND asset=outcome_asset
   -- 确认 available >= outcome_amount
   UPDATE accounts SET available=available-outcome_amount

2. 增加基础资产:
   SELECT * FROM accounts WHERE user_id=? AND asset=base_asset
   -- 无则 INSERT
   UPDATE accounts SET available=available+base_amount

3. INSERT INTO balance_operations (结果代币扣减 + 基础资产增加)

COMMIT
```

### 11.4 其他操作一览

| RPC | 逻辑 | 事务 |
|-----|------|------|
| GetBalance | 查询 accounts 表 | 无 |
| GetBalances | 查询用户所有资产 | 无 |
| Freeze | available -= amount, frozen += amount | IMMEDIATE |
| Unfreeze | frozen -= amount, available += amount | IMMEDIATE |
| Deposit | available += amount | IMMEDIATE |
| Withdraw | available -= amount | IMMEDIATE |
| Transfer | from.available -= amount, to.available += amount | IMMEDIATE |
| Lock | available -= amount, locked += amount | IMMEDIATE |
| Unlock | locked -= amount, available += amount | IMMEDIATE |
| CheckBalance | 仅查询 available >= amount | 无 |
| CheckFrozen | 仅查询 frozen | 无 |
| BatchGetBalances | 批量查询 | 无 |

---

## 12. 错误码映射

| 错误码 | gRPC Status | 说明 |
|--------|-------------|------|
| ACCOUNT_NOT_FOUND | NOT_FOUND | 账户不存在 |
| ASSET_NOT_SUPPORTED | INVALID_ARGUMENT | 资产不支持 |
| INSUFFICIENT_BALANCE | FAILED_PRECONDITION | 可用余额不足 |
| INSUFFICIENT_FROZEN | FAILED_PRECONDITION | 冻结余额不足 |
| INSUFFICIENT_LOCKED | FAILED_PRECONDITION | 锁定余额不足 |
| AMOUNT_INVALID | INVALID_ARGUMENT | 金额无效 (负数或零) |
| AMOUNT_PRECISION_ERROR | INVALID_ARGUMENT | 金额精度错误 |
| ACCOUNT_LOCKED | PERMISSION_DENIED | 账户被风控锁定 |
| OUTCOME_ASSET_INVALID | INVALID_ARGUMENT | 结果代币标识无效 |
| SETTLEMENT_FAILED | INTERNAL | 结算派彩失败 |
| DUPLICATE_OPERATION | ALREADY_EXISTS | 重复操作 |

---

## 13. 开发任务分解

### Phase 1: 基础框架搭建

| # | 任务 | 文件 |
|---|------|------|
| 1.1 | 重写 Cargo.toml，添加 tonic/db/serde_yaml 依赖 | Cargo.toml |
| 1.2 | 创建 build.rs (tonic-build 编译 proto) | build.rs |
| 1.3 | 创建 account.proto (金额字段使用 int64) | src/pb/account.proto |
| 1.4 | 创建 config.rs + account_service.yaml (含 DB 配置 + 资产精度) | src/config.rs, config/ |
| 1.5 | 创建 error.rs (Error -> tonic::Status 转换) | src/error.rs |
| 1.6 | 创建 models.rs (Account/BalanceOperation 使用 i64) | src/models.rs |
| 1.7 | 创建 precision.rs (AssetPrecision 精度管理) | src/precision.rs |

### Phase 2: 数据层

| # | 任务 | 文件 |
|---|------|------|
| 2.1 | 创建 account_repo.rs (get_or_create, get, get_by_user, update_balance, batch_get) | src/repository/account_repo.rs |
| 2.2 | 创建 operation_repo.rs (record) | src/repository/operation_repo.rs |
| 2.3 | 创建 server.rs (DBManager 初始化 + 自行建表 + gRPC Server 启动) | src/server.rs |

### Phase 3: 服务实现

| # | 任务 | 文件 |
|---|------|------|
| 3.1 | 实现 GetBalance / GetBalances / BatchGetBalances | src/services/account_service_impl.rs |
| 3.2 | 实现 CheckBalance / CheckFrozen (只读查询) | 同上 |
| 3.3 | 实现 CheckAndFreeze (原子操作，核心) | 同上 |
| 3.4 | 实现 Freeze / Unfreeze | 同上 |
| 3.5 | 实现 FreezeAndDeduct (成交清算) | 同上 |
| 3.6 | 实现 Deposit / Withdraw | 同上 |
| 3.7 | 实现 Transfer | 同上 |
| 3.8 | 实现 Lock / Unlock | 同上 |
| 3.9 | 实现 Settle (结算派彩) | 同上 |

### Phase 4: 入口和集成

| # | 任务 | 文件 |
|---|------|------|
| 4.1 | 重写 main.rs (加载配置 -> DBManager::init -> 启动 gRPC) | src/main.rs |
| 4.2 | 重写 lib.rs (模块导出) | src/lib.rs |
| 4.3 | 删除旧文件 (account.rs, handlers.rs) | - |
| 4.4 | 编译验证 cargo build -p account-service | - |
| 4.5 | 单元测试 cargo test -p account-service | - |

---

## 14. 关键设计决策

### 14.1 使用 common/db 组件

使用 `DBPool`/`DBManager`/`Config`/`MergedConfig` 管理 DB 连接。服务配置中 `database` 字段直接使用 `db::Config`，通过 `merge()` 链合并优先级 (服务配置 > 组件默认 > 硬编码)。自行建表而非使用 `DBPool::create_tables()` (后者内建了 user-service 的表结构)。

### 14.2 金额使用 i64 整数存储

所有金额以 i64 存储，表示"最小单位"的数量。配合 `AssetPrecision` 管理：
- USDT 精度 6: 1.5 USDT → 存储 1500000
- 结果代币精度 4: 100 个 → 存储 1000000

优势: 避免 Decimal/字符串比较开销，整数运算天然原子，Proto int64 直接映射，无精度损失。

### 14.3 资产精度可配置

通过 `assets` 配置段定义:
- `base.precision` — 基础资产(USDT)精度
- `outcome.precision` — 结果代币默认精度

`AssetPrecision` 根据 asset 名称自动匹配精度: 已注册的基础资产用配置值，结果代币 (`{market_id}_{outcome}` 格式) 用 outcome 默认值。

### 14.4 SQLite 事务隔离

SQLite 使用 `BEGIN IMMEDIATE` 替代 `FOR UPDATE`，确保写入串行化。对于单服务部署足够。

### 14.5 资产标识设计

- 基础资产: `"USDT"`
- 结果代币: `"{market_id}_{outcome}"` (如 `"12345_yes"`)
- 无需预定义资产列表，任意 asset 字符串均可自动创建账户

### 14.6 自动创建账户

`get_or_create` 语义：首次查询不存在的 (user_id, asset) 组合时自动创建余额为 0 的账户。避免调用方需要先"开户"。

### 14.7 Kafka 事件暂不实现

当前 common/queue 的 Kafka 实现是 stub，Phase 1 仅实现同步 gRPC 调用。Kafka 事件发布作为后续任务。
