//! Wallet Repository (placeholder)

use async_trait::async_trait;
use crate::models::wallet::{WalletAddress, WalletType, ChainType};
use crate::models::session::UserSession;
use crate::models::settings::UserSettings;
use crate::models::risk::UserRisk;

pub struct WalletRepository;

impl WalletRepository {
    pub async fn new() -> Self { Self }
}

pub struct SessionRepository;

impl SessionRepository {
    pub async fn new() -> Self { Self }
}

pub struct SettingsRepository;

impl SettingsRepository {
    pub async fn new() -> Self { Self }
}

pub struct RiskRepository;

impl RiskRepository {
    pub async fn new() -> Self { Self }
}

pub struct TagRepository;

impl TagRepository {
    pub async fn new() -> Self { Self }
}