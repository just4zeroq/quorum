//! Trade Repository

use sqlx::{PgPool, Row};
use crate::models::Trade;

pub struct TradeRepository;

impl TradeRepository {
    /// 获取近期成交
    pub async fn get_recent_trades(
        pool: &PgPool,
        market_id: i64,
        outcome_id: Option<i64>,
        limit: i32,
        from_trade_id: Option<i64>,
    ) -> sqlx::Result<Vec<Trade>> {
        let query = match (outcome_id, from_trade_id) {
            (Some(oid), Some(fid)) => format!(
                "SELECT id, market_id, outcome_id, user_id, side, price, quantity, amount, fee, created_at
                 FROM market_trades
                 WHERE market_id = {} AND outcome_id = {} AND id < {}
                 ORDER BY id DESC LIMIT {}",
                market_id, oid, fid, limit
            ),
            (Some(oid), None) => format!(
                "SELECT id, market_id, outcome_id, user_id, side, price, quantity, amount, fee, created_at
                 FROM market_trades
                 WHERE market_id = {} AND outcome_id = {}
                 ORDER BY id DESC LIMIT {}",
                market_id, oid, limit
            ),
            (None, Some(fid)) => format!(
                "SELECT id, market_id, outcome_id, user_id, side, price, quantity, amount, fee, created_at
                 FROM market_trades
                 WHERE market_id = {} AND id < {}
                 ORDER BY id DESC LIMIT {}",
                market_id, fid, limit
            ),
            (None, None) => format!(
                "SELECT id, market_id, outcome_id, user_id, side, price, quantity, amount, fee, created_at
                 FROM market_trades
                 WHERE market_id = {}
                 ORDER BY id DESC LIMIT {}",
                market_id, limit
            ),
        };

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let mut trades = Vec::new();
        for row in rows {
            trades.push(Trade {
                id: row.get("id"),
                market_id: row.get("market_id"),
                outcome_id: row.get("outcome_id"),
                user_id: row.get("user_id"),
                side: row.get("side"),
                price: row.get::<String, _>("price").parse().unwrap_or_default(),
                quantity: row.get::<String, _>("quantity").parse().unwrap_or_default(),
                amount: row.get::<String, _>("amount").parse().unwrap_or_default(),
                fee: row.get::<String, _>("fee").parse().unwrap_or_default(),
                created_at: row.get("created_at"),
            });
        }

        Ok(trades)
    }

    /// 创建成交记录 (用于模拟数据)
    pub async fn create_trade(
        pool: &PgPool,
        trade: &Trade,
    ) -> sqlx::Result<i64> {
        let row = sqlx::query(
            "INSERT INTO market_trades (market_id, outcome_id, user_id, side, price, quantity, amount, fee, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"
        )
        .bind(trade.market_id)
        .bind(trade.outcome_id)
        .bind(trade.user_id)
        .bind(&trade.side)
        .bind(&trade.price.to_string())
        .bind(&trade.quantity.to_string())
        .bind(&trade.amount.to_string())
        .bind(&trade.fee.to_string())
        .bind(trade.created_at)
        .fetch_one(pool)
        .await?;

        Ok(row.get("id"))
    }

    /// 统计24h成交笔数
    pub async fn count_24h_trades(
        pool: &PgPool,
        market_id: i64,
        since: i64,
    ) -> sqlx::Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM market_trades WHERE market_id = $1 AND created_at >= $2"
        )
        .bind(market_id)
        .bind(since)
        .fetch_one(pool)
        .await?;

        Ok(row.0)
    }
}