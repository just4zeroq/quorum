//! Trade Repository

use sqlx::{PgPool, Row};
use crate::models::{Trade, TradeSide};

fn parse_side(s: &str) -> TradeSide {
    match s.to_lowercase().as_str() {
        "buy" => TradeSide::Buy,
        _ => TradeSide::Sell,
    }
}

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
            let id: i64 = row.get("id");
            let side_str: String = row.get("side");
            let fee: rust_decimal::Decimal = row.get::<String, _>("fee").parse().unwrap_or_default();
            trades.push(Trade {
                id: id.to_string(),
                trade_id: id.to_string(),
                order_id: String::new(),
                counter_order_id: String::new(),
                market_id: row.get("market_id"),
                outcome_id: row.get("outcome_id"),
                maker_user_id: row.get("user_id"),
                taker_user_id: row.get("user_id"),
                side: parse_side(&side_str),
                price: row.get::<String, _>("price").parse().unwrap_or_default(),
                quantity: row.get::<String, _>("quantity").parse().unwrap_or_default(),
                amount: row.get::<String, _>("amount").parse().unwrap_or_default(),
                maker_fee: fee,
                taker_fee: fee,
                fee_token: "USDC".to_string(),
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
        .bind(trade.taker_user_id)
        .bind(trade.side.to_string())
        .bind(&trade.price.to_string())
        .bind(&trade.quantity.to_string())
        .bind(&trade.amount.to_string())
        .bind(&trade.taker_fee.to_string())
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