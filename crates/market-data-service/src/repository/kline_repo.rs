//! Kline Repository

use sqlx::{PgPool, Row};
use crate::models::{Kline, KlineInterval};

fn parse_interval(s: &str) -> KlineInterval {
    match s {
        "1m" => KlineInterval::Interval1m,
        "5m" => KlineInterval::Interval5m,
        "15m" => KlineInterval::Interval15m,
        "1h" => KlineInterval::Interval1h,
        "4h" => KlineInterval::Interval4h,
        "1d" => KlineInterval::Interval1d,
        _ => KlineInterval::Interval1h,
    }
}

fn interval_to_string(interval: &KlineInterval) -> String {
    match interval {
        KlineInterval::Interval1m => "1m".to_string(),
        KlineInterval::Interval5m => "5m".to_string(),
        KlineInterval::Interval15m => "15m".to_string(),
        KlineInterval::Interval1h => "1h".to_string(),
        KlineInterval::Interval4h => "4h".to_string(),
        KlineInterval::Interval1d => "1d".to_string(),
    }
}

pub struct KlineRepository;

impl KlineRepository {
    /// 获取K线数据
    pub async fn get_klines(
        pool: &PgPool,
        market_id: i64,
        interval: &str,
        start_time: i64,
        end_time: i64,
        limit: i32,
    ) -> sqlx::Result<Vec<Kline>> {
        let rows = sqlx::query(
            "SELECT market_id, interval, open, high, low, close, volume, quote_volume, timestamp
             FROM market_klines
             WHERE market_id = $1 AND interval = $2 AND timestamp >= $3 AND timestamp <= $4
             ORDER BY timestamp DESC LIMIT $5"
        )
        .bind(market_id)
        .bind(interval)
        .bind(start_time)
        .bind(end_time)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let mut klines = Vec::new();
        for row in rows {
            let interval_str: String = row.get("interval");
            klines.push(Kline {
                market_id: row.get("market_id"),
                outcome_id: 0,
                interval: parse_interval(&interval_str),
                open: row.get::<String, _>("open").parse().unwrap_or_default(),
                high: row.get::<String, _>("high").parse().unwrap_or_default(),
                low: row.get::<String, _>("low").parse().unwrap_or_default(),
                close: row.get::<String, _>("close").parse().unwrap_or_default(),
                volume: row.get::<String, _>("volume").parse().unwrap_or_default(),
                quote_volume: row.get::<String, _>("quote_volume").parse().unwrap_or_default(),
                timestamp: row.get("timestamp"),
            });
        }

        Ok(klines)
    }

    /// 获取最新K线
    pub async fn get_latest_kline(
        pool: &PgPool,
        market_id: i64,
        interval: &str,
    ) -> sqlx::Result<Option<Kline>> {
        let row = sqlx::query(
            "SELECT market_id, interval, open, high, low, close, volume, quote_volume, timestamp
             FROM market_klines
             WHERE market_id = $1 AND interval = $2
             ORDER BY timestamp DESC LIMIT 1"
        )
        .bind(market_id)
        .bind(interval)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| {
            let interval_str: String = r.get("interval");
            Kline {
                market_id: r.get("market_id"),
                outcome_id: 0,
                interval: parse_interval(&interval_str),
                open: r.get::<String, _>("open").parse().unwrap_or_default(),
                high: r.get::<String, _>("high").parse().unwrap_or_default(),
                low: r.get::<String, _>("low").parse().unwrap_or_default(),
                close: r.get::<String, _>("close").parse().unwrap_or_default(),
                volume: r.get::<String, _>("volume").parse().unwrap_or_default(),
                quote_volume: r.get::<String, _>("quote_volume").parse().unwrap_or_default(),
                timestamp: r.get("timestamp"),
            }
        }))
    }

    /// 创建K线 (用于模拟数据)
    pub async fn create_kline(
        pool: &PgPool,
        kline: &Kline,
    ) -> sqlx::Result<i64> {
        let row = sqlx::query(
            "INSERT INTO market_klines (market_id, interval, open, high, low, close, volume, quote_volume, timestamp)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"
        )
        .bind(kline.market_id)
        .bind(interval_to_string(&kline.interval))
        .bind(&kline.open.to_string())
        .bind(&kline.high.to_string())
        .bind(&kline.low.to_string())
        .bind(&kline.close.to_string())
        .bind(&kline.volume.to_string())
        .bind(&kline.quote_volume.to_string())
        .bind(kline.timestamp)
        .fetch_one(pool)
        .await?;

        Ok(row.get("id"))
    }
}