//! Wallet Service

use std::sync::Arc;
use crate::config::Config;
use crate::repository::UserRepository;

pub struct WalletService {
    _repo: Arc<UserRepository>,
    _config: Arc<Config>,
}

impl WalletService {
    pub fn new(repo: Arc<UserRepository>, config: Arc<Config>) -> Self {
        Self { _repo: repo, _config: config }
    }
}