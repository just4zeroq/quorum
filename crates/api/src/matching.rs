// Matching engine types - manually defined (no proto)
use prost::Message;

#[derive(Clone, Message, PartialEq)]
pub struct OrderCommandRequest {
    #[prost(bytes, tag = "1")]
    pub command_data: Vec<u8>,
}

#[derive(Clone, Message, PartialEq)]
pub struct OrderResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(bytes, tag = "3")]
    pub result_data: Vec<u8>,
}

#[derive(Clone, Message, PartialEq)]
pub struct CancelRequest {
    #[prost(bytes, tag = "1")]
    pub cancel_data: Vec<u8>,
}

#[derive(Clone, Message, PartialEq)]
pub struct CancelResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct OrderBookRequest {
    #[prost(uint64, tag = "1")]
    pub market_id: u64,
    #[prost(string, tag = "2")]
    pub outcome_asset: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct OrderBookResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(bytes, tag = "3")]
    pub orderbook_data: Vec<u8>,
}

// ========== gRPC Client ==========

pub mod matching_service_client {
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct MatchingServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl MatchingServiceClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }

    impl<T> MatchingServiceClient<T>
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

        pub async fn submit_order(
            &mut self,
            request: impl tonic::IntoRequest<super::OrderCommandRequest>,
        ) -> std::result::Result<tonic::Response<super::OrderResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/matching.MatchingService/SubmitOrder");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("matching.MatchingService", "SubmitOrder"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn cancel_order(
            &mut self,
            request: impl tonic::IntoRequest<super::CancelRequest>,
        ) -> std::result::Result<tonic::Response<super::CancelResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/matching.MatchingService/CancelOrder");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("matching.MatchingService", "CancelOrder"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_order_book(
            &mut self,
            request: impl tonic::IntoRequest<super::OrderBookRequest>,
        ) -> std::result::Result<tonic::Response<super::OrderBookResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/matching.MatchingService/GetOrderBook");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("matching.MatchingService", "GetOrderBook"));
            self.inner.unary(req, path, codec).await
        }
    }
}

// ========== gRPC Server Trait ==========

pub mod matching_service_server {
    use tonic::codegen::*;
    use async_trait::async_trait;

    #[async_trait]
    pub trait MatchingService: std::marker::Send + std::marker::Sync + 'static {
        async fn submit_order(
            &self,
            request: tonic::Request<super::OrderCommandRequest>,
        ) -> std::result::Result<tonic::Response<super::OrderResponse>, tonic::Status>;

        async fn cancel_order(
            &self,
            request: tonic::Request<super::CancelRequest>,
        ) -> std::result::Result<tonic::Response<super::CancelResponse>, tonic::Status>;

        async fn get_order_book(
            &self,
            request: tonic::Request<super::OrderBookRequest>,
        ) -> std::result::Result<tonic::Response<super::OrderBookResponse>, tonic::Status>;
    }

    #[derive(Debug)]
    pub struct MatchingServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }

    impl<T> MatchingServiceServer<T> {
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

        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
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

    impl<T, B> tonic::codegen::Service<http::Request<B>> for MatchingServiceServer<T>
    where
        T: MatchingService,
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
            match req.uri().path() {
                "/matching.MatchingService/SubmitOrder" => {
                    struct SubmitOrderSvc<T: MatchingService>(pub Arc<T>);
                    impl<T: MatchingService> tonic::server::UnaryService<super::OrderCommandRequest>
                        for SubmitOrderSvc<T> {
                        type Response = super::OrderResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::OrderCommandRequest>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { <T as MatchingService>::submit_order(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let method = SubmitOrderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        Ok(tonic::server::Grpc::new(codec).unary(method, req).await)
                    })
                }
                "/matching.MatchingService/CancelOrder" => {
                    struct CancelOrderSvc<T: MatchingService>(pub Arc<T>);
                    impl<T: MatchingService> tonic::server::UnaryService<super::CancelRequest>
                        for CancelOrderSvc<T> {
                        type Response = super::CancelResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::CancelRequest>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { <T as MatchingService>::cancel_order(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let method = CancelOrderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        Ok(tonic::server::Grpc::new(codec).unary(method, req).await)
                    })
                }
                "/matching.MatchingService/GetOrderBook" => {
                    struct GetOrderBookSvc<T: MatchingService>(pub Arc<T>);
                    impl<T: MatchingService> tonic::server::UnaryService<super::OrderBookRequest>
                        for GetOrderBookSvc<T> {
                        type Response = super::OrderBookResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::OrderBookRequest>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { <T as MatchingService>::get_order_book(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let method = GetOrderBookSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        Ok(tonic::server::Grpc::new(codec).unary(method, req).await)
                    })
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(empty_body());
                        let headers = response.headers_mut();
                        headers.insert(tonic::Status::GRPC_STATUS, (tonic::Code::Unimplemented as i32).into());
                        headers.insert(http::header::CONTENT_TYPE, tonic::metadata::GRPC_CONTENT_TYPE);
                        Ok(response)
                    })
                }
            }
        }
    }

    impl<T> Clone for MatchingServiceServer<T> {
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

    pub const SERVICE_NAME: &str = "matching.MatchingService";
    impl<T> tonic::server::NamedService for MatchingServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}

// ========== Client Helper ==========

pub async fn create_matching_client(addr: &str) -> Result<matching_service_client::MatchingServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    matching_service_client::MatchingServiceClient::connect(addr.to_owned()).await
}
