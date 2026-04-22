//! 订单管理器

use common::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// 订单管理器
pub struct OrderManager {
    /// 订单存储
    orders: RwLock<HashMap<String, Order>>,
}

impl OrderManager {
    pub fn new() -> Self {
        Self {
            orders: RwLock::new(HashMap::new()),
        }
    }

    /// 创建订单
    pub fn create_order(&self, order: Order) -> Result<Order, Error> {
        // 参数校验
        if order.quantity <= Decimal::ZERO {
            return Err(Error::InvalidParam("Quantity must be positive".to_string()));
        }
        if order.price <= Decimal::ZERO {
            return Err(Error::InvalidParam("Price must be positive".to_string()));
        }

        // 订单类型校验
        match order.order_type {
            OrderType::Limit => {}
            OrderType::Market => {}
            OrderType::IOC | OrderType::FOK => {
                if order.price <= Decimal::ZERO {
                    return Err(Error::InvalidParam("IOC/FOK orders require a price".to_string()));
                }
            }
            OrderType::PostOnly => {
                if order.price <= Decimal::ZERO {
                    return Err(Error::InvalidParam("PostOnly orders require a price".to_string()));
                }
            }
        }

        // 存储订单
        let mut orders = self.orders.write();
        orders.insert(order.id.clone(), order.clone());

        tracing::info!("Order created: {}", order.id);
        Ok(order)
    }

    /// 获取订单
    pub fn get_order(&self, order_id: &str) -> Result<Order, Error> {
        let orders = self.orders.read();
        orders
            .get(order_id)
            .cloned()
            .ok_or_else(|| Error::OrderNotFound(order_id.to_string()))
    }

    /// 取消订单
    pub fn cancel_order(&self, order_id: &str) -> Result<Order, Error> {
        let mut orders = self.orders.write();
        if let Some(order) = orders.get_mut(order_id) {
            if order.is_completed() {
                return Err(Error::Business("Order already completed".to_string()));
            }
            order.status = OrderStatus::Cancelled;
            order.updated_at = chrono::Utc::now();
            tracing::info!("Order cancelled: {}", order_id);
            Ok(order.clone())
        } else {
            Err(Error::OrderNotFound(order_id.to_string()))
        }
    }

    /// 更新订单状态
    pub fn update_order_status(&self, order_id: &str, status: OrderStatus, filled: Decimal) -> Result<(), Error> {
        let mut orders = self.orders.write();
        if let Some(order) = orders.get_mut(order_id) {
            order.filled_quantity = filled;
            order.status = status;
            order.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(Error::OrderNotFound(order_id.to_string()))
        }
    }

    /// 获取用户订单列表
    pub fn get_user_orders(&self, user_id: &str, symbol: Option<&str>) -> Vec<Order> {
        let orders = self.orders.read();
        orders
            .values()
            .filter(|o| o.user_id == user_id)
            .filter(|o| symbol.map_or(true, |s| o.symbol == s))
            .cloned()
            .collect()
    }
}

impl Default for OrderManager {
    fn default() -> Self {
        Self::new()
    }
}

pub type GlobalOrderManager = Arc<OrderManager>;

pub fn create_order_manager() -> GlobalOrderManager {
    Arc::new(OrderManager::new())
}