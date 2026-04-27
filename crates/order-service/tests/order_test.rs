//! Order Service SQLite 单元测试

#[cfg(test)]
mod database_test {
    use sqlx::SqlitePool;
    use sqlx::sqlite::SqlitePoolOptions;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    use order_service::models::{Order, OrderQuery, OrderSide, OrderType, OrderStatus};
    use order_service::repository::OrderRepository;
    use db::DBPool;

    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap()
    }

    async fn init_tables(pool: &SqlitePool) {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS orders (
                id TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                market_id INTEGER NOT NULL,
                outcome_id INTEGER NOT NULL,
                side TEXT NOT NULL,
                order_type TEXT NOT NULL,
                price TEXT NOT NULL,
                quantity TEXT NOT NULL,
                filled_quantity TEXT NOT NULL DEFAULT '0',
                filled_amount TEXT NOT NULL DEFAULT '0',
                status TEXT NOT NULL DEFAULT 'pending',
                client_order_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id)")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_market_id ON orders(market_id)")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status)")
            .execute(pool)
            .await
            .unwrap();
    }

    fn create_repo(pool: SqlitePool) -> OrderRepository {
        OrderRepository::new(DBPool::Sqlite(pool))
    }

    fn make_order(
        id: &str,
        user_id: i64,
        market_id: i64,
        outcome_id: i64,
        side: OrderSide,
        order_type: OrderType,
        price: &str,
        quantity: &str,
        status: OrderStatus,
    ) -> Order {
        let now = chrono::Utc::now().timestamp_millis();
        Order {
            id: id.to_string(),
            user_id,
            market_id,
            outcome_id,
            side,
            order_type,
            price: Decimal::from_str(price).unwrap(),
            quantity: Decimal::from_str(quantity).unwrap(),
            filled_quantity: Decimal::ZERO,
            filled_amount: Decimal::ZERO,
            status,
            client_order_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    // ========== Create 测试 ==========

    #[tokio::test]
    async fn test_create_order() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let order_id = repo.create(&make_order(
            "ord_test_001", 1, 100, 1,
            OrderSide::Buy, OrderType::Limit, "10.5", "100",
            OrderStatus::Pending,
        )).await.unwrap();

        assert!(!order_id.is_empty());
    }

    #[tokio::test]
    async fn test_create_duplicate_id() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let order = make_order(
            "ord_dup", 1, 100, 1,
            OrderSide::Buy, OrderType::Limit, "10.5", "100",
            OrderStatus::Pending,
        );
        repo.create(&order).await.unwrap();

        let result = repo.create(&order).await;
        assert!(result.is_err());
    }

    // ========== GetByID 测试 ==========

    #[tokio::test]
    async fn test_get_order_by_id() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let order = make_order(
            "ord_get_001", 1, 100, 1,
            OrderSide::Buy, OrderType::Limit, "10.5", "100",
            OrderStatus::Pending,
        );
        repo.create(&order).await.unwrap();

        let found = repo.get_by_id("ord_get_001").await.unwrap();
        assert!(found.is_some());
        let o = found.unwrap();
        assert_eq!(o.user_id, 1);
        assert_eq!(o.side, OrderSide::Buy);
        assert_eq!(o.price, Decimal::from_str("10.5").unwrap());
        assert_eq!(o.status, OrderStatus::Pending);
    }

    #[tokio::test]
    async fn test_get_nonexistent_order() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let found = repo.get_by_id("ord_not_exists").await.unwrap();
        assert!(found.is_none());
    }

    // ========== Update Status 测试 ==========

    #[tokio::test]
    async fn test_update_order_status() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let order = make_order(
            "ord_upd_001", 1, 100, 1,
            OrderSide::Buy, OrderType::Limit, "10.5", "100",
            OrderStatus::Pending,
        );
        repo.create(&order).await.unwrap();

        let updated = repo.update_status("ord_upd_001", "submitted", "0", "0").await.unwrap();
        assert!(updated);

        let found = repo.get_by_id("ord_upd_001").await.unwrap().unwrap();
        assert_eq!(found.status, OrderStatus::Submitted);
    }

    #[tokio::test]
    async fn test_update_filled_quantity() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let order = make_order(
            "ord_fill_001", 1, 100, 1,
            OrderSide::Buy, OrderType::Limit, "10.5", "100",
            OrderStatus::Submitted,
        );
        repo.create(&order).await.unwrap();

        let updated = repo.update_status("ord_fill_001", "partially_filled", "50", "525")
            .await.unwrap();
        assert!(updated);

        let found = repo.get_by_id("ord_fill_001").await.unwrap().unwrap();
        assert_eq!(found.filled_quantity, Decimal::from_str("50").unwrap());
        assert_eq!(found.filled_amount, Decimal::from_str("525").unwrap());
        assert_eq!(found.status, OrderStatus::PartiallyFilled);
    }

    #[tokio::test]
    async fn test_update_nonexistent_order() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let updated = repo.update_status("ord_noexist", "filled", "100", "1000")
            .await.unwrap();
        assert!(!updated);
    }

    // ========== Cancel 测试 ==========

    #[tokio::test]
    async fn test_cancel_pending_order() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        repo.create(&make_order(
            "ord_cancel_001", 1, 100, 1,
            OrderSide::Buy, OrderType::Limit, "10.5", "100",
            OrderStatus::Pending,
        )).await.unwrap();

        let cancelled = repo.cancel("ord_cancel_001").await.unwrap();
        assert!(cancelled);

        let found = repo.get_by_id("ord_cancel_001").await.unwrap().unwrap();
        assert_eq!(found.status, OrderStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_cancel_filled_order_should_fail() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        repo.create(&make_order(
            "ord_nocancel", 1, 100, 1,
            OrderSide::Buy, OrderType::Limit, "10.5", "100",
            OrderStatus::Filled,
        )).await.unwrap();

        let cancelled = repo.cancel("ord_nocancel").await.unwrap();
        assert!(!cancelled);
    }

    // ========== GetByUser 测试 ==========

    #[tokio::test]
    async fn test_get_orders_by_user() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        for i in 0..5 {
            let order_id = format!("ord_user_{}", i);
            repo.create(&make_order(
                &order_id, 42, 100, 1,
                OrderSide::Buy, OrderType::Limit, "10.5", "100",
                OrderStatus::Pending,
            )).await.unwrap();
        }

        let query = OrderQuery {
            user_id: Some(42),
            page: 1,
            page_size: 10,
            ..Default::default()
        };
        let (orders, total) = repo.get_by_user(&query).await.unwrap();
        assert_eq!(orders.len(), 5);
        assert_eq!(total, 5);
    }

    #[tokio::test]
    async fn test_get_orders_by_user_empty() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let query = OrderQuery {
            user_id: Some(999),
            page: 1,
            page_size: 10,
            ..Default::default()
        };
        let (orders, total) = repo.get_by_user(&query).await.unwrap();
        assert!(orders.is_empty());
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_get_orders_with_pagination() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        for i in 0..10 {
            let order_id = format!("ord_page_{}", i);
            repo.create(&make_order(
                &order_id, 1, 100, 1,
                OrderSide::Buy, OrderType::Limit, "10.5", "100",
                OrderStatus::Pending,
            )).await.unwrap();
        }

        let query = OrderQuery {
            user_id: Some(1),
            page: 1,
            page_size: 3,
            ..Default::default()
        };
        let (orders, total) = repo.get_by_user(&query).await.unwrap();
        assert_eq!(orders.len(), 3);
        assert_eq!(total, 10);
    }

    #[tokio::test]
    async fn test_get_orders_filter_by_status() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        repo.create(&make_order(
            "ord_fs_001", 1, 100, 1,
            OrderSide::Buy, OrderType::Limit, "10.5", "100",
            OrderStatus::Pending,
        )).await.unwrap();
        repo.create(&make_order(
            "ord_fs_002", 1, 100, 1,
            OrderSide::Buy, OrderType::Limit, "10.5", "100",
            OrderStatus::Filled,
        )).await.unwrap();
        repo.create(&make_order(
            "ord_fs_003", 1, 100, 1,
            OrderSide::Buy, OrderType::Limit, "10.5", "100",
            OrderStatus::Cancelled,
        )).await.unwrap();

        let query = OrderQuery {
            user_id: Some(1),
            status: Some(OrderStatus::Filled),
            page: 1,
            page_size: 10,
            ..Default::default()
        };
        let (orders, total) = repo.get_by_user(&query).await.unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(orders[0].id, "ord_fs_002");
    }

    // ========== GetByMarket 测试 ==========

    #[tokio::test]
    async fn test_get_orders_by_market() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        for i in 0..5 {
            let order_id = format!("ord_mkt_{}", i);
            repo.create(&make_order(
                &order_id, 1, 200, 1,
                OrderSide::Sell, OrderType::Limit, "20.0", "50",
                OrderStatus::Submitted,
            )).await.unwrap();
        }

        let orders = repo.get_by_market(200, None, None, 10).await.unwrap();
        assert_eq!(orders.len(), 5);
    }

    #[tokio::test]
    async fn test_get_orders_by_market_pending_not_returned() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        repo.create(&make_order(
            "ord_mp_001", 1, 300, 1,
            OrderSide::Buy, OrderType::Limit, "10.0", "100",
            OrderStatus::Pending,
        )).await.unwrap();

        // pending 不在 get_by_market 返回的范围内
        let orders = repo.get_by_market(300, None, None, 10).await.unwrap();
        assert!(orders.is_empty());
    }
}
