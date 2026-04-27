use prost::Message;

// =============================================================================
// Message Types
// =============================================================================

#[derive(Clone, Message, PartialEq)]
pub struct CreateMarketRequest {
    #[prost(string, tag = "1")]
    pub question: String,
    #[prost(string, tag = "2")]
    pub description: String,
    #[prost(string, tag = "3")]
    pub category: String,
    #[prost(string, tag = "4")]
    pub image_url: String,
    #[prost(message, repeated, tag = "5")]
    pub outcomes: Vec<CreateOutcomeRequest>,
    #[prost(int64, tag = "6")]
    pub start_time: i64,
    #[prost(int64, tag = "7")]
    pub end_time: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct CreateOutcomeRequest {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub description: String,
    #[prost(string, tag = "3")]
    pub image_url: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct CreateMarketResponse {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(message, optional, tag = "2")]
    pub market: Option<PredictionMarket>,
    #[prost(message, repeated, tag = "3")]
    pub outcomes: Vec<MarketOutcome>,
}

#[derive(Clone, Message, PartialEq)]
pub struct UpdateMarketRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(string, tag = "2")]
    pub question: String,
    #[prost(string, tag = "3")]
    pub description: String,
    #[prost(string, tag = "4")]
    pub image_url: String,
    #[prost(int64, tag = "5")]
    pub end_time: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct UpdateMarketResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(message, optional, tag = "2")]
    pub market: Option<PredictionMarket>,
}

#[derive(Clone, Message, PartialEq)]
pub struct CloseMarketRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct CloseMarketResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetMarketRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetMarketResponse {
    #[prost(message, optional, tag = "1")]
    pub market: Option<PredictionMarket>,
    #[prost(message, repeated, tag = "2")]
    pub outcomes: Vec<MarketOutcome>,
}

#[derive(Clone, Message, PartialEq)]
pub struct ListMarketsRequest {
    #[prost(string, tag = "1")]
    pub category: String,
    #[prost(string, tag = "2")]
    pub status: String,
    #[prost(int32, tag = "3")]
    pub page: i32,
    #[prost(int32, tag = "4")]
    pub page_size: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct ListMarketsResponse {
    #[prost(message, repeated, tag = "1")]
    pub markets: Vec<PredictionMarket>,
    #[prost(int64, tag = "2")]
    pub total: i64,
    #[prost(int32, tag = "3")]
    pub page: i32,
    #[prost(int32, tag = "4")]
    pub page_size: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct AddOutcomeRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, tag = "3")]
    pub description: String,
    #[prost(string, tag = "4")]
    pub image_url: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct AddOutcomeResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(message, optional, tag = "2")]
    pub outcome: Option<MarketOutcome>,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetOutcomesRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetOutcomesResponse {
    #[prost(message, repeated, tag = "1")]
    pub outcomes: Vec<MarketOutcome>,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetMarketPriceRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetMarketPriceResponse {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(message, repeated, tag = "2")]
    pub prices: Vec<OutcomePrice>,
}

#[derive(Clone, Message, PartialEq)]
pub struct OutcomePrice {
    #[prost(int64, tag = "1")]
    pub outcome_id: i64,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, tag = "3")]
    pub price: String,
    #[prost(string, tag = "4")]
    pub volume: String,
    #[prost(string, tag = "5")]
    pub probability: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetMarketDepthRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(int32, tag = "2")]
    pub depth: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetMarketDepthResponse {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(message, repeated, tag = "2")]
    pub bids: Vec<OrderBookLevel>,
    #[prost(message, repeated, tag = "3")]
    pub asks: Vec<OrderBookLevel>,
}

#[derive(Clone, Message, PartialEq)]
pub struct OrderBookLevel {
    #[prost(string, tag = "1")]
    pub price: String,
    #[prost(string, tag = "2")]
    pub quantity: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct ResolveMarketRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(int64, tag = "2")]
    pub outcome_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct ResolveMarketResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(message, optional, tag = "3")]
    pub resolution: Option<MarketResolution>,
}

#[derive(Clone, Message, PartialEq)]
pub struct CalculatePayoutRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct CalculatePayoutResponse {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(string, tag = "3")]
    pub total_payout: String,
    #[prost(message, repeated, tag = "4")]
    pub positions: Vec<UserPositionPayout>,
}

#[derive(Clone, Message, PartialEq)]
pub struct UserPositionPayout {
    #[prost(int64, tag = "1")]
    pub outcome_id: i64,
    #[prost(string, tag = "2")]
    pub outcome_name: String,
    #[prost(string, tag = "3")]
    pub quantity: String,
    #[prost(string, tag = "4")]
    pub avg_price: String,
    #[prost(string, tag = "5")]
    pub payout: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetUserPositionsRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(int64, tag = "2")]
    pub market_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetUserPositionsResponse {
    #[prost(message, repeated, tag = "1")]
    pub positions: Vec<UserPosition>,
}

#[derive(Clone, Message, PartialEq)]
pub struct PredictionMarket {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(string, tag = "2")]
    pub question: String,
    #[prost(string, tag = "3")]
    pub description: String,
    #[prost(string, tag = "4")]
    pub category: String,
    #[prost(string, tag = "5")]
    pub image_url: String,
    #[prost(int64, tag = "6")]
    pub start_time: i64,
    #[prost(int64, tag = "7")]
    pub end_time: i64,
    #[prost(string, tag = "8")]
    pub status: String,
    #[prost(int64, tag = "9")]
    pub resolved_outcome_id: i64,
    #[prost(int64, tag = "10")]
    pub resolved_at: i64,
    #[prost(string, tag = "11")]
    pub total_volume: String,
    #[prost(int64, tag = "12")]
    pub created_at: i64,
    #[prost(int64, tag = "13")]
    pub updated_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct MarketOutcome {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(int64, tag = "2")]
    pub market_id: i64,
    #[prost(string, tag = "3")]
    pub name: String,
    #[prost(string, tag = "4")]
    pub description: String,
    #[prost(string, tag = "5")]
    pub image_url: String,
    #[prost(string, tag = "6")]
    pub price: String,
    #[prost(string, tag = "7")]
    pub volume: String,
    #[prost(string, tag = "8")]
    pub probability: String,
    #[prost(int64, tag = "9")]
    pub created_at: i64,
    #[prost(int64, tag = "10")]
    pub updated_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct UserPosition {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(int64, tag = "3")]
    pub market_id: i64,
    #[prost(int64, tag = "4")]
    pub outcome_id: i64,
    #[prost(string, tag = "5")]
    pub quantity: String,
    #[prost(string, tag = "6")]
    pub avg_price: String,
    #[prost(int64, tag = "7")]
    pub created_at: i64,
    #[prost(int64, tag = "8")]
    pub updated_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct MarketResolution {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(int64, tag = "2")]
    pub market_id: i64,
    #[prost(int64, tag = "3")]
    pub outcome_id: i64,
    #[prost(string, tag = "4")]
    pub total_payout: String,
    #[prost(string, tag = "5")]
    pub winning_quantity: String,
    #[prost(string, tag = "6")]
    pub payout_ratio: String,
    #[prost(int64, tag = "7")]
    pub resolved_at: i64,
}

// =============================================================================
// gRPC Client Module
// =============================================================================

pub mod prediction_market_service_client {
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct PredictionMarketServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl PredictionMarketServiceClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }

    impl<T> PredictionMarketServiceClient<T>
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

        pub async fn create_market(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateMarketRequest>,
        ) -> std::result::Result<tonic::Response<super::CreateMarketResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/CreateMarket");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "CreateMarket"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn update_market(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateMarketRequest>,
        ) -> std::result::Result<tonic::Response<super::UpdateMarketResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/UpdateMarket");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "UpdateMarket"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn close_market(
            &mut self,
            request: impl tonic::IntoRequest<super::CloseMarketRequest>,
        ) -> std::result::Result<tonic::Response<super::CloseMarketResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/CloseMarket");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "CloseMarket"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_market(
            &mut self,
            request: impl tonic::IntoRequest<super::GetMarketRequest>,
        ) -> std::result::Result<tonic::Response<super::GetMarketResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/GetMarket");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "GetMarket"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn list_markets(
            &mut self,
            request: impl tonic::IntoRequest<super::ListMarketsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListMarketsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/ListMarkets");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "ListMarkets"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn add_outcome(
            &mut self,
            request: impl tonic::IntoRequest<super::AddOutcomeRequest>,
        ) -> std::result::Result<tonic::Response<super::AddOutcomeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/AddOutcome");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "AddOutcome"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_outcomes(
            &mut self,
            request: impl tonic::IntoRequest<super::GetOutcomesRequest>,
        ) -> std::result::Result<tonic::Response<super::GetOutcomesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/GetOutcomes");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "GetOutcomes"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_market_price(
            &mut self,
            request: impl tonic::IntoRequest<super::GetMarketPriceRequest>,
        ) -> std::result::Result<tonic::Response<super::GetMarketPriceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/GetMarketPrice");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "GetMarketPrice"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_market_depth(
            &mut self,
            request: impl tonic::IntoRequest<super::GetMarketDepthRequest>,
        ) -> std::result::Result<tonic::Response<super::GetMarketDepthResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/GetMarketDepth");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "GetMarketDepth"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn resolve_market(
            &mut self,
            request: impl tonic::IntoRequest<super::ResolveMarketRequest>,
        ) -> std::result::Result<tonic::Response<super::ResolveMarketResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/ResolveMarket");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "ResolveMarket"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn calculate_payout(
            &mut self,
            request: impl tonic::IntoRequest<super::CalculatePayoutRequest>,
        ) -> std::result::Result<tonic::Response<super::CalculatePayoutResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/CalculatePayout");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "CalculatePayout"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_user_positions(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUserPositionsRequest>,
        ) -> std::result::Result<tonic::Response<super::GetUserPositionsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/prediction_market.PredictionMarketService/GetUserPositions");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("prediction_market.PredictionMarketService", "GetUserPositions"));
            self.inner.unary(req, path, codec).await
        }
    }
}

pub async fn create_prediction_market_client(addr: &str) -> Result<prediction_market_service_client::PredictionMarketServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    prediction_market_service_client::PredictionMarketServiceClient::connect(addr.to_owned()).await
}

// ========== gRPC Server Trait ==========

pub mod prediction_market_service_server {
    use tonic::codegen::*;
    use async_trait::async_trait;

    #[async_trait]
    pub trait PredictionMarketService: std::marker::Send + std::marker::Sync + 'static {
        async fn create_market(&self, request: tonic::Request<super::CreateMarketRequest>) -> std::result::Result<tonic::Response<super::CreateMarketResponse>, tonic::Status>;
        async fn update_market(&self, request: tonic::Request<super::UpdateMarketRequest>) -> std::result::Result<tonic::Response<super::UpdateMarketResponse>, tonic::Status>;
        async fn close_market(&self, request: tonic::Request<super::CloseMarketRequest>) -> std::result::Result<tonic::Response<super::CloseMarketResponse>, tonic::Status>;
        async fn get_market(&self, request: tonic::Request<super::GetMarketRequest>) -> std::result::Result<tonic::Response<super::GetMarketResponse>, tonic::Status>;
        async fn list_markets(&self, request: tonic::Request<super::ListMarketsRequest>) -> std::result::Result<tonic::Response<super::ListMarketsResponse>, tonic::Status>;
        async fn add_outcome(&self, request: tonic::Request<super::AddOutcomeRequest>) -> std::result::Result<tonic::Response<super::AddOutcomeResponse>, tonic::Status>;
        async fn get_outcomes(&self, request: tonic::Request<super::GetOutcomesRequest>) -> std::result::Result<tonic::Response<super::GetOutcomesResponse>, tonic::Status>;
        async fn get_market_price(&self, request: tonic::Request<super::GetMarketPriceRequest>) -> std::result::Result<tonic::Response<super::GetMarketPriceResponse>, tonic::Status>;
        async fn get_market_depth(&self, request: tonic::Request<super::GetMarketDepthRequest>) -> std::result::Result<tonic::Response<super::GetMarketDepthResponse>, tonic::Status>;
        async fn resolve_market(&self, request: tonic::Request<super::ResolveMarketRequest>) -> std::result::Result<tonic::Response<super::ResolveMarketResponse>, tonic::Status>;
        async fn calculate_payout(&self, request: tonic::Request<super::CalculatePayoutRequest>) -> std::result::Result<tonic::Response<super::CalculatePayoutResponse>, tonic::Status>;
        async fn get_user_positions(&self, request: tonic::Request<super::GetUserPositionsRequest>) -> std::result::Result<tonic::Response<super::GetUserPositionsResponse>, tonic::Status>;
    }

    #[derive(Debug)]
    pub struct PredictionMarketServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }

    impl<T> PredictionMarketServiceServer<T> {
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

    impl<T, B> tonic::codegen::Service<http::Request<B>> for PredictionMarketServiceServer<T>
    where
        T: PredictionMarketService,
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

    impl<T> Clone for PredictionMarketServiceServer<T> {
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

    pub const SERVICE_NAME: &str = "prediction_market.PredictionMarketService";
    impl<T> tonic::server::NamedService for PredictionMarketServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
