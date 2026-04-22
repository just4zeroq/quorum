//! 结果选项仓库

use sqlx::{PgPool, Row};
use crate::models::MarketOutcome;

pub struct OutcomeRepository;

impl OutcomeRepository {
    /// 创建选项
    pub async fn create(pool: &PgPool, outcome: &MarketOutcome) -> sqlx::Result<i64> {
        let row = sqlx::query(
            r#"
            INSERT INTO market_outcomes
                (market_id, name, description, image_url, price, volume, probability, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(outcome.market_id)
        .bind(&outcome.name)
        .bind(&outcome.description)
        .bind(&outcome.image_url)
        .bind(&outcome.price.to_string())
        .bind(&outcome.volume.to_string())
        .bind(&outcome.probability.to_string())
        .bind(outcome.created_at)
        .bind(outcome.updated_at)
        .fetch_one(pool)
        .await?;

        Ok(row.get("id"))
    }

    /// 根据市场ID获取选项列表
    pub async fn find_by_market(pool: &PgPool, market_id: i64) -> sqlx::Result<Vec<MarketOutcome>> {
        let rows = sqlx::query(
            "SELECT id, market_id, name, description, image_url, price, volume, probability, created_at, updated_at
             FROM market_outcomes WHERE market_id = $1 ORDER BY id"
        )
        .bind(market_id)
        .fetch_all(pool)
        .await?;

        let mut outcomes = Vec::new();
        for row in rows {
            outcomes.push(MarketOutcome {
                id: row.get("id"),
                market_id: row.get("market_id"),
                name: row.get("name"),
                description: row.get("description"),
                image_url: row.get("image_url"),
                price: row.get::<String, _>("price").parse().unwrap_or_default(),
                volume: row.get::<String, _>("volume").parse().unwrap_or_default(),
                probability: row.get::<String, _>("probability").parse().unwrap_or_default(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(outcomes)
    }

    /// 根据ID获取选项
    pub async fn find_by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<MarketOutcome>> {
        let row = sqlx::query(
            "SELECT id, market_id, name, description, image_url, price, volume, probability, created_at, updated_at
             FROM market_outcomes WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| MarketOutcome {
            id: r.get("id"),
            market_id: r.get("market_id"),
            name: r.get("name"),
            description: r.get("description"),
            image_url: r.get("image_url"),
            price: r.get::<String, _>("price").parse().unwrap_or_default(),
            volume: r.get::<String, _>("volume").parse().unwrap_or_default(),
            probability: r.get::<String, _>("probability").parse().unwrap_or_default(),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// 更新选项价格和成交量
    pub async fn update_price(
        pool: &PgPool,
        id: i64,
        price: &str,
        volume: &str,
        probability: &str,
    ) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            UPDATE market_outcomes
            SET price = $1, volume = $2, probability = $3, updated_at = $4
            WHERE id = $5
            "#,
        )
        .bind(price)
        .bind(volume)
        .bind(probability)
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}