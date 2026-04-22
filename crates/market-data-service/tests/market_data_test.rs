//! Market Data Service SQLite 单元测试

#[cfg(test)]
mod database_test {
    use sqlx::SqlitePool;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap()
    }

    async fn init_tables(pool: &SqlitePool) {
        // prediction_markets 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS prediction_markets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                question TEXT NOT NULL,
                description TEXT,
                category TEXT NOT NULL,
                image_url TEXT,
                start_time INTEGER NOT NULL,
                end_time INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'open',
                resolved_outcome_id INTEGER,
                resolved_at INTEGER,
                total_volume TEXT NOT NULL DEFAULT '0',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#
        )
        .execute(pool)
        .await
        .unwrap();

        // market_outcomes 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS market_outcomes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                market_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                image_url TEXT,
                price TEXT NOT NULL DEFAULT '0.5',
                volume TEXT NOT NULL DEFAULT '0',
                probability TEXT NOT NULL DEFAULT '0',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#
        )
        .execute(pool)
        .await
        .unwrap();

        // market_klines 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS market_klines (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                market_id INTEGER NOT NULL,
                interval TEXT NOT NULL,
                open TEXT NOT NULL,
                high TEXT NOT NULL,
                low TEXT NOT NULL,
                close TEXT NOT NULL,
                volume TEXT NOT NULL,
                quote_volume TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )
            "#
        )
        .execute(pool)
        .await
        .unwrap();

        // market_trades 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS market_trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                market_id INTEGER NOT NULL,
                outcome_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                side TEXT NOT NULL,
                price TEXT NOT NULL,
                quantity TEXT NOT NULL,
                amount TEXT NOT NULL,
                fee TEXT NOT NULL DEFAULT '0',
                created_at INTEGER NOT NULL
            )
            "#
        )
        .execute(pool)
        .await
        .unwrap();
    }

    // ========== Create 测试 ==========

    #[tokio::test]
    async fn test_create_market() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();
        let result = sqlx::query(
            "INSERT INTO prediction_markets (question, description, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'open', '0', ?, ?)"
        )
        .bind("Will BTC reach 100k?")
        .bind("Test description")
        .bind("crypto")
        .bind(1700000000000i64)
        .bind(1731532800000i64)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        let market_id = result.last_insert_rowid();
        assert_eq!(market_id, 1);
    }

    #[tokio::test]
    async fn test_create_outcomes() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();

        // 创建市场
        sqlx::query(
            "INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)"
        )
        .bind("Test market")
        .bind("crypto")
        .bind(1700000000000i64)
        .bind(1731532800000i64)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // 创建 Yes 选项
        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.5', '1000', '0.5', ?, ?)")
            .bind(1i64).bind("Yes").bind(now).bind(now)
            .execute(&pool).await.unwrap();

        // 创建 No 选项
        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.5', '800', '0.5', ?, ?)")
            .bind(1i64).bind("No").bind(now).bind(now)
            .execute(&pool).await.unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM market_outcomes WHERE market_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count.0, 2);
    }

    // ========== Read 测试 ==========

    #[tokio::test]
    async fn test_get_markets() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();

        for i in 1..=3 {
            sqlx::query(
                "INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)"
            )
            .bind(format!("Question {}", i))
            .bind("crypto")
            .bind(1700000000000i64)
            .bind(1731532800000i64)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();
        }

        let markets: Vec<(i64, String)> = sqlx::query_as("SELECT id, question FROM prediction_markets ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(markets.len(), 3);
    }

    #[tokio::test]
    async fn test_get_market_by_category() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();

        // 创建不同分类的市场
        sqlx::query("INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)")
            .bind("Q1").bind("crypto").bind(1700000000000i64).bind(1731532800000i64).bind(now).bind(now)
            .execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)")
            .bind("Q2").bind("crypto").bind(1700000000000i64).bind(1731532800000i64).bind(now).bind(now)
            .execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)")
            .bind("Q3").bind("sports").bind(1700000000000i64).bind(1731532800000i64).bind(now).bind(now)
            .execute(&pool).await.unwrap();

        let crypto: Vec<(i64,)> = sqlx::query_as("SELECT id FROM prediction_markets WHERE category = 'crypto'")
            .fetch_all(&pool)
            .await
            .unwrap();

        let sports: Vec<(i64,)> = sqlx::query_as("SELECT id FROM prediction_markets WHERE category = 'sports'")
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(crypto.len(), 2);
        assert_eq!(sports.len(), 1);
    }

    #[tokio::test]
    async fn test_get_outcomes_with_prices() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();

        // 创建市场
        sqlx::query("INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)")
            .bind("Test").bind("crypto").bind(1700000000000i64).bind(1731532800000i64).bind(now).bind(now)
            .execute(&pool).await.unwrap();

        // 创建选项
        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.7', '7000', '0.7', ?, ?)")
            .bind(1i64).bind("Yes").bind(now).bind(now)
            .execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.3', '3000', '0.3', ?, ?)")
            .bind(1i64).bind("No").bind(now).bind(now)
            .execute(&pool).await.unwrap();

        let outcomes: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT name, price, volume FROM market_outcomes WHERE market_id = 1 ORDER BY id"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(outcomes[0].0, "Yes");
        assert_eq!(outcomes[0].1, "0.7");
        assert_eq!(outcomes[1].0, "No");
    }

    // ========== K线测试 ==========

    #[tokio::test]
    async fn test_create_and_get_klines() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();

        // 创建市场
        sqlx::query("INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)")
            .bind("Test").bind("crypto").bind(1700000000000i64).bind(1731532800000i64).bind(now).bind(now)
            .execute(&pool).await.unwrap();

        // 创建K线
        for i in 0..5 {
            let ts = now - (i * 60000); // 每分钟
            sqlx::query(
                "INSERT INTO market_klines (market_id, interval, open, high, low, close, volume, quote_volume, timestamp) VALUES (?, '1m', ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(1i64)
            .bind("0.5")
            .bind("0.6")
            .bind("0.4")
            .bind("0.55")
            .bind("1000")
            .bind("500")
            .bind(ts)
            .execute(&pool)
            .await
            .unwrap();
        }

        let klines: Vec<(i64, String)> = sqlx::query_as(
            "SELECT id, close FROM market_klines WHERE market_id = 1 ORDER BY timestamp DESC LIMIT 3"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(klines.len(), 3);
    }

    // ========== 成交记录测试 ==========

    #[tokio::test]
    async fn test_create_and_get_trades() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();

        // 创建市场
        sqlx::query("INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)")
            .bind("Test").bind("crypto").bind(1700000000000i64).bind(1731532800000i64).bind(now).bind(now)
            .execute(&pool).await.unwrap();

        // 创建选项
        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.5', '0', '0', ?, ?)")
            .bind(1i64).bind("Yes").bind(now).bind(now)
            .execute(&pool).await.unwrap();

        // 创建成交
        sqlx::query(
            "INSERT INTO market_trades (market_id, outcome_id, user_id, side, price, quantity, amount, fee, created_at) VALUES (?, ?, ?, 'buy', '0.5', '100', '50', '0.05', ?)"
        )
        .bind(1i64).bind(1i64).bind(1i64).bind(now)
        .execute(&pool)
        .await
        .unwrap();

        let trades: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT side, price, quantity FROM market_trades WHERE market_id = 1"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(trades[0].0, "buy");
        assert_eq!(trades[0].1, "0.5");
        assert_eq!(trades[0].2, "100");
    }

    // ========== 分页测试 ==========

    #[tokio::test]
    async fn test_pagination() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();

        // 创建 5 个市场
        for i in 1..=5 {
            sqlx::query(
                "INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)"
            )
            .bind(format!("Market {}", i))
            .bind("test")
            .bind(1700000000000i64)
            .bind(1731532800000i64)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();
        }

        // 分页查询
        let page1: Vec<(i64,)> = sqlx::query_as("SELECT id FROM prediction_markets ORDER BY id LIMIT 2 OFFSET 0")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(page1.len(), 2);

        let page2: Vec<(i64,)> = sqlx::query_as("SELECT id FROM prediction_markets ORDER BY id LIMIT 2 OFFSET 2")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(page2.len(), 2);
    }

    // ========== 完整流程测试 ==========

    #[tokio::test]
    async fn test_full_workflow() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();

        // 1. 创建市场
        sqlx::query(
            "INSERT INTO prediction_markets (question, description, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'open', '0', ?, ?)"
        )
        .bind("Will ETH reach 5000?")
        .bind("By end of year")
        .bind("crypto")
        .bind(1700000000000i64)
        .bind(1731532800000i64)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // 2. 创建选项 Yes
        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.6', '6000', '0.6', ?, ?)")
            .bind(1i64).bind("Yes").bind(now).bind(now)
            .execute(&pool).await.unwrap();

        // 3. 创建选项 No
        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.4', '4000', '0.4', ?, ?)")
            .bind(1i64).bind("No").bind(now).bind(now)
            .execute(&pool).await.unwrap();

        // 4. 查询市场
        let market: (String, String) = sqlx::query_as(
            "SELECT question, category FROM prediction_markets WHERE id = 1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(market.0, "Will ETH reach 5000?");

        // 5. 查询选项价格
        let outcomes: Vec<(String, String)> = sqlx::query_as(
            "SELECT name, price FROM market_outcomes WHERE market_id = 1"
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(outcomes.len(), 2);

        // 6. 更新成交量
        sqlx::query("UPDATE prediction_markets SET total_volume = '10000', updated_at = ? WHERE id = 1")
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();

        let volume: (String,) = sqlx::query_as("SELECT total_volume FROM prediction_markets WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(volume.0, "10000");

        // 7. 结算市场
        sqlx::query("UPDATE prediction_markets SET status = 'resolved', resolved_outcome_id = 1, resolved_at = ?, updated_at = ? WHERE id = 1")
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();

        let status: (String, Option<i64>) = sqlx::query_as("SELECT status, resolved_outcome_id FROM prediction_markets WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status.0, "resolved");
    }
}