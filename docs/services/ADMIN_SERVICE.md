# Admin Service - 管理服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50007 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | 所有业务服务 |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 用户管理 | 用户列表、冻结、解冻 |
| ⚠️ KYC 审核 | 身份认证审核 |
| ⚠️ 提现审核 | 大额提现人工审核 |
| 市场管理 | 创建市场、修改配置 |
| 系统管理 | 系统配置、参数调整 |
| 报表 | 业务报表查看 |

### 1.1.2 功能列表

```
Admin Service
├── 用户管理
│   ├── ListUsers - 用户列表
│   ├── GetUserDetail - 用户详情
│   ├── FreezeUser - 冻结用户
│   └── UnfreezeUser - 解冻用户
├── KYC 审核 (⚠️ 待实现)
│   ├── ListKYCApplications - KYC 申请列表
│   ├── ReviewKYC - 审核 KYC
│   ├── GetKYCDetail - 查看 KYC 详情
│   └── RejectKYC - 拒绝 KYC
├── 提现审核 (⚠️ 待实现)
│   ├── ListPendingWithdraws - 待审核提现列表
│   ├── ApproveWithdraw - 批准提现
│   ├── RejectWithdraw - 拒绝提现
│   └── GetWithdrawDetail - 提现详情
├── 市场管理
│   ├── AdminCreateMarket - 创建市场
│   ├── AdminUpdateMarket - 更新市场
│   ├── AdminCloseMarket - 关闭市场
│   └── AdminResolveMarket - 结算市场
├── 系统配置
│   ├── GetFeeConfig - 手续费配置
│   ├── SetFeeConfig - 设置手续费
│   ├── GetRiskConfig - 风控配置
│   └── SetRiskConfig - 设置风控
└── 报表统计
    ├── GetStats - 系统统计
    ├── GetTradingVolume - 交易量统计
    └── GetUserStats - 用户统计
```

## 2. Proto 接口

```protobuf
syntax = "proto3";

package admin;

service AdminService {
    // ========== 用户管理 ==========
    rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
    rpc GetUserDetail(GetUserDetailRequest) returns (GetUserDetailResponse);
    rpc FreezeUser(FreezeUserRequest) returns (FreezeUserResponse);
    rpc UnfreezeUser(UnfreezeUserRequest) returns (UnfreezeUserResponse);

    // ========== KYC 审核 (⚠️ 待实现) ==========
    rpc ListKYCApplications(ListKYCApplicationsRequest) returns (ListKYCApplicationsResponse);
    rpc GetKYCDetail(GetKYCDetailRequest) returns (GetKYCDetailResponse);
    rpc ReviewKYC(ReviewKYCRequest) returns (ReviewKYCResponse);
    rpc RejectKYC(RejectKYCRequest) returns (RejectKYCResponse);

    // ========== 提现审核 (⚠️ 待实现) ==========
    rpc ListPendingWithdraws(ListPendingWithdrawsRequest) returns (ListPendingWithdrawsResponse);
    rpc GetWithdrawDetail(GetWithdrawDetailRequest) returns (GetWithdrawDetailResponse);
    rpc ApproveWithdraw(ApproveWithdrawRequest) returns (ApproveWithdrawResponse);
    rpc RejectWithdraw(RejectWithdrawRequest) returns (RejectWithdrawResponse);

    // ========== 市场管理 ==========
    rpc AdminCreateMarket(AdminCreateMarketRequest) returns (AdminCreateMarketResponse);
    rpc AdminUpdateMarket(AdminUpdateMarketRequest) returns (AdminUpdateMarketResponse);
    rpc AdminCloseMarket(AdminCloseMarketRequest) returns (AdminCloseMarketResponse);
    rpc AdminResolveMarket(AdminResolveMarketRequest) returns (AdminResolveMarketResponse);

    // ========== 系统配置 ==========
    rpc GetFeeConfig(GetFeeConfigRequest) returns (GetFeeConfigResponse);
    rpc SetFeeConfig(SetFeeConfigRequest) returns (SetFeeConfigResponse);
    rpc GetRiskConfig(GetRiskConfigRequest) returns (GetRiskConfigResponse);
    rpc SetRiskConfig(SetRiskConfigRequest) returns (SetRiskConfigResponse);

    // ========== 报表统计 ==========
    rpc GetStats(GetStatsRequest) returns (GetStatsResponse);
    rpc GetTradingVolume(GetTradingVolumeRequest) returns (GetTradingVolumeResponse);
    rpc GetUserStats(GetUserStatsRequest) returns (GetUserStatsResponse);
}

// ==================== 用户管理 ====================

message ListUsersRequest {
    string status = 1;  // "active", "frozen", "pending_kyc"
    int32 page = 2;
    int32 page_size = 3;
}

message ListUsersResponse {
    repeated UserInfo users = 1;
    int64 total = 2;
}

message UserInfo {
    int64 user_id = 1;
    string email = 2;
    string status = 3;
    string kyc_status = 4;  // "none", "pending", "approved", "rejected"
    int64 created_at = 5;
}

message GetUserDetailRequest {
    int64 user_id = 1;
}

message GetUserDetailResponse {
    int64 user_id = 1;
    string email = 2;
    string status = 3;
    string kyc_status = 4;
    int64 created_at = 5;
    int64 last_login_at = 6;
    string total_deposit = 7;
    string total_withdraw = 8;
    string total_volume = 9;
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

// ==================== KYC 审核 (⚠️ 待实现) ====================

message ListKYCApplicationsRequest {
    string status = 1;  // "pending", "approved", "rejected"
    int32 page = 2;
    int32 page_size = 3;
}

message ListKYCApplicationsResponse {
    repeated KYCApplicationSummary applications = 1;
    int64 total = 2;
}

message KYCApplicationSummary {
    int64 user_id = 1;
    string email = 2;
    string real_name = 3;
    string id_type = 4;  // "passport", "id_card"
    string status = 5;
    int64 submitted_at = 6;
}

message GetKYCDetailRequest {
    int64 user_id = 1;
}

message GetKYCDetailResponse {
    int64 user_id = 1;
    string real_name = 2;
    string id_type = 3;
    string id_number = 4;  // 加密存储
    string id_front_image = 5;
    string id_back_image = 6;
    string selfie_image = 7;
    string status = 8;
    string reject_reason = 9;
    int64 submitted_at = 10;
    int64 reviewed_at = 11;
}

message ReviewKYCRequest {
    int64 user_id = 1;
    int64 reviewer_id = 2;  // 审核员ID
    bool approved = 3;
}

message ReviewKYCResponse {
    bool success = 1;
    string message = 2;
}

message RejectKYCRequest {
    int64 user_id = 1;
    int64 reviewer_id = 2;
    string reason = 3;
}

message RejectKYCResponse {
    bool success = 1;
    string message = 2;
}

// ==================== 提现审核 (⚠️ 待实现) ====================

message ListPendingWithdrawsRequest {
    int64 user_id = 1;
    string asset = 2;
    string status = 3;  // "pending", "approved", "rejected"
    int32 page = 4;
    int32 page_size = 5;
}

message ListPendingWithdrawsResponse {
    repeated WithdrawApplicationSummary withdrawals = 1;
    int64 total = 2;
}

message WithdrawApplicationSummary {
    string withdraw_id = 1;
    int64 user_id = 2;
    string asset = 3;
    string amount = 4;
    string fee = 5;
    string to_address = 6;
    string status = 7;
    int64 created_at = 8;
}

message GetWithdrawDetailRequest {
    string withdraw_id = 1;
}

message GetWithdrawDetailResponse {
    string withdraw_id = 1;
    int64 user_id = 2;
    string asset = 3;
    string amount = 4;
    string fee = 5;
    string to_address = 6;
    string status = 7;
    string tx_id = 8;
    int64 created_at = 9;
    int64 reviewed_at = 10;
}

message ApproveWithdrawRequest {
    string withdraw_id = 1;
    int64 reviewer_id = 2;
    string note = 3;
}

message ApproveWithdrawResponse {
    bool success = 1;
    string message = 2;
}

message RejectWithdrawRequest {
    string withdraw_id = 1;
    int64 reviewer_id = 2;
    string reason = 3;
}

message RejectWithdrawResponse {
    bool success = 1;
    string message = 2;
}

// ==================== 市场管理 ====================

message AdminCreateMarketRequest {
    string question = 1;
    string description = 2;
    string category = 3;
    string image_url = 4;
    int64 start_time = 5;
    int64 end_time = 6;
    repeated CreateOutcomeRequest outcomes = 7;
}

message CreateOutcomeRequest {
    string name = 1;
    string description = 2;
    string image_url = 3;
}

message AdminCreateMarketResponse {
    bool success = 1;
    int64 market_id = 2;
    string message = 3;
}

message AdminUpdateMarketRequest {
    int64 market_id = 1;
    string question = 2;
    string description = 3;
    string image_url = 4;
}

message AdminUpdateMarketResponse {
    bool success = 1;
    string message = 2;
}

message AdminCloseMarketRequest {
    int64 market_id = 1;
    string reason = 2;
}

message AdminCloseMarketResponse {
    bool success = 1;
    string message = 2;
}

message AdminResolveMarketRequest {
    int64 market_id = 1;
    int64 winning_outcome_id = 2;
    int64 resolver_id = 3;
    string reason = 4;            // 结算原因，必须留痕
    string evidence_url = 5;      // 结算依据链接 (可选)
}

message AdminResolveMarketResponse {
    bool success = 1;
    string message = 2;
    int64 market_id = 3;
}

// ==================== 系统配置 ====================

message GetFeeConfigRequest {
    int64 market_id = 1;  // 可选
}

message GetFeeConfigResponse {
    int64 market_id = 1;
    string maker_fee = 2;
    string taker_fee = 3;
}

message SetFeeConfigRequest {
    int64 market_id = 1;  // 可选，null 表示全局
    string maker_fee = 2;
    string taker_fee = 3;
}

message SetFeeConfigResponse {
    bool success = 1;
    string message = 2;
}

message GetRiskConfigRequest {
    string config_key = 1;  // 可选
}

message GetRiskConfigResponse {
    map<string, string> configs = 1;
}

message SetRiskConfigRequest {
    string config_key = 1;
    string config_value = 2;
}

message SetRiskConfigResponse {
    bool success = 1;
    string message = 2;
}

// ==================== 报表统计 ====================

message GetStatsRequest {}

message GetStatsResponse {
    int64 total_users = 1;
    int64 active_users_24h = 2;
    int64 total_markets = 3;
    int64 open_markets = 4;
    int64 total_orders_24h = 5;
    string total_volume_24h = 6;
    string total_fee_24h = 7;
}

message GetTradingVolumeRequest {
    int64 market_id = 1;
    int64 start_time = 2;
    int64 end_time = 3;
    string interval = 4;  // "hour", "day"
}

message GetTradingVolumeResponse {
    repeated VolumePoint volumes = 1;
}

message VolumePoint {
    int64 timestamp = 1;
    string volume = 2;
    int64 order_count = 3;
}

message GetUserStatsRequest {
    int64 user_id = 1;
    int64 start_time = 2;
    int64 end_time = 3;
}

message GetUserStatsResponse {
    int64 user_id = 1;
    string total_deposit = 2;
    string total_withdraw = 3;
    string total_volume = 4;
    int64 total_orders = 5;
    string realized_pnl = 6;
}
```

## 3. 服务间通信

### 3.1 gRPC 调用

| 被调用方 | 接口 | 场景 |
|----------|------|------|
| User Service (50001) | GetUser, FreezeUser | 用户管理 |
| Account Service (50019) | Lock, Unlock | 冻结用户时锁定/解锁账户余额 |
| Wallet Service (50002) | GetWithdrawHistory | 提现审核 |
| Market Data Service (50006) | GetStats | 统计 |
| Order Service (50003) | GetOrders | 订单统计 |
| Prediction Market Service (50010) | CreateMarket, ResolveMarket | 市场管理 |
| Clearing Service (50008) | GetFeeConfig, SetFeeConfig | 手续费管理 |
| Risk Service (50004) | GetRiskConfig, UpdateRiskConfig | 风控配置 |

### 3.2 gRPC 被调用

| 调用方 | 接口 | 场景 |
|----------|------|------|
| API Gateway | 各类管理接口 | 后台管理操作 |

## 4. 配置

```yaml
# crates/admin-service/config/admin_service.yaml
service:
  host: "0.0.0.0"
  port: 50007

database:
  driver: "sqlite"
  url: "sqlite:./data/admin.db"
  max_connections: 20

user_service:
  addr: "localhost:50001"
  timeout_ms: 3000

wallet_service:
  addr: "localhost:50002"
  timeout_ms: 3000

kyc:
  # KYC 审核配置
  require_kyc: false
  auto_approve: false

withdraw:
  # 提现审核配置
  require_review: true
  large_amount_threshold: "10000"
```

## 5. 错误码

| 错误码 | 说明 |
|--------|------|
| USER_NOT_FOUND | 用户不存在 |
| USER_ALREADY_FROZEN | 用户已冻结 |
| USER_ALREADY_UNFROZEN | 用户已解冻 |
| MARKET_NOT_FOUND | 市场不存在 |
| MARKET_NOT_OPEN | 市场未开放 |
| KYC_NOT_FOUND | KYC 记录不存在 |
| KYC_ALREADY_REVIEWED | KYC 已审核 |
| WITHDRAW_NOT_FOUND | 提现记录不存在 |
| WITHDRAW_ALREADY_REVIEWED | 提现已审核 |
| INSUFFICIENT_PERMISSION | 权限不足 |

## 6. 审计日志

> 所有管理操作（冻结用户、结算市场、审核提现/KYC、修改配置等）必须记录审计日志，不可篡改。
> 预测市场的结算决策争议性高，审计追踪是合规刚需。

### 6.1 数据模型

```rust
/// 审计日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i64,
    pub operator_id: i64,        // 操作人ID (管理员)
    pub operator_role: String,    // 操作人角色
    pub action: String,           // 操作类型: freeze_user, resolve_market, approve_withdraw, ...
    pub target_type: String,      // 目标类型: user, market, withdraw, kyc, config
    pub target_id: String,        // 目标ID
    pub detail: String,           // 操作详情 (JSON)
    pub ip_address: String,       // 操作IP
    pub user_agent: String,       // 操作UA
    pub created_at: i64,
}
```

### 6.2 数据库表

**audit_logs 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| operator_id | BIGINT | NOT NULL | 操作人ID |
| operator_role | VARCHAR(32) | NOT NULL | 操作人角色 |
| action | VARCHAR(64) | NOT NULL | 操作类型 |
| target_type | VARCHAR(32) | NOT NULL | 目标类型 |
| target_id | VARCHAR(64) | NOT NULL | 目标ID |
| detail | TEXT | | 操作详情 (JSON) |
| ip_address | VARCHAR(45) | | 操作IP |
| user_agent | VARCHAR(512) | | 操作UA |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `INDEX idx_operator_id (operator_id)`
- `INDEX idx_action (action)`
- `INDEX idx_target (target_type, target_id)`
- `INDEX idx_created_at (created_at)`

### 6.3 审计操作类型

| action | target_type | 说明 |
|--------|-------------|------|
| freeze_user | user | 冻结用户 |
| unfreeze_user | user | 解冻用户 |
| approve_kyc | kyc | 通过KYC |
| reject_kyc | kyc | 拒绝KYC |
| approve_withdraw | withdraw | 批准提现 |
| reject_withdraw | withdraw | 拒绝提现 |
| create_market | market | 创建市场 |
| update_market | market | 更新市场 |
| close_market | market | 关闭市场 |
| resolve_market | market | 结算市场 (必须记录reason) |
| set_fee_config | config | 修改手续费 |
| set_risk_config | config | 修改风控配置 |

## 6. 目录结构

```
crates/admin-service/
├── Cargo.toml
├── build.rs
├── config/
│   └── admin_service.yaml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── config.rs
    ├── server.rs
    ├── error.rs
    ├── pb.rs
    ├── pb/
    │   ├── admin.proto
    │   └── admin.rs
    ├── repository/
    │   ├── mod.rs
    │   ├── admin_repo.rs
    │   └── audit_log_repo.rs
    ├── services/
    │   ├── mod.rs
    │   ├── admin_service_impl.rs
    │   └── audit_service.rs
    └── clients/
        ├── mod.rs
        ├── user_service_client.rs
        ├── account_service_client.rs
        └── wallet_service_client.rs
```
