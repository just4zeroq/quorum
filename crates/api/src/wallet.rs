use prost::Message;

// Wallet message types
#[derive(Clone, Message, PartialEq)]
pub struct DepositAddressSummary {
    #[prost(string, tag = "1")]
    pub address: String,
    #[prost(string, tag = "2")]
    pub chain: String,
    #[prost(int64, tag = "3")]
    pub created_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct DepositRecord {
    #[prost(string, tag = "1")]
    pub tx_id: String,
    #[prost(string, tag = "2")]
    pub chain: String,
    #[prost(string, tag = "3")]
    pub amount: String,
    #[prost(string, tag = "4")]
    pub address: String,
    #[prost(int64, tag = "5")]
    pub created_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct WithdrawRecordSummary {
    #[prost(string, tag = "1")]
    pub withdraw_id: String,
    #[prost(string, tag = "2")]
    pub asset: String,
    #[prost(string, tag = "3")]
    pub amount: String,
    #[prost(string, tag = "4")]
    pub fee: String,
    #[prost(string, tag = "5")]
    pub to_address: String,
    #[prost(string, tag = "6")]
    pub status: String,
    #[prost(string, tag = "7")]
    pub tx_id: String,
    #[prost(int64, tag = "8")]
    pub created_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct WhitelistAddressSummary {
    #[prost(string, tag = "1")]
    pub chain: String,
    #[prost(string, tag = "2")]
    pub address: String,
    #[prost(string, tag = "3")]
    pub label: String,
    #[prost(int64, tag = "4")]
    pub created_at: i64,
}

// Request/Response types
#[derive(Clone, Message, PartialEq)]
pub struct GetDepositAddressRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub chain: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetDepositAddressResponse {
    #[prost(string, tag = "1")]
    pub address: String,
    #[prost(string, tag = "2")]
    pub chain: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ListDepositAddressesRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct ListDepositAddressesResponse {
    #[prost(message, repeated, tag = "1")]
    pub addresses: Vec<DepositAddressSummary>,
}

#[derive(Clone, Message, PartialEq)]
pub struct ConfirmDepositRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub tx_id: String,
    #[prost(string, tag = "3")]
    pub chain: String,
    #[prost(string, tag = "4")]
    pub amount: String,
    #[prost(string, tag = "5")]
    pub address: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ConfirmDepositResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetDepositHistoryRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub chain: String,
    #[prost(int32, tag = "3")]
    pub page: i32,
    #[prost(int32, tag = "4")]
    pub page_size: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetDepositHistoryResponse {
    #[prost(message, repeated, tag = "1")]
    pub deposits: Vec<DepositRecord>,
    #[prost(int64, tag = "2")]
    pub total: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct CreateWithdrawRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub asset: String,
    #[prost(string, tag = "3")]
    pub amount: String,
    #[prost(string, tag = "4")]
    pub to_address: String,
    #[prost(string, tag = "5")]
    pub chain: String,
    #[prost(string, tag = "6")]
    pub payment_password: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct CreateWithdrawResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(string, tag = "3")]
    pub withdraw_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ConfirmWithdrawRequest {
    #[prost(string, tag = "1")]
    pub withdraw_id: String,
    #[prost(string, tag = "2")]
    pub signature: String,
    #[prost(string, tag = "3")]
    pub otp_code: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ConfirmWithdrawResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(string, tag = "3")]
    pub tx_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct CancelWithdrawRequest {
    #[prost(string, tag = "1")]
    pub withdraw_id: String,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct CancelWithdrawResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetWithdrawHistoryRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(int32, tag = "2")]
    pub page: i32,
    #[prost(int32, tag = "3")]
    pub page_size: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetWithdrawHistoryResponse {
    #[prost(message, repeated, tag = "1")]
    pub withdrawals: Vec<WithdrawRecordSummary>,
    #[prost(int64, tag = "2")]
    pub total: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetPendingWithdrawsRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetPendingWithdrawsResponse {
    #[prost(message, repeated, tag = "1")]
    pub withdrawals: Vec<WithdrawRecordSummary>,
}

#[derive(Clone, Message, PartialEq)]
pub struct AddWhitelistAddressRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub chain: String,
    #[prost(string, tag = "3")]
    pub address: String,
    #[prost(string, tag = "4")]
    pub label: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct AddWhitelistAddressResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct RemoveWhitelistAddressRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub address: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct RemoveWhitelistAddressResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ListWhitelistAddressesRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub chain: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ListWhitelistAddressesResponse {
    #[prost(message, repeated, tag = "1")]
    pub addresses: Vec<WhitelistAddressSummary>,
}

#[derive(Clone, Message, PartialEq)]
pub struct IsWhitelistedRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub address: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct IsWhitelistedResponse {
    #[prost(bool, tag = "1")]
    pub is_whitelisted: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct SetPaymentPasswordRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub password: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct SetPaymentPasswordResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct VerifyPaymentPasswordRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub password: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct VerifyPaymentPasswordResponse {
    #[prost(bool, tag = "1")]
    pub valid: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ResetPaymentPasswordRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub old_password: String,
    #[prost(string, tag = "3")]
    pub new_password: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ResetPaymentPasswordResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct HasPaymentPasswordRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct HasPaymentPasswordResponse {
    #[prost(bool, tag = "1")]
    pub has_password: bool,
}

// ========== gRPC Client ==========

pub mod wallet_service_client {
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct WalletServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl WalletServiceClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }

    impl<T> WalletServiceClient<T>
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

        pub async fn get_deposit_address(&mut self, request: impl tonic::IntoRequest<super::GetDepositAddressRequest>) -> std::result::Result<tonic::Response<super::GetDepositAddressResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/GetDepositAddress");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "GetDepositAddress"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn list_deposit_addresses(&mut self, request: impl tonic::IntoRequest<super::ListDepositAddressesRequest>) -> std::result::Result<tonic::Response<super::ListDepositAddressesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/ListDepositAddresses");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "ListDepositAddresses"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn confirm_deposit(&mut self, request: impl tonic::IntoRequest<super::ConfirmDepositRequest>) -> std::result::Result<tonic::Response<super::ConfirmDepositResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/ConfirmDeposit");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "ConfirmDeposit"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_deposit_history(&mut self, request: impl tonic::IntoRequest<super::GetDepositHistoryRequest>) -> std::result::Result<tonic::Response<super::GetDepositHistoryResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/GetDepositHistory");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "GetDepositHistory"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn create_withdraw(&mut self, request: impl tonic::IntoRequest<super::CreateWithdrawRequest>) -> std::result::Result<tonic::Response<super::CreateWithdrawResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/CreateWithdraw");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "CreateWithdraw"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn confirm_withdraw(&mut self, request: impl tonic::IntoRequest<super::ConfirmWithdrawRequest>) -> std::result::Result<tonic::Response<super::ConfirmWithdrawResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/ConfirmWithdraw");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "ConfirmWithdraw"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn cancel_withdraw(&mut self, request: impl tonic::IntoRequest<super::CancelWithdrawRequest>) -> std::result::Result<tonic::Response<super::CancelWithdrawResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/CancelWithdraw");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "CancelWithdraw"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_withdraw_history(&mut self, request: impl tonic::IntoRequest<super::GetWithdrawHistoryRequest>) -> std::result::Result<tonic::Response<super::GetWithdrawHistoryResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/GetWithdrawHistory");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "GetWithdrawHistory"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_pending_withdraws(&mut self, request: impl tonic::IntoRequest<super::GetPendingWithdrawsRequest>) -> std::result::Result<tonic::Response<super::GetPendingWithdrawsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/GetPendingWithdraws");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "GetPendingWithdraws"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn add_whitelist_address(&mut self, request: impl tonic::IntoRequest<super::AddWhitelistAddressRequest>) -> std::result::Result<tonic::Response<super::AddWhitelistAddressResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/AddWhitelistAddress");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "AddWhitelistAddress"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn remove_whitelist_address(&mut self, request: impl tonic::IntoRequest<super::RemoveWhitelistAddressRequest>) -> std::result::Result<tonic::Response<super::RemoveWhitelistAddressResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/RemoveWhitelistAddress");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "RemoveWhitelistAddress"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn list_whitelist_addresses(&mut self, request: impl tonic::IntoRequest<super::ListWhitelistAddressesRequest>) -> std::result::Result<tonic::Response<super::ListWhitelistAddressesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/ListWhitelistAddresses");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "ListWhitelistAddresses"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn is_whitelisted(&mut self, request: impl tonic::IntoRequest<super::IsWhitelistedRequest>) -> std::result::Result<tonic::Response<super::IsWhitelistedResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/IsWhitelisted");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "IsWhitelisted"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn set_payment_password(&mut self, request: impl tonic::IntoRequest<super::SetPaymentPasswordRequest>) -> std::result::Result<tonic::Response<super::SetPaymentPasswordResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/SetPaymentPassword");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "SetPaymentPassword"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn verify_payment_password(&mut self, request: impl tonic::IntoRequest<super::VerifyPaymentPasswordRequest>) -> std::result::Result<tonic::Response<super::VerifyPaymentPasswordResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/VerifyPaymentPassword");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "VerifyPaymentPassword"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn reset_payment_password(&mut self, request: impl tonic::IntoRequest<super::ResetPaymentPasswordRequest>) -> std::result::Result<tonic::Response<super::ResetPaymentPasswordResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/ResetPaymentPassword");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "ResetPaymentPassword"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn has_payment_password(&mut self, request: impl tonic::IntoRequest<super::HasPaymentPasswordRequest>) -> std::result::Result<tonic::Response<super::HasPaymentPasswordResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/WalletService/HasPaymentPassword");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("WalletService", "HasPaymentPassword"));
            self.inner.unary(req, path, codec).await
        }
    }
}

pub async fn create_wallet_client(addr: &str) -> Result<wallet_service_client::WalletServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    wallet_service_client::WalletServiceClient::connect(addr.to_owned()).await
}

// ========== gRPC Server Trait ==========

pub mod wallet_service_server {
    use tonic::codegen::*;
    use async_trait::async_trait;

    #[async_trait]
    pub trait WalletService: std::marker::Send + std::marker::Sync + 'static {
        async fn get_deposit_address(&self, request: tonic::Request<super::GetDepositAddressRequest>) -> std::result::Result<tonic::Response<super::GetDepositAddressResponse>, tonic::Status>;
        async fn list_deposit_addresses(&self, request: tonic::Request<super::ListDepositAddressesRequest>) -> std::result::Result<tonic::Response<super::ListDepositAddressesResponse>, tonic::Status>;
        async fn confirm_deposit(&self, request: tonic::Request<super::ConfirmDepositRequest>) -> std::result::Result<tonic::Response<super::ConfirmDepositResponse>, tonic::Status>;
        async fn get_deposit_history(&self, request: tonic::Request<super::GetDepositHistoryRequest>) -> std::result::Result<tonic::Response<super::GetDepositHistoryResponse>, tonic::Status>;
        async fn create_withdraw(&self, request: tonic::Request<super::CreateWithdrawRequest>) -> std::result::Result<tonic::Response<super::CreateWithdrawResponse>, tonic::Status>;
        async fn confirm_withdraw(&self, request: tonic::Request<super::ConfirmWithdrawRequest>) -> std::result::Result<tonic::Response<super::ConfirmWithdrawResponse>, tonic::Status>;
        async fn cancel_withdraw(&self, request: tonic::Request<super::CancelWithdrawRequest>) -> std::result::Result<tonic::Response<super::CancelWithdrawResponse>, tonic::Status>;
        async fn get_withdraw_history(&self, request: tonic::Request<super::GetWithdrawHistoryRequest>) -> std::result::Result<tonic::Response<super::GetWithdrawHistoryResponse>, tonic::Status>;
        async fn get_pending_withdraws(&self, request: tonic::Request<super::GetPendingWithdrawsRequest>) -> std::result::Result<tonic::Response<super::GetPendingWithdrawsResponse>, tonic::Status>;
        async fn add_whitelist_address(&self, request: tonic::Request<super::AddWhitelistAddressRequest>) -> std::result::Result<tonic::Response<super::AddWhitelistAddressResponse>, tonic::Status>;
        async fn remove_whitelist_address(&self, request: tonic::Request<super::RemoveWhitelistAddressRequest>) -> std::result::Result<tonic::Response<super::RemoveWhitelistAddressResponse>, tonic::Status>;
        async fn list_whitelist_addresses(&self, request: tonic::Request<super::ListWhitelistAddressesRequest>) -> std::result::Result<tonic::Response<super::ListWhitelistAddressesResponse>, tonic::Status>;
        async fn is_whitelisted(&self, request: tonic::Request<super::IsWhitelistedRequest>) -> std::result::Result<tonic::Response<super::IsWhitelistedResponse>, tonic::Status>;
        async fn set_payment_password(&self, request: tonic::Request<super::SetPaymentPasswordRequest>) -> std::result::Result<tonic::Response<super::SetPaymentPasswordResponse>, tonic::Status>;
        async fn verify_payment_password(&self, request: tonic::Request<super::VerifyPaymentPasswordRequest>) -> std::result::Result<tonic::Response<super::VerifyPaymentPasswordResponse>, tonic::Status>;
        async fn reset_payment_password(&self, request: tonic::Request<super::ResetPaymentPasswordRequest>) -> std::result::Result<tonic::Response<super::ResetPaymentPasswordResponse>, tonic::Status>;
        async fn has_payment_password(&self, request: tonic::Request<super::HasPaymentPasswordRequest>) -> std::result::Result<tonic::Response<super::HasPaymentPasswordResponse>, tonic::Status>;
    }

    #[derive(Debug)]
    pub struct WalletServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }

    impl<T> WalletServiceServer<T> {
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

    impl<T, B> tonic::codegen::Service<http::Request<B>> for WalletServiceServer<T>
    where
        T: WalletService,
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
            let path = req.uri().path().to_string();
            Box::pin(async move {
                let mut response = http::Response::new(empty_body());
                let headers = response.headers_mut();
                headers.insert(tonic::Status::GRPC_STATUS, (tonic::Code::Unimplemented as i32).into());
                headers.insert(http::header::CONTENT_TYPE, tonic::metadata::GRPC_CONTENT_TYPE);
                Ok(response)
            })
        }
    }

    impl<T> Clone for WalletServiceServer<T> {
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

    pub const SERVICE_NAME: &str = "wallet.WalletService";
    impl<T> tonic::server::NamedService for WalletServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
