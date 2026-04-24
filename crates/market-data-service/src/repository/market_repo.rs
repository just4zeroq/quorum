//! Market Data Repository

use sqlx::{PgPool, Row};
use crate::models::{PredictionMarket, MarketOutcome, MarketStatus, OrderBook, OrderBookLevel, Market24hStats};
use rust_decimal::Decimal;

pub struct MarketRepository;

impl MarketRepository {
    /// 获取市场列表
    pub async fn get_markets(
        pool: &PgPool,
        category: Option<&str>,
        status: Option<&str>,
        sort_by: &str,
        descending: bool,
        page: i32,
        page_size: i32,
    ) -> sqlx::Result<Vec<PredictionMarket>> {
        let order = if descending { "DESC" } else { "ASC" };
        let offset = (page - 1) * page_size;

        let query = match (category, status) {
            (Some(cat), Some(st)) => format!(
                "SELECT id, question, description, category, image_url, start_time, end_time,
                        status, resolved_outcome_id, resolved_at, total_volume, created_at, updated_at
                 FROM prediction_markets WHERE category = '{}' AND status = '{}'
                 ORDER BY {} {} LIMIT {} OFFSET {}",
                cat, st, sort_by, order, page_size, offset
            ),
            (Some(cat), None) => format!(
                "SELECT id, question, description, category, image_url, start_time, end_time,
                        status, resolved_outcome_id, resolved_at, total_volume, created_at, updated_at
                 FROM prediction_markets WHERE category = '{}'
                 ORDER BY {} {} LIMIT {} OFFSET {}",
                cat, sort_by, order, page_size, offset
            ),
            (None, Some(st)) => format!(
                "SELECT id, question, description, category, image_url, start_time, end_time,
                        status, resolved_outcome_id, resolved_at, total_volume, created_at, updated_at
                 FROM prediction_markets WHERE status = '{}'
                 ORDER BY {} {} LIMIT {} OFFSET {}",
                st, sort_by, order, page_size, offset
            ),
            (None, None) => format!(
                "SELECT id, question, description, category, image_url, start_time, end_time,
                        status, resolved_outcome_id, resolved_at, total_volume, created_at, updated_at
                 FROM prediction_markets ORDER BY {} {} LIMIT {} OFFSET {}",
                sort_by, order, page_size, offset
            ),
        };

        let rows = sqlx::query(&query).fetch_all(pool).await?;
        let mut markets = Vec::new();

        for row in rows {
            markets.push(PredictionMarket {
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
            });
        }

        Ok(markets)
    }

    /// 获取市场总数
    pub async fn count_markets(
        pool: &PgPool,
        category: Option<&str>,
        status: Option<&str>,
    ) -> sqlx::Result<i64> {
        let query = match (category, status) {
            (Some(cat), Some(st)) => format!(
                "SELECT COUNT(*) FROM prediction_markets WHERE category = '{}' AND status = '{}'",
                cat, st
            ),
            (Some(cat), None) => format!(
                "SELECT COUNT(*) FROM prediction_markets WHERE category = '{}'",
                cat
            ),
            (None, Some(st)) => format!(
                "SELECT COUNT(*) FROM prediction_markets WHERE status = '{}'",
                st
            ),
            (None, None) => "SELECT COUNT(*) FROM prediction_markets".to_string(),
        };

        let row: (i64,) = sqlx::query_as(&query).fetch_one(pool).await?;
        Ok(row.0)
    }

    /// 获取市场详情
    pub async fn get_market_by_id(pool: &PgPool, market_id: i64) -> sqlx::Result<Option<PredictionMarket>> {
        let row = sqlx::query(
            "SELECT id, question, description, category, image_url, start_time, end_time,
                    status, resolved_outcome_id, resolved_at, total_volume, created_at, updated_at
             FROM prediction_markets WHERE id = $1"
        )
        .bind(market_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| PredictionMarket {
            id: r.get("id"),
            question: r.get("question"),
            description: r.get("description"),
            category: r.get("category"),
            image_url: r.get("image_url"),
            start_time: r.get("start_time"),
            end_time: r.get("end_time"),
            status: MarketStatus::from_str(&r.get::<String, _>("status")),
            resolved_outcome_id: r.get("resolved_outcome_id"),
            resolved_at: r.get("resolved_at"),
            total_volume: r.get::<String, _>("total_volume").parse().unwrap_or_default(),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// 获取市场选项列表
    pub async fn get_outcomes(pool: &PgPool, market_id: i64) -> sqlx::Result<Vec<MarketOutcome>> {
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

    /// 获取选项价格
    pub async fn get_outcome_prices(
        pool: &PgPool,
        market_id: i64,
        outcome_ids: &[i64],
    ) -> sqlx::Result<Vec<MarketOutcome>> {
        if outcome_ids.is_empty() {
            return Self::get_outcomes(pool, market_id).await;
        }

        let placeholders: Vec<String> = outcome_ids.iter().enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect();
        let query = format!(
            "SELECT id, market_id, name, description, image_url, price, volume, probability, created_at, updated_at
             FROM market_outcomes WHERE market_id = $1 AND id IN ({})",
            placeholders.join(", ")
        );

        let mut q = sqlx::query(&query).bind(market_id);
        for id in outcome_ids {
            q = q.bind(id);
        }

        let rows = q.fetch_all(pool).await?;
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

    /// 获取订单簿 (简化版 - 基于模拟数据)
    pub async fn get_orderbook(
        pool: &PgPool,
        market_id: i64,
        depth: i32,
    ) -> sqlx::Result<OrderBook> {
        let outcomes = Self::get_outcomes(pool, market_id).await?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for outcome in outcomes {
            // 模拟订单簿数据
            // 实际应该从 order-service 或专门的订单簿服务获取
            let price = outcome.price;

            // 买单 (买方 - 想要买入)
            if price > Decimal::ZERO {
                let bid_qty = outcome.volume / Decimal::from(2);
                bids.push(OrderBookLevel {
                    price,
                    quantity: bid_qty,
                    orders: 1,
                });
            }

            // 卖单 (卖方 - 想要卖出)
            let ask_price = price + Decimal::from_str("0.01").unwrap_or_default();
            let ask_qty = outcome.volume / Decimal::from(2);
            asks.push(OrderBookLevel {
                price: ask_price,
                quantity: ask_qty,
                orders: 1,
            });
        }

        // 排序
        bids.sort_by(|a, b| b.price.cmp(&a.price)); // 买盘价格从高到低
        asks.sort_by(|a, b| a.price.cmp(&b.price)); // 卖盘价格从低到高

        // 截取深度
        bids.truncate(depth as usize);
        asks.truncate(depth as usize);

        Ok(OrderBook {
            market_id,
            outcome_id: 0,
            bids,
            asks,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// 获取24h统计数据
    pub async fn get_24h_stats(pool: &PgPool, market_id: i64) -> sqlx::Result<Market24hStats> {
        let market = Self::get_market_by_id(pool, market_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        // 从交易记录计算24h统计
        // 简化处理，直接使用市场当前数据
        let stats = Market24hStats {
            market_id,
            volume_24h: market.total_volume,
            amount_24h: market.total_volume,
            high_24h: Decimal::from_str("1").unwrap_or_default(),
            low_24h: Decimal::from_str("0.01").unwrap_or_default(),
            price_change: Decimal::ZERO,
            price_change_percent: Decimal::ZERO,
            trade_count_24h: 0,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        Ok(stats)
    }
}

use std::str::FromStr;