//! Order Service Implementation

use tonic::{Request, Response, Status};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::models::{Order, OrderQuery};
use crate::repository::OrderRepository;
use crate::pb::{
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
    pool: sqlx::SqlitePool,
}

impl OrderServiceImpl {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
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

        // 创建订单
        let order = Order::new(
            req.user_id,
            req.market_id,
            req.outcome_id,
            req.side,
            order_type,
            price,
            quantity,
            if req.client_order_id.is_empty() { None } else { Some(req.client_order_id) },
        );

        // 保存到数据库
        OrderRepository::create(&self.pool, &order)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tracing::info!("Order created: {}", order.id);

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

        // 验证订单存在且属于该用户
        let order = OrderRepository::get_by_id(&self.pool, &req.order_id)
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
        OrderRepository::cancel(&self.pool, &req.order_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tracing::info!("Order cancelled: {}", req.order_id);

        let updated_order = OrderRepository::get_by_id(&self.pool, &req.order_id)
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

        let order = OrderRepository::get_by_id(&self.pool, &req.order_id)
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

        let query = OrderQuery {
            user_id: Some(req.user_id),
            market_id: if req.market_id == 0 { None } else { Some(req.market_id) },
            outcome_id: None,
            status: if req.status.is_empty() { None } else { Some(req.status) },
            side: None,
            page,
            page_size,
        };

        let (orders, total) = OrderRepository::get_by_user(&self.pool, &query)
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

        let orders = OrderRepository::get_by_market(
            &self.pool,
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

        let success = OrderRepository::update_status(
            &self.pool,
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
            side: order.side,
            order_type: order.order_type,
            price: order.price.to_string(),
            quantity: order.quantity.to_string(),
            filled_quantity: order.filled_quantity.to_string(),
            filled_amount: order.filled_amount.to_string(),
            status: order.status,
            client_order_id: order.client_order_id.unwrap_or_default(),
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}