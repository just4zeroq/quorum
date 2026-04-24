//! 用户持仓仓库

use sqlx::{PgPool, Row};
use crate::models::UserPosition;

pub struct PositionRepository;

impl PositionRepository {
    /// 创建持仓记录
    pub async fn create(pool: &PgPool, position: &UserPosition) -> sqlx::Result<i64> {
        let row = sqlx::query(
            r#"
            INSERT INTO user_positions
                (user_id, market_id, outcome_id, quantity, avg_price, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(position.user_id)
        .bind(position.market_id)
        .bind(position.outcome_id)
        .bind(&position.quantity.to_string())
        .bind(&position.avg_price.to_string())
        .bind(position.created_at)
        .bind(position.updated_at)
        .fetch_one(pool)
        .await?;

        Ok(row.get("id"))
    }

    /// 根据用户ID和市场ID获取持仓列表
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: i64,
        market_id: Option<i64>,
    ) -> sqlx::Result<Vec<UserPosition>> {
        let query = if let Some(mid) = market_id {
            sqlx::query(
                "SELECT id, user_id, market_id, outcome_id, quantity, avg_price, created_at, updated_at
                 FROM user_positions WHERE user_id = $1 AND market_id = $2 ORDER BY created_at DESC"
            )
            .bind(user_id)
            .bind(mid)
        } else {
            sqlx::query(
                "SELECT id, user_id, market_id, outcome_id, quantity, avg_price, created_at, updated_at
                 FROM user_positions WHERE user_id = $1 ORDER BY created_at DESC"
            )
            .bind(user_id)
        };

        let rows = query.fetch_all(pool).await?;
        let mut positions = Vec::new();
        for row in rows {
            positions.push(Self::row_to_position(row));
        }
        Ok(positions)
    }

    /// 根据ID获取持仓
    pub async fn find_by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<UserPosition>> {
        let row = sqlx::query(
            "SELECT id, user_id, market_id, outcome_id, quantity, avg_price, created_at, updated_at
             FROM user_positions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| Self::row_to_position(r)))
    }

    fn row_to_position(row: sqlx::postgres::PgRow) -> UserPosition {
        UserPosition {
            id: row.get("id"),
            user_id: row.get("user_id"),
            market_id: row.get("market_id"),
            outcome_id: row.get("outcome_id"),
            quantity: row.get::<String, _>("quantity").parse().unwrap_or_default(),
            avg_price: row.get::<String, _>("avg_price").parse().unwrap_or_default(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
