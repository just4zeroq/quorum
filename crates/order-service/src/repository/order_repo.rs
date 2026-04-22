//! Order Repository - 订单数据访问

use sqlx::{Pool, Sqlite, Row};
use crate::models::{Order, OrderQuery};

pub struct OrderRepository;

impl OrderRepository {
    /// 创建订单
    pub async fn create(
        pool: &Pool<Sqlite>,
        order: &Order,
    ) -> Result<String, sqlx::Error> {
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
        .bind(&order.side)
        .bind(&order.order_type)
        .bind(order.price.to_string())
        .bind(order.quantity.to_string())
        .bind(order.filled_quantity.to_string())
        .bind(order.filled_amount.to_string())
        .bind(&order.status)
        .bind(&order.client_order_id)
        .bind(order.created_at)
        .bind(order.updated_at)
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid().to_string())
    }

    /// 根据ID获取订单
    pub async fn get_by_id(
        pool: &Pool<Sqlite>,
        order_id: &str,
    ) -> Result<Option<Order>, sqlx::Error> {
        // 先检查是否存在
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM orders WHERE id = ?")
            .bind(order_id)
            .fetch_one(pool)
            .await?;

        if exists.0 == 0 {
            return Ok(None);
        }

        // 获取订单
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
    pub async fn get_by_user(
        pool: &Pool<Sqlite>,
        query: &OrderQuery,
    ) -> Result<(Vec<Order>, i64), sqlx::Error> {
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

        // 排序和分页
        let offset = (query.page - 1) * query.page_size;
        sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", query.page_size, offset));

        let rows = sqlx::query(&sql)
            .fetch_all(pool)
            .await?;

        let orders: Vec<Order> = rows.iter().map(|r| Self::row_to_order(r)).collect();

        // 获取总数
        let total: (i64,) = sqlx::query_as(&count_sql)
            .fetch_one(pool)
            .await?;

        Ok((orders, total.0))
    }

    /// 获取市场订单列表
    pub async fn get_by_market(
        pool: &Pool<Sqlite>,
        market_id: i64,
        side: Option<&str>,
        _status: Option<&str>,
        limit: i32,
    ) -> Result<Vec<Order>, sqlx::Error> {
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
        pool: &Pool<Sqlite>,
        order_id: &str,
        status: &str,
        filled_quantity: &str,
        filled_amount: &str,
    ) -> Result<bool, sqlx::Error> {
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
    pub async fn cancel(
        pool: &Pool<Sqlite>,
        order_id: &str,
    ) -> Result<bool, sqlx::Error> {
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

    /// 行数据转订单 - 使用 SqliteRow
    fn row_to_order(row: &sqlx::sqlite::SqliteRow) -> Order {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        Order {
            id: row.get("id"),
            user_id: row.get("user_id"),
            market_id: row.get("market_id"),
            outcome_id: row.get("outcome_id"),
            side: row.get("side"),
            order_type: row.get("order_type"),
            price: Decimal::from_str(&row.get::<String, _>("price")).unwrap_or_default(),
            quantity: Decimal::from_str(&row.get::<String, _>("quantity")).unwrap_or_default(),
            filled_quantity: Decimal::from_str(&row.get::<String, _>("filled_quantity")).unwrap_or_default(),
            filled_amount: Decimal::from_str(&row.get::<String, _>("filled_amount")).unwrap_or_default(),
            status: row.get("status"),
            client_order_id: row.get("client_order_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}