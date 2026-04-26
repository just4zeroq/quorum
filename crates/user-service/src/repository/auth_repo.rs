//! Auth Repository

use sqlx::PgPool;
use crate::errors::AuthError;
use crate::models::auth::{UserSession, ApiKey};

pub struct AuthRepository {
    pool: PgPool,
}

impl AuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create or update session
    pub async fn upsert_session(&self, session: &UserSession) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, token_hash, refresh_token_hash, device_id, ip_address, user_agent, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                token_hash = EXCLUDED.token_hash,
                refresh_token_hash = EXCLUDED.refresh_token_hash
            "#,
        )
        .bind(&session.id)
        .bind(&session.user_id)
        .bind(&session.token_hash)
        .bind(&session.refresh_token_hash)
        .bind(&session.device_id)
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .bind(&session.expires_at)
        .bind(&session.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<UserSession>, AuthError> {
        let session = sqlx::query_as::<_, UserSession>(
            "SELECT * FROM user_sessions WHERE id = $1 AND expires_at > NOW()",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(session)
    }

    /// Get session by token hash
    pub async fn get_session_by_token(&self, token_hash: &str) -> Result<Option<UserSession>, AuthError> {
        let session = sqlx::query_as::<_, UserSession>(
            "SELECT * FROM user_sessions WHERE token_hash = $1 AND expires_at > NOW()",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(session)
    }

    /// Delete session
    pub async fn delete_session(&self, session_id: &str) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM user_sessions WHERE id = $1")
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Delete all user sessions
    pub async fn delete_user_sessions(&self, user_id: &str) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Create API key
    pub async fn create_api_key(&self, api_key: &ApiKey) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (id, user_id, key_hash, secret_hash, name, permissions, is_active, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&api_key.id)
        .bind(&api_key.user_id)
        .bind(&api_key.key_hash)
        .bind(&api_key.secret_hash)
        .bind(&api_key.name)
        .bind(&api_key.permissions)
        .bind(api_key.is_active)
        .bind(&api_key.expires_at)
        .bind(&api_key.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get API key by key hash
    pub async fn get_api_key(&self, key_hash: &str) -> Result<Option<ApiKey>, AuthError> {
        let api_key = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true AND (expires_at IS NULL OR expires_at > NOW())",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(api_key)
    }

    /// Update API key last used
    pub async fn update_api_key_used(&self, key_id: &str) -> Result<(), AuthError> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Delete API key
    pub async fn delete_api_key(&self, key_id: &str) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}