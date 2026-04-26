// Order service types - manually maintained (no proto)
use prost::Message;

// ========== Requests ==========

#[derive(Clone, Message, PartialEq)]
pub struct CreateOrderRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(int64, tag = "2")]
    pub market_id: i64,
    #[prost(int64, tag = "3")]
    pub outcome_id: i64,
    /// buy, sell
    #[prost(string, tag = "4")]
    pub side: String,
    /// limit, market, ioc, fok, post_only
    #[prost(string, tag = "5")]
    pub order_type: String,
    /// 价格 (0-1 for prediction market)
    #[prost(string, tag = "6")]
    pub price: String,
    /// 数量
    #[prost(string, tag = "7")]
    pub quantity: String,
    /// 客户端订单ID (可选)
    #[prost(string, tag = "8")]
    pub client_order_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct CancelOrderRequest {
    #[prost(string, tag = "1")]
    pub order_id: String,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetOrderRequest {
    #[prost(string, tag = "1")]
    pub order_id: String,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetUserOrdersRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    /// 可选
    #[prost(int64, tag = "2")]
    pub market_id: i64,
    /// 可选过滤
    #[prost(string, tag = "3")]
    pub status: String,
    #[prost(int32, tag = "4")]
    pub page: i32,
    #[prost(int32, tag = "5")]
    pub page_size: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetOrdersByMarketRequest {
    #[prost(int64, tag = "1")]
    pub market_id: i64,
    /// 可选过滤
    #[prost(string, tag = "2")]
    pub side: String,
    /// 可选过滤
    #[prost(string, tag = "3")]
    pub status: String,
    #[prost(int32, tag = "4")]
    pub limit: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct UpdateOrderStatusRequest {
    #[prost(string, tag = "1")]
    pub order_id: String,
    /// submitted, partially_filled, filled, cancelled, rejected
    #[prost(string, tag = "2")]
    pub status: String,
    #[prost(string, tag = "3")]
    pub filled_quantity: String,
    #[prost(string, tag = "4")]
    pub filled_amount: String,
}

// ========== Responses ==========

#[derive(Clone, Message, PartialEq)]
pub struct CreateOrderResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub order_id: String,
    #[prost(string, tag = "3")]
    pub message: String,
    #[prost(message, optional, tag = "4")]
    pub order: Option<Order>,
}

#[derive(Clone, Message, PartialEq)]
pub struct CancelOrderResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(message, optional, tag = "3")]
    pub order: Option<Order>,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetOrderResponse {
    #[prost(message, optional, tag = "1")]
    pub order: Option<Order>,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetUserOrdersResponse {
    #[prost(message, repeated, tag = "1")]
    pub orders: Vec<Order>,
    #[prost(int64, tag = "2")]
    pub total: i64,
    #[prost(int32, tag = "3")]
    pub page: i32,
    #[prost(int32, tag = "4")]
    pub page_size: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct GetOrdersByMarketResponse {
    #[prost(message, repeated, tag = "1")]
    pub orders: Vec<Order>,
    #[prost(int32, tag = "2")]
    pub count: i32,
}

#[derive(Clone, Message, PartialEq)]
pub struct UpdateOrderStatusResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

// ========== Data Models ==========

#[derive(Clone, Message, PartialEq)]
pub struct Order {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(int64, tag = "3")]
    pub market_id: i64,
    #[prost(int64, tag = "4")]
    pub outcome_id: i64,
    #[prost(string, tag = "5")]
    pub side: String,
    #[prost(string, tag = "6")]
    pub order_type: String,
    #[prost(string, tag = "7")]
    pub price: String,
    #[prost(string, tag = "8")]
    pub quantity: String,
    #[prost(string, tag = "9")]
    pub filled_quantity: String,
    #[prost(string, tag = "10")]
    pub filled_amount: String,
    #[prost(string, tag = "11")]
    pub status: String,
    #[prost(string, tag = "12")]
    pub client_order_id: String,
    #[prost(int64, tag = "13")]
    pub created_at: i64,
    #[prost(int64, tag = "14")]
    pub updated_at: i64,
}

// ========== gRPC Client ==========

pub mod order_service_client {
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct OrderServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl OrderServiceClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }

    impl<T> OrderServiceClient<T>
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

        pub async fn create_order(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::CreateOrderResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/order.OrderService/CreateOrder");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("order.OrderService", "CreateOrder"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn cancel_order(
            &mut self,
            request: impl tonic::IntoRequest<super::CancelOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::CancelOrderResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/order.OrderService/CancelOrder");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("order.OrderService", "CancelOrder"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_order(
            &mut self,
            request: impl tonic::IntoRequest<super::GetOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::GetOrderResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/order.OrderService/GetOrder");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("order.OrderService", "GetOrder"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_user_orders(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUserOrdersRequest>,
        ) -> std::result::Result<tonic::Response<super::GetUserOrdersResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/order.OrderService/GetUserOrders");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("order.OrderService", "GetUserOrders"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn get_orders_by_market(
            &mut self,
            request: impl tonic::IntoRequest<super::GetOrdersByMarketRequest>,
        ) -> std::result::Result<tonic::Response<super::GetOrdersByMarketResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/order.OrderService/GetOrdersByMarket");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("order.OrderService", "GetOrdersByMarket"));
            self.inner.unary(req, path, codec).await
        }

        pub async fn update_order_status(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateOrderStatusRequest>,
        ) -> std::result::Result<tonic::Response<super::UpdateOrderStatusResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/order.OrderService/UpdateOrderStatus");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("order.OrderService", "UpdateOrderStatus"));
            self.inner.unary(req, path, codec).await
        }
    }
}

// ========== gRPC Server Trait ==========

pub mod order_service_server {
    use tonic::codegen::*;
    use async_trait::async_trait;

    #[async_trait]
    pub trait OrderService: std::marker::Send + std::marker::Sync + 'static {
        async fn create_order(
            &self,
            request: tonic::Request<super::CreateOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::CreateOrderResponse>, tonic::Status>;

        async fn cancel_order(
            &self,
            request: tonic::Request<super::CancelOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::CancelOrderResponse>, tonic::Status>;

        async fn get_order(
            &self,
            request: tonic::Request<super::GetOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::GetOrderResponse>, tonic::Status>;

        async fn get_user_orders(
            &self,
            request: tonic::Request<super::GetUserOrdersRequest>,
        ) -> std::result::Result<tonic::Response<super::GetUserOrdersResponse>, tonic::Status>;

        async fn get_orders_by_market(
            &self,
            request: tonic::Request<super::GetOrdersByMarketRequest>,
        ) -> std::result::Result<tonic::Response<super::GetOrdersByMarketResponse>, tonic::Status>;

        async fn update_order_status(
            &self,
            request: tonic::Request<super::UpdateOrderStatusRequest>,
        ) -> std::result::Result<tonic::Response<super::UpdateOrderStatusResponse>, tonic::Status>;
    }

    #[derive(Debug)]
    pub struct OrderServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }

    impl<T> OrderServiceServer<T> {
        pub fn new(inner: T) -> Self { Self::from_arc(Arc::new(inner)) }
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
        where F: tonic::service::Interceptor {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding); self
        }
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding); self
        }
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit); self
        }
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit); self
        }
    }

    impl<T, B> tonic::codegen::Service<http::Request<B>> for OrderServiceServer<T>
    where
        T: OrderService,
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
                "/order.OrderService/CreateOrder" => {
                    struct S<T: OrderService>(pub Arc<T>);
                    impl<T: OrderService> tonic::server::UnaryService<super::CreateOrderRequest> for S<T> {
                        type Response = super::CreateOrderResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::CreateOrderRequest>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            Box::pin(async move { <T as OrderService>::create_order(&inner, request).await })
                        }
                    }
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let method = S(inner);
                        Ok(tonic::server::Grpc::new(tonic::codec::ProstCodec::default()).unary(method, req).await)
                    })
                }
                "/order.OrderService/CancelOrder" => {
                    struct S<T: OrderService>(pub Arc<T>);
                    impl<T: OrderService> tonic::server::UnaryService<super::CancelOrderRequest> for S<T> {
                        type Response = super::CancelOrderResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::CancelOrderRequest>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            Box::pin(async move { <T as OrderService>::cancel_order(&inner, request).await })
                        }
                    }
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let method = S(inner);
                        Ok(tonic::server::Grpc::new(tonic::codec::ProstCodec::default()).unary(method, req).await)
                    })
                }
                "/order.OrderService/GetOrder" => {
                    struct S<T: OrderService>(pub Arc<T>);
                    impl<T: OrderService> tonic::server::UnaryService<super::GetOrderRequest> for S<T> {
                        type Response = super::GetOrderResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::GetOrderRequest>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            Box::pin(async move { <T as OrderService>::get_order(&inner, request).await })
                        }
                    }
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let method = S(inner);
                        Ok(tonic::server::Grpc::new(tonic::codec::ProstCodec::default()).unary(method, req).await)
                    })
                }
                "/order.OrderService/GetUserOrders" => {
                    struct S<T: OrderService>(pub Arc<T>);
                    impl<T: OrderService> tonic::server::UnaryService<super::GetUserOrdersRequest> for S<T> {
                        type Response = super::GetUserOrdersResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::GetUserOrdersRequest>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            Box::pin(async move { <T as OrderService>::get_user_orders(&inner, request).await })
                        }
                    }
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let method = S(inner);
                        Ok(tonic::server::Grpc::new(tonic::codec::ProstCodec::default()).unary(method, req).await)
                    })
                }
                "/order.OrderService/GetOrdersByMarket" => {
                    struct S<T: OrderService>(pub Arc<T>);
                    impl<T: OrderService> tonic::server::UnaryService<super::GetOrdersByMarketRequest> for S<T> {
                        type Response = super::GetOrdersByMarketResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::GetOrdersByMarketRequest>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            Box::pin(async move { <T as OrderService>::get_orders_by_market(&inner, request).await })
                        }
                    }
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let method = S(inner);
                        Ok(tonic::server::Grpc::new(tonic::codec::ProstCodec::default()).unary(method, req).await)
                    })
                }
                "/order.OrderService/UpdateOrderStatus" => {
                    struct S<T: OrderService>(pub Arc<T>);
                    impl<T: OrderService> tonic::server::UnaryService<super::UpdateOrderStatusRequest> for S<T> {
                        type Response = super::UpdateOrderStatusResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::UpdateOrderStatusRequest>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            Box::pin(async move { <T as OrderService>::update_order_status(&inner, request).await })
                        }
                    }
                    let inner = self.inner.clone();
                    Box::pin(async move {
                        let method = S(inner);
                        Ok(tonic::server::Grpc::new(tonic::codec::ProstCodec::default()).unary(method, req).await)
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

    impl<T> Clone for OrderServiceServer<T> {
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

    pub const SERVICE_NAME: &str = "order.OrderService";
    impl<T> tonic::server::NamedService for OrderServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}

// ========== Client Helper ==========

pub async fn create_order_client(addr: &str) -> Result<order_service_client::OrderServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    order_service_client::OrderServiceClient::connect(addr.to_owned()).await
}
