// Auth service types - manually defined (no proto)
use prost::Message;

// ========== Auth Requests/Responses ==========

#[derive(Clone, Message, PartialEq)]
pub struct LoginRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub password: String,
    #[prost(string, tag = "3")]
    pub device_id: String,
    #[prost(string, tag = "4")]
    pub ip_address: String,
    #[prost(string, tag = "5")]
    pub user_agent: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct LoginResponse {
    #[prost(string, tag = "1")]
    pub access_token: String,
    #[prost(string, tag = "2")]
    pub refresh_token: String,
    #[prost(int64, tag = "3")]
    pub expires_in: i64,
    #[prost(string, tag = "4")]
    pub token_type: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct LogoutRequest {
    #[prost(string, tag = "1")]
    pub session_id: String,
}

#[derive(Clone, Copy, Message, PartialEq)]
pub struct LogoutResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct ValidateTokenRequest {
    #[prost(string, tag = "1")]
    pub token: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ValidateTokenResponse {
    #[prost(bool, tag = "1")]
    pub valid: bool,
    #[prost(string, tag = "2")]
    pub user_id: String,
    #[prost(string, tag = "3")]
    pub session_id: String,
    #[prost(string, repeated, tag = "4")]
    pub permissions: Vec<String>,
}

#[derive(Clone, Message, PartialEq)]
pub struct RefreshTokenRequest {
    #[prost(string, tag = "1")]
    pub refresh_token: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct RefreshTokenResponse {
    #[prost(string, tag = "1")]
    pub access_token: String,
    #[prost(int64, tag = "2")]
    pub expires_in: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct ValidateApiKeyRequest {
    #[prost(string, tag = "1")]
    pub api_key_id: String,
    #[prost(string, tag = "2")]
    pub api_key_secret: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ValidateApiKeyResponse {
    #[prost(bool, tag = "1")]
    pub valid: bool,
    #[prost(string, tag = "2")]
    pub user_id: String,
    #[prost(string, repeated, tag = "3")]
    pub permissions: Vec<String>,
}

#[derive(Clone, Message, PartialEq)]
pub struct CreateApiKeyRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, repeated, tag = "3")]
    pub permissions: Vec<String>,
}

#[derive(Clone, Message, PartialEq)]
pub struct CreateApiKeyResponse {
    #[prost(string, tag = "1")]
    pub api_key_id: String,
    #[prost(string, tag = "2")]
    pub api_key_secret: String,
    #[prost(string, tag = "3")]
    pub created_at: String,
}

// ========== gRPC Client ==========

pub mod auth_service_client {
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct AuthServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl AuthServiceClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }

    impl<T> AuthServiceClient<T>
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

        pub async fn login(
            &mut self,
            request: impl tonic::IntoRequest<super::LoginRequest>,
        ) -> std::result::Result<tonic::Response<super::LoginResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/auth.AuthService/Login");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("auth.AuthService", "Login"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn logout(
            &mut self,
            request: impl tonic::IntoRequest<super::LogoutRequest>,
        ) -> std::result::Result<tonic::Response<super::LogoutResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/auth.AuthService/Logout");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("auth.AuthService", "Logout"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn validate_token(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidateTokenRequest>,
        ) -> std::result::Result<tonic::Response<super::ValidateTokenResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/auth.AuthService/ValidateToken");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("auth.AuthService", "ValidateToken"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn refresh_token(
            &mut self,
            request: impl tonic::IntoRequest<super::RefreshTokenRequest>,
        ) -> std::result::Result<tonic::Response<super::RefreshTokenResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/auth.AuthService/RefreshToken");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("auth.AuthService", "RefreshToken"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn validate_api_key(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidateApiKeyRequest>,
        ) -> std::result::Result<tonic::Response<super::ValidateApiKeyResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/auth.AuthService/ValidateApiKey");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("auth.AuthService", "ValidateApiKey"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn create_api_key(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateApiKeyRequest>,
        ) -> std::result::Result<tonic::Response<super::CreateApiKeyResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/auth.AuthService/CreateApiKey");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("auth.AuthService", "CreateApiKey"));
            self.inner.unary(req, path, codec).await
        }
    }
}

// ========== Client Helper ==========

pub async fn create_auth_client(addr: &str) -> Result<auth_service_client::AuthServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    auth_service_client::AuthServiceClient::connect(addr.to_owned()).await
}

// ========== gRPC Server Trait ==========

pub mod auth_service_server {
    use tonic::codegen::*;
    use async_trait::async_trait;

    #[async_trait]
    pub trait AuthService: std::marker::Send + std::marker::Sync + 'static {
        async fn login(&self, request: tonic::Request<super::LoginRequest>) -> std::result::Result<tonic::Response<super::LoginResponse>, tonic::Status>;
        async fn logout(&self, request: tonic::Request<super::LogoutRequest>) -> std::result::Result<tonic::Response<super::LogoutResponse>, tonic::Status>;
        async fn validate_token(&self, request: tonic::Request<super::ValidateTokenRequest>) -> std::result::Result<tonic::Response<super::ValidateTokenResponse>, tonic::Status>;
        async fn refresh_token(&self, request: tonic::Request<super::RefreshTokenRequest>) -> std::result::Result<tonic::Response<super::RefreshTokenResponse>, tonic::Status>;
        async fn validate_api_key(&self, request: tonic::Request<super::ValidateApiKeyRequest>) -> std::result::Result<tonic::Response<super::ValidateApiKeyResponse>, tonic::Status>;
        async fn create_api_key(&self, request: tonic::Request<super::CreateApiKeyRequest>) -> std::result::Result<tonic::Response<super::CreateApiKeyResponse>, tonic::Status>;
    }

    #[derive(Debug)]
    pub struct AuthServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }

    impl<T> AuthServiceServer<T> {
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

    impl<T, B> tonic::codegen::Service<http::Request<B>> for AuthServiceServer<T>
    where
        T: AuthService,
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

    impl<T> Clone for AuthServiceServer<T> {
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

    pub const SERVICE_NAME: &str = "auth.AuthService";
    impl<T> tonic::server::NamedService for AuthServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
