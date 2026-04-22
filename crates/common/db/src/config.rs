use serde::{Deserialize, Serialize};
use std::path::Path;

/// 数据库配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// 数据库类型: sqlite, postgres
    pub db_type: Option<String>,
    /// PostgreSQL 配置
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
    /// 数据库名 (业务配置，服务传入)
    pub database: Option<String>,
    /// 连接池配置
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connection_timeout_ms: Option<u64>,
    pub idle_timeout_ms: Option<u64>,
    /// SQLite 配置 (文件路径)
    pub file_path: Option<String>,
}

impl Config {
    /// 从指定路径加载配置文件
    pub fn from_file<P: AsRef<Path>>(path: P) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_yaml::from_str(&content).ok()
    }

    /// 加载组件默认配置文件
    pub fn load_default() -> Self {
        // 默认读取组件目录下的 config.yaml
        let config_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("crates/common/db/config/config.yaml"))
            .unwrap_or_else(|| Path::new("crates/common/db/config/config.yaml").to_path_buf());

        Self::from_file(&config_path).unwrap_or_default()
    }
}

impl Default for Config {
    fn default() -> Self {
        // Return empty config without recursion
        Self {
            db_type: Some("postgres".to_string()),
            host: Some("localhost".to_string()),
            port: Some(5432),
            user: Some("postgres".to_string()),
            password: Some("postgres".to_string()),
            database: Some("cex_dex".to_string()),
            max_connections: Some(100),
            min_connections: Some(5),
            connection_timeout_ms: Some(5000),
            idle_timeout_ms: Some(600000),
            file_path: None,
        }
    }
}

impl Config {
    /// 合并配置
    /// 优先级: 服务传入的配置 > 组件默认配置文件 > 硬编码默认
    pub fn merge(&self) -> MergedConfig {
        // 先获取组件默认配置
        let default_config = Self::load_default();

        // 服务配置覆盖默认配置
        self.merge_with(&default_config)
    }

    /// 合并配置 (服务配置优先，没有则用传入的公共配置)
    pub fn merge_with(&self, common: &Config) -> MergedConfig {
        let db_type = self
            .db_type
            .clone()
            .or_else(|| common.db_type.clone())
            .unwrap_or_else(|| "postgres".to_string());

        MergedConfig {
            db_type: db_type.clone(),
            host: self.host.clone().or_else(|| common.host.clone()),
            port: self.port.unwrap_or(common.port.unwrap_or(5432)),
            user: self.user.clone().or_else(|| common.user.clone()),
            password: self.password.clone().or_else(|| common.password.clone()),
            database: self.database.clone().or_else(|| common.database.clone()),
            max_connections: self.max_connections.unwrap_or(common.max_connections.unwrap_or(100)),
            min_connections: self.min_connections.unwrap_or(common.min_connections.unwrap_or(5)),
            connection_timeout_ms: self
                .connection_timeout_ms
                .unwrap_or(common.connection_timeout_ms.unwrap_or(5000)),
            idle_timeout_ms: self
                .idle_timeout_ms
                .unwrap_or(common.idle_timeout_ms.unwrap_or(600000)),
            file_path: self.file_path.clone().or_else(|| common.file_path.clone()),
            is_sqlite: db_type == "sqlite",
        }
    }
}

/// 合并后的数据库配置
pub struct MergedConfig {
    pub db_type: String,
    pub host: Option<String>,
    pub port: u16,
    pub user: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_ms: u64,
    pub idle_timeout_ms: u64,
    pub file_path: Option<String>,
    pub is_sqlite: bool,
}

impl MergedConfig {
    /// 获取 PostgreSQL 连接字符串
    pub fn postgres_url(&self) -> Option<String> {
        if self.is_sqlite {
            return None;
        }
        let host = self.host.as_ref()?;
        let user = self.user.as_ref()?;
        let password = self.password.as_ref()?;
        let database = self.database.as_ref()?;

        Some(format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, self.port, database
        ))
    }

    /// 获取 SQLite 连接字符串
    pub fn sqlite_url(&self) -> Option<String> {
        if !self.is_sqlite {
            return None;
        }
        let file_path = self.file_path.as_ref()?;
        Some(format!("sqlite:{}?mode=rwc", file_path))
    }

    /// 获取数据库连接 URL
    pub fn url(&self) -> Option<String> {
        if self.is_sqlite {
            self.sqlite_url()
        } else {
            self.postgres_url()
        }
    }
}