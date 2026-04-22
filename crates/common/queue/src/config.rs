use serde::{Deserialize, Serialize};
use std::path::Path;

/// 消息队列后端类型
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Backend {
    #[default]
    Redis,  // 默认使用 Redis
    Kafka,
}

impl Backend {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "kafka" => Backend::Kafka,
            _ => Backend::Redis,
        }
    }

    pub fn or_else(self, _other: Self) -> Self {
        self
    }
}

/// 消息队列配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// 后端类型: redis, kafka
    pub backend: Option<String>,
    /// Redis 配置
    pub host: Option<String>,
    pub port: Option<u16>,
    pub db: Option<u8>,
    pub password: Option<String>,
    /// Kafka 配置
    pub brokers: Option<Vec<String>>,
    pub topic: Option<String>,
    pub group_id: Option<String>,
    /// 通用配置
    pub connection_timeout_ms: Option<u64>,
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
            .map(|p| p.join("crates/common/queue/config/config.yaml"))
            .unwrap_or_else(|| Path::new("crates/common/queue/config/config.yaml").to_path_buf());

        Self::from_file(&config_path).unwrap_or_default()
    }

    /// 获取后端类型
    pub fn backend(&self) -> Backend {
        self.backend
            .as_deref()
            .map(Backend::from_str)
            .unwrap_or(Backend::Redis)
    }
}

impl Default for Config {
    fn default() -> Self {
        // 默认配置使用 Redis
        Self {
            backend: Some("redis".to_string()),
            host: Some("localhost".to_string()),
            port: Some(6379),
            db: Some(0),
            password: None,
            brokers: None,
            topic: None,
            group_id: None,
            connection_timeout_ms: Some(5000),
        }
    }
}

impl Config {
    /// 合并配置
    pub fn merge(&self) -> MergedConfig {
        let default_config = Self::load_default();
        self.merge_with(&default_config)
    }

    /// 合并配置
    pub fn merge_with(&self, common: &Config) -> MergedConfig {
        let backend = self.backend();

        MergedConfig {
            backend,
            // Redis 配置
            redis_host: self.host.clone().or_else(|| common.host.clone())
                .unwrap_or_else(|| "localhost".to_string()),
            redis_port: self.port.unwrap_or(common.port.unwrap_or(6379)),
            redis_db: self.db.unwrap_or(common.db.unwrap_or(0)),
            redis_password: self.password.clone().or_else(|| common.password.clone()),
            // Kafka 配置
            kafka_brokers: self.brokers.clone().or_else(|| common.brokers.clone())
                .unwrap_or_else(|| vec!["localhost:9092".to_string()]),
            kafka_topic: self.topic.clone().or_else(|| common.topic.clone()),
            kafka_group_id: self.group_id.clone().or_else(|| common.group_id.clone()),
            // 通用配置
            connection_timeout_ms: self.connection_timeout_ms
                .or(common.connection_timeout_ms)
                .unwrap_or(5000),
        }
    }
}

/// 合并后的配置
#[derive(Clone)]
pub struct MergedConfig {
    pub backend: Backend,
    // Redis
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_db: u8,
    pub redis_password: Option<String>,
    // Kafka
    pub kafka_brokers: Vec<String>,
    pub kafka_topic: Option<String>,
    pub kafka_group_id: Option<String>,
    // 通用
    pub connection_timeout_ms: u64,
}

impl MergedConfig {
    /// 获取 Redis URL
    pub fn redis_url(&self) -> String {
        if let Some(ref password) = self.redis_password {
            format!("redis://:{}@{}:{}/{}", password, self.redis_host, self.redis_port, self.redis_db)
        } else {
            format!("redis://{}:{}/{}", self.redis_host, self.redis_port, self.redis_db)
        }
    }

    /// 获取 Kafka broker 字符串
    pub fn kafka_brokers_str(&self) -> String {
        self.kafka_brokers.join(",")
    }
}