//! 配置模块

use serde::{Deserialize, Serialize};

/// 服务配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// 服务配置
    pub service: ServiceConfig,
    /// 数据库配置 (覆盖公共配置)
    pub db: Option<db::Config>,
    /// 缓存配置 (覆盖公共配置)
    pub cache: Option<cache::Config>,
    /// JWT 配置
    pub jwt: JwtConfig,
    /// 安全配置
    pub security: SecurityConfig,
}

/// 服务元信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    pub name: String,
    pub port: u16,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "user-service".to_string(),
            port: 50001,
        }
    }
}

/// JWT 配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    /// 密钥
    pub secret: String,
    /// Token 过期时间 (秒)
    pub expires: i64,
    /// Refresh Token 过期时间 (秒)
    pub refresh_expires: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "change-me-in-production".to_string(),
            expires: 604800,      // 7天
            refresh_expires: 2592000, // 30天
        }
    }
}

/// 安全配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    /// 登录失败最大次数
    pub login_max_failures: i32,
    /// 登录锁定时长 (秒)
    pub login_lock_duration: i64,
    /// 钱包 nonce 过期时间 (秒)
    pub wallet_nonce_expires: i64,
    /// 密码最小长度
    pub password_min_length: i32,
    /// 是否需要大写字母
    pub password_require_uppercase: bool,
    /// 是否需要小写字母
    pub password_require_lowercase: bool,
    /// 是否需要数字
    pub password_require_digit: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            login_max_failures: 5,
            login_lock_duration: 900,     // 15分钟
            wallet_nonce_expires: 300,    // 5分钟
            password_min_length: 8,
            password_require_uppercase: true,
            password_require_lowercase: true,
            password_require_digit: true,
        }
    }
}

impl Config {
    /// 从配置文件加载
    pub fn load(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 加载并合并配置
    pub fn load_merged(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::load(config_path)?;

        // 合并数据库配置
        if config.db.is_none() {
            config.db = Some(db::Config::default());
        }

        // 合并缓存配置
        if config.cache.is_none() {
            config.cache = Some(cache::Config::default());
        }

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service: ServiceConfig::default(),
            db: None,
            cache: None,
            jwt: JwtConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}