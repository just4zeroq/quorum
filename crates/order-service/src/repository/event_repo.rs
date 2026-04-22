//! Order Event Repository - 订单事件数据访问

use sqlx::{Pool, Sqlite, Row};
use crate::models::OrderEventRecord;

pub struct OrderEventRepository;

impl OrderEventRepository {
    /// 创建订单事件
    pub async fn create(
        pool: &Pool<Sqlite>,
        event: &OrderEventRecord,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO order_events (order_id, event_type, old_status, new_status,
                filled_quantity, filled_amount, price, reason, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&event.order_id)
        .bind(&event.event_type)
        .bind(event.old_status.as_ref().map(|s| s.to_string()))
        .bind(event.new_status.to_string())
        .bind(event.filled_quantity.map(|q| q.to_string()))
        .bind(event.filled_amount.map(|a| a.to_string()))
        .bind(event.price.map(|p| p.to_string()))
        .bind(&event.reason)
        .bind(event.created_at)
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 根据订单ID查询事件
    pub async fn get_by_order_id(
        pool: &Pool<Sqlite>,
        order_id: &str,
    ) -> Result<Vec<OrderEventRecord>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, order_id, event_type, old_status, new_status,
                filled_quantity, filled_amount, price, reason, created_at
            FROM order_events
            WHERE order_id = ?
            ORDER BY created_at ASC
            "#
        )
        .bind(order_id)
        .fetch_all(pool)
        .await?;

        let events: Vec<OrderEventRecord> = rows.iter().map(Self::row_to_event).collect();
        Ok(events)
    }

    /// 查询最近的订单事件
    pub async fn get_recent(
        pool: &Pool<Sqlite>,
        limit: i32,
    ) -> Result<Vec<OrderEventRecord>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, order_id, event_type, old_status, new_status,
                filled_quantity, filled_amount, price, reason, created_at
            FROM order_events
            ORDER BY created_at DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let events: Vec<OrderEventRecord> = rows.iter().map(Self::row_to_event).collect();
        Ok(events)
    }

    /// 行数据转订单事件 - 使用 SqliteRow
    fn row_to_event(row: &sqlx::sqlite::SqliteRow) -> OrderEventRecord {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        fn parse_decimal(v: Option<String>) -> Option<Decimal> {
            v.and_then(|s| Decimal::from_str(&s).ok())
        }

        fn parse_status(v: Option<String>) -> Option<domain::order::model::OrderStatus> {
            v.and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok())
        }

        OrderEventRecord {
            id: row.get("id"),
            order_id: row.get("order_id"),
            event_type: row.get("event_type"),
            old_status: parse_status(row.get("old_status")),
            new_status: parse_status(row.get("new_status")).unwrap_or_default(),
            filled_quantity: parse_decimal(row.get("filled_quantity")),
            filled_amount: parse_decimal(row.get("filled_amount")),
            price: parse_decimal(row.get("price")),
            reason: row.get("reason"),
            created_at: row.get("created_at"),
        }
    }
}