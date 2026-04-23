//! Auth Service Entry Point

use std::time::Duration;
use auth_service::AuthServiceImpl;
use auth_service::repository::AuthRepository;
use auth_service::pb::auth::auth_service_server::AuthServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::load()?;

    // Create database pool
    let pool = sqlx::PgPool::connect(&config.database_url).await?;

    // Create repository
    let repo = AuthRepository::new(pool);

    // Create service
    let service = AuthServiceImpl::new(
        repo,
        config.jwt_secret,
        Duration::from_secs(config.access_token_ttl as u64),
        Duration::from_secs(config.refresh_token_ttl as u64),
    );

    // Start gRPC server
    let addr = format!("[::1]:{}", config.port).parse()?;
    tracing::info!("Auth service listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(AuthServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

struct Config {
    database_url: String,
    jwt_secret: String,
    port: u16,
    access_token_ttl: i64,
    refresh_token_ttl: i64,
}

impl Config {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost:5432/auth".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "50010".to_string())
                .parse()?,
            access_token_ttl: std::env::var("ACCESS_TOKEN_TTL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()?,
            refresh_token_ttl: std::env::var("REFRESH_TOKEN_TTL")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()?,
        })
    }
}
