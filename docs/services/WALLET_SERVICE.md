# Wallet Service - 钱包服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50002 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | Account Service (50019), User Service (50001), Risk Service (50004) |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 充值 | ⚠️ 链上充值监听、入账 |
| 提现 | 提现申请、签名、广播 |
| 地址管理 | 充值地址生成 |
| 提现白名单 | ⚠️ 提现地址白名单管理 |
| 支付密码 | ⚠️ 资金密码验证 |

### 1.1.2 功能列表

```
Wallet Service
├── 充值管理
│   ├── GetDepositAddress - 获取充值地址
│   ├── ListDepositAddresses - 列出所有充值地址
│   ├── ConfirmDeposit - 确认充值入账
│   └── ⚠️ MonitorChainDeposit - 监听链上充值 (待实现)
├── 提现管理
│   ├── CreateWithdraw - 创建提现
│   ├── ConfirmWithdraw - 确认提现 (签名广播)
│   ├── CancelWithdraw - 取消提现
│   ├── GetWithdrawHistory - 提现历史
│   └── ⚠️ GetPendingWithdraws - 待确认提现
├── 地址白名单
│   ├── AddWhitelistAddress - 添加白名单
│   ├── RemoveWhitelistAddress - 移除白名单
│   ├── ListWhitelistAddresses - 列出白名单
│   └── IsWhitelisted - 检查白名单
├── 支付密码
│   ├── SetPaymentPassword - 设置支付密码
│   ├── VerifyPaymentPassword - 验证支付密码
│   └── ResetPaymentPassword - 重置支付密码
└── 链上操作 (⚠️ 需实现)
    ├── ScanChainBlocks - 扫描链上区块
    ├── ParseChainDeposit - 解析链上充值交易
    └── BroadcastWithdrawTx - 广播提现交易
```

## 2. 数据模型

### 2.1 Domain Model

```rust
/// 充值地址
pub struct DepositAddress {
    pub id: i64,
    pub user_id: i64,
    pub chain: String,        // "ETH", "BSC", "ARBITRUM"
    pub address: String,
    pub created_at: i64,
}

/// 提现记录
pub struct WithdrawRecord {
    pub id: i64,
    pub user_id: i64,
    pub asset: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub to_address: String,
    pub chain: String,
    pub status: WithdrawStatus,
    pub tx_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 提现状态
pub enum WithdrawStatus {
    Pending,      // 待确认
    Confirmed,    // 已确认(已广播)
    Completed,    // 已完成
    Cancelled,    // 已取消
    Failed,       // 失败
}

/// 地址白名单
pub struct WhitelistAddress {
    pub id: i64,
    pub user_id: i64,
    pub chain: String,
    pub address: String,
    pub label: Option<String>,
    pub created_at: i64,
}

/// 用户支付密码
pub struct PaymentPassword {
    pub id: i64,
    pub user_id: i64,
    pub password_hash: String,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### 2.2 数据库表

**deposit_addresses 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| user_id | BIGINT | NOT NULL | 用户ID |
| chain | VARCHAR(20) | NOT NULL | 链名 |
| address | VARCHAR(100) | NOT NULL | 地址 |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `PRIMARY KEY (id)`
- `INDEX idx_user_id (user_id)`
- `UNIQUE INDEX idx_user_chain (user_id, chain)`

**withdraw_records 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| user_id | BIGINT | NOT NULL | 用户ID |
| asset | VARCHAR(20) | NOT NULL | 资产 |
| amount | TEXT | NOT NULL | 提现金额 |
| fee | TEXT | NOT NULL | 手续费 |
| to_address | VARCHAR(100) | NOT NULL | 目标地址 |
| chain | VARCHAR(20) | NOT NULL | 链名 |
| status | VARCHAR(20) | NOT NULL | 状态 |
| tx_id | VARCHAR(100) | | 链上交易ID |
| created_at | BIGINT | NOT NULL | 创建时间 |
| updated_at | BIGINT | NOT NULL | 更新时间 |

**whitelist_addresses 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| user_id | BIGINT | NOT NULL | 用户ID |
| chain | VARCHAR(20) | NOT NULL | 链名 |
| address | VARCHAR(100) | NOT NULL | 地址 |
| label | VARCHAR(100) | | 标签 |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `PRIMARY KEY (id)`
- `INDEX idx_user_id (user_id)`

**payment_passwords 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| user_id | BIGINT | UNIQUE, NOT NULL | 用户ID |
| password_hash | VARCHAR(100) | NOT NULL | 密码Hash |
| created_at | BIGINT | NOT NULL | 创建时间 |
| updated_at | BIGINT | NOT NULL | 更新时间 |

## 3. Proto 接口

```protobuf
syntax = "proto3";

package wallet;

service WalletService {
    // ========== 充值地址 ==========
    rpc GetDepositAddress(GetDepositAddressRequest) returns (GetDepositAddressResponse);
    rpc ListDepositAddresses(ListDepositAddressesRequest) returns (ListDepositAddressesResponse);

    // ========== 充值 ==========
    rpc ConfirmDeposit(ConfirmDepositRequest) returns (ConfirmDepositResponse);
    rpc GetDepositHistory(GetDepositHistoryRequest) returns (GetDepositHistoryResponse);

    // ========== 提现 ==========
    rpc CreateWithdraw(CreateWithdrawRequest) returns (CreateWithdrawResponse);
    rpc ConfirmWithdraw(ConfirmWithdrawRequest) returns (ConfirmWithdrawResponse);
    rpc CancelWithdraw(CancelWithdrawRequest) returns (CancelWithdrawResponse);
    rpc GetWithdrawHistory(GetWithdrawHistoryRequest) returns (GetWithdrawHistoryResponse);
    rpc GetPendingWithdraws(GetPendingWithdrawsRequest) returns (GetPendingWithdrawsResponse);

    // ========== 地址白名单 ==========
    rpc AddWhitelistAddress(AddWhitelistAddressRequest) returns (AddWhitelistAddressResponse);
    rpc RemoveWhitelistAddress(RemoveWhitelistAddressRequest) returns (RemoveWhitelistAddressResponse);
    rpc ListWhitelistAddresses(ListWhitelistAddressesRequest) returns (ListWhitelistAddressesResponse);
    rpc IsWhitelisted(IsWhitelistedRequest) returns (IsWhitelistedResponse);

    // ========== 支付密码 ==========
    rpc SetPaymentPassword(SetPaymentPasswordRequest) returns (SetPaymentPasswordResponse);
    rpc VerifyPaymentPassword(VerifyPaymentPasswordRequest) returns (VerifyPaymentPasswordResponse);
    rpc ResetPaymentPassword(ResetPaymentPasswordRequest) returns (ResetPaymentPasswordResponse);
    rpc HasPaymentPassword(HasPaymentPasswordRequest) returns (HasPaymentPasswordResponse);
}

// ==================== 充值地址 ====================

message GetDepositAddressRequest {
    int64 user_id = 1;
    string chain = 2;  // "ETH", "BSC", "ARBITRUM"
}

message GetDepositAddressResponse {
    string address = 1;
    string chain = 2;
}

message ListDepositAddressesRequest {
    int64 user_id = 1;
}

message ListDepositAddressesResponse {
    repeated DepositAddressSummary addresses = 1;
}

message DepositAddressSummary {
    string address = 1;
    string chain = 2;
    int64 created_at = 3;
}

// ==================== 充值 ====================

message ConfirmDepositRequest {
    int64 user_id = 1;
    string tx_id = 2;
    string chain = 3;
    string amount = 4;
    string address = 5;
}

message ConfirmDepositResponse {
    bool success = 1;
    string message = 2;
}

message GetDepositHistoryRequest {
    int64 user_id = 1;
    string chain = 2;
    int32 page = 3;
    int32 page_size = 4;
}

message GetDepositHistoryResponse {
    repeated DepositRecord deposits = 1;
    int64 total = 2;
}

message DepositRecord {
    string tx_id = 1;
    string chain = 2;
    string amount = 3;
    string address = 4;
    int64 created_at = 5;
}

// ==================== 提现 ====================

message CreateWithdrawRequest {
    int64 user_id = 1;
    string asset = 2;
    string amount = 3;
    string to_address = 4;
    string chain = 5;
    string payment_password = 6;  // ⚠️ 需验证支付密码
}

message CreateWithdrawResponse {
    bool success = 1;
    string message = 2;
    string withdraw_id = 3;
}

message ConfirmWithdrawRequest {
    string withdraw_id = 1;
    string signature = 2;
    string otp_code = 3;  // 可选，2FA验证码
}

message ConfirmWithdrawResponse {
    bool success = 1;
    string message = 2;
    string tx_id = 3;
}

message CancelWithdrawRequest {
    string withdraw_id = 1;
    int64 user_id = 2;
}

message CancelWithdrawResponse {
    bool success = 1;
    string message = 2;
}

message GetWithdrawHistoryRequest {
    int64 user_id = 1;
    int32 page = 2;
    int32 page_size = 3;
}

message GetWithdrawHistoryResponse {
    repeated WithdrawRecordSummary withdrawals = 1;
    int64 total = 2;
}

message WithdrawRecordSummary {
    string withdraw_id = 1;
    string asset = 2;
    string amount = 3;
    string fee = 4;
    string to_address = 5;
    string status = 6;
    string tx_id = 7;
    int64 created_at = 8;
}

message GetPendingWithdrawsRequest {
    int64 user_id = 1;
}

message GetPendingWithdrawsResponse {
    repeated WithdrawRecordSummary withdrawals = 1;
}

// ==================== 地址白名单 ====================

message AddWhitelistAddressRequest {
    int64 user_id = 1;
    string chain = 2;
    string address = 3;
    string label = 4;
}

message AddWhitelistAddressResponse {
    bool success = 1;
    string message = 2;
}

message RemoveWhitelistAddressRequest {
    int64 user_id = 1;
    string address = 2;
}

message RemoveWhitelistAddressResponse {
    bool success = 1;
    string message = 2;
}

message ListWhitelistAddressesRequest {
    int64 user_id = 1;
    string chain = 2;
}

message ListWhitelistAddressesResponse {
    repeated WhitelistAddressSummary addresses = 1;
}

message WhitelistAddressSummary {
    string chain = 1;
    string address = 2;
    string label = 3;
    int64 created_at = 4;
}

message IsWhitelistedRequest {
    int64 user_id = 1;
    string address = 2;
}

message IsWhitelistedResponse {
    bool is_whitelisted = 1;
}

// ==================== 支付密码 ====================

message SetPaymentPasswordRequest {
    int64 user_id = 1;
    string password = 2;
}

message SetPaymentPasswordResponse {
    bool success = 1;
    string message = 2;
}

message VerifyPaymentPasswordRequest {
    int64 user_id = 1;
    string password = 2;
}

message VerifyPaymentPasswordResponse {
    bool valid = 1;
    string message = 2;
}

message ResetPaymentPasswordRequest {
    int64 user_id = 1;
    string old_password = 2;
    string new_password = 3;
}

message ResetPaymentPasswordResponse {
    bool success = 1;
    string message = 2;
}

message HasPaymentPasswordRequest {
    int64 user_id = 1;
}

message HasPaymentPasswordResponse {
    bool has_password = 1;
}
```

## 4. 服务间通信

### 4.1 gRPC 调用

| 被调用方 | 接口 | 场景 |
|----------|------|------|
| Account Service (50019) | Deposit | 充值入账 |
| Account Service (50019) | Withdraw | 提现扣账 |
| Risk Service (50004) | CheckWithdraw | 提现风控检查 |
| User Service (50001) | GetUserById | 获取用户信息 |

### 4.2 gRPC 被调用

| 调用方 | 接口 | 场景 |
|----------|------|------|
| API Gateway | 充值/提现/地址管理 | 用户操作 |

### 4.3 Kafka 消费

| Topic | 处理 |
|-------|------|
| user_events | 用户注册时生成充值地址 |
| chain_events | ⚠️ 链上充值事件 (待实现) |

## 5. 核心流程

### 5.1 充值流程

```
用户充值 (链上转账)
    │
    ▼
⚠️ 待实现: Chain Scanner 监听链上交易
    │
    ▼
┌─────────────────────────────────────────┐
│ 1. 解析链上交易                          │
│    - tx_id, from_address, to_address    │
│    - amount, chain                      │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 2. 验证充值地址                          │
│    - to_address 是否为系统充值地址       │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 3. 调用 Account Service 入账            │
│    - Deposit                            │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 4. 保存充值记录                          │
└─────────────────┬───────────────────────┘
```

### 5.2 提现流程

```
用户发起提现
    │
    ▼
┌─────────────────────────────────────────┐
│ 1. 验证支付密码 ⚠️                        │
│    - WalletService.VerifyPaymentPassword│
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 2. 调用 Risk Service 风控检查           │
│    - CheckWithdraw                      │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 3. 检查地址白名单 ⚠️                       │
│    - IsWhitelisted                      │
│    - 如果开启白名单则必须通过             │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 4. 调用 Account Service 扣款            │
│    - Withdraw                           │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 5. 创建待确认提现记录                    │
│    - status = Pending                   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
用户确认 (2FA/邮件)
    │
    ▼
┌─────────────────────────────────────────┐
│ 6. 签名并广播交易                        │
│    - ⚠️ 待实现: 离线签名                 │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 7. 更新提现状态                          │
│    - status = Confirmed                 │
│    - tx_id                              │
└─────────────────┬───────────────────────┘
```

## 6. ⚠️ 重要待实现功能

1. **链上充值监听 (Chain Scanner)**
   - 扫描区块链节点/区块
   - 解析链上交易
   - 自动确认充值

2. **离线签名**
   - 提现交易离线签名
   - 多签支持

3. **Admin 审核接口**
   - 大额提现人工审核
   - 充值异常处理

## 7. 配置

```yaml
# crates/wallet-service/config/wallet_service.yaml
service:
  host: "0.0.0.0"
  port: 50002

database:
  driver: "sqlite"
  url: "sqlite:./data/wallet.db"
  max_connections: 20

account_service:
  addr: "localhost:50019"
  timeout_ms: 5000

risk_service:
  addr: "localhost:50004"
  timeout_ms: 3000

kafka:
  brokers:
    - "localhost:9092"
  topics:
    user_events: "user_events"
    chain_events: "chain_events"

wallet:
  # 支持的链
  supported_chains:
    - ETH
    - BSC
    - ARBITRUM

  # 提现配置
  withdraw:
    # 是否开启白名单
    require_whitelist: false
    # 是否需要支付密码
    require_payment_password: true
    # 大额提现审核阈值
    large_amount_threshold: "10000"
    # 手续费
    fee:
      ETH: "0.001"
      BSC: "0.0005"
      ARBITRUM: "0.0001"
```

## 8. 错误码

| 错误码 | 说明 |
|--------|------|
| ADDRESS_NOT_FOUND | 地址不存在 |
| DEPOSIT_NOT_FOUND | 充值记录不存在 |
| DEPOSIT_ALREADY_CONFIRMED | 充值已确认 |
| WITHDRAW_NOT_FOUND | 提现记录不存在 |
| WITHDRAW_ALREADY_CONFIRMED | 提现已确认 |
| WITHDRAW_NOT_PENDING | 提现状态不可操作 |
| INSUFFICIENT_BALANCE | 余额不足 |
| INSUFFICIENT_FEE | 手续费不足 |
| ADDRESS_NOT_WHITELISTED | 地址不在白名单 |
| PAYMENT_PASSWORD_REQUIRED | 需要支付密码 |
| PAYMENT_PASSWORD_INVALID | 支付密码错误 |
| CHAIN_NOT_SUPPORTED | 不支持的链 |
| BROADCAST_FAILED | 广播失败 |

## 9. 目录结构

```
crates/wallet-service/
├── Cargo.toml
├── build.rs
├── config/
│   └── wallet_service.yaml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── config.rs
    ├── server.rs
    ├── error.rs
    ├── pb.rs
    ├── pb/
    │   ├── wallet.proto
    │   └── wallet.rs
    ├── repository/
    │   ├── mod.rs
    │   ├── deposit_repo.rs
    │   ├── withdraw_repo.rs
    │   ├── whitelist_repo.rs
    │   └── payment_password_repo.rs
    ├── services/
    │   ├── mod.rs
    │   └── wallet_service_impl.rs
    ├── clients/
    │   ├── mod.rs
    │   ├── account_service_client.rs
    │   └── risk_service_client.rs
    └── utils/
        └── mod.rs
```
