# Risk Service - 风控服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50004 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | 无 |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| Pre-trade 检查 | ⚠️ 下单前必须调用风控检查 |
| 提现风控 | 提现限额检查 |
| 用户风控 | 用户状态检查 |

### 1.1.2 功能列表

```
Risk Service
├── 下单风控
│   ├── CheckOrder - 下单前风控检查
│   ├── CheckOrderBatch - 批量订单检查
│   └── GetOrderQuota - 查询用户订单限额
├── 提现风控
│   ├── CheckWithdraw - 提现风控检查
│   └── GetWithdrawQuota - 查询用户提现限额
├── 用户风控
│   ├── CheckUserStatus - 用户状态检查
│   ├── FreezeUser - 冻结用户交易
│   └── UnfreezeUser - 解冻用户
└── 风控配置
    ├── GetRiskConfig - 获取风控配置
    └── UpdateRiskConfig - 更新风控配置
```

## 2. 服务间通信

### 2.1 gRPC 被调用

| 调用方 | 接口 | 场景 | 状态 |
|----------|------|------|------|
| Order Service (50003) | CheckOrder | 下单前检查 | ⚠️ 文档说调用，但代码未实现 |
| Wallet Service (50002) | CheckWithdraw | 提现前检查 | 待确认 |
| API Gateway | CheckUserStatus | 用户状态检查 | - |

## 2. Proto 接口

```protobuf
syntax = "proto3";

package risk;

service RiskService {
    // ========== 下单风控 ==========
    rpc CheckOrder(CheckOrderRequest) returns (CheckOrderResponse);
    rpc CheckOrderBatch(CheckOrderBatchRequest) returns (CheckOrderBatchResponse);
    rpc GetOrderQuota(GetOrderQuotaRequest) returns (GetOrderQuotaResponse);

    // ========== 提现风控 ==========
    rpc CheckWithdraw(CheckWithdrawRequest) returns (CheckWithdrawResponse);
    rpc GetWithdrawQuota(GetWithdrawQuotaRequest) returns (GetWithdrawQuotaResponse);

    // ========== 用户风控 ==========
    rpc CheckUserStatus(CheckUserStatusRequest) returns (CheckUserStatusResponse);
    rpc FreezeUser(FreezeUserRequest) returns (FreezeUserResponse);
    rpc UnfreezeUser(UnfreezeUserRequest) returns (UnfreezeUserResponse);

    // ========== 风控配置 ==========
    rpc GetRiskConfig(GetRiskConfigRequest) returns (GetRiskConfigResponse);
    rpc UpdateRiskConfig(UpdateRiskConfigRequest) returns (UpdateRiskConfigResponse);
}

// ==================== 下单风控 ====================

message CheckOrderRequest {
    int64 user_id = 1;
    int64 market_id = 2;
    string side = 3;           // "buy" or "sell"
    string order_type = 4;     // "limit", "market", etc.
    string price = 5;
    string quantity = 6;
    string total_amount = 7;   // price * quantity
}

message CheckOrderResponse {
    bool pass = 1;
    string reason = 2;
    string rejected_rule = 3;  // 触发的规则名称
}

message CheckOrderBatchRequest {
    repeated CheckOrderRequest orders = 1;
}

message CheckOrderBatchResponse {
    repeated CheckOrderResponse results = 1;
}

message GetOrderQuotaRequest {
    int64 user_id = 1;
}

message GetOrderQuotaResponse {
    int64 user_id = 1;
    string daily_limit = 2;
    string daily_used = 3;
    string remaining = 4;
}

// ==================== 提现风控 ====================

message CheckWithdrawRequest {
    int64 user_id = 1;
    string asset = 2;
    string amount = 3;
    string address = 4;
}

message CheckWithdrawResponse {
    bool pass = 1;
    string reason = 2;
    string rejected_rule = 3;
}

message GetWithdrawQuotaRequest {
    int64 user_id = 1;
    string asset = 2;
}

message GetWithdrawQuotaResponse {
    int64 user_id = 1;
    string asset = 2;
    string daily_limit = 3;
    string daily_used = 4;
    string remaining = 5;
}

// ==================== 用户风控 ====================

message CheckUserStatusRequest {
    int64 user_id = 1;
}

message CheckUserStatusResponse {
    bool pass = 1;
    string status = 2;         // "active", "frozen", "restricted"
    string reason = 3;
}

message FreezeUserRequest {
    int64 user_id = 1;
    string reason = 2;
}

message FreezeUserResponse {
    bool success = 1;
    string message = 2;
}

message UnfreezeUserRequest {
    int64 user_id = 1;
    string reason = 2;
}

message UnfreezeUserResponse {
    bool success = 1;
    string message = 2;
}

// ==================== 风控配置 ====================

message GetRiskConfigRequest {
    string config_key = 1;  // 可选，不传则返回全部
}

message GetRiskConfigResponse {
    map<string, string> configs = 1;
}

message UpdateRiskConfigRequest {
    string config_key = 1;
    string config_value = 2;
}

message UpdateRiskConfigResponse {
    bool success = 1;
    string message = 2;
}
```

## 3. 风控规则

### 3.1 下单风控规则

| 规则 | 说明 | 默认值 |
|------|------|--------|
| max_order_amount | 单笔最大交易金额 | 1,000,000 |
| daily_order_limit | 24小时累计交易限额 | 5,000,000 |
| max_order_count | 单日最大订单数 | 1000 |
| min_order_amount | 单笔最小交易金额 | 1 |

### 3.2 提现风控规则

| 规则 | 说明 | 默认值 |
|------|------|--------|
| max_withdraw | 单笔最大提现金额 | 100,000 |
| daily_withdraw_limit | 24小时累计提现限额 | 500,000 |
| require_withdraw_whitelist | 必须提现到白名单地址 | false |
| require_payment_password | 必须验证支付密码 | true |

### 3.3 用户状态检查

| 状态 | 说明 |
|------|------|
| active | 正常交易 |
| frozen | 已冻结，无法交易 |
| restricted | 限制交易（如需要额外验证） |

## 4. 配置

```yaml
# crates/risk-service/config/risk_service.yaml
service:
  host: "0.0.0.0"
  port: 50004

database:
  driver: "sqlite"
  url: "sqlite:./data/risk.db"
  max_connections: 10

kafka:
  brokers:
    - "localhost:9092"
  topics:
    risk_events: "risk_events"

risk:
  # 交易风控
  max_order_amount: "1000000"
  daily_order_limit: "5000000"
  max_order_count: 1000
  min_order_amount: "1"

  # 提现风控
  max_withdraw: "100000"
  daily_withdraw_limit: "500000"
  require_withdraw_whitelist: false
  require_payment_password: true

  # 用户风控
  enable_user_status_check: true
  auto_freeze_suspicious: true
```

## 5. 错误码

| 错误码 | 说明 |
|--------|------|
| ORDER_AMOUNT_EXCEEDED | 订单金额超限 |
| DAILY_LIMIT_EXCEEDED | 每日限额超限 |
| ORDER_COUNT_EXCEEDED | 订单数超限 |
| USER_FROZEN | 用户已被冻结 |
| USER_RESTRICTED | 用户受限 |
| WITHDRAW_AMOUNT_EXCEEDED | 提现金额超限 |
| WHITELIST_REQUIRED | 需要提现白名单 |

## 6. 目录结构

```
crates/risk-service/
├── Cargo.toml
├── build.rs
├── config/
│   └── risk_service.yaml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── config.rs
    ├── server.rs
    ├── error.rs
    ├── pb.rs
    ├── pb/
    │   ├── risk.proto
    │   └── risk.rs
    ├── repository/
    │   ├── mod.rs
    │   └── risk_config_repo.rs
    ├── services/
    │   ├── mod.rs
    │   ├── risk_service_impl.rs
    │   └── rules/
    │       ├── mod.rs
    │       ├── order_rule.rs
    │       ├── withdraw_rule.rs
    │       └── user_rule.rs
    └── utils/
        └── mod.rs
```

## 7. ⚠️ 重要注意事项

**Order Service 必须调用 Risk Service 进行下单前风控检查！**

```
创建订单流程:
1. 参数校验
2. 生成订单号
3. ⚠️ 调用 RiskService.CheckOrder  ← 必须
4. 检查并冻结余额
5. 保存订单
6. 提交到 Matching Engine
```

如果 Order Service 未集成风控检查，将无法阻止：
- 异常大额订单
- 超出日限额的订单
- 被冻结用户的订单
