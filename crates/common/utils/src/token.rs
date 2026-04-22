//! Token generation and validation - Token 生成与验证

use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Token 错误
#[derive(Error, Debug)]
pub enum TokenError {
    #[error("Token 生成失败: {0}")]
    GenerateError(String),
    #[error("Token 验证失败: {0}")]
    ValidateError(String),
    #[error("Token 解析失败: {0}")]
    ParseError(String),
    #[error("Token 已过期")]
    Expired,
}

/// Token 载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 用户ID
    pub sub: String,
    /// 邮箱
    pub email: Option<String>,
    /// 登录方式
    pub login_method: String,
    /// 签发时间
    pub iat: i64,
    /// 过期时间
    pub exp: i64,
    /// Token 类型: access / refresh
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
}

impl Claims {
    /// 创建 Access Token 载荷
    pub fn new_access(user_id: i64, email: Option<String>, login_method: &str, expires_in: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            email,
            login_method: login_method.to_string(),
            iat: now,
            exp: now + expires_in,
            token_type: Some("access".to_string()),
        }
    }

    /// 创建 Refresh Token 载荷
    pub fn new_refresh(user_id: i64, expires_in: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            email: None,
            login_method: "refresh".to_string(),
            iat: now,
            exp: now + expires_in,
            token_type: Some("refresh".to_string()),
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() >= self.exp
    }
}

/// Token 生成器
pub struct TokenGenerator {
    secret: String,
    algorithm: Algorithm,
}

impl TokenGenerator {
    /// 创建 Token 生成器
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            algorithm: Algorithm::HS256,
        }
    }

    /// 生成 Access Token
    pub fn generate_access(
        &self,
        user_id: i64,
        email: Option<String>,
        login_method: &str,
        expires_in_seconds: i64,
    ) -> Result<String, TokenError> {
        let claims = Claims::new_access(user_id, email, login_method, expires_in_seconds);
        self.generate_token(claims)
    }

    /// 生成 Refresh Token
    pub fn generate_refresh(&self, user_id: i64, expires_in_seconds: i64) -> Result<String, TokenError> {
        let claims = Claims::new_refresh(user_id, expires_in_seconds);
        self.generate_token(claims)
    }

    /// 生成 Token
    fn generate_token(&self, claims: Claims) -> Result<String, TokenError> {
        let header = Header::new(self.algorithm);
        let encoding_key = EncodingKey::from_secret(self.secret.as_bytes());

        encode(&header, &claims, &encoding_key)
            .map_err(|e| TokenError::GenerateError(e.to_string()))
    }

    /// 验证并解析 Token
    pub fn verify(&self, token: &str) -> Result<Claims, TokenError> {
        let validation = Validation::new(self.algorithm);
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());

        let token_data: TokenData<Claims> =
            decode(token, &decoding_key, &validation)
                .map_err(|e| match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
                    _ => TokenError::ValidateError(e.to_string()),
                })?;

        Ok(token_data.claims)
    }

    /// 验证并解析 Token (带自定义验证)
    #[allow(dead_code)]
    pub fn verify_with_validation(&self, token: &str, mut validation: Validation) -> Result<Claims, TokenError> {
        validation.validate_exp = true;
        validation.set_required_spec_claims(&["sub", "exp", "iat"]);
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());

        let token_data: TokenData<Claims> =
            decode(token, &decoding_key, &validation)
                .map_err(|e| match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
                    _ => TokenError::ValidateError(e.to_string()),
                })?;

        Ok(token_data.claims)
    }
}

/// Token 验证器 (只读)
pub struct TokenValidator {
    secret: String,
    algorithm: Algorithm,
}

impl TokenValidator {
    /// 创建 Token 验证器
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            algorithm: Algorithm::HS256,
        }
    }

    /// 验证 Token
    pub fn verify(&self, token: &str) -> Result<Claims, TokenError> {
        let validation = Validation::new(self.algorithm);
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());

        let token_data: TokenData<Claims> =
            decode(token, &decoding_key, &validation)
                .map_err(|e| match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
                    _ => TokenError::ValidateError(e.to_string()),
                })?;

        Ok(token_data.claims)
    }

    /// 从 Token 中提取用户ID
    pub fn get_user_id(&self, token: &str) -> Result<i64, TokenError> {
        let claims = self.verify(token)?;
        claims.sub.parse().map_err(|_| TokenError::ParseError("Invalid user_id".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generate_and_verify() {
        let generator = TokenGenerator::new("test_secret_key_12345");

        // 生成 Access Token
        let token = generator
            .generate_access(123, Some("test@example.com".to_string()), "email", 3600)
            .unwrap();
        assert!(!token.is_empty());

        // 验证 Token
        let validator = TokenValidator::new("test_secret_key_12345");
        let claims = validator.verify(&token).unwrap();

        assert_eq!(claims.sub, "123");
        assert_eq!(claims.email, Some("test@example.com".to_string()));
        assert_eq!(claims.login_method, "email");
        assert_eq!(claims.token_type, Some("access".to_string()));
    }

    #[test]
    fn test_token_expired() {
        let generator = TokenGenerator::new("test_secret_key_12345");

        // 生成已过期的 Token (exp = iat - 3600, 1小时前过期)
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: "123".to_string(),
            email: None,
            login_method: "email".to_string(),
            iat: now,
            exp: now - 3600, // 1小时前过期
            token_type: Some("access".to_string()),
        };
        let token = generator.generate_token(claims).unwrap();

        let validator = TokenValidator::new("test_secret_key_12345");
        let result = validator.verify(&token);
        assert!(matches!(result, Err(TokenError::Expired)));
    }

    #[test]
    fn test_refresh_token() {
        let generator = TokenGenerator::new("test_secret_key_12345");

        // 生成 Refresh Token
        let token = generator.generate_refresh(123, 604800).unwrap(); // 7 days

        let validator = TokenValidator::new("test_secret_key_12345");
        let claims = validator.verify(&token).unwrap();

        assert_eq!(claims.sub, "123");
        assert_eq!(claims.token_type, Some("refresh".to_string()));
    }
}