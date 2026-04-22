# Ledger Service - 账本服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50011 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | 无 |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 账本记录 | 不可变流水记录 |
| 复式记账 | 借贷配对，每笔借必有对应贷 |
| 对账基础 | 可重算验证余额 |
| 审计追踪 | 所有资金变动的不可篡改记录 |

## 2. 数据模型

### 2.1 业务类型枚举

```
BizType:
  deposit           - 充值
  withdraw          - 提现
  freeze            - 冻结 (下单)
  unfreeze          - 解冻 (撤单)
  deduct            - 扣减 (成交)
  fee               - 手续费
  transfer_in       - 转入
  transfer_out      - 转出
  lock              - 风控锁定
  unlock            - 风控解锁
  settlement_win    - 结算派彩 (获胜)
  settlement_lose   - 结算清零 (失败)
```

### 2.2 数据库表

**ledger_entries 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| biz_type | VARCHAR(30) | NOT NULL | 业务类型 (见枚举) |
| ref_id | VARCHAR(64) | | 关联ID (order_id, trade_id, settlement_id) |
| user_id | BIGINT | NOT NULL | 用户ID |
| asset | VARCHAR(32) | NOT NULL | 资产标识 ("USDT" 或 "{market_id}_{outcome}") |
| entry_type | VARCHAR(10) | NOT NULL | DEBIT/CREDIT |
| amount | TEXT | NOT NULL | 金额 |
| balance_after | TEXT | NOT NULL | 操作后余额 |
| counterpart_entry_id | BIGINT | | 配对分录ID (复式记账) |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `PRIMARY KEY (id)`
- `INDEX idx_user_asset (user_id, asset)`
- `INDEX idx_ref_id (ref_id)`
- `INDEX idx_biz_type (biz_type)`
- `INDEX idx_counterpart (counterpart_entry_id)`
- `INDEX idx_created_at (created_at)`

## 3. Proto 接口

```protobuf
syntax = "proto3";

package ledger;

service LedgerService {
    rpc GetEntries(GetEntriesRequest) returns (GetEntriesResponse);
    rpc GetBalanceSummary(GetBalanceSummaryRequest) returns (GetBalanceSummaryResponse);
    rpc VerifyBalance(VerifyBalanceRequest) returns (VerifyBalanceResponse);
}

message GetEntriesRequest {
    int64 user_id = 1;
    string asset = 2;
    string biz_type = 3;         // 可选，按业务类型过滤
    int64 start_time = 4;
    int64 end_time = 5;
    int32 page = 6;
    int32 page_size = 7;
}

message GetEntriesResponse {
    repeated LedgerEntry entries = 1;
    int64 total = 2;
}

message LedgerEntry {
    int64 id = 1;
    string biz_type = 2;
    string ref_id = 3;
    string entry_type = 4;
    string amount = 5;
    string balance_after = 6;
    int64 counterpart_entry_id = 7;
    int64 created_at = 8;
}

message GetBalanceSummaryRequest {
    int64 user_id = 1;
    string asset = 2;
}

message GetBalanceSummaryResponse {
    int64 user_id = 1;
    string asset = 2;
    string total_debit = 3;
    string total_credit = 4;
    string net_balance = 5;
}

message VerifyBalanceRequest {
    int64 user_id = 1;
    string asset = 2;
}

message VerifyBalanceResponse {
    bool valid = 1;
    string calculated_balance = 2;   // 重算得出的余额
    string recorded_balance = 3;     // 最新分录记录的余额
    string discrepancy = 4;          // 差额 (0 表示一致)
}
```

## 4. Kafka 消费

| Topic | 处理 |
|-------|------|
| balance_updates | 记录账本流水 (充值/提现/冻结/解冻/扣减等) |
| settlement_events | 记录结算派彩流水 (派彩/清零) |

## 5. 配置

```yaml
service:
  port: 50011
database:
  driver: "sqlite"
  url: "sqlite:./data/ledger.db"
kafka:
  brokers:
    - "localhost:9092"
  topics:
    balance_updates: "balance_updates"
    settlement_events: "settlement_events"
```

## 6. 目录结构

```
crates/ledger-service/
├── Cargo.toml, build.rs, config/, src/
```
