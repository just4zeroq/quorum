use serde::{Deserialize, Serialize};
use std::path::Path;

/// Redis 缓存配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Redis 主机地址
    pub host: Option<String>,
    /// Redis 端口
    pub port: Option<u16>,
    /// 密码
    pub password: Option<String>,
    /// 数据库编号
    pub db: Option<u8>,
    /// 连接池大小
    pub pool_size: Option<u32>,
    /// 连接超时 (毫秒)
    pub connect_timeout_ms: Option<u64>,
    /// 读写超时 (毫秒)
    pub read_timeout_ms: Option<u64>,
}

impl Config {
    /// 从指定路径加载配置文件
    pub fn from_file<P: AsRef<Path>>(path: P) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_yaml::from_str(&content).ok()
    }

    /// 加载组件默认配置文件
    pub fn load_default() -> Self {
        let config_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("crates/common/cache/config/config.yaml"))
            .unwrap_or_else(|| Path::new("crates/common/cache/config/config.yaml").to_path_buf());

        Self::from_file(&config_path).unwrap_or_default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: Some("localhost".to_string()),
            port: Some(6379),
            db: Some(0),
            password: None,
            pool_size: Some(10),
            connect_timeout_ms: Some(5000),
            read_timeout_ms: Some(3000),
        }
    }
}

impl Config {
    /// 合并配置
    /// 优先级: 服务传入的配置 > 组件默认配置文件 > 硬编码默认
    pub fn merge(&self) -> MergedConfig {
        let default_config = Self::load_default();
        self.merge_with(&default_config)
    }

    /// 合并配置 (服务配置优先，没有则用传入的配置)
    pub fn merge_with(&self, common: &Config) -> MergedConfig {
        MergedConfig {
            host: self
                .host
                .clone()
                .or_else(|| common.host.clone())
                .unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(common.port.unwrap_or(6379)),
            password: self.password.clone().or_else(|| common.password.clone()),
            db: self.db.unwrap_or(common.db.unwrap_or(0)),
            pool_size: self.pool_size.unwrap_or(common.pool_size.unwrap_or(10)),
            connect_timeout_ms: self
                .connect_timeout_ms
                .unwrap_or(common.connect_timeout_ms.unwrap_or(5000)),
            read_timeout_ms: self
                .read_timeout_ms
                .unwrap_or(common.read_timeout_ms.unwrap_or(3000)),
        }
    }
}

/// 合并后的 Redis 配置
pub struct MergedConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
    pub pool_size: u32,
    pub connect_timeout_ms: u64,
    pub read_timeout_ms: u64,
}

impl MergedConfig {
    /// 获取 Redis 连接地址
    pub fn addr(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }

    /// 获取 Redis 连接 URL (带密码)
    pub fn url(&self) -> String {
        if let Some(ref password) = self.password {
            format!("redis://:{}@{}:{}/{}", password, self.host, self.port, self.db)
        } else {
            format!("redis://{}:{}/{}", self.host, self.port, self.db)
        }
    }
}