//! Portfolio Service Entry Point

use db::{DBManager, Config, MigrationRunner};
use registry::ServiceRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Load config
    let config = portfolio_service::config::Config::default();

    // Load database config
    let db_config = Config::load_default().merge();

    // Init connection pool
    let db_manager = DBManager::new(db_config);
    db_manager.init().await.map_err(|e| format!("DB init failed: {}", e))?;

    let db_pool = db_manager.get_pool().await.ok_or("Failed to get DB pool")?;

    // Run migrations
    let migrations_dir = std::path::PathBuf::from(
        std::env::var("MIGRATIONS_DIR")
            .unwrap_or_else(|_| "crates/migrations/portfolio".to_string()),
    );
    MigrationRunner::run_migrations(&db_pool, &migrations_dir).await
        .map_err(|e| format!("Migration failed: {}", e))?;

    tracing::info!("Portfolio Service migrations applied successfully");

    // Create repository and service
    let repo = portfolio_service::repository::PortfolioRepository::from_db_pool(db_pool);
    let portfolio_svc = portfolio_service::service::PortfolioServiceImpl::new(repo);

    // Start gRPC server
    let addr: std::net::SocketAddr = format!("{}:{}", config.service.host, config.service.port).parse()?;
    tracing::info!("Portfolio service listening on {}", addr);

    // Register to etcd
    let registry = ServiceRegistry::new(
        "portfolio-service",
        &format!("http://{}", addr),
        &config.etcd_endpoints,
    )
    .await
    .map_err(|e| format!("Failed to create service registry: {}", e))?;
    registry
        .register(30)
        .await
        .map_err(|e| format!("Failed to register service: {}", e))?;
    let _heartbeat_handle = registry.clone().start_heartbeat(30, 10);

    tonic::transport::Server::builder()
        .add_service(api::portfolio::portfolio_service_server::PortfolioServiceServer::new(portfolio_svc))
        .serve(addr)
        .await?;

    Ok(())
}
