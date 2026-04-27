use prost::Message;

// User message types
#[derive(Clone, Message, PartialEq)]
pub struct User {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub username: String,
    #[prost(string, tag = "3")]
    pub email: String,
    #[prost(string, tag = "4")]
    pub phone: String,
    #[prost(string, tag = "5")]
    pub status: String,
    #[prost(string, tag = "6")]
    pub kyc_status: String,
    #[prost(bool, tag = "7")]
    pub two_factor_enabled: bool,
    #[prost(int64, tag = "8")]
    pub created_at: i64,
    #[prost(int64, tag = "9")]
    pub updated_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct Notifications {
    #[prost(bool, tag = "1")]
    pub email: bool,
    #[prost(bool, tag = "2")]
    pub sms: bool,
    #[prost(bool, tag = "3")]
    pub push: bool,
    #[prost(bool, tag = "4")]
    pub order_trade: bool,
    #[prost(bool, tag = "5")]
    pub price_alert: bool,
    #[prost(bool, tag = "6")]
    pub deposit_withdraw: bool,
    #[prost(bool, tag = "7")]
    pub system: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct TradingPreferences {
    #[prost(bool, tag = "1")]
    pub confirm_order: bool,
    #[prost(bool, tag = "2")]
    pub confirm_cancel: bool,
    #[prost(string, tag = "3")]
    pub default_order_type: String,
    #[prost(string, tag = "4")]
    pub default_time_in_force: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct UserSettings {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub language: String,
    #[prost(string, tag = "3")]
    pub theme: String,
    #[prost(string, tag = "4")]
    pub timezone: String,
    #[prost(bool, tag = "5")]
    pub show_balance: bool,
    #[prost(bool, tag = "6")]
    pub show_pnl: bool,
    #[prost(bool, tag = "7")]
    pub compact_view: bool,
    #[prost(message, tag = "8")]
    pub notifications: Option<Notifications>,
    #[prost(message, tag = "9")]
    pub trading_preferences: Option<TradingPreferences>,
}

#[derive(Clone, Message, PartialEq)]
pub struct UserRisk {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(int32, tag = "2")]
    pub risk_level: i32,
    #[prost(int32, tag = "3")]
    pub kyc_level: i32,
    #[prost(bool, tag = "4")]
    pub frozen: bool,
    #[prost(string, tag = "5")]
    pub frozen_reason: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct UserTag {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub description: String,
    #[prost(string, tag = "3")]
    pub created_at: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct WalletAddress {
    #[prost(string, tag = "1")]
    pub address: String,
    #[prost(string, tag = "2")]
    pub wallet_type: String,
    #[prost(string, tag = "3")]
    pub chain_type: String,
    #[prost(string, tag = "4")]
    pub tag: String,
    #[prost(bool, tag = "5")]
    pub is_default: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct SessionInfo {
    #[prost(string, tag = "1")]
    pub session_id: String,
    #[prost(string, tag = "2")]
    pub device_id: String,
    #[prost(string, tag = "3")]
    pub device_type: String,
    #[prost(string, tag = "4")]
    pub ip_address: String,
    #[prost(string, tag = "5")]
    pub user_agent: String,
    #[prost(string, tag = "6")]
    pub location: String,
    #[prost(int64, tag = "7")]
    pub created_at: i64,
    #[prost(int64, tag = "8")]
    pub last_active_at: i64,
    #[prost(bool, tag = "9")]
    pub current: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct AccountSummary {
    #[prost(bool, tag = "1")]
    pub spot_enabled: bool,
    #[prost(bool, tag = "2")]
    pub futures_enabled: bool,
    #[prost(message, repeated, tag = "3")]
    pub deposit_addresses: Vec<WalletAddress>,
}

// Request/Response types
#[derive(Clone, Message, PartialEq)]
pub struct RegisterRequest {
    #[prost(string, tag = "1")]
    pub username: String,
    #[prost(string, tag = "2")]
    pub email: String,
    #[prost(string, tag = "3")]
    pub password: String,
    #[prost(string, tag = "4")]
    pub invite_code: String,
    #[prost(string, tag = "5")]
    pub ip_address: String,
    #[prost(string, tag = "6")]
    pub user_agent: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct RegisterResponse {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub token: String,
    #[prost(message, tag = "3")]
    pub user: Option<User>,
}

#[derive(Clone, Message, PartialEq)]
pub struct LoginRequest {
    #[prost(string, tag = "1")]
    pub email: String,
    #[prost(string, tag = "2")]
    pub password: String,
    #[prost(string, tag = "3")]
    pub code_2fa: String,
    #[prost(string, tag = "4")]
    pub ip_address: String,
    #[prost(string, tag = "5")]
    pub user_agent: String,
    #[prost(string, tag = "6")]
    pub device_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct LoginResponse {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub token: String,
    #[prost(string, tag = "3")]
    pub refresh_token: String,
    #[prost(int64, tag = "4")]
    pub expires_at: i64,
    #[prost(bool, tag = "5")]
    pub need_2fa: bool,
    #[prost(message, tag = "6")]
    pub user: Option<User>,
    #[prost(message, tag = "7")]
    pub accounts: Option<AccountSummary>,
}

#[derive(Clone, Message, PartialEq)]
pub struct LogoutRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub token: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct LogoutResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct RefreshTokenRequest {
    #[prost(string, tag = "1")]
    pub refresh_token: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct RefreshTokenResponse {
    #[prost(string, tag = "1")]
    pub token: String,
    #[prost(string, tag = "2")]
    pub refresh_token: String,
    #[prost(int64, tag = "3")]
    pub expires_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetWalletNonceRequest {
    #[prost(string, tag = "1")]
    pub wallet_address: String,
    #[prost(string, tag = "2")]
    pub wallet_type: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetWalletNonceResponse {
    #[prost(string, tag = "1")]
    pub nonce: String,
    #[prost(int64, tag = "2")]
    pub expires_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct WalletLoginRequest {
    #[prost(string, tag = "1")]
    pub wallet_address: String,
    #[prost(string, tag = "2")]
    pub wallet_type: String,
    #[prost(string, tag = "3")]
    pub signature: String,
    #[prost(string, tag = "4")]
    pub ip_address: String,
    #[prost(string, tag = "5")]
    pub user_agent: String,
    #[prost(string, tag = "6")]
    pub device_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct WalletLoginResponse {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub token: String,
    #[prost(string, tag = "3")]
    pub refresh_token: String,
    #[prost(int64, tag = "4")]
    pub expires_at: i64,
    #[prost(bool, tag = "5")]
    pub is_new_user: bool,
    #[prost(message, tag = "6")]
    pub user: Option<User>,
    #[prost(bool, tag = "7")]
    pub need_bind_email: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct WalletBindRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub email: String,
    #[prost(string, tag = "3")]
    pub password: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct WalletBindResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetUserRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetUserResponse {
    #[prost(message, tag = "1")]
    pub user: Option<User>,
}

#[derive(Clone, Message, PartialEq)]
pub struct UpdateUserRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub username: String,
    #[prost(string, tag = "3")]
    pub email: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct UpdateUserResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(message, tag = "2")]
    pub user: Option<User>,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetUserProfileRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetUserProfileResponse {
    #[prost(message, tag = "1")]
    pub user: Option<User>,
    #[prost(message, tag = "2")]
    pub settings: Option<UserSettings>,
    #[prost(message, tag = "3")]
    pub risk: Option<UserRisk>,
    #[prost(message, repeated, tag = "4")]
    pub tags: Vec<UserTag>,
}

#[derive(Clone, Message, PartialEq)]
pub struct ChangePasswordRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub old_password: String,
    #[prost(string, tag = "3")]
    pub new_password: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ChangePasswordResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct Enable2FARequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub password: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct Enable2FAResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub secret: String,
    #[prost(string, tag = "3")]
    pub qr_code: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct Disable2FARequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub code_2fa: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct Disable2FAResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct Verify2FARequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub code_2fa: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct Verify2FAResponse {
    #[prost(bool, tag = "1")]
    pub valid: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct SubmitKYCRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub real_name: String,
    #[prost(string, tag = "3")]
    pub id_number: String,
    #[prost(string, tag = "4")]
    pub id_type: String,
    #[prost(string, tag = "5")]
    pub front_image: String,
    #[prost(string, tag = "6")]
    pub back_image: String,
    #[prost(string, tag = "7")]
    pub selfie_image: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct SubmitKYCResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetKYCStatusRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetKYCStatusResponse {
    #[prost(string, tag = "1")]
    pub status: String,
    #[prost(string, tag = "2")]
    pub reason: String,
    #[prost(int32, tag = "3")]
    pub level: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct ListSessionsRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(int32, tag = "2")]
    pub page: i32,
    #[prost(int32, tag = "3")]
    pub page_size: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct ListSessionsResponse {
    #[prost(message, repeated, tag = "1")]
    pub sessions: Vec<SessionInfo>,
    #[prost(int32, tag = "2")]
    pub total: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct KillSessionRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub session_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct KillSessionResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct KillAllSessionsRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub except_session_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct KillAllSessionsResponse {
    #[prost(int32, tag = "1")]
    pub killed_count: i32,
}

// ========== gRPC Client ==========

pub mod user_service_client {
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct UserServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl UserServiceClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }

    impl<T> UserServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }

        pub async fn register(&mut self, request: impl tonic::IntoRequest<super::RegisterRequest>) -> std::result::Result<tonic::Response<super::RegisterResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/Register");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "Register"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn login(&mut self, request: impl tonic::IntoRequest<super::LoginRequest>) -> std::result::Result<tonic::Response<super::LoginResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/Login");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "Login"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn logout(&mut self, request: impl tonic::IntoRequest<super::LogoutRequest>) -> std::result::Result<tonic::Response<super::LogoutResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/Logout");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "Logout"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn refresh_token(&mut self, request: impl tonic::IntoRequest<super::RefreshTokenRequest>) -> std::result::Result<tonic::Response<super::RefreshTokenResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/RefreshToken");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "RefreshToken"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn wallet_login(&mut self, request: impl tonic::IntoRequest<super::WalletLoginRequest>) -> std::result::Result<tonic::Response<super::WalletLoginResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/WalletLogin");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "WalletLogin"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn wallet_bind(&mut self, request: impl tonic::IntoRequest<super::WalletBindRequest>) -> std::result::Result<tonic::Response<super::WalletBindResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/WalletBind");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "WalletBind"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_wallet_nonce(&mut self, request: impl tonic::IntoRequest<super::GetWalletNonceRequest>) -> std::result::Result<tonic::Response<super::GetWalletNonceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/GetWalletNonce");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "GetWalletNonce"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_user(&mut self, request: impl tonic::IntoRequest<super::GetUserRequest>) -> std::result::Result<tonic::Response<super::GetUserResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/GetUser");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "GetUser"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn update_user(&mut self, request: impl tonic::IntoRequest<super::UpdateUserRequest>) -> std::result::Result<tonic::Response<super::UpdateUserResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/UpdateUser");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "UpdateUser"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_user_profile(&mut self, request: impl tonic::IntoRequest<super::GetUserProfileRequest>) -> std::result::Result<tonic::Response<super::GetUserProfileResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/GetUserProfile");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "GetUserProfile"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn change_password(&mut self, request: impl tonic::IntoRequest<super::ChangePasswordRequest>) -> std::result::Result<tonic::Response<super::ChangePasswordResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/ChangePassword");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "ChangePassword"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn enable_2fa(&mut self, request: impl tonic::IntoRequest<super::Enable2FARequest>) -> std::result::Result<tonic::Response<super::Enable2FAResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/Enable2FA");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "Enable2FA"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn disable_2fa(&mut self, request: impl tonic::IntoRequest<super::Disable2FARequest>) -> std::result::Result<tonic::Response<super::Disable2FAResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/Disable2FA");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "Disable2FA"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn verify_2fa(&mut self, request: impl tonic::IntoRequest<super::Verify2FARequest>) -> std::result::Result<tonic::Response<super::Verify2FAResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/Verify2FA");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "Verify2FA"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn submit_kyc(&mut self, request: impl tonic::IntoRequest<super::SubmitKYCRequest>) -> std::result::Result<tonic::Response<super::SubmitKYCResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/SubmitKYC");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "SubmitKYC"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_kyc_status(&mut self, request: impl tonic::IntoRequest<super::GetKYCStatusRequest>) -> std::result::Result<tonic::Response<super::GetKYCStatusResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/GetKYCStatus");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "GetKYCStatus"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn list_sessions(&mut self, request: impl tonic::IntoRequest<super::ListSessionsRequest>) -> std::result::Result<tonic::Response<super::ListSessionsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/ListSessions");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "ListSessions"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn kill_session(&mut self, request: impl tonic::IntoRequest<super::KillSessionRequest>) -> std::result::Result<tonic::Response<super::KillSessionResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/KillSession");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "KillSession"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn kill_all_sessions(&mut self, request: impl tonic::IntoRequest<super::KillAllSessionsRequest>) -> std::result::Result<tonic::Response<super::KillAllSessionsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/UserService/KillAllSessions");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("UserService", "KillAllSessions"));
            self.inner.unary(req, path, codec).await
        }
    }
}

pub async fn create_user_client(addr: &str) -> Result<user_service_client::UserServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    user_service_client::UserServiceClient::connect(addr.to_owned()).await
}

// ========== gRPC Server Trait ==========

pub mod user_service_server {
    use tonic::codegen::*;
    use async_trait::async_trait;

    #[async_trait]
    pub trait UserService: std::marker::Send + std::marker::Sync + 'static {
        async fn register(&self, request: tonic::Request<super::RegisterRequest>) -> std::result::Result<tonic::Response<super::RegisterResponse>, tonic::Status>;
        async fn login(&self, request: tonic::Request<super::LoginRequest>) -> std::result::Result<tonic::Response<super::LoginResponse>, tonic::Status>;
        async fn logout(&self, request: tonic::Request<super::LogoutRequest>) -> std::result::Result<tonic::Response<super::LogoutResponse>, tonic::Status>;
        async fn refresh_token(&self, request: tonic::Request<super::RefreshTokenRequest>) -> std::result::Result<tonic::Response<super::RefreshTokenResponse>, tonic::Status>;
        async fn wallet_login(&self, request: tonic::Request<super::WalletLoginRequest>) -> std::result::Result<tonic::Response<super::WalletLoginResponse>, tonic::Status>;
        async fn wallet_bind(&self, request: tonic::Request<super::WalletBindRequest>) -> std::result::Result<tonic::Response<super::WalletBindResponse>, tonic::Status>;
        async fn get_wallet_nonce(&self, request: tonic::Request<super::GetWalletNonceRequest>) -> std::result::Result<tonic::Response<super::GetWalletNonceResponse>, tonic::Status>;
        async fn get_user(&self, request: tonic::Request<super::GetUserRequest>) -> std::result::Result<tonic::Response<super::GetUserResponse>, tonic::Status>;
        async fn update_user(&self, request: tonic::Request<super::UpdateUserRequest>) -> std::result::Result<tonic::Response<super::UpdateUserResponse>, tonic::Status>;
        async fn get_user_profile(&self, request: tonic::Request<super::GetUserProfileRequest>) -> std::result::Result<tonic::Response<super::GetUserProfileResponse>, tonic::Status>;
        async fn change_password(&self, request: tonic::Request<super::ChangePasswordRequest>) -> std::result::Result<tonic::Response<super::ChangePasswordResponse>, tonic::Status>;
        async fn enable_2fa(&self, request: tonic::Request<super::Enable2FARequest>) -> std::result::Result<tonic::Response<super::Enable2FAResponse>, tonic::Status>;
        async fn disable_2fa(&self, request: tonic::Request<super::Disable2FARequest>) -> std::result::Result<tonic::Response<super::Disable2FAResponse>, tonic::Status>;
        async fn verify_2fa(&self, request: tonic::Request<super::Verify2FARequest>) -> std::result::Result<tonic::Response<super::Verify2FAResponse>, tonic::Status>;
        async fn submit_kyc(&self, request: tonic::Request<super::SubmitKYCRequest>) -> std::result::Result<tonic::Response<super::SubmitKYCResponse>, tonic::Status>;
        async fn get_kyc_status(&self, request: tonic::Request<super::GetKYCStatusRequest>) -> std::result::Result<tonic::Response<super::GetKYCStatusResponse>, tonic::Status>;
        async fn list_sessions(&self, request: tonic::Request<super::ListSessionsRequest>) -> std::result::Result<tonic::Response<super::ListSessionsResponse>, tonic::Status>;
        async fn kill_session(&self, request: tonic::Request<super::KillSessionRequest>) -> std::result::Result<tonic::Response<super::KillSessionResponse>, tonic::Status>;
        async fn kill_all_sessions(&self, request: tonic::Request<super::KillAllSessionsRequest>) -> std::result::Result<tonic::Response<super::KillAllSessionsResponse>, tonic::Status>;
    }

    #[derive(Debug)]
    pub struct UserServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }

    impl<T> UserServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }

        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }

        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }

        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }

        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }

        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }

    impl<T, B> tonic::codegen::Service<http::Request<B>> for UserServiceServer<T>
    where
        T: UserService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let _path = req.uri().path().to_string();
            Box::pin(async move {
                let mut response = http::Response::new(empty_body());
                let headers = response.headers_mut();
                headers.insert(tonic::Status::GRPC_STATUS, (tonic::Code::Unimplemented as i32).into());
                headers.insert(http::header::CONTENT_TYPE, tonic::metadata::GRPC_CONTENT_TYPE);
                Ok(response)
            })
        }
    }

    impl<T> Clone for UserServiceServer<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }

    pub const SERVICE_NAME: &str = "user.UserService";
    impl<T> tonic::server::NamedService for UserServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
