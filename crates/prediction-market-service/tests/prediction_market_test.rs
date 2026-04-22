//! Prediction Market Service SQLite 单元测试

// ========== SQLite 数据库测试 ==========

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
        // 创建 prediction_markets 表
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

        // 创建 market_outcomes 表
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
    }

    // ========== Create 测试 ==========

    #[tokio::test]
    async fn test_create_market() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();
        let result = sqlx::query(
            r#"
            INSERT INTO prediction_markets
                (question, description, category, image_url, start_time, end_time, status,
                 total_volume, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, 'open', '0', ?, ?)
            "#
        )
        .bind("Will BTC reach 100k?")
        .bind("By end of 2024")
        .bind("crypto")
        .bind("")
        .bind(1700000000000i64)
        .bind(1731532800000i64)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        let market_id = result.last_insert_rowid();
        assert_eq!(market_id, 1);

        // 验证
        let row: (i64, String, String) = sqlx::query_as(
            "SELECT id, question, category FROM prediction_markets WHERE id = ?"
        )
        .bind(market_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(row.0, 1);
        assert_eq!(row.1, "Will BTC reach 100k?");
        assert_eq!(row.2, "crypto");
    }

    // ========== Read 测试 ==========

    #[tokio::test]
    async fn test_find_market() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO prediction_markets (question, description, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'open', '0', ?, ?)"
        )
        .bind("Test question")
        .bind("Test description")
        .bind("sports")
        .bind(1700000000000i64)
        .bind(1731532800000i64)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        let row: (i64, String, String, String) = sqlx::query_as(
            "SELECT id, question, category, description FROM prediction_markets WHERE id = 1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(row.1, "Test question");
        assert_eq!(row.2, "sports");
        assert_eq!(row.3, "Test description");
    }

    #[tokio::test]
    async fn test_find_nonexistent() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let row: Option<(i64,)> = sqlx::query_as("SELECT id FROM prediction_markets WHERE id = 999")
            .fetch_optional(&pool)
            .await
            .unwrap();

        assert!(row.is_none());
    }

    // ========== Update 测试 ==========

    #[tokio::test]
    async fn test_update_market_status() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)"
        )
        .bind("Test")
        .bind("crypto")
        .bind(1700000000000i64)
        .bind(1731532800000i64)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // 更新为已结算
        sqlx::query(
            "UPDATE prediction_markets SET status = 'resolved', resolved_outcome_id = 1, resolved_at = ?, updated_at = ? WHERE id = 1"
        )
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        let row: (String, Option<i64>) = sqlx::query_as(
            "SELECT status, resolved_outcome_id FROM prediction_markets WHERE id = 1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(row.0, "resolved");
        assert_eq!(row.1, Some(1));
    }

    #[tokio::test]
    async fn test_update_volume() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)"
        )
        .bind("Test")
        .bind("crypto")
        .bind(1700000000000i64)
        .bind(1731532800000i64)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // 增加成交量
        sqlx::query("UPDATE prediction_markets SET total_volume = total_volume + '1000', updated_at = ? WHERE id = 1")
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();

        let volume: (String,) = sqlx::query_as("SELECT total_volume FROM prediction_markets WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(volume.0, "1000");
    }

    // ========== Delete 测试 ==========

    #[tokio::test]
    async fn test_delete_market() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)"
        )
        .bind("To delete")
        .bind("test")
        .bind(1700000000000i64)
        .bind(1731532800000i64)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // 删除
        sqlx::query("DELETE FROM prediction_markets WHERE id = 1")
            .execute(&pool)
            .await
            .unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM prediction_markets")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count.0, 0);
    }

    // ========== Outcome 测试 ==========

    #[tokio::test]
    async fn test_create_outcome() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();

        // 先创建市场
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

        // 创建选项
        sqlx::query(
            "INSERT INTO market_outcomes (market_id, name, description, price, volume, probability, created_at, updated_at) VALUES (?, ?, ?, '0.5', '0', '0', ?, ?)"
        )
        .bind(1i64)
        .bind("Yes")
        .bind("")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        let row: (i64, String, String) = sqlx::query_as(
            "SELECT id, name, price FROM market_outcomes WHERE market_id = 1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(row.1, "Yes");
        assert_eq!(row.2, "0.5");
    }

    #[tokio::test]
    async fn test_multiple_outcomes() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let now = chrono::Utc::now().timestamp_millis();

        // 创建市场
        sqlx::query(
            "INSERT INTO prediction_markets (question, category, start_time, end_time, status, total_volume, created_at, updated_at) VALUES (?, ?, ?, ?, 'open', '0', ?, ?)"
        )
        .bind("Test")
        .bind("crypto")
        .bind(1700000000000i64)
        .bind(1731532800000i64)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // 创建 Yes 和 No 选项
        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.5', '0', '0', ?, ?)")
            .bind(1i64).bind("Yes").bind(now).bind(now)
            .execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.5', '0', '0', ?, ?)")
            .bind(1i64).bind("No").bind(now).bind(now)
            .execute(&pool).await.unwrap();

        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM market_outcomes WHERE market_id = 1 ORDER BY id"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "Yes");
        assert_eq!(rows[1].0, "No");
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
        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.5', '0', '0', ?, ?)")
            .bind(1i64).bind("Yes").bind(now).bind(now)
            .execute(&pool).await.unwrap();

        // 3. 创建选项 No
        sqlx::query("INSERT INTO market_outcomes (market_id, name, price, volume, probability, created_at, updated_at) VALUES (?, ?, '0.5', '0', '0', ?, ?)")
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

        // 5. 查询选项
        let outcomes: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM market_outcomes WHERE market_id = 1"
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(outcomes.len(), 2);

        // 6. 更新成交量
        sqlx::query("UPDATE prediction_markets SET total_volume = total_volume + '1000', updated_at = ? WHERE id = 1")
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();

        let volume: (String,) = sqlx::query_as("SELECT total_volume FROM prediction_markets WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(volume.0, "1000");

        // 7. 结算市场
        sqlx::query("UPDATE prediction_markets SET status = 'resolved', resolved_outcome_id = 1, resolved_at = ?, updated_at = ? WHERE id = 1")
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();

        let status: (String, Option<i64>) = sqlx::query_as(
            "SELECT status, resolved_outcome_id FROM prediction_markets WHERE id = 1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(status.0, "resolved");
        assert_eq!(status.1, Some(1));
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
        let page1: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM prediction_markets ORDER BY id LIMIT 2 OFFSET 0"
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(page1.len(), 2);

        let page2: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM prediction_markets ORDER BY id LIMIT 2 OFFSET 2"
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(page2.len(), 2);

        let page3: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM prediction_markets ORDER BY id LIMIT 2 OFFSET 4"
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(page3.len(), 1);
    }

    // ========== 空结果测试 ==========

    #[tokio::test]
    async fn test_empty_result() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM prediction_markets")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count.0, 0);
    }
}