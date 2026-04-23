//! Auth Service Implementation

use std::sync::Arc;
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Sha256, Digest};
use tonic::{Request, Response, Status};
use std::result::Result as StdResult;

use crate::errors::AuthError;
use crate::models::{AuthContext, JwtClaims, UserSession, ApiKey};
use crate::repository::AuthRepository;
use crate::pb::auth::auth_service_server::AuthService;
use crate::pb::auth::*;

pub struct AuthServiceImpl {
    repo: Arc<AuthRepository>,
    jwt_secret: String,
    access_token_ttl: Duration,
    refresh_token_ttl: Duration,
}

impl AuthServiceImpl {
    pub fn new(
        repo: AuthRepository,
        jwt_secret: String,
        access_token_ttl: Duration,
        refresh_token_ttl: Duration,
    ) -> Self {
        Self {
            repo: Arc::new(repo),
            jwt_secret,
            access_token_ttl,
            refresh_token_ttl,
        }
    }

    /// Hash token using SHA256
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate JWT access token
    fn generate_access_token(&self, user_id: &str, session_id: &str) -> Result<String, AuthError> {
        let now = Utc::now().timestamp();
        let claims = JwtClaims {
            sub: user_id.to_string(),
            sid: session_id.to_string(),
            exp: now + self.access_token_ttl.as_secs() as i64,
            iat: now,
            ttype: "access".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::InternalError(e.to_string()))
    }

    /// Generate JWT refresh token
    fn generate_refresh_token(&self, user_id: &str, session_id: &str) -> Result<String, AuthError> {
        let now = Utc::now().timestamp();
        let claims = JwtClaims {
            sub: user_id.to_string(),
            sid: session_id.to_string(),
            exp: now + self.refresh_token_ttl.as_secs() as i64,
            iat: now,
            ttype: "refresh".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::InternalError(e.to_string()))
    }

    /// Validate JWT token
    fn validate_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(token_data.claims)
    }

    /// Validate API key
    async fn validate_api_key(&self, api_key_id: &str, api_key_secret: &str) -> Result<AuthContext, AuthError> {
        let key_hash = Self::hash_token(api_key_id);
        let api_key = self.repo.get_api_key(&key_hash)
            .await?
            .ok_or(AuthError::InvalidApiKey)?;

        // Verify secret (in production, use constant-time comparison)
        let secret_hash = Self::hash_token(api_key_secret);
        if secret_hash != api_key.secret_hash {
            return Err(AuthError::InvalidApiKey);
        }

        // Update last used
        self.repo.update_api_key_used(&api_key.id).await?;

        Ok(AuthContext::new(api_key.user_id.clone(), api_key.id)
            .with_permissions(api_key.permissions))
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    /// Login with username/password
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> StdResult<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        // In production, verify password hash against stored hash
        // For now, just create session
        let user_id = req.user_id;
        let session_id = uuid::Uuid::new_v4().to_string();

        // Generate tokens
        let access_token = self.generate_access_token(&user_id, &session_id)
            .map_err(|e| Status::internal(e.to_string()))?;
        let refresh_token = self.generate_refresh_token(&user_id, &session_id)
            .map_err(|e| Status::internal(e.to_string()))?;

        // Store session
        let session = UserSession {
            id: session_id.clone(),
            user_id: user_id.clone(),
            token_hash: Self::hash_token(&access_token),
            refresh_token_hash: Some(Self::hash_token(&refresh_token)),
            device_id: None,
            ip_address: None,
            user_agent: None,
            expires_at: Utc::now() + ChronoDuration::from_std(self.access_token_ttl).unwrap(),
            created_at: Utc::now(),
        };

        self.repo.upsert_session(&session)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(LoginResponse {
            access_token,
            refresh_token,
            expires_in: self.access_token_ttl.as_secs() as i64,
            token_type: "Bearer".to_string(),
        }))
    }

    /// Logout - invalidate session
    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> StdResult<Response<LogoutResponse>, Status> {
        let req = request.into_inner();

        self.repo.delete_session(&req.session_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(LogoutResponse { success: true }))
    }

    /// Validate token and get auth context
    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> StdResult<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        match self.validate_token(&req.token) {
            Ok(claims) => {
                // Verify session exists
                let session = self.repo.get_session(&claims.sid)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                if session.is_some() {
                    Ok(Response::new(ValidateTokenResponse {
                        valid: true,
                        user_id: claims.sub,
                        session_id: claims.sid,
                        permissions: vec![],
                    }))
                } else {
                    Ok(Response::new(ValidateTokenResponse {
                        valid: false,
                        user_id: String::new(),
                        session_id: String::new(),
                        permissions: vec![],
                    }))
                }
            }
            Err(_) => Ok(Response::new(ValidateTokenResponse {
                valid: false,
                user_id: String::new(),
                session_id: String::new(),
                permissions: vec![],
            })),
        }
    }

    /// Refresh access token using refresh token
    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> StdResult<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();

        // Validate refresh token
        let claims = self.validate_token(&req.refresh_token)
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        if claims.ttype != "refresh" {
            return Err(Status::unauthenticated("Invalid token type"));
        }

        // Verify session and refresh token hash
        let session = self.repo.get_session(&claims.sid)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::unauthenticated("Session not found"))?;

        let refresh_hash = Self::hash_token(&req.refresh_token);
        if session.refresh_token_hash.as_ref() != Some(&refresh_hash) {
            return Err(Status::unauthenticated("Invalid refresh token"));
        }

        // Generate new access token
        let access_token = self.generate_access_token(&claims.sub, &claims.sid)
            .map_err(|e| Status::internal(e.to_string()))?;

        // Update session token hash
        let mut session = session;
        session.token_hash = Self::hash_token(&access_token);
        self.repo.upsert_session(&session)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RefreshTokenResponse {
            access_token,
            expires_in: self.access_token_ttl.as_secs() as i64,
        }))
    }

    /// Validate API key
    async fn validate_api_key(
        &self,
        request: Request<ValidateApiKeyRequest>,
    ) -> StdResult<Response<ValidateApiKeyResponse>, Status> {
        let req = request.into_inner();

        match self.validate_api_key(&req.api_key_id, &req.api_key_secret).await {
            Ok(ctx) => Ok(Response::new(ValidateApiKeyResponse {
                valid: true,
                user_id: ctx.user_id,
                permissions: ctx.permissions,
            })),
            Err(_) => Ok(Response::new(ValidateApiKeyResponse {
                valid: false,
                user_id: String::new(),
                permissions: vec![],
            })),
        }
    }

    /// Create API key for user
    async fn create_api_key(
        &self,
        request: Request<CreateApiKeyRequest>,
    ) -> StdResult<Response<CreateApiKeyResponse>, Status> {
        let req = request.into_inner();

        let key_id = format!("ak_{}", uuid::Uuid::new_v4());
        let key_secret = format!("sk_{}", uuid::Uuid::new_v4());

        let api_key = ApiKey {
            id: key_id.clone(),
            user_id: req.user_id,
            key_hash: Self::hash_token(&key_id),
            secret_hash: Self::hash_token(&key_secret),
            name: req.name,
            permissions: req.permissions,
            is_active: true,
            last_used_at: None,
            expires_at: None,
            created_at: Utc::now(),
        };

        self.repo.create_api_key(&api_key)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateApiKeyResponse {
            api_key_id: key_id,
            api_key_secret: key_secret,
            created_at: Utc::now().to_rfc3339(),
        }))
    }
}
