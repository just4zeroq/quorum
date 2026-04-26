//! Order Service Implementation

use tonic::{Request, Response, Status};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::models::{Order, OrderQuery, OrderSide, OrderType, OrderStatus};
use crate::repository::OrderRepository;
use db::DBPool;
use api::order::{
    CreateOrderRequest, CreateOrderResponse,
    CancelOrderRequest, CancelOrderResponse,
    GetOrderRequest, GetOrderResponse,
    GetUserOrdersRequest, GetUserOrdersResponse,
    GetOrdersByMarketRequest, GetOrdersByMarketResponse,
    UpdateOrderStatusRequest, UpdateOrderStatusResponse,
    Order as PbOrder,
    order_service_server::OrderService,
};

pub struct OrderServiceImpl {
    pool: DBPool,
    queue_producer: Option<crate::queue_producer::OrderCommandProducer>,
}

impl OrderServiceImpl {
    pub fn new(pool: DBPool) -> Self {
        Self { pool, queue_producer: None }
    }

    pub fn with_queue_producer(mut self, producer: crate::queue_producer::OrderCommandProducer) -> Self {
        self.queue_producer = Some(producer);
        self
    }
}

#[tonic::async_trait]
impl OrderService for OrderServiceImpl {
    async fn create_order(
        &self,
        request: Request<CreateOrderRequest>,
    ) -> Result<Response<CreateOrderResponse>, Status> {
        let req = request.into_inner();

        // 参数校验
        let quantity = Decimal::from_str(&req.quantity)
            .map_err(|_| Status::invalid_argument("Invalid quantity"))?;
        if quantity <= Decimal::ZERO {
            return Err(Status::invalid_argument("Quantity must be positive"));
        }

        let price = Decimal::from_str(&req.price)
            .map_err(|_| Status::invalid_argument("Invalid price"))?;
        if price <= Decimal::ZERO {
            return Err(Status::invalid_argument("Price must be positive"));
        }

        // 验证订单类型
        let order_type = match req.order_type.as_str() {
            "limit" | "market" | "ioc" | "fok" | "post_only" => req.order_type.clone(),
            _ => return Err(Status::invalid_argument("Invalid order type")),
        };

        // 验证订单方向
        if !matches!(req.side.as_str(), "buy" | "sell") {
            return Err(Status::invalid_argument("Invalid side, must be 'buy' or 'sell'"));
        }

        // 解析方向和类型为枚举
        let side = match req.side.as_str() {
            "buy" => OrderSide::Buy,
            "sell" => OrderSide::Sell,
            _ => return Err(Status::invalid_argument("Invalid side")),
        };

        let order_type_enum = match order_type.as_str() {
            "limit" => OrderType::Limit,
            "market" => OrderType::Market,
            "ioc" => OrderType::IOC,
            "fok" => OrderType::FOK,
            "post_only" => OrderType::PostOnly,
            _ => return Err(Status::invalid_argument("Invalid order type")),
        };

        // 创建订单
        let order = Order::new(
            req.user_id,
            req.market_id,
            req.outcome_id,
            side,
            order_type_enum,
            price,
            quantity,
            if req.client_order_id.is_empty() { None } else { Some(req.client_order_id) },
        );

        // 保存到数据库
        let repo = OrderRepository::new(self.pool.clone());
        repo.create(&order)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tracing::info!("Order created: {}", order.id);

        // 发送到消息队列
        if let Some(ref producer) = self.queue_producer {
            if let Err(e) = producer.send_place_order(&order).await {
                tracing::error!("Failed to send order to queue: {}", e);
                // 注意: 订单已创建，不回滚。后续可以添加重试机制
            }
        }

        Ok(Response::new(CreateOrderResponse {
            success: true,
            order_id: order.id.clone(),
            message: "Order created successfully".to_string(),
            order: Some(Self::to_pb_order(order)),
        }))
    }

    async fn cancel_order(
        &self,
        request: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderResponse>, Status> {
        let req = request.into_inner();

        let repo = OrderRepository::new(self.pool.clone());

        // 验证订单存在且属于该用户
        let order = repo.get_by_id(&req.order_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Order not found"))?;

        if order.user_id != req.user_id {
            return Err(Status::permission_denied("Order does not belong to user"));
        }

        if !order.can_cancel() {
            return Err(Status::failed_precondition("Order cannot be cancelled"));
        }

        // 取消订单
        repo.cancel(&req.order_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tracing::info!("Order cancelled: {}", req.order_id);

        // 发送取消命令到消息队列
        if let Some(ref producer) = self.queue_producer {
            if let Err(e) = producer.send_cancel_order(&req.order_id, order.user_id).await {
                tracing::error!("Failed to send cancel order to queue: {}", e);
            }
        }

        let updated_order = repo.get_by_id(&req.order_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .unwrap();

        Ok(Response::new(CancelOrderResponse {
            success: true,
            message: "Order cancelled successfully".to_string(),
            order: Some(Self::to_pb_order(updated_order)),
        }))
    }

    async fn get_order(
        &self,
        request: Request<GetOrderRequest>,
    ) -> Result<Response<GetOrderResponse>, Status> {
        let req = request.into_inner();

        let repo = OrderRepository::new(self.pool.clone());
        let order = repo.get_by_id(&req.order_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Order not found"))?;

        Ok(Response::new(GetOrderResponse {
            order: Some(Self::to_pb_order(order)),
        }))
    }

    async fn get_user_orders(
        &self,
        request: Request<GetUserOrdersRequest>,
    ) -> Result<Response<GetUserOrdersResponse>, Status> {
        let req = request.into_inner();

        let page = if req.page <= 0 { 1 } else { req.page };
        let page_size = if req.page_size <= 0 { 20 } else { req.page_size };

        // 解析 status 字符串为 OrderStatus 枚举
        let status = if req.status.is_empty() {
            None
        } else {
            match req.status.as_str() {
                "pending" => Some(OrderStatus::Pending),
                "submitted" => Some(OrderStatus::Submitted),
                "partially_filled" => Some(OrderStatus::PartiallyFilled),
                "filled" => Some(OrderStatus::Filled),
                "cancelled" => Some(OrderStatus::Cancelled),
                "rejected" => Some(OrderStatus::Rejected),
                _ => return Err(Status::invalid_argument("Invalid status")),
            }
        };

        let query = OrderQuery {
            user_id: Some(req.user_id),
            market_id: if req.market_id == 0 { None } else { Some(req.market_id) },
            outcome_id: None,
            status,
            side: None,
            page,
            page_size,
        };

        let repo = OrderRepository::new(self.pool.clone());
        let (orders, total) = repo.get_by_user(&query)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let pb_orders: Vec<PbOrder> = orders.into_iter().map(Self::to_pb_order).collect();

        Ok(Response::new(GetUserOrdersResponse {
            orders: pb_orders,
            total,
            page,
            page_size,
        }))
    }

    async fn get_orders_by_market(
        &self,
        request: Request<GetOrdersByMarketRequest>,
    ) -> Result<Response<GetOrdersByMarketResponse>, Status> {
        let req = request.into_inner();

        let limit = if req.limit <= 0 { 100 } else { req.limit };

        let repo = OrderRepository::new(self.pool.clone());
        let orders = repo.get_by_market(
            req.market_id,
            if req.side.is_empty() { None } else { Some(&req.side) },
            if req.status.is_empty() { None } else { Some(&req.status) },
            limit,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let pb_orders: Vec<PbOrder> = orders.into_iter().map(Self::to_pb_order).collect();
        let count = pb_orders.len() as i32;

        Ok(Response::new(GetOrdersByMarketResponse {
            orders: pb_orders,
            count,
        }))
    }

    async fn update_order_status(
        &self,
        request: Request<UpdateOrderStatusRequest>,
    ) -> Result<Response<UpdateOrderStatusResponse>, Status> {
        let req = request.into_inner();

        let filled_quantity = if req.filled_quantity.is_empty() {
            "0".to_string()
        } else {
            req.filled_quantity.clone()
        };

        let filled_amount = if req.filled_amount.is_empty() {
            "0".to_string()
        } else {
            req.filled_amount.clone()
        };

        let repo = OrderRepository::new(self.pool.clone());
        let success = repo.update_status(
            &req.order_id,
            &req.status,
            &filled_quantity,
            &filled_amount,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        if success {
            tracing::info!("Order status updated: {} -> {}", req.order_id, req.status);
        }

        Ok(Response::new(UpdateOrderStatusResponse {
            success,
            message: if success { "Status updated".to_string() } else { "Order not found".to_string() },
        }))
    }
}

impl OrderServiceImpl {
    fn to_pb_order(order: Order) -> PbOrder {
        PbOrder {
            id: order.id,
            user_id: order.user_id,
            market_id: order.market_id,
            outcome_id: order.outcome_id,
            side: order.side.to_string(),
            order_type: order.order_type.to_string(),
            price: order.price.to_string(),
            quantity: order.quantity.to_string(),
            filled_quantity: order.filled_quantity.to_string(),
            filled_amount: order.filled_amount.to_string(),
            status: order.status.to_string(),
            client_order_id: order.client_order_id.unwrap_or_default(),
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}
