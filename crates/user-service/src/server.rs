//! User Service gRPC Server

use std::net::SocketAddr;
use std::sync::Arc;

use api::user::user_service_server::UserServiceServer;
use tonic::transport::Server;

use crate::config::Config;
use crate::repository::UserRepository;
use crate::services::UserServiceImpl;

pub struct UserGrpcServer {
    config: Arc<Config>,
    user_service: Arc<UserServiceImpl>,
}

impl UserGrpcServer {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(config);
        let repo = Arc::new(UserRepository::new(&config).await?);
        let user_service = Arc::new(UserServiceImpl::new(repo));

        Ok(Self {
            config,
            user_service,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("[::1]:{}", self.config.service.port).parse()?;

        tracing::info!("User service gRPC server listening on {}", addr);

        Server::builder()
            .add_service(UserServiceServer::from_arc(self.user_service.clone()))
            .serve(addr)
            .await?;

        Ok(())
    }
}
