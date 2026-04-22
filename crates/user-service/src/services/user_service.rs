//! User Service

use std::sync::Arc;
use crate::config::Config;
use crate::repository::UserRepository;

pub struct UserService {
    _repo: Arc<UserRepository>,
    _config: Arc<Config>,
}

impl UserService {
    pub fn new(repo: Arc<UserRepository>, config: Arc<Config>) -> Self {
        Self { _repo: repo, _config: config }
    }
}