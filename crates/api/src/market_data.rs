use prost::Message;

// =============================================================================
// Message Types
// =============================================================================

#[derive(Clone, Message, PartialEq)]
pub struct GetMarketsRequest {
    #[prost(string, tag = "1")]
    pub category: String,
    #[prost(string, tag = "2")]
    pub status: String,
    #[prost(string, tag = "3")]
    pub sort_by: String,
    #[prost(bool, tag = "4")]
    pub descending: bool,
    #[prost(int32, tag = "5")]
    pub page: i32,
    #[prost(int32, tag = "6")]
    pub page_size: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetMarketsResponse {
    #[prost(message, repeated, tag = "1")]
    pub markets: Vec<MarketSummary>,
    #[prost(int64, tag = "2")]
    pub total: i64,
    #[prost(int32, tag = "3")]
    pub page: i32,
    #[prost(int32, tag = "4")]
    pub page_size: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct MarketSummary {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(string, tag = "2")]
    pub question: String,
    #[prost(string, tag = "3")]
    pub category: String,
    #[prost(int64, tag = "4")]
    pub end_time: i64,
    #[prost(string, tag = "5")]
    pub status: String,
    #[prost(string, tag = "6")]
    pub total_volume: String,
    #[prost(message, repeated, tag = "7")]
    pub outcomes: Vec<OutcomeSummary>,
    #[prost(int64, tag = "8")]
    pub created_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct OutcomeSummary {
    #[prost(int64, tag = "1")]
    pub id: i64,
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
pub struct GetMarketDetailRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetMarketDetailResponse {
    #[prost(message, optional, tag = "1")]
    pub market: Option<MarketDetail>,
    #[prost(message, repeated, tag = "2")]
    pub outcomes: Vec<OutcomeDetail>,
}

#[derive(Clone, Message, PartialEq)]
pub struct MarketDetail {
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
    #[prost(string, tag = "9")]
    pub total_volume: String,
    #[prost(int64, tag = "10")]
    pub resolved_outcome_id: i64,
    #[prost(int64, tag = "11")]
    pub resolved_at: i64,
    #[prost(int64, tag = "12")]
    pub created_at: i64,
    #[prost(int64, tag = "13")]
    pub updated_at: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct OutcomeDetail {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, tag = "3")]
    pub description: String,
    #[prost(string, tag = "4")]
    pub image_url: String,
    #[prost(string, tag = "5")]
    pub price: String,
    #[prost(string, tag = "6")]
    pub volume: String,
    #[prost(string, tag = "7")]
    pub probability: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetOutcomePricesRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(int64, repeated, tag = "2")]
    pub outcome_ids: Vec<i64>,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetOutcomePricesResponse {
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
    #[prost(string, tag = "6")]
    pub change_24h: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetOrderBookRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(int32, tag = "2")]
    pub depth: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetOrderBookResponse {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(message, repeated, tag = "2")]
    pub bids: Vec<OrderBookLevel>,
    #[prost(message, repeated, tag = "3")]
    pub asks: Vec<OrderBookLevel>,
    #[prost(int64, tag = "4")]
    pub timestamp: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct OrderBookLevel {
    #[prost(string, tag = "1")]
    pub price: String,
    #[prost(string, tag = "2")]
    pub quantity: String,
    #[prost(string, tag = "3")]
    pub orders: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetKlinesRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(string, tag = "2")]
    pub interval: String,
    #[prost(int64, tag = "3")]
    pub start_time: i64,
    #[prost(int64, tag = "4")]
    pub end_time: i64,
    #[prost(int32, tag = "5")]
    pub limit: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetKlinesResponse {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(string, tag = "2")]
    pub interval: String,
    #[prost(message, repeated, tag = "3")]
    pub klines: Vec<KlineData>,
}

#[derive(Clone, Message, PartialEq)]
pub struct KlineData {
    #[prost(int64, tag = "1")]
    pub timestamp: i64,
    #[prost(string, tag = "2")]
    pub open: String,
    #[prost(string, tag = "3")]
    pub high: String,
    #[prost(string, tag = "4")]
    pub low: String,
    #[prost(string, tag = "5")]
    pub close: String,
    #[prost(string, tag = "6")]
    pub volume: String,
    #[prost(string, tag = "7")]
    pub quote_volume: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetLatestKlineRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(string, tag = "2")]
    pub interval: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetLatestKlineResponse {
    #[prost(message, optional, tag = "1")]
    pub kline: Option<KlineData>,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetRecentTradesRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(int64, tag = "2")]
    pub outcome_id: i64,
    #[prost(int32, tag = "3")]
    pub limit: i32,
    #[prost(int64, tag = "4")]
    pub from_trade_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetRecentTradesResponse {
    #[prost(message, repeated, tag = "1")]
    pub trades: Vec<TradeData>,
    #[prost(int64, tag = "2")]
    pub next_from_trade_id: i64,
    #[prost(bool, tag = "3")]
    pub has_more: bool,
}

#[derive(Clone, Message, PartialEq)]
pub struct TradeData {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(int64, tag = "2")]
    pub market_id: i64,
    #[prost(int64, tag = "3")]
    pub outcome_id: i64,
    #[prost(int64, tag = "4")]
    pub user_id: i64,
    #[prost(string, tag = "5")]
    pub side: String,
    #[prost(string, tag = "6")]
    pub price: String,
    #[prost(string, tag = "7")]
    pub quantity: String,
    #[prost(string, tag = "8")]
    pub amount: String,
    #[prost(string, tag = "9")]
    pub fee: String,
    #[prost(int64, tag = "10")]
    pub timestamp: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct Get24hStatsRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct Get24hStatsResponse {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    #[prost(string, tag = "2")]
    pub volume_24h: String,
    #[prost(string, tag = "3")]
    pub amount_24h: String,
    #[prost(string, tag = "4")]
    pub high_24h: String,
    #[prost(string, tag = "5")]
    pub low_24h: String,
    #[prost(string, tag = "6")]
    pub price_change: String,
    #[prost(string, tag = "7")]
    pub price_change_percent: String,
    #[prost(int64, tag = "8")]
    pub trade_count_24h: i64,
    #[prost(int64, tag = "9")]
    pub timestamp: i64,
}

// =============================================================================
// gRPC Client Module
// =============================================================================

pub mod market_data_service_client {
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct MarketDataServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl MarketDataServiceClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }

    impl<T> MarketDataServiceClient<T>
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

        pub async fn get_markets(
            &mut self,
            request: impl tonic::IntoRequest<super::GetMarketsRequest>,
        ) -> std::result::Result<tonic::Response<super::GetMarketsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/market_data.MarketDataService/GetMarkets");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("market_data.MarketDataService", "GetMarkets"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_market_detail(
            &mut self,
            request: impl tonic::IntoRequest<super::GetMarketDetailRequest>,
        ) -> std::result::Result<tonic::Response<super::GetMarketDetailResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/market_data.MarketDataService/GetMarketDetail");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("market_data.MarketDataService", "GetMarketDetail"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_outcome_prices(
            &mut self,
            request: impl tonic::IntoRequest<super::GetOutcomePricesRequest>,
        ) -> std::result::Result<tonic::Response<super::GetOutcomePricesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/market_data.MarketDataService/GetOutcomePrices");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("market_data.MarketDataService", "GetOutcomePrices"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_order_book(
            &mut self,
            request: impl tonic::IntoRequest<super::GetOrderBookRequest>,
        ) -> std::result::Result<tonic::Response<super::GetOrderBookResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/market_data.MarketDataService/GetOrderBook");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("market_data.MarketDataService", "GetOrderBook"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_klines(
            &mut self,
            request: impl tonic::IntoRequest<super::GetKlinesRequest>,
        ) -> std::result::Result<tonic::Response<super::GetKlinesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/market_data.MarketDataService/GetKlines");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("market_data.MarketDataService", "GetKlines"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_latest_kline(
            &mut self,
            request: impl tonic::IntoRequest<super::GetLatestKlineRequest>,
        ) -> std::result::Result<tonic::Response<super::GetLatestKlineResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/market_data.MarketDataService/GetLatestKline");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("market_data.MarketDataService", "GetLatestKline"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_recent_trades(
            &mut self,
            request: impl tonic::IntoRequest<super::GetRecentTradesRequest>,
        ) -> std::result::Result<tonic::Response<super::GetRecentTradesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/market_data.MarketDataService/GetRecentTrades");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("market_data.MarketDataService", "GetRecentTrades"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_24h_stats(
            &mut self,
            request: impl tonic::IntoRequest<super::Get24hStatsRequest>,
        ) -> std::result::Result<tonic::Response<super::Get24hStatsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/market_data.MarketDataService/Get24hStats");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("market_data.MarketDataService", "Get24hStats"));
            self.inner.unary(req, path, codec).await
        }
    }
}

pub async fn create_market_data_client(addr: &str) -> Result<market_data_service_client::MarketDataServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    market_data_service_client::MarketDataServiceClient::connect(addr.to_owned()).await
}

// ========== gRPC Server Trait ==========

pub mod market_data_service_server {
    use tonic::codegen::*;
    use async_trait::async_trait;

    #[async_trait]
    pub trait MarketDataService: std::marker::Send + std::marker::Sync + 'static {
        async fn get_markets(&self, request: tonic::Request<super::GetMarketsRequest>) -> std::result::Result<tonic::Response<super::GetMarketsResponse>, tonic::Status>;
        async fn get_market_detail(&self, request: tonic::Request<super::GetMarketDetailRequest>) -> std::result::Result<tonic::Response<super::GetMarketDetailResponse>, tonic::Status>;
        async fn get_outcome_prices(&self, request: tonic::Request<super::GetOutcomePricesRequest>) -> std::result::Result<tonic::Response<super::GetOutcomePricesResponse>, tonic::Status>;
        async fn get_order_book(&self, request: tonic::Request<super::GetOrderBookRequest>) -> std::result::Result<tonic::Response<super::GetOrderBookResponse>, tonic::Status>;
        async fn get_klines(&self, request: tonic::Request<super::GetKlinesRequest>) -> std::result::Result<tonic::Response<super::GetKlinesResponse>, tonic::Status>;
        async fn get_latest_kline(&self, request: tonic::Request<super::GetLatestKlineRequest>) -> std::result::Result<tonic::Response<super::GetLatestKlineResponse>, tonic::Status>;
        async fn get_recent_trades(&self, request: tonic::Request<super::GetRecentTradesRequest>) -> std::result::Result<tonic::Response<super::GetRecentTradesResponse>, tonic::Status>;
        async fn get_24h_stats(&self, request: tonic::Request<super::Get24hStatsRequest>) -> std::result::Result<tonic::Response<super::Get24hStatsResponse>, tonic::Status>;
    }

    #[derive(Debug)]
    pub struct MarketDataServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }

    impl<T> MarketDataServiceServer<T> {
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

    impl<T, B> tonic::codegen::Service<http::Request<B>> for MarketDataServiceServer<T>
    where
        T: MarketDataService,
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

    impl<T> Clone for MarketDataServiceServer<T> {
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

    pub const SERVICE_NAME: &str = "market_data.MarketDataService";
    impl<T> tonic::server::NamedService for MarketDataServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
