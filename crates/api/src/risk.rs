// Risk service types - manually defined (no proto)
use prost::Message;

#[derive(Clone, Message, PartialEq)]
pub struct CheckRiskRequest {
    #[prost(string, tag = "1")]
    pub user_id: String,
    #[prost(uint64, tag = "2")]
    pub market_id: u64,
    #[prost(uint64, tag = "3")]
    pub outcome_id: u64,
    /// YES / NO
    #[prost(string, tag = "4")]
    pub side: String,
    /// limit / market
    #[prost(string, tag = "5")]
    pub order_type: String,
    /// decimal as string
    #[prost(string, tag = "6")]
    pub price: String,
    /// decimal as string
    #[prost(string, tag = "7")]
    pub quantity: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct CheckRiskResponse {
    #[prost(bool, tag = "1")]
    pub accepted: bool,
    /// empty if accepted
    #[prost(string, tag = "2")]
    pub reason: String,
}

// ========== gRPC Client ==========

pub mod risk_service_client {
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct RiskServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl RiskServiceClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }

    impl<T> RiskServiceClient<T>
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

        pub async fn check_risk(
            &mut self,
            request: impl tonic::IntoRequest<super::CheckRiskRequest>,
        ) -> std::result::Result<tonic::Response<super::CheckRiskResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/risk.RiskService/CheckRisk");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("risk.RiskService", "CheckRisk"));
            self.inner.unary(req, path, codec).await
        }
    }
}

// ========== gRPC Server Trait ==========

pub mod risk_service_server {
    use tonic::codegen::*;
    use async_trait::async_trait;

    #[async_trait]
    pub trait RiskService: std::marker::Send + std::marker::Sync + 'static {
        async fn check_risk(
            &self,
            request: tonic::Request<super::CheckRiskRequest>,
        ) -> std::result::Result<tonic::Response<super::CheckRiskResponse>, tonic::Status>;
    }

    #[derive(Debug)]
    pub struct RiskServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }

    impl<T> RiskServiceServer<T> {
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

    impl<T, B> tonic::codegen::Service<http::Request<B>> for RiskServiceServer<T>
    where
        T: RiskService,
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
                "/risk.RiskService/CheckRisk" => {
                    struct CheckRiskSvc<T: RiskService>(pub Arc<T>);
                    impl<T: RiskService> tonic::server::UnaryService<super::CheckRiskRequest>
                        for CheckRiskSvc<T> {
                        type Response = super::CheckRiskResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::CheckRiskRequest>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { <T as RiskService>::check_risk(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let method = CheckRiskSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(accept_compression_encodings, send_compression_encodings)
                            .apply_max_message_size_config(max_decoding_message_size, max_encoding_message_size);
                        Ok(grpc.unary(method, req).await)
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

    impl<T> Clone for RiskServiceServer<T> {
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

    pub const SERVICE_NAME: &str = "risk.RiskService";
    impl<T> tonic::server::NamedService for RiskServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}

// ========== Client Helper ==========

pub async fn create_risk_client(addr: &str) -> Result<risk_service_client::RiskServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    risk_service_client::RiskServiceClient::connect(addr.to_owned()).await
}
