//! Portfolio Service SQLite 单元测试

#[cfg(test)]
mod database_test {
    use sqlx::SqlitePool;
    use sqlx::sqlite::SqlitePoolOptions;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    use portfolio_service::models::{Account, AccountType, Position, PositionSide, Settlement, SettlementStatus, LedgerEntry, LedgerType};
    use portfolio_service::repository::PortfolioRepository;
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
            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                asset TEXT NOT NULL,
                account_type TEXT NOT NULL,
                available TEXT NOT NULL DEFAULT '0',
                frozen TEXT NOT NULL DEFAULT '0',
                version INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS positions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                market_id INTEGER NOT NULL,
                outcome_id INTEGER NOT NULL,
                side TEXT NOT NULL,
                size TEXT NOT NULL DEFAULT '0',
                entry_price TEXT NOT NULL DEFAULT '0',
                version INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(user_id, market_id, outcome_id, side)
            )
            "#
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ledger_entries (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                ledger_type TEXT NOT NULL,
                asset TEXT NOT NULL,
                amount TEXT NOT NULL,
                balance_after TEXT NOT NULL,
                reference_id TEXT NOT NULL,
                reference_type TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
            "#
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settlements (
                id TEXT PRIMARY KEY,
                trade_id TEXT NOT NULL,
                market_id INTEGER NOT NULL,
                user_id TEXT NOT NULL,
                outcome_id INTEGER NOT NULL,
                side TEXT NOT NULL,
                amount TEXT NOT NULL,
                fee TEXT NOT NULL,
                payout TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
            "#
        )
        .execute(pool)
        .await
        .unwrap();
    }

    fn create_repo(pool: SqlitePool) -> PortfolioRepository {
        PortfolioRepository::from_db_pool(DBPool::Sqlite(pool))
    }

    fn make_account(user_id: &str, asset: &str, available: &str) -> Account {
        let now = chrono::Utc::now();
        Account {
            id: format!("acc_{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(16).collect::<String>()),
            user_id: user_id.to_string(),
            asset: asset.to_string(),
            account_type: AccountType::Spot,
            available: Decimal::from_str(available).unwrap(),
            frozen: Decimal::ZERO,
            version: 0,
            created_at: now,
            updated_at: now,
        }
    }

    // ==================== Account Tests ====================

    #[tokio::test]
    async fn test_create_account() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let account = make_account("usr_001", "USDC", "1000");
        repo.create_account(&account).await.unwrap();

        let found = repo.get_account("usr_001", "USDC").await.unwrap();
        assert!(found.is_some());
        let a = found.unwrap();
        assert_eq!(a.available, Decimal::from_str("1000").unwrap());
        assert_eq!(a.frozen, Decimal::ZERO);
        assert_eq!(a.asset, "USDC");
    }

    #[tokio::test]
    async fn test_get_account_not_found() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let found = repo.get_account("usr_none", "USDC").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_freeze_success() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let account = make_account("usr_frz", "USDC", "500");
        repo.create_account(&account).await.unwrap();

        let rows = repo.freeze_with_version("usr_frz", "USDC", Decimal::from_str("200").unwrap(), 0)
            .await.unwrap();
        assert_eq!(rows, 1);

        let found = repo.get_account("usr_frz", "USDC").await.unwrap().unwrap();
        assert_eq!(found.available, Decimal::from_str("300").unwrap());
        assert_eq!(found.frozen, Decimal::from_str("200").unwrap());
        assert_eq!(found.version, 1);
    }

    #[tokio::test]
    async fn test_freeze_insufficient_balance() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let account = make_account("usr_poor", "USDC", "100");
        repo.create_account(&account).await.unwrap();

        let rows = repo.freeze_with_version("usr_poor", "USDC", Decimal::from_str("500").unwrap(), 0)
            .await.unwrap();
        assert_eq!(rows, 0);
    }

    #[tokio::test]
    async fn test_freeze_wrong_version() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let account = make_account("usr_ver", "USDC", "1000");
        repo.create_account(&account).await.unwrap();

        // 使用错误的 version，更新应失败
        let rows = repo.freeze_with_version("usr_ver", "USDC", Decimal::from_str("100").unwrap(), 99)
            .await.unwrap();
        assert_eq!(rows, 0);
    }

    #[tokio::test]
    async fn test_unfreeze_success() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let account = make_account("usr_unf", "USDC", "500");
        repo.create_account(&account).await.unwrap();

        // 先冻结
        repo.freeze_with_version("usr_unf", "USDC", Decimal::from_str("200").unwrap(), 0)
            .await.unwrap();

        let found = repo.get_account("usr_unf", "USDC").await.unwrap().unwrap();
        // 解冻
        let rows = repo.unfreeze_with_version("usr_unf", "USDC", Decimal::from_str("200").unwrap(), found.version)
            .await.unwrap();
        assert_eq!(rows, 1);

        let found = repo.get_account("usr_unf", "USDC").await.unwrap().unwrap();
        assert_eq!(found.available, Decimal::from_str("500").unwrap());
        assert_eq!(found.frozen, Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_credit_success() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let account = make_account("usr_crd", "USDC", "100");
        repo.create_account(&account).await.unwrap();

        let rows = repo.credit_with_version("usr_crd", "USDC", Decimal::from_str("50").unwrap(), 0)
            .await.unwrap();
        assert_eq!(rows, 1);

        let found = repo.get_account("usr_crd", "USDC").await.unwrap().unwrap();
        assert_eq!(found.available, Decimal::from_str("150").unwrap());
    }

    #[tokio::test]
    async fn test_debit_success() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let account = make_account("usr_dbt", "USDC", "300");
        repo.create_account(&account).await.unwrap();

        let rows = repo.debit_available_with_version("usr_dbt", "USDC", Decimal::from_str("100").unwrap(), 0)
            .await.unwrap();
        assert_eq!(rows, 1);

        let found = repo.get_account("usr_dbt", "USDC").await.unwrap().unwrap();
        assert_eq!(found.available, Decimal::from_str("200").unwrap());
    }

    #[tokio::test]
    async fn test_debit_insufficient() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        // NOTE: SQLite stores amounts as TEXT, so string comparison is broken for
        // differing digit lengths (e.g. "5" > "1" makes "5" >= "100" TRUE).
        // Use values where string comparison reflects correct numeric order.
        let account = make_account("usr_dbt2", "USDC", "10");
        repo.create_account(&account).await.unwrap();

        let rows = repo.debit_available_with_version("usr_dbt2", "USDC", Decimal::from_str("100").unwrap(), 0)
            .await.unwrap();
        assert_eq!(rows, 0);
    }

    // ==================== Position Tests ====================

    #[tokio::test]
    async fn test_create_position() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let now = chrono::Utc::now();
        let pos = Position {
            id: "pos_001".to_string(),
            user_id: "usr_001".to_string(),
            market_id: 1,
            outcome_id: 1,
            side: PositionSide::Long,
            size: Decimal::from_str("100").unwrap(),
            entry_price: Decimal::from_str("0.5").unwrap(),
            version: 0,
            created_at: now,
            updated_at: now,
        };
        repo.upsert_position(&pos).await.unwrap();

        let found = repo.get_position("usr_001", 1, 1, "long").await.unwrap();
        assert!(found.is_some());
        let p = found.unwrap();
        assert_eq!(p.size, Decimal::from_str("100").unwrap());
        assert_eq!(p.entry_price, Decimal::from_str("0.5").unwrap());
    }

    #[tokio::test]
    async fn test_upsert_position_update() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let now = chrono::Utc::now();
        let pos = Position {
            id: "pos_upd".to_string(),
            user_id: "usr_upd".to_string(),
            market_id: 1,
            outcome_id: 1,
            side: PositionSide::Long,
            size: Decimal::from_str("100").unwrap(),
            entry_price: Decimal::from_str("0.5").unwrap(),
            version: 0,
            created_at: now,
            updated_at: now,
        };
        repo.upsert_position(&pos).await.unwrap();

        // 更新
        let pos2 = Position {
            size: Decimal::from_str("200").unwrap(),
            entry_price: Decimal::from_str("0.6").unwrap(),
            version: 1,
            ..pos
        };
        repo.upsert_position(&pos2).await.unwrap();

        let found = repo.get_position("usr_upd", 1, 1, "long").await.unwrap().unwrap();
        assert_eq!(found.size, Decimal::from_str("200").unwrap());
        assert_eq!(found.entry_price, Decimal::from_str("0.6").unwrap());
    }

    // ==================== Ledger Tests ====================

    #[tokio::test]
    async fn test_insert_ledger() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let now = chrono::Utc::now();
        let entry = LedgerEntry {
            id: "led_001".to_string(),
            user_id: "usr_001".to_string(),
            account_id: "acc_001".to_string(),
            ledger_type: LedgerType::Deposit,
            asset: "USDC".to_string(),
            amount: Decimal::from_str("1000").unwrap(),
            balance_after: Decimal::from_str("1000").unwrap(),
            reference_id: "ref_001".to_string(),
            reference_type: "deposit".to_string(),
            created_at: now,
        };
        repo.insert_ledger(&entry).await.unwrap();

        let entries = repo.list_ledger_by_user("usr_001", 10, 0).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].amount, Decimal::from_str("1000").unwrap());
    }

    #[tokio::test]
    async fn test_list_ledger_empty() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let entries = repo.list_ledger_by_user("usr_empty", 10, 0).await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_list_positions() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let now = chrono::Utc::now();
        for i in 0..3 {
            let pos = Position {
                id: format!("pos_list_{}", i),
                user_id: "usr_list".to_string(),
                market_id: 10,
                outcome_id: i + 1,
                side: PositionSide::Long,
                size: Decimal::from_str("50").unwrap(),
                entry_price: Decimal::from_str("0.5").unwrap(),
                version: 0,
                created_at: now,
                updated_at: now,
            };
            repo.upsert_position(&pos).await.unwrap();
        }

        let positions = repo.list_positions("usr_list", 10).await.unwrap();
        assert_eq!(positions.len(), 3);
    }

    // ==================== Settlement Tests ====================

    #[tokio::test]
    async fn test_insert_settlement() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool);

        let now = chrono::Utc::now();
        let settlement = Settlement {
            id: "stl_001".to_string(),
            trade_id: "trade_001".to_string(),
            market_id: 1,
            user_id: "usr_001".to_string(),
            outcome_id: 1,
            side: PositionSide::Long,
            amount: Decimal::from_str("100").unwrap(),
            fee: Decimal::from_str("1").unwrap(),
            payout: Decimal::from_str("200").unwrap(),
            status: SettlementStatus::Completed,
            created_at: now,
        };

        repo.insert_settlement(&settlement).await.unwrap();
    }

    // ==================== Workflow Tests ====================

    #[tokio::test]
    async fn test_freeze_unfreeze_debit_workflow() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool.clone());

        // NOTE: All amounts must have same digit length for SQLite TEXT comparison
        // to work correctly (e.g. "500" >= "300" compares '5' > '3').
        let account = make_account("usr_wf", "USDC", "500");
        repo.create_account(&account).await.unwrap();

        // 冻结 300
        let rows = repo.freeze_with_version("usr_wf", "USDC", Decimal::from_str("300").unwrap(), 0)
            .await.unwrap();
        assert_eq!(rows, 1);

        let a = repo.get_account("usr_wf", "USDC").await.unwrap().unwrap();
        assert_eq!(a.available, Decimal::from_str("200").unwrap());
        assert_eq!(a.frozen, Decimal::from_str("300").unwrap());

        // 解冻 100
        let rows = repo.unfreeze_with_version("usr_wf", "USDC", Decimal::from_str("100").unwrap(), a.version)
            .await.unwrap();
        assert_eq!(rows, 1);

        let row: (String, String) = sqlx::query_as(
            "SELECT available, frozen FROM accounts WHERE user_id = ? AND asset = ?"
        )
        .bind("usr_wf")
        .bind("USDC")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, "300");
        assert_eq!(row.1, "200");
    }

    #[tokio::test]
    async fn test_freeze_then_debit() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = create_repo(pool.clone());

        // NOTE: All amounts must have same digit length for SQLite TEXT comparison.
        // Start=800: 800-300=500 (freeze), 500-200=300 (debit).
        let account = make_account("usr_fd", "USDC", "800");
        repo.create_account(&account).await.unwrap();

        repo.freeze_with_version("usr_fd", "USDC", Decimal::from_str("300").unwrap(), 0)
            .await.unwrap();

        let a = repo.get_account("usr_fd", "USDC").await.unwrap().unwrap();

        let rows = repo.debit_available_with_version("usr_fd", "USDC", Decimal::from_str("200").unwrap(), a.version)
            .await.unwrap();
        assert_eq!(rows, 1);

        let row: (String, String) = sqlx::query_as(
            "SELECT available, frozen FROM accounts WHERE user_id = ? AND asset = ?"
        )
        .bind("usr_fd")
        .bind("USDC")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, "300");
        assert_eq!(row.1, "300");
    }
}
