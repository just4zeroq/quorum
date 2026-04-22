//! Server 模块

use salvo::prelude::*;
use std::sync::Arc;

use crate::config::Config;
use crate::repository::UserRepository;
use crate::services::{UserService, AuthService, WalletService};
use crate::handlers::*;

pub struct Server {
    config: Arc<Config>,
    user_service: Arc<UserService>,
    auth_service: Arc<AuthService>,
    wallet_service: Arc<WalletService>,
}

impl Server {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(config);
        let repo = match UserRepository::new(&config).await {
            Ok(r) => Arc::new(r),
            Err(e) => return Err(e.into()),
        };

        let user_service = Arc::new(UserService::new(repo.clone(), config.clone()));
        let auth_service = Arc::new(AuthService::new(repo.clone(), config.clone()));
        let wallet_service = Arc::new(WalletService::new(repo, config.clone()));

        Ok(Self {
            config,
            user_service,
            auth_service,
            wallet_service,
        })
    }

    pub fn create_router(&self) -> Router {
        Router::new()
            .push(Router::with_path("/api/v1/users")
                .push(Router::with_path("/register").post(register))
                .push(Router::with_path("/login").post(login))
                .push(Router::with_path("/logout").post(logout))
                .push(Router::with_path("/wallet/nonce").post(get_wallet_nonce))
                .push(Router::with_path("/wallet/login").post(wallet_login))
                .push(Router::with_path("/me").get(get_user))
            )
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let router = self.create_router();

        tracing::info!("Starting user-service on port {}", self.config.service.port);

        let acceptor = TcpListener::new(format!("0.0.0.0:{}", self.config.service.port))
            .bind()
            .await;

        // Use Salvo's Server to serve
        salvo::Server::new(acceptor).serve(router).await;

        Ok(())
    }
}