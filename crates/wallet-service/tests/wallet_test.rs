//! Wallet Service SQLite 单元测试

#[cfg(test)]
mod database_test {
    use sqlx::SqlitePool;
    use sqlx::sqlite::SqlitePoolOptions;

    use wallet_service::repository::{
        DepositRepository, WithdrawRepository, WhitelistRepository, PaymentPasswordRepository,
    };

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
            CREATE TABLE IF NOT EXISTS deposit_addresses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                chain TEXT NOT NULL,
                address TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_deposit_addresses_user_id ON deposit_addresses(user_id)")
            .execute(pool)
            .await
            .unwrap();

        sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_deposit_addresses_user_chain ON deposit_addresses(user_id, chain)")
            .execute(pool)
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS deposit_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                tx_id TEXT NOT NULL,
                chain TEXT NOT NULL,
                amount TEXT NOT NULL,
                address TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS withdraw_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                asset TEXT NOT NULL,
                amount TEXT NOT NULL,
                fee TEXT NOT NULL,
                to_address TEXT NOT NULL,
                chain TEXT NOT NULL,
                status TEXT NOT NULL,
                tx_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_withdraw_records_user_id ON withdraw_records(user_id)")
            .execute(pool)
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS whitelist_addresses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                chain TEXT NOT NULL,
                address TEXT NOT NULL,
                label TEXT,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_whitelist_addresses_user_id ON whitelist_addresses(user_id)")
            .execute(pool)
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS payment_passwords (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    // ==================== Deposit Repository Tests ====================

    #[tokio::test]
    async fn test_create_deposit_address() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = DepositRepository::new(pool);

        let id = repo.create_address(1, "ETH", "0x1234").await.unwrap();
        assert!(id > 0);

        let addr = repo.get_address(1, "ETH").await.unwrap().unwrap();
        assert_eq!(addr.address, "0x1234");
        assert_eq!(addr.chain, "ETH");
    }

    #[tokio::test]
    async fn test_create_duplicate_address() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = DepositRepository::new(pool);

        repo.create_address(1, "ETH", "0x1234").await.unwrap();
        let result = repo.create_address(1, "ETH", "0x5678").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_address_not_found() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = DepositRepository::new(pool);

        let addr = repo.get_address(1, "BTC").await.unwrap();
        assert!(addr.is_none());
    }

    #[tokio::test]
    async fn test_list_addresses() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = DepositRepository::new(pool);

        repo.create_address(1, "ETH", "0x1111").await.unwrap();
        repo.create_address(1, "BTC", "1abc").await.unwrap();

        let addrs = repo.list_addresses(1).await.unwrap();
        assert_eq!(addrs.len(), 2);

        let addrs_other = repo.list_addresses(2).await.unwrap();
        assert!(addrs_other.is_empty());
    }

    #[tokio::test]
    async fn test_create_deposit_record() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = DepositRepository::new(pool);

        let id = repo.create_record(1, "tx_001", "ETH", "1000", "0x1234").await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_deposit_history_empty() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = DepositRepository::new(pool);

        let (records, total) = repo.get_history(1, "ETH", 1, 10).await.unwrap();
        assert!(records.is_empty());
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_deposit_history_pagination() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = DepositRepository::new(pool);

        for i in 0..5 {
            repo.create_record(1, &format!("tx_{}", i), "ETH", "100", "0x1234").await.unwrap();
        }

        let (records, total) = repo.get_history(1, "ETH", 1, 3).await.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(total, 5);

        let (records2, _) = repo.get_history(1, "ETH", 2, 3).await.unwrap();
        assert_eq!(records2.len(), 2);

        // diff user should be empty
        let (records3, total3) = repo.get_history(2, "ETH", 1, 10).await.unwrap();
        assert!(records3.is_empty());
        assert_eq!(total3, 0);
    }

    // ==================== Withdraw Repository Tests ====================

    #[tokio::test]
    async fn test_create_withdraw() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WithdrawRepository::new(pool);

        let id = repo.create(1, "USDC", "500", "1", "0xdest", "ETH").await.unwrap();
        assert!(id > 0);

        let record = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(record.status, "pending");
        assert_eq!(record.amount, "500");
        assert_eq!(record.asset, "USDC");
    }

    #[tokio::test]
    async fn test_get_withdraw_not_found() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WithdrawRepository::new(pool);

        let record = repo.get_by_id(999).await.unwrap();
        assert!(record.is_none());
    }

    #[tokio::test]
    async fn test_update_withdraw_status() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WithdrawRepository::new(pool);

        let id = repo.create(1, "USDC", "500", "1", "0xdest", "ETH").await.unwrap();

        repo.update_status(id, "confirmed", Some("tx_abc")).await.unwrap();

        let record = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(record.status, "confirmed");
        assert_eq!(record.tx_id, Some("tx_abc".to_string()));
    }

    #[tokio::test]
    async fn test_update_withdraw_status_null_txid() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WithdrawRepository::new(pool);

        let id = repo.create(1, "USDC", "500", "1", "0xdest", "ETH").await.unwrap();

        repo.update_status(id, "failed", None).await.unwrap();

        let record = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(record.status, "failed");
        assert_eq!(record.tx_id, None);
    }

    #[tokio::test]
    async fn test_get_withdraw_history() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WithdrawRepository::new(pool);

        for _i in 0..4 {
            repo.create(1, "USDC", "100", "1", "0xdest", "ETH").await.unwrap();
        }

        let (records, total) = repo.get_history(1, 1, 10).await.unwrap();
        assert_eq!(records.len(), 4);
        assert_eq!(total, 4);

        // other user empty
        let (records2, total2) = repo.get_history(2, 1, 10).await.unwrap();
        assert!(records2.is_empty());
        assert_eq!(total2, 0);
    }

    #[tokio::test]
    async fn test_get_withdraw_history_pagination() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WithdrawRepository::new(pool);

        for _i in 0..7 {
            repo.create(1, "USDC", "100", "1", "0xdest", "ETH").await.unwrap();
        }

        let (records, total) = repo.get_history(1, 1, 3).await.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(total, 7);
    }

    #[tokio::test]
    async fn test_get_pending_withdraws() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WithdrawRepository::new(pool);

        let id1 = repo.create(1, "USDC", "100", "1", "0xdest", "ETH").await.unwrap();
        let id2 = repo.create(1, "USDT", "200", "1", "0xdest2", "ETH").await.unwrap();
        repo.update_status(id2, "confirmed", Some("tx_001")).await.unwrap();

        let pending = repo.get_pending(1).await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, id1);
    }

    #[tokio::test]
    async fn test_get_pending_empty() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WithdrawRepository::new(pool);

        let pending = repo.get_pending(1).await.unwrap();
        assert!(pending.is_empty());
    }

    // ==================== Whitelist Repository Tests ====================

    #[tokio::test]
    async fn test_add_whitelist_address() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WhitelistRepository::new(pool);

        repo.add(1, "ETH", "0x1234", Some("my wallet")).await.unwrap();

        let addrs = repo.list(1, "").await.unwrap();
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].address, "0x1234");
        assert_eq!(addrs[0].label, Some("my wallet".to_string()));
    }

    #[tokio::test]
    async fn test_add_whitelist_null_label() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WhitelistRepository::new(pool);

        repo.add(1, "ETH", "0x1234", None).await.unwrap();

        let addrs = repo.list(1, "").await.unwrap();
        assert_eq!(addrs[0].label, None);
    }

    #[tokio::test]
    async fn test_remove_whitelist_address() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WhitelistRepository::new(pool);

        repo.add(1, "ETH", "0x1234", None).await.unwrap();
        repo.remove(1, "0x1234").await.unwrap();

        let addrs = repo.list(1, "").await.unwrap();
        assert!(addrs.is_empty());
    }

    #[tokio::test]
    async fn test_list_whitelist_filter_by_chain() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WhitelistRepository::new(pool);

        repo.add(1, "ETH", "0xeth", None).await.unwrap();
        repo.add(1, "BTC", "1btc", None).await.unwrap();

        let eth_addrs = repo.list(1, "ETH").await.unwrap();
        assert_eq!(eth_addrs.len(), 1);
        assert_eq!(eth_addrs[0].chain, "ETH");

        let all_addrs = repo.list(1, "").await.unwrap();
        assert_eq!(all_addrs.len(), 2);
    }

    #[tokio::test]
    async fn test_is_whitelisted() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = WhitelistRepository::new(pool);

        repo.add(1, "ETH", "0x1234", None).await.unwrap();

        assert!(repo.is_whitelisted(1, "0x1234").await.unwrap());
        assert!(!repo.is_whitelisted(1, "0x9999").await.unwrap());
        assert!(!repo.is_whitelisted(2, "0x1234").await.unwrap());
    }

    // ==================== Payment Password Repository Tests ====================

    #[tokio::test]
    async fn test_set_payment_password() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = PaymentPasswordRepository::new(pool);

        repo.set(1, "hash123").await.unwrap();

        let pw = repo.get(1).await.unwrap().unwrap();
        assert_eq!(pw.password_hash, "hash123");
    }

    #[tokio::test]
    async fn test_upsert_payment_password() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = PaymentPasswordRepository::new(pool);

        repo.set(1, "hash_old").await.unwrap();
        repo.set(1, "hash_new").await.unwrap();

        let pw = repo.get(1).await.unwrap().unwrap();
        assert_eq!(pw.password_hash, "hash_new");
    }

    #[tokio::test]
    async fn test_get_payment_password_not_found() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = PaymentPasswordRepository::new(pool);

        let pw = repo.get(1).await.unwrap();
        assert!(pw.is_none());
    }

    #[tokio::test]
    async fn test_has_payment_password() {
        let pool = create_test_pool().await;
        init_tables(&pool).await;
        let repo = PaymentPasswordRepository::new(pool);

        assert!(!repo.has(1).await.unwrap());

        repo.set(1, "hash123").await.unwrap();

        assert!(repo.has(1).await.unwrap());
    }
}
