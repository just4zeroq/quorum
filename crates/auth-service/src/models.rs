//! Auth Service Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// User session stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub device_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// API Key stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: String,
    pub user_id: String,
    pub key_hash: String,
    pub secret_hash: String,
    pub name: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,           // user_id
    pub sid: String,           // session_id
    pub exp: i64,              // expiration time
    pub iat: i64,              // issued at
    pub ttype: String,         // token type: access/refresh
}

/// Auth context passed to services
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub session_id: String,
    pub permissions: Vec<String>,
}

impl AuthContext {
    pub fn new(user_id: String, session_id: String) -> Self {
        Self {
            user_id,
            session_id,
            permissions: vec![],
        }
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission || p == "*")
    }
}
