# Clearing Service - 清算服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50008 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | Position Service (50005), Account Service (50019), Trade Service (50013) |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 成交清算 | 结算交易双方的资金与结果代币 (Maker/Taker) |
| 派彩计算 | 市场结算后计算用户盈亏（预测市场） |
| 手续费结算 | 收取并分配手续费 |
| 资产兑换 | 结算时结果代币 ↔ 基础资产兑换 |

### 1.1.2 功能列表

```
Clearing Service
├── 成交清算
│   ├── ClearTrade - 清算单笔成交
│   ├── BatchClearTrades - 批量清算
│   └── GetClearingRecord - 查询清算记录
├── 预测市场派彩
│   ├── SettleMarket - 结算市场（被 Prediction Market Service 调用）
│   ├── CalculatePayout - 计算用户派彩
│   ├── GetPayoutDetails - 查询派彩详情
│   └── ListSettlements - 结算历史
├── 手续费管理
│   ├── GetFeeConfig - 获取手续费配置
│   ├── SetFeeConfig - 设置手续费
│   └── CalculateFee - 计算手续费
└── 资产兑换
    └── SettleOutcomeToken - 结算时结果代币兑换基础资产 (调用 Account Service.Settle)
```

## 2. 数据模型

### 2.1 Domain Model

```rust
/// 清算记录
pub struct ClearingRecord {
    pub id: i64,
    pub trade_id: String,
    pub maker_user_id: i64,
    pub taker_user_id: i64,
    pub market_id: i64,
    pub outcome_id: i64,
    pub outcome_asset: String,    // 结果代币标识 (如 "12345_yes")
    pub maker_side: OrderSide,
    pub price: Decimal,
    pub quantity: Decimal,
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
    pub created_at: i64,
}

/// 市场结算记录
pub struct MarketSettlement {
    pub id: i64,
    pub market_id: i64,
    pub winning_outcome_id: i64,
    pub winning_outcome_asset: String,  // 获胜结果代币标识
    pub total_volume: Decimal,
    pub loser_volume: Decimal,
    pub total_payout: Decimal,
    pub fee: Decimal,
    pub settled_at: i64,
}

/// 用户派彩记录
pub struct UserPayout {
    pub id: i64,
    pub settlement_id: i64,
    pub user_id: i64,
    pub outcome_id: i64,
    pub outcome_asset: String,    // 结果代币标识
    pub quantity: Decimal,
    pub avg_price: Decimal,
    pub payout: Decimal,
    pub is_winner: bool,          // 是否获胜方
    pub created_at: i64,
}

/// 手续费配置
pub struct FeeConfig {
    pub id: i64,
    pub market_id: Option<i64>,  // null 表示全局配置
    pub maker_fee: Decimal,
    pub taker_fee: Decimal,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### 2.2 数据库表

**clearing_records 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| trade_id | VARCHAR(64) | UNIQUE, NOT NULL | 成交ID |
| maker_user_id | BIGINT | NOT NULL | Maker用户 |
| taker_user_id | BIGINT | NOT NULL | Taker用户 |
| market_id | BIGINT | NOT NULL | 市场ID |
| outcome_id | BIGINT | NOT NULL | 选项ID |
| outcome_asset | VARCHAR(32) | NOT NULL | 结果代币标识 |
| maker_side | VARCHAR(10) | NOT NULL | Maker方向 |
| price | TEXT | NOT NULL | 成交价格 |
| quantity | TEXT | NOT NULL | 成交数量 |
| maker_fee | TEXT | NOT NULL | Maker手续费 |
| taker_fee | TEXT | NOT NULL | Taker手续费 |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `PRIMARY KEY (id)`
- `UNIQUE INDEX idx_trade_id (trade_id)`
- `INDEX idx_maker_user (maker_user_id)`
- `INDEX idx_taker_user (taker_user_id)`

**market_settlements 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| market_id | BIGINT | UNIQUE, NOT NULL | 市场ID |
| winning_outcome_id | BIGINT | NOT NULL | 获胜选项ID |
| winning_outcome_asset | VARCHAR(32) | NOT NULL | 获胜结果代币标识 |
| total_volume | TEXT | NOT NULL | 总成交量 |
| loser_volume | TEXT | NOT NULL | 输家总量 |
| total_payout | TEXT | NOT NULL | 总派彩金额 |
| fee | TEXT | NOT NULL | 手续费 |
| settled_at | BIGINT | NOT NULL | 结算时间 |

**索引**:
- `PRIMARY KEY (id)`
- `UNIQUE INDEX idx_market_id (market_id)`

**user_payouts 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| settlement_id | BIGINT | NOT NULL, FK | 结算记录ID |
| user_id | BIGINT | NOT NULL | 用户ID |
| outcome_id | BIGINT | NOT NULL | 选项ID |
| outcome_asset | VARCHAR(32) | NOT NULL | 结果代币标识 |
| quantity | TEXT | NOT NULL | 持仓数量 |
| avg_price | TEXT | NOT NULL | 平均价格 |
| payout | TEXT | NOT NULL | 派彩金额 |
| is_winner | BOOLEAN | NOT NULL | 是否获胜方 |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `PRIMARY KEY (id)`
- `INDEX idx_settlement_id (settlement_id)`
- `INDEX idx_user_id (user_id)`

**fee_configs 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| market_id | BIGINT | NULL | 市场ID (NULL=全局) |
| maker_fee | TEXT | NOT NULL | Maker手续费率 |
| taker_fee | TEXT | NOT NULL | Taker手续费率 |
| created_at | BIGINT | NOT NULL | 创建时间 |
| updated_at | BIGINT | NOT NULL | 更新时间 |

**索引**:
- `PRIMARY KEY (id)`
- `UNIQUE INDEX idx_market_id (market_id)`

## 3. Proto 接口

```protobuf
syntax = "proto3";

package clearing;

service ClearingService {
    // ========== 成交清算 ==========
    rpc ClearTrade(ClearTradeRequest) returns (ClearTradeResponse);
    rpc BatchClearTrades(BatchClearTradesRequest) returns (BatchClearTradesResponse);

    // ========== 市场结算派彩 ==========
    rpc SettleMarket(SettleMarketRequest) returns (SettleMarketResponse);
    rpc CalculatePayout(CalculatePayoutRequest) returns (CalculatePayoutResponse);
    rpc GetPayoutDetails(GetPayoutDetailsRequest) returns (GetPayoutDetailsResponse);
    rpc ListSettlements(ListSettlementsRequest) returns (ListSettlementsResponse);

    // ========== 手续费管理 ==========
    rpc GetFeeConfig(GetFeeConfigRequest) returns (GetFeeConfigResponse);
    rpc SetFeeConfig(SetFeeConfigRequest) returns (SetFeeConfigResponse);
    rpc CalculateFee(CalculateFeeRequest) returns (CalculateFeeResponse);

    // ========== 查询 ==========
    rpc GetClearingRecord(GetClearingRecordRequest) returns (GetClearingRecordResponse);
}

// ==================== 成交清算 ====================

message ClearTradeRequest {
    string trade_id = 1;
    int64 maker_user_id = 2;
    int64 taker_user_id = 3;
    int64 market_id = 4;
    int64 outcome_id = 5;
    string outcome_asset = 6;    // 结果代币标识 (如 "12345_yes")
    string side = 7;
    string price = 8;
    string quantity = 9;
    string maker_fee = 10;
    string taker_fee = 11;
}

message ClearTradeResponse {
    bool success = 1;
    string message = 2;
}

message BatchClearTradesRequest {
    repeated ClearTradeRequest trades = 1;
}

message BatchClearTradesResponse {
    bool success = 1;
    string message = 2;
    int32 processed = 3;
    int32 failed = 4;
}

// ==================== 市场结算派彩 ====================

message SettleMarketRequest {
    int64 market_id = 1;
    int64 winning_outcome_id = 2;
}

message SettleMarketResponse {
    bool success = 1;
    string message = 2;
    int64 market_id = 3;
    string total_payout = 4;
    int32 payout_count = 5;
    int32 loser_count = 6;       // 输家用户数
}

message CalculatePayoutRequest {
    int64 market_id = 1;
    int64 outcome_id = 2;
    int64 user_id = 3;
}

message CalculatePayoutResponse {
    int64 user_id = 1;
    int64 market_id = 2;
    int64 outcome_id = 3;
    string outcome_asset = 4;    // 结果代币标识
    string quantity = 5;
    string avg_price = 6;
    string payout = 7;
    bool is_winner = 8;
}

message GetPayoutDetailsRequest {
    int64 market_id = 1;
    int64 user_id = 2;
}

message GetPayoutDetailsResponse {
    repeated UserPayoutSummary payouts = 1;
}

message UserPayoutSummary {
    int64 user_id = 1;
    int64 market_id = 2;
    string outcome_asset = 3;
    string quantity = 4;
    string avg_price = 5;
    string payout = 6;
    bool is_winner = 7;
}

message ListSettlementsRequest {
    int64 market_id = 1;
    int32 page = 2;
    int32 page_size = 3;
}

message ListSettlementsResponse {
    repeated SettlementSummary settlements = 1;
    int64 total = 2;
    int32 page = 3;
    int32 page_size = 4;
}

message SettlementSummary {
    int64 market_id = 1;
    int64 winning_outcome_id = 2;
    string total_volume = 3;
    string total_payout = 4;
    int64 settled_at = 5;
}

// ==================== 手续费管理 ====================

message GetFeeConfigRequest {
    int64 market_id = 1;  // 可选，不传则查全局配置
}

message GetFeeConfigResponse {
    int64 market_id = 1;
    string maker_fee = 2;
    string taker_fee = 3;
}

message SetFeeConfigRequest {
    int64 market_id = 1;  // 可选，null 表示全局配置
    string maker_fee = 2;
    string taker_fee = 3;
}

message SetFeeConfigResponse {
    bool success = 1;
    string message = 2;
}

message CalculateFeeRequest {
    string price = 1;
    string quantity = 2;
    string side = 3;           // "maker" or "taker"
    int64 market_id = 4;       // 可选
}

message CalculateFeeResponse {
    string fee = 1;
    string fee_rate = 2;
}

// ==================== 查询 ====================

message GetClearingRecordRequest {
    string trade_id = 1;
}

message GetClearingRecordResponse {
    string trade_id = 1;
    int64 maker_user_id = 2;
    int64 taker_user_id = 3;
    string outcome_asset = 4;
    string price = 5;
    string quantity = 6;
    string maker_fee = 7;
    string taker_fee = 8;
    int64 created_at = 9;
}
```

## 4. 服务间通信

### 4.1 gRPC 调用

| 被调用方 | 接口 | 场景 |
|----------|------|------|
| Account Service (50019) | FreezeAndDeduct | 成交清算：扣减冻结并增加结果代币 |
| Account Service (50019) | Settle | 结算派彩：结果代币兑换为基础资产 |
| Position Service (50005) | GetUserPositions | 查询用户持仓用于派彩计算 |
| Trade Service (50013) | GetTrades | 查询市场成交记录 |

### 4.2 gRPC 被调用

| 调用方 | 接口 | 场景 |
|----------|------|------|
| Prediction Market Service (50010) | SettleMarket | 市场结算时调用 |
| Matching Engine (50009) | ClearTrade | 成交时触发清算 |
| API Gateway | 各类查询接口 | 清算记录查询 |

### 4.3 Kafka 消费

| Topic | 处理 |
|-------|------|
| trade_executed | 清算成交（Matching Engine 发布） |
| market_events | 市场结算触发派彩（Prediction Market 发布） |

### 4.4 Kafka 生产

| 事件 | Topic | 说明 |
|------|-------|------|
| TradeCleared | trade_events | 成交清算完成 |
| MarketSettled | settlement_events | 市场结算完成 |

## 5. 核心流程

### 5.1 成交清算流程

> 预测市场成交模型：买方用 USDT 购买结果代币，卖方用结果代币换回 USDT。

```
Matching Engine 成交
    │
    ▼ 发布 Kafka: trade_executed
┌─────────────────────────────────────────┐
│ Clearing Service 消费 trade_executed    │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 1. 计算手续费                           │
│    - Maker Fee = price * quantity * maker_fee_rate│
│    - Taker Fee = price * quantity * taker_fee_rate│
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 2. 执行资金清算 (调用 Account Service)  │
│                                         │
│  买方 (Buy Side):                       │
│    - USDT: FreezeAndDeduct 扣减冻结金额 │
│    - 结果代币: available += quantity    │
│                                         │
│  卖方 (Sell Side):                      │
│    - 结果代币: FreezeAndDeduct 扣减冻结 │
│    - USDT: available += (price * qty - fee)│
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 3. 更新持仓 (调用 Position Service)     │
│    - 买方: 增加对应结果代币持仓         │
│    - 卖方: 减少对应结果代币持仓         │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 4. 保存清算记录                         │
│    - clearing_records 表                │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 5. 发布 Kafka: TradeCleared             │
└─────────────────────────────────────────┘
```

### 5.2 市场结算派彩流程

> 预测市场结算：获胜结果代币兑换为 USDT，失败结果代币清零。

```
Prediction Market Service 调用 SettleMarket
    │
    ▼
┌─────────────────────────────────────────┐
│ 1. 查询市场所有用户持仓                  │
│    - 从 Position Service 获取           │
│    - 按获胜/失败选项分组                 │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 2. 计算输家资金                         │
│    - 输家 = 持有失败结果代币的用户       │
│    - 输家资金 = sum(输家持仓 * 买入均价) │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 3. 计算赢家派彩                         │
│    - 赢家 = 持有获胜结果代币的用户       │
│    - 总派彩池 = 输家资金 * (1 - 平台手续费率)│
│    - 每用户派彩 =                       │
│      (用户持仓 / 赢家总持仓) * 总派彩池  │
│    - 每用户净收益 = 派彩 - 买入成本      │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 4. 执行资产兑换 (调用 Account Service)  │
│                                         │
│  赢家:                                   │
│    - 调用 Settle: 结果代币清零 → USDT增加│
│    - 消耗获胜结果代币，获得派彩 USDT     │
│                                         │
│  输家:                                   │
│    - 调用 Settle: 失败结果代币清零       │
│    - 结果代币余额归零，无 USDT 返还      │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 5. 保存结算记录                         │
│    - market_settlements 表              │
│    - user_payouts 表                    │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│ 6. 发布 Kafka: MarketSettled            │
└─────────────────────────────────────────┘
                  │
                  ▼
返回 SettleMarketResponse
```

## 6. 配置

```yaml
# crates/clearing-service/config/clearing_service.yaml
service:
  host: "0.0.0.0"
  port: 50008

database:
  driver: "sqlite"
  url: "sqlite:./data/clearing.db"
  max_connections: 20

account_service:
  addr: "localhost:50019"
  timeout_ms: 5000

position_service:
  addr: "localhost:50005"
  timeout_ms: 3000

trade_service:
  addr: "localhost:50013"
  timeout_ms: 3000

kafka:
  brokers:
    - "localhost:9092"
  topics:
    trade_events: "trade_events"
    settlement_events: "settlement_events"

clearing:
  # 默认手续费率
  default_maker_fee: "0.001"  # 0.1%
  default_taker_fee: "0.001"  # 0.1%
  # 派彩手续费率
  payout_fee_rate: "0.02"     # 2%
```

## 7. 错误码

| 错误码 | 说明 |
|--------|------|
| TRADE_NOT_FOUND | 成交记录不存在 |
| TRADE_ALREADY_CLEARED | 成交已清算 |
| MARKET_NOT_FOUND | 市场不存在 |
| SETTLEMENT_FAILED | 结算失败 |
| SETTLEMENT_ALREADY_DONE | 市场已结算 |
| INSUFFICIENT_BALANCE | 余额不足 |
| FEE_CONFIG_NOT_FOUND | 手续费配置不存在 |
| OUTCOME_ASSET_INVALID | 结果代币标识无效 |

## 8. 目录结构

```
crates/clearing-service/
├── Cargo.toml
├── build.rs
├── config/
│   └── clearing_service.yaml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── config.rs
    ├── server.rs
    ├── error.rs
    ├── pb.rs
    ├── pb/
    │   ├── clearing.proto
    │   └── clearing.rs
    ├── repository/
    │   ├── mod.rs
    │   ├── clearing_repo.rs
    │   ├── settlement_repo.rs
    │   ├── payout_repo.rs
    │   └── fee_config_repo.rs
    ├── services/
    │   ├── mod.rs
    │   └── clearing_service_impl.rs
    ├── clients/
    │   ├── mod.rs
    │   ├── account_service_client.rs
    │   ├── position_service_client.rs
    │   └── trade_service_client.rs
    └── utils/
        └── mod.rs
```
