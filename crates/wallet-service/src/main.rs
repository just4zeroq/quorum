//! Wallet Service Entry Point

use std::net::SocketAddr;
use tonic::transport::Server;

use wallet_service::services::WalletServiceImpl;
use wallet_service::repository::{
    DepositRepository, WithdrawRepository, WhitelistRepository, PaymentPasswordRepository,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load config
    let config = wallet_service::config::Config::default();

    // Create database pool
    let pool = sqlx::SqlitePool::connect(&config.database.url).await?;

    // Initialize tables
    init_tables(&pool).await?;

    // Create repositories
    let deposit_repo = DepositRepository::new(pool.clone());
    let withdraw_repo = WithdrawRepository::new(pool.clone());
    let whitelist_repo = WhitelistRepository::new(pool.clone());
    let payment_password_repo = PaymentPasswordRepository::new(pool.clone());

    // Create service
    let service = WalletServiceImpl::new(
        deposit_repo,
        withdraw_repo,
        whitelist_repo,
        payment_password_repo,
    );

    // Start gRPC server
    let addr: SocketAddr = format!("{}:{}", config.service.host, config.service.port).parse()?;
    tracing::info!("Wallet service listening on {}", addr);

    Server::builder()
        .add_service(wallet_service::pb::wallet_service_server::WalletServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

async fn init_tables(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    // deposit_addresses
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
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_deposit_addresses_user_id ON deposit_addresses(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_deposit_addresses_user_chain ON deposit_addresses(user_id, chain)")
        .execute(pool)
        .await?;

    // deposit_records
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
    .await?;

    // withdraw_records
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
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_withdraw_records_user_id ON withdraw_records(user_id)")
        .execute(pool)
        .await?;

    // whitelist_addresses
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
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_whitelist_addresses_user_id ON whitelist_addresses(user_id)")
        .execute(pool)
        .await?;

    // payment_passwords
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
    .await?;

    tracing::info!("Wallet tables initialized");
    Ok(())
}
