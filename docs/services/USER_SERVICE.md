# User Service - 用户服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50001 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | 无 |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 用户注册 | 邮箱/用户名注册 |
| 用户登录 | 邮箱密码登录、会话管理 |
| 钱包登录 | Ethereum 钱包签名登录 (EIP-191) |
| 2FA | TOTP 二次验证 |
| KYC | 身份认证状态管理 |
| 会话管理 | JWT Token 签发与验证 |

### 1.1.2 功能列表

```
User Service
├── 认证
│   ├── Register - 邮箱注册
│   ├── Login - 邮箱登录
│   ├── Logout - 登出
│   └── RefreshToken - 刷新Token
├── 钱包登录
│   ├── WalletLogin - 钱包登录
│   └── WalletBind - 绑定钱包
├── 用户信息
│   ├── GetUser - 获取用户信息
│   └── UpdateUser - 更新用户信息
├── 安全
│   ├── Enable2FA - 启用2FA
│   ├── Disable2FA - 禁用2FA
│   ├── Verify2FA - 验证2FA
│   └── ChangePassword - 修改密码
├── KYC
│   ├── SubmitKYC - 提交KYC
│   └── GetKYCStatus - 查询KYC状态
└── 内部服务
    ├── ValidateToken - 验证Token
    └── GetUserById - 通过ID查询用户
```

## 2. 数据模型

### 2.1 Domain Model (domain/user)

已在 `crates/domain/src/user/model/mod.rs` 定义：

```rust
/// 用户状态
pub enum UserStatus {
    Active,
    Frozen,
    Closed,
}

/// KYC 状态
pub enum KycStatus {
    None,
    Submitting,
    Pending,
    Verified,
    Rejected,
}

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub phone: Option<String>,
    pub password_hash: Option<String>,
    pub kyc_status: KycStatus,
    pub kyc_level: i32,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
    pub status: UserStatus,
    pub status_reason: Option<String>,
    pub frozen_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_login_at: Option<i64>,
}

/// 用户会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
    pub created_at: i64,
}

/// 钱包地址
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddress {
    pub id: i64,
    pub user_id: i64,
    pub wallet_address: String,
    pub wallet_type: String,
    pub chain_type: String,
    pub is_primary: bool,
    pub created_at: i64,
}
```

### 2.2 数据库表

**users 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | 用户ID |
| username | VARCHAR(50) | UNIQUE, NOT NULL | 用户名 |
| email | VARCHAR(255) | UNIQUE, NOT NULL | 邮箱 |
| phone | VARCHAR(20) | UNIQUE | 手机号 |
| password_hash | VARCHAR(255) | | 密码哈希 |
| kyc_status | VARCHAR(20) | NOT NULL DEFAULT 'none' | KYC状态 |
| kyc_level | INT | NOT NULL DEFAULT 0 | KYC等级 |
| kyc_submitted_at | BIGINT | | KYC提交时间 |
| kyc_verified_at | BIGINT | | KYC认证时间 |
| two_factor_enabled | BOOLEAN | NOT NULL DEFAULT FALSE | 2FA启用 |
| two_factor_secret | VARCHAR(255) | | 2FA密钥 |
| status | VARCHAR(20) | NOT NULL DEFAULT 'active' | 账户状态 |
| status_reason | VARCHAR(255) | | 状态原因 |
| frozen_at | BIGINT | | 冻结时间 |
| created_at | BIGINT | NOT NULL | 创建时间 |
| updated_at | BIGINT | NOT NULL | 更新时间 |
| last_login_at | BIGINT | | 最后登录时间 |

**索引**:
- `PRIMARY KEY (id)`
- `UNIQUE INDEX idx_email (email)`
- `UNIQUE INDEX idx_username (username)`
- `INDEX idx_status (status)`

**user_sessions 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | 会话ID |
| user_id | BIGINT | NOT NULL, FK | 用户ID |
| token | TEXT | UNIQUE, NOT NULL | JWT Token |
| refresh_token | TEXT | | 刷新Token |
| token_type | VARCHAR(20) | NOT NULL | Token类型 |
| expires_at | BIGINT | NOT NULL | 过期时间 |
| ip_address | VARCHAR(45) | | IP地址 |
| user_agent | TEXT | | User-Agent |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `INDEX idx_user_id (user_id)`
- `UNIQUE INDEX idx_token (token)`
- `INDEX idx_expires_at (expires_at)`

**wallet_addresses 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| user_id | BIGINT | NOT NULL, FK | 用户ID |
| wallet_address | VARCHAR(100) | NOT NULL | 钱包地址 |
| wallet_type | VARCHAR(20) | NOT NULL | 钱包类型 |
| chain_type | VARCHAR(20) | NOT NULL | 链类型 |
| is_primary | BOOLEAN | NOT NULL DEFAULT FALSE | 是否主地址 |
| created_at | BIGINT | NOT NULL | 创建时间 |

**索引**:
- `UNIQUE INDEX idx_user_wallet (user_id, wallet_address)`
- `INDEX idx_wallet_address (wallet_address)`

## 3. Proto 接口定义

### 3.1 服务定义

```protobuf
syntax = "proto3";

package user;

service UserService {
    // 认证
    rpc Register(RegisterRequest) returns (RegisterResponse);
    rpc Login(LoginRequest) returns (LoginResponse);
    rpc Logout(LogoutRequest) returns (LogoutResponse);
    rpc RefreshToken(RefreshTokenRequest) returns (RefreshTokenResponse);

    // 钱包登录
    rpc WalletLogin(WalletLoginRequest) returns (WalletLoginResponse);
    rpc WalletBind(WalletBindRequest) returns (WalletBindResponse);

    // 用户信息
    rpc GetUser(GetUserRequest) returns (GetUserResponse);
    rpc UpdateUser(UpdateUserRequest) returns (UpdateUserResponse);

    // 安全
    rpc Enable2FA(Enable2FARequest) returns (Enable2FAResponse);
    rpc Disable2FA(Disable2FARequest) returns (Disable2FAResponse);
    rpc Verify2FA(Verify2FARequest) returns (Verify2FAResponse);
    rpc ChangePassword(ChangePasswordRequest) returns (ChangePasswordResponse);

    // KYC
    rpc SubmitKYC(SubmitKYCRequest) returns (SubmitKYCResponse);
    rpc GetKYCStatus(GetKYCStatusRequest) returns (GetKYCStatusResponse);

    // 内部服务
    rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);
    rpc GetUserById(GetUserByIdRequest) returns (GetUserByIdResponse);
}
```

### 3.2 消息定义

```protobuf
// ==================== 认证 ====================

message RegisterRequest {
    string username = 1;
    string email = 2;
    string password = 3;
}

message RegisterResponse {
    bool success = 1;
    string message = 2;
    int64 user_id = 3;
}

message LoginRequest {
    string email = 1;
    string password = 2;
    string code_2fa = 3;
    string ip_address = 4;
    string user_agent = 5;
}

message LoginResponse {
    bool success = 1;
    string message = 2;
    int64 user_id = 3;
    string token = 4;
    string refresh_token = 5;
    int64 expires_at = 6;
    bool need_2fa = 7;
}

message LogoutRequest {
    string token = 1;
}

message LogoutResponse {
    bool success = 1;
    string message = 2;
}

message RefreshTokenRequest {
    string refresh_token = 1;
}

message RefreshTokenResponse {
    bool success = 1;
    string token = 2;
    string refresh_token = 3;
    int64 expires_at = 4;
}

// ==================== 钱包登录 ====================

message WalletLoginRequest {
    string wallet_address = 1;
    string signature = 2;
    string message = 3;
    string ip_address = 4;
}

message WalletLoginResponse {
    bool success = 1;
    string message = 2;
    int64 user_id = 3;
    bool is_new_user = 4;
    string token = 5;
}

message WalletBindRequest {
    int64 user_id = 1;
    string wallet_address = 2;
    string signature = 3;
    string message = 4;
}

message WalletBindResponse {
    bool success = 1;
    string message = 2;
}

// ==================== 用户信息 ====================

message GetUserRequest {
    int64 user_id = 1;
}

message GetUserResponse {
    int64 id = 1;
    string username = 2;
    string email = 3;
    string phone = 4;
    string kyc_status = 5;
    int32 kyc_level = 6;
    bool two_factor_enabled = 7;
    string status = 8;
    int64 created_at = 9;
    int64 updated_at = 10;
}

message UpdateUserRequest {
    int64 user_id = 1;
    string username = 2;
    string phone = 3;
}

message UpdateUserResponse {
    bool success = 1;
    string message = 2;
    int64 user_id = 3;
}

// ==================== 安全 ====================

message Enable2FARequest {
    int64 user_id = 1;
}

message Enable2FAResponse {
    bool success = 1;
    string message = 2;
    string secret = 3;
    string qr_code_url = 4;
}

message Disable2FARequest {
    int64 user_id = 1;
    string code_2fa = 2;
}

message Disable2FAResponse {
    bool success = 1;
    string message = 2;
}

message Verify2FARequest {
    int64 user_id = 1;
    string code = 2;
}

message Verify2FAResponse {
    bool success = 1;
    string message = 2;
}

message ChangePasswordRequest {
    int64 user_id = 1;
    string old_password = 2;
    string new_password = 3;
}

message ChangePasswordResponse {
    bool success = 1;
    string message = 2;
}

// ==================== KYC ====================

message SubmitKYCRequest {
    int64 user_id = 1;
    string real_name = 2;
    string id_number = 3;
}

message SubmitKYCResponse {
    bool success = 1;
    string message = 2;
}

message GetKYCStatusRequest {
    int64 user_id = 1;
}

message GetKYCStatusResponse {
    string status = 1;
    int32 level = 2;
    string rejection_reason = 3;
}

// ==================== 内部服务 ====================

message ValidateTokenRequest {
    string token = 1;
}

message ValidateTokenResponse {
    bool valid = 1;
    int64 user_id = 2;
    string token_type = 3;
}

message GetUserByIdRequest {
    int64 user_id = 1;
}

message GetUserByIdResponse {
    int64 id = 1;
    string username = 2;
    string email = 3;
    string status = 4;
}
```

## 4. Kafka 事件

### 4.1 生产的事件

| 事件 | Topic | 说明 |
|------|-------|------|
| UserRegistered | user_events | 用户注册 |
| UserLoggedIn | user_events | 用户登录 |
| UserLoggedOut | user_events | 用户登出 |
| UserKYCSubmitted | user_events | KYC 提交 |
| UserKYCVerified | user_events | KYC 认证通过 |
| UserFrozen | user_events | 用户冻结 |
| UserUnfrozen | user_events | 用户解冻 |

### 4.2 事件 Schema

已在 `crates/domain/src/user/event/mod.rs` 定义：

```rust
pub enum UserEvent {
    Registered {
        user_id: i64,
        email: String,
        username: String,
    },
    Login {
        user_id: i64,
        method: String,  // "password" | "wallet"
    },
    Logout {
        user_id: i64,
    },
    Frozen {
        user_id: i64,
        reason: Option<String>,
    },
    Unfrozen {
        user_id: i64,
    },
}
```

## 5. 服务间通信

### 5.1 gRPC 调用 (被调用)

| 调用方 | 接口 | 说明 |
|--------|------|------|
| API Gateway | ValidateToken | 验证用户 Token |
| Order Service | GetUserById | 获取用户信息 |
| Wallet Service | GetUserById | 关联用户 |

### 5.2 Kafka 消费

无（User Service 不消费任何事件）

## 6. 配置

```yaml
# crates/user-service/config/user_service.yaml
service:
  host: "0.0.0.0"
  port: 50001

database:
  driver: "sqlite"  # or "postgres"
  url: "sqlite:./data/users.db"
  max_connections: 10

jwt:
  secret: "${JWT_SECRET}"
  expiry_seconds: 86400      # 24 hours
  refresh_expiry_days: 30

password:
  min_length: 8
  require_uppercase: true
  require_lowercase: true
  require_number: true

two_factor:
  issuer: "PredictionMarket"

rate_limiter:
  enabled: true
  requests_per_minute: 100
```

## 7. 错误码

| 错误码 | 说明 |
|--------|------|
| USER_NOT_FOUND | 用户不存在 |
| USER_ALREADY_EXISTS | 用户已存在 |
| INVALID_PASSWORD | 密码错误 |
| WEAK_PASSWORD | 密码强度不足 |
| INVALID_TOKEN | Token无效 |
| TOKEN_EXPIRED | Token过期 |
| 2FA_REQUIRED | 需要2FA验证 |
| 2FA_INVALID | 2FA验证码错误 |
| USER_FROZEN | 账户已冻结 |
| USER_CLOSED | 账户已关闭 |
| KYC_ALREADY_SUBMITTED | KYC已提交 |
| KYC_REJECTED | KYC已拒绝 |
| INVALID_SIGNATURE | 签名无效 |
| WALLET_ALREADY_BOUND | 钱包已绑定 |

## 8. 目录结构

```
crates/user-service/
├── Cargo.toml
├── build.rs
├── config/
│   └── user_service.yaml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── config.rs
    ├── server.rs
    ├── error.rs
    ├── pb.rs
    ├── pb/
    │   ├── user.proto
    │   └── user.rs
    ├── repository/
    │   ├── mod.rs
    │   ├── user_repo.rs
    │   ├── session_repo.rs
    │   └── wallet_repo.rs
    ├── services/
    │   ├── mod.rs
    │   └── user_service_impl.rs
    └── utils/
        ├── mod.rs
        ├── password.rs
        └── jwt.rs
```
