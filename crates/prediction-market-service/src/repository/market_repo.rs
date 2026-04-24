//! 预测市场仓库

use sqlx::{PgPool, Row};
use crate::models::{PredictionMarket, MarketStatus};

pub struct MarketRepository;

impl MarketRepository {
    /// 创建市场
    pub async fn create(pool: &PgPool, market: &PredictionMarket) -> sqlx::Result<i64> {
        let row = sqlx::query(
            r#"
            INSERT INTO prediction_markets
                (question, description, category, image_url, start_time, end_time, status,
                 total_volume, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
            "#,
        )
        .bind(&market.question)
        .bind(&market.description)
        .bind(&market.category)
        .bind(&market.image_url)
        .bind(market.start_time)
        .bind(market.end_time)
        .bind(&market.status.to_string())
        .bind(&market.total_volume.to_string())
        .bind(market.created_at)
        .bind(market.updated_at)
        .fetch_one(pool)
        .await?;

        Ok(row.get("id"))
    }

    /// 根据ID获取市场
    pub async fn find_by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<PredictionMarket>> {
        let row = sqlx::query(
            "SELECT id, question, description, category, image_url, start_time, end_time,
                    status, resolved_outcome_id, resolved_at, total_volume, created_at, updated_at
             FROM prediction_markets WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| Self::row_to_market(r)))
    }

    /// 获取市场列表
    pub async fn list(
        pool: &PgPool,
        category: Option<&str>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> sqlx::Result<Vec<PredictionMarket>> {
        let mut markets = Vec::new();

        let query = match (category, status) {
            (Some(cat), Some(st)) => {
                sqlx::query(
                    "SELECT id, question, description, category, image_url, start_time, end_time,
                            status, resolved_outcome_id, resolved_at, total_volume, created_at, updated_at
                     FROM prediction_markets WHERE category = $1 AND status = $2
                     ORDER BY created_at DESC LIMIT $3 OFFSET $4"
                )
                .bind(cat)
                .bind(st)
                .bind(limit)
                .bind(offset)
            }
            (Some(cat), None) => {
                sqlx::query(
                    "SELECT id, question, description, category, image_url, start_time, end_time,
                            status, resolved_outcome_id, resolved_at, total_volume, created_at, updated_at
                     FROM prediction_markets WHERE category = $1
                     ORDER BY created_at DESC LIMIT $2 OFFSET $3"
                )
                .bind(cat)
                .bind(limit)
                .bind(offset)
            }
            (None, Some(st)) => {
                sqlx::query(
                    "SELECT id, question, description, category, image_url, start_time, end_time,
                            status, resolved_outcome_id, resolved_at, total_volume, created_at, updated_at
                     FROM prediction_markets WHERE status = $1
                     ORDER BY created_at DESC LIMIT $2 OFFSET $3"
                )
                .bind(st)
                .bind(limit)
                .bind(offset)
            }
            (None, None) => {
                sqlx::query(
                    "SELECT id, question, description, category, image_url, start_time, end_time,
                            status, resolved_outcome_id, resolved_at, total_volume, created_at, updated_at
                     FROM prediction_markets ORDER BY created_at DESC LIMIT $1 OFFSET $2"
                )
                .bind(limit)
                .bind(offset)
            }
        };

        let rows = query.fetch_all(pool).await?;

        for row in rows {
            markets.push(Self::row_to_market(row));
        }

        Ok(markets)
    }

    fn row_to_market(row: sqlx::postgres::PgRow) -> PredictionMarket {
        PredictionMarket {
            id: row.get("id"),
            question: row.get("question"),
            description: row.get("description"),
            category: row.get("category"),
            image_url: row.get("image_url"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            status: MarketStatus::from_str(&row.get::<String, _>("status")),
            resolved_outcome_id: row.get("resolved_outcome_id"),
            resolved_at: row.get("resolved_at"),
            total_volume: row.get::<String, _>("total_volume").parse().unwrap_or_default(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    /// 更新市场状态
    pub async fn update_status(
        pool: &PgPool,
        id: i64,
        status: &str,
        resolved_outcome_id: Option<i64>,
        resolved_at: Option<i64>,
    ) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            UPDATE prediction_markets
            SET status = $1, resolved_outcome_id = $2, resolved_at = $3, updated_at = $4
            WHERE id = $5
            "#,
        )
        .bind(status)
        .bind(resolved_outcome_id)
        .bind(resolved_at)
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 更新市场信息
    pub async fn update(
        pool: &PgPool,
        id: i64,
        question: Option<&str>,
        description: Option<&str>,
        image_url: Option<&str>,
        end_time: Option<i64>,
    ) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            UPDATE prediction_markets
            SET question = COALESCE($1, question),
                description = COALESCE($2, description),
                image_url = COALESCE($3, image_url),
                end_time = COALESCE($4, end_time),
                updated_at = $5
            WHERE id = $6
            "#,
        )
        .bind(question)
        .bind(description)
        .bind(image_url)
        .bind(end_time)
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 更新市场成交量
    pub async fn add_volume(pool: &PgPool, id: i64, volume: &str) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            UPDATE prediction_markets
            SET total_volume = total_volume + $1, updated_at = $2
            WHERE id = $3
            "#,
        )
        .bind(volume)
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}