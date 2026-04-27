use prost::Message;

// =============================================================================
// Message Types
// =============================================================================

/// Account
#[derive(Clone, Message, PartialEq)]
pub struct GetBalanceRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub asset: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetBalanceResponse {
    #[prost(string, tag = "1")]
    pub account_id: String,
    #[prost(string, tag = "2")]
    pub asset: String,
    #[prost(string, tag = "3")]
    pub available: String,
    #[prost(string, tag = "4")]
    pub frozen: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct CreditRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub asset: String,
    #[prost(string, tag = "3")]
    pub amount: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct CreditResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub available_after: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct DebitRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub asset: String,
    #[prost(string, tag = "3")]
    pub amount: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct DebitResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub available_after: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct FreezeRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub asset: String,
    #[prost(string, tag = "3")]
    pub amount: String,
    #[prost(string, tag = "4")]
    pub order_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct FreezeResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub available_after: String,
    #[prost(string, tag = "3")]
    pub frozen_after: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct UnfreezeRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub asset: String,
    #[prost(string, tag = "3")]
    pub amount: String,
    #[prost(string, tag = "4")]
    pub order_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct UnfreezeResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}

/// Position
#[derive(Clone, Message, PartialEq)]
pub struct GetPositionsRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(uint64, tag = "2")]
    pub market_id: u64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetPositionsResponse {
    #[prost(message, repeated, tag = "1")]
    pub positions: Vec<Position>,
}

#[derive(Clone, Message, PartialEq)]
pub struct Position {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(uint64, tag = "2")]
    pub market_id: u64,
    #[prost(uint64, tag = "3")]
    pub outcome_id: u64,
    #[prost(string, tag = "4")]
    pub side: String,
    #[prost(string, tag = "5")]
    pub size: String,
    #[prost(string, tag = "6")]
    pub entry_price: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetPositionRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(uint64, tag = "2")]
    pub market_id: u64,
    #[prost(uint64, tag = "3")]
    pub outcome_id: u64,
    #[prost(string, tag = "4")]
    pub side: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetPositionResponse {
    #[prost(message, optional, tag = "1")]
    pub position: Option<Position>,
}

/// Clearing
#[derive(Clone, Message, PartialEq)]
pub struct SettleTradeRequest {
    #[prost(string, tag = "1")]
    pub trade_id: String,
    #[prost(uint64, tag = "2")]
    pub market_id: u64,
    #[prost(string, tag = "3")]
    pub buyer_id: String,
    #[prost(string, tag = "4")]
    pub seller_id: String,
    #[prost(string, tag = "5")]
    pub price: String,
    #[prost(string, tag = "6")]
    pub size: String,
    #[prost(string, tag = "7")]
    pub taker_fee_rate: String,
    #[prost(string, tag = "8")]
    pub maker_fee_rate: String,
    #[prost(uint64, tag = "9")]
    pub outcome_id: u64,
}

#[derive(Clone, Message, PartialEq)]
pub struct SettleTradeResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub settlement_id: String,
}

/// Market Settlement (payout distribution for resolved markets)
#[derive(Clone, Message, PartialEq)]
pub struct SettleMarketRequest {
    #[prost(uint64, tag = "1")]
    pub market_id: u64,
    #[prost(uint64, tag = "2")]
    pub winning_outcome_id: u64,
    #[prost(message, repeated, tag = "3")]
    pub payouts: Vec<UserPayout>,
}

#[derive(Clone, Message, PartialEq)]
pub struct UserPayout {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub quantity: String,
    #[prost(string, tag = "3")]
    pub payout_amount: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct SettleMarketResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(int32, tag = "2")]
    pub users_credited: i32,
}

/// Ledger
#[derive(Clone, Message, PartialEq)]
pub struct GetLedgerRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(string, tag = "2")]
    pub account_id: String,
    #[prost(int32, tag = "3")]
    pub limit: i32,
    #[prost(int32, tag = "4")]
    pub offset: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetLedgerResponse {
    #[prost(message, repeated, tag = "1")]
    pub entries: Vec<LedgerEntry>,
}

#[derive(Clone, Message, PartialEq)]
pub struct LedgerEntry {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub ledger_type: String,
    #[prost(string, tag = "3")]
    pub asset: String,
    #[prost(string, tag = "4")]
    pub amount: String,
    #[prost(string, tag = "5")]
    pub balance_after: String,
    #[prost(string, tag = "6")]
    pub reference_id: String,
    #[prost(string, tag = "7")]
    pub created_at: String,
}

// =============================================================================
// gRPC Client Module
// =============================================================================

pub mod portfolio_service_client {
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct PortfolioServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl PortfolioServiceClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }

    impl<T> PortfolioServiceClient<T>
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

        pub async fn get_balance(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBalanceRequest>,
        ) -> std::result::Result<tonic::Response<super::GetBalanceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/portfolio.PortfolioService/GetBalance");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("portfolio.PortfolioService", "GetBalance"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn credit(
            &mut self,
            request: impl tonic::IntoRequest<super::CreditRequest>,
        ) -> std::result::Result<tonic::Response<super::CreditResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/portfolio.PortfolioService/Credit");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("portfolio.PortfolioService", "Credit"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn debit(
            &mut self,
            request: impl tonic::IntoRequest<super::DebitRequest>,
        ) -> std::result::Result<tonic::Response<super::DebitResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/portfolio.PortfolioService/Debit");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("portfolio.PortfolioService", "Debit"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn freeze(
            &mut self,
            request: impl tonic::IntoRequest<super::FreezeRequest>,
        ) -> std::result::Result<tonic::Response<super::FreezeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/portfolio.PortfolioService/Freeze");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("portfolio.PortfolioService", "Freeze"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn unfreeze(
            &mut self,
            request: impl tonic::IntoRequest<super::UnfreezeRequest>,
        ) -> std::result::Result<tonic::Response<super::UnfreezeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/portfolio.PortfolioService/Unfreeze");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("portfolio.PortfolioService", "Unfreeze"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_positions(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPositionsRequest>,
        ) -> std::result::Result<tonic::Response<super::GetPositionsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/portfolio.PortfolioService/GetPositions");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("portfolio.PortfolioService", "GetPositions"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_position(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPositionRequest>,
        ) -> std::result::Result<tonic::Response<super::GetPositionResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/portfolio.PortfolioService/GetPosition");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("portfolio.PortfolioService", "GetPosition"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn settle_trade(
            &mut self,
            request: impl tonic::IntoRequest<super::SettleTradeRequest>,
        ) -> std::result::Result<tonic::Response<super::SettleTradeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/portfolio.PortfolioService/SettleTrade");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("portfolio.PortfolioService", "SettleTrade"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn settle_market(
            &mut self,
            request: impl tonic::IntoRequest<super::SettleMarketRequest>,
        ) -> std::result::Result<tonic::Response<super::SettleMarketResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/portfolio.PortfolioService/SettleMarket");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("portfolio.PortfolioService", "SettleMarket"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_ledger(
            &mut self,
            request: impl tonic::IntoRequest<super::GetLedgerRequest>,
        ) -> std::result::Result<tonic::Response<super::GetLedgerResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/portfolio.PortfolioService/GetLedger");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("portfolio.PortfolioService", "GetLedger"));
            self.inner.unary(req, path, codec).await
        }
    }
}

pub async fn create_portfolio_client(addr: &str) -> Result<portfolio_service_client::PortfolioServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    portfolio_service_client::PortfolioServiceClient::connect(addr.to_owned()).await
}

// ========== gRPC Server Trait ==========

pub mod portfolio_service_server {
    use tonic::codegen::*;
    use async_trait::async_trait;

    #[async_trait]
    pub trait PortfolioService: std::marker::Send + std::marker::Sync + 'static {
        async fn get_balance(&self, request: tonic::Request<super::GetBalanceRequest>) -> std::result::Result<tonic::Response<super::GetBalanceResponse>, tonic::Status>;
        async fn credit(&self, request: tonic::Request<super::CreditRequest>) -> std::result::Result<tonic::Response<super::CreditResponse>, tonic::Status>;
        async fn debit(&self, request: tonic::Request<super::DebitRequest>) -> std::result::Result<tonic::Response<super::DebitResponse>, tonic::Status>;
        async fn freeze(&self, request: tonic::Request<super::FreezeRequest>) -> std::result::Result<tonic::Response<super::FreezeResponse>, tonic::Status>;
        async fn unfreeze(&self, request: tonic::Request<super::UnfreezeRequest>) -> std::result::Result<tonic::Response<super::UnfreezeResponse>, tonic::Status>;
        async fn get_positions(&self, request: tonic::Request<super::GetPositionsRequest>) -> std::result::Result<tonic::Response<super::GetPositionsResponse>, tonic::Status>;
        async fn get_position(&self, request: tonic::Request<super::GetPositionRequest>) -> std::result::Result<tonic::Response<super::GetPositionResponse>, tonic::Status>;
        async fn settle_trade(&self, request: tonic::Request<super::SettleTradeRequest>) -> std::result::Result<tonic::Response<super::SettleTradeResponse>, tonic::Status>;
        async fn settle_market(&self, request: tonic::Request<super::SettleMarketRequest>) -> std::result::Result<tonic::Response<super::SettleMarketResponse>, tonic::Status>;
        async fn get_ledger(&self, request: tonic::Request<super::GetLedgerRequest>) -> std::result::Result<tonic::Response<super::GetLedgerResponse>, tonic::Status>;
    }

    #[derive(Debug)]
    pub struct PortfolioServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }

    impl<T> PortfolioServiceServer<T> {
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

    impl<T, B> tonic::codegen::Service<http::Request<B>> for PortfolioServiceServer<T>
    where
        T: PortfolioService,
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

    impl<T> Clone for PortfolioServiceServer<T> {
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

    pub const SERVICE_NAME: &str = "portfolio.PortfolioService";
    impl<T> tonic::server::NamedService for PortfolioServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
