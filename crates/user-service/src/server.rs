//! User Service gRPC Server

use std::net::SocketAddr;
use std::sync::Arc;
use sqlx::PgPool;

use api::user::user_service_server::UserServiceServer;
use tonic::transport::Server;

use crate::config::Config;
use crate::repository::UserRepository;
use crate::services::{UserServiceImpl, AuthServiceImpl, AuthGrpcServer};

pub struct UserGrpcServer {
    config: Arc<Config>,
    user_service: Arc<UserServiceImpl>,
    auth_service: Option<Arc<AuthServiceImpl>>,
}

impl UserGrpcServer {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(config);
        let repo = Arc::new(UserRepository::new(&config).await?);
        let user_service = Arc::new(UserServiceImpl::new(repo));

        // Create auth service if PostgreSQL is configured
        let auth_service = if let Some(pg_url) = config.db.as_ref()
            .map(|db| db.merge().postgres_url())
            .flatten()
        {
            let pool = PgPool::connect(&pg_url).await?;
            let auth_repo = crate::repository::AuthRepository::new(pool);
            Some(Arc::new(AuthServiceImpl::new(
                auth_repo,
                config.jwt.secret.clone(),
                std::time::Duration::from_secs(config.jwt.expires as u64),
                std::time::Duration::from_secs(config.jwt.refresh_expires as u64),
            )))
        } else {
            tracing::warn!("Auth service disabled: PostgreSQL not configured");
            None
        };

        Ok(Self {
            config,
            user_service,
            auth_service,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("[::1]:{}", self.config.service.port).parse()?;

        tracing::info!("User service gRPC server listening on {}", addr);

        let mut server = Server::builder()
            .add_service(UserServiceServer::from_arc(self.user_service.clone()));

        // Add auth service if available
        if let Some(auth_service) = &self.auth_service {
            server = server.add_service(AuthGrpcServer::from_arc(auth_service.clone()));
            tracing::info!("Auth service enabled");
        }

        server.serve(addr).await?;

        Ok(())
    }
}
