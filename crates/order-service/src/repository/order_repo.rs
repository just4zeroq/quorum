//! Order Repository - 订单数据访问

use db::{DBPool, DBError};
use db::SqliteRow;
use crate::models::{Order, OrderQuery};
use sqlx::Row;

pub struct OrderRepository {
    pool: DBPool,
}

impl OrderRepository {
    pub fn new(pool: DBPool) -> Self {
        Self { pool }
    }

    /// 创建订单
    pub async fn create(&self, order: &Order) -> Result<String, DBError> {
        let pool = self.pool.sqlite_pool().ok_or_else(|| DBError::Config("Not a SQLite pool".to_string()))?;
        let result = sqlx::query(
            r#"
            INSERT INTO orders (id, user_id, market_id, outcome_id, side, order_type,
                price, quantity, filled_quantity, filled_amount, status, client_order_id,
                created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&order.id)
        .bind(order.user_id)
        .bind(order.market_id)
        .bind(order.outcome_id)
        .bind(&order.side.to_string())
        .bind(&order.order_type.to_string())
        .bind(order.price.to_string())
        .bind(order.quantity.to_string())
        .bind(order.filled_quantity.to_string())
        .bind(order.filled_amount.to_string())
        .bind(&order.status.to_string())
        .bind(&order.client_order_id)
        .bind(order.created_at)
        .bind(order.updated_at)
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid().to_string())
    }

    /// 根据ID获取订单
    pub async fn get_by_id(&self, order_id: &str) -> Result<Option<Order>, DBError> {
        let pool = self.pool.sqlite_pool().ok_or_else(|| DBError::Config("Not a SQLite pool".to_string()))?;
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM orders WHERE id = ?")
            .bind(order_id)
            .fetch_one(pool)
            .await?;

        if exists.0 == 0 {
            return Ok(None);
        }

        let order = sqlx::query(
            r#"
            SELECT id, user_id, market_id, outcome_id, side, order_type,
                price, quantity, filled_quantity, filled_amount, status, client_order_id,
                created_at, updated_at
            FROM orders WHERE id = ?
            "#
        )
        .bind(order_id)
        .fetch_one(pool)
        .await?;

        Ok(Some(Self::row_to_order(&order)))
    }

    /// 获取用户订单列表
    pub async fn get_by_user(&self, query: &OrderQuery) -> Result<(Vec<Order>, i64), DBError> {
        let pool = self.pool.sqlite_pool().ok_or_else(|| DBError::Config("Not a SQLite pool".to_string()))?;
        let mut sql = String::from("SELECT * FROM orders WHERE 1=1");
        let mut count_sql = String::from("SELECT COUNT(*) FROM orders WHERE 1=1");

        if let Some(user_id) = query.user_id {
            sql.push_str(&format!(" AND user_id = {}", user_id));
            count_sql.push_str(&format!(" AND user_id = {}", user_id));
        }
        if let Some(market_id) = query.market_id {
            sql.push_str(&format!(" AND market_id = {}", market_id));
            count_sql.push_str(&format!(" AND market_id = {}", market_id));
        }
        if let Some(ref status) = query.status {
            sql.push_str(&format!(" AND status = '{}'", status));
            count_sql.push_str(&format!(" AND status = '{}'", status));
        }
        if let Some(ref side) = query.side {
            sql.push_str(&format!(" AND side = '{}'", side));
            count_sql.push_str(&format!(" AND side = '{}'", side));
        }

        let offset = (query.page - 1) * query.page_size;
        sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", query.page_size, offset));

        let rows = sqlx::query(&sql)
            .fetch_all(pool)
            .await?;

        let orders: Vec<Order> = rows.iter().map(|r| Self::row_to_order(r)).collect();

        let total: (i64,) = sqlx::query_as(&count_sql)
            .fetch_one(pool)
            .await?;

        Ok((orders, total.0))
    }

    /// 获取市场订单列表
    pub async fn get_by_market(
        &self,
        market_id: i64,
        side: Option<&str>,
        _status: Option<&str>,
        limit: i32,
    ) -> Result<Vec<Order>, DBError> {
        let pool = self.pool.sqlite_pool().ok_or_else(|| DBError::Config("Not a SQLite pool".to_string()))?;
        let mut sql = String::from("SELECT * FROM orders WHERE market_id = ? AND status IN ('submitted', 'partially_filled')");

        if let Some(s) = side {
            sql = format!("{} AND side = '{}'", sql, s);
        }

        sql = format!("{} ORDER BY price ASC, created_at ASC LIMIT {}", sql, limit);

        let rows = sqlx::query(&sql)
            .bind(market_id)
            .fetch_all(pool)
            .await?;

        Ok(rows.iter().map(|r| Self::row_to_order(r)).collect())
    }

    /// 更新订单状态
    pub async fn update_status(
        &self,
        order_id: &str,
        status: &str,
        filled_quantity: &str,
        filled_amount: &str,
    ) -> Result<bool, DBError> {
        let pool = self.pool.sqlite_pool().ok_or_else(|| DBError::Config("Not a SQLite pool".to_string()))?;
        let now = chrono::Utc::now().timestamp_millis();
        let result = sqlx::query(
            r#"
            UPDATE orders
            SET status = ?, filled_quantity = ?, filled_amount = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(status)
        .bind(filled_quantity)
        .bind(filled_amount)
        .bind(now)
        .bind(order_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 取消订单
    pub async fn cancel(&self, order_id: &str) -> Result<bool, DBError> {
        let pool = self.pool.sqlite_pool().ok_or_else(|| DBError::Config("Not a SQLite pool".to_string()))?;
        let now = chrono::Utc::now().timestamp_millis();
        let result = sqlx::query(
            "UPDATE orders SET status = 'cancelled', updated_at = ? WHERE id = ? AND status IN ('pending', 'submitted', 'partially_filled')"
        )
        .bind(now)
        .bind(order_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 行数据转订单
    fn row_to_order(row: &SqliteRow) -> Order {
        use rust_decimal::Decimal;
        use std::str::FromStr;
        use crate::models::{OrderStatus, OrderSide, OrderType};

        let side_str: String = row.get("side");
        let order_type_str: String = row.get("order_type");
        let status_str: String = row.get("status");

        let side = match side_str.as_str() {
            "buy" => OrderSide::Buy,
            _ => OrderSide::Sell,
        };

        let order_type = match order_type_str.as_str() {
            "limit" => OrderType::Limit,
            "market" => OrderType::Market,
            "ioc" => OrderType::IOC,
            "fok" => OrderType::FOK,
            "post_only" => OrderType::PostOnly,
            _ => OrderType::Limit,
        };

        let status = match status_str.as_str() {
            "pending" => OrderStatus::Pending,
            "submitted" => OrderStatus::Submitted,
            "partially_filled" => OrderStatus::PartiallyFilled,
            "filled" => OrderStatus::Filled,
            "cancelled" => OrderStatus::Cancelled,
            "rejected" => OrderStatus::Rejected,
            _ => OrderStatus::Pending,
        };

        Order {
            id: row.get("id"),
            user_id: row.get("user_id"),
            market_id: row.get("market_id"),
            outcome_id: row.get("outcome_id"),
            side,
            order_type,
            price: Decimal::from_str(&row.get::<String, _>("price")).unwrap_or_default(),
            quantity: Decimal::from_str(&row.get::<String, _>("quantity")).unwrap_or_default(),
            filled_quantity: Decimal::from_str(&row.get::<String, _>("filled_quantity")).unwrap_or_default(),
            filled_amount: Decimal::from_str(&row.get::<String, _>("filled_amount")).unwrap_or_default(),
            status,
            client_order_id: row.get("client_order_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
