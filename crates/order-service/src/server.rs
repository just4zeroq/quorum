//! Order Service gRPC Server

use std::net::SocketAddr;
use tonic::transport::Server;
use tonic_reflection::server::Builder;

use crate::config::Config;
use crate::services::OrderServiceImpl;
use crate::pb::order_service_server::OrderServiceServer;
use crate::kafka_consumer::MatchEventConsumer;
use crate::kafka_producer::OrderCommandProducer;
use db::{DBPool, Config as DBConfig};
use queue::{ConsumerManager, ProducerManager, Config as QueueConfig};
use crate::repository::OrderRepository;

pub struct OrderServer {
    config: Config,
    pool: DBPool,
    kafka_consumer: Option<MatchEventConsumer>,
    kafka_producer: Option<OrderCommandProducer>,
}

impl OrderServer {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // 创建数据库连接池
        let db_config: DBConfig = config.database.clone().into();
        let pool = DBPool::new(&db_config.merge()).await
            .map_err(|e| format!("Failed to create DB pool: {}", e))?;

        // 初始化表
        Self::init_tables(&pool).await?;

        // 初始化 Kafka
        let queue_config = QueueConfig {
            backend: Some(config.queue.backend.clone()),
            host: Some("localhost".to_string()),
            port: Some(6379),
            db: Some(0),
            password: None,
            brokers: Some(config.queue.brokers.clone()),
            topic: Some("order.commands".to_string()),
            group_id: Some(config.queue.group_id.clone()),
            connection_timeout_ms: Some(5000),
        };
        let merged_config = queue_config.merge();

        // 创建 Kafka Producer
        let producer = ProducerManager::new(merged_config.clone());
        producer.init().await.map_err(|e| format!("Failed to init producer: {}", e))?;
        let kafka_producer = OrderCommandProducer::new(producer);

        // 创建 Kafka Consumer
        let consumer = ConsumerManager::new(merged_config, vec!["match.events".to_string()]);
        consumer.init().await.map_err(|e| format!("Failed to init consumer: {}", e))?;
        let order_repo = OrderRepository::new(pool.clone());
        let kafka_consumer = MatchEventConsumer::new(consumer, order_repo);

        Ok(Self {
            config,
            pool,
            kafka_consumer: Some(kafka_consumer),
            kafka_producer: Some(kafka_producer),
        })
    }

    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 创建服务
        let order_service = OrderServiceImpl::new(self.pool.clone());

        let addr = format!("{}:{}", self.config.service.host, self.config.service.port)
            .parse::<SocketAddr>()?;

        tracing::info!("Starting Order Service on {}", addr);

        // 添加反射服务
        let reflection_service = Builder::configure()
            .register_encoded_file_descriptor_set(include_bytes!("pb/order_service.desc"))
            .build_v1()?;

        // 启动 Kafka 消费者
        if let Some(consumer) = self.kafka_consumer.take() {
            let handle = tokio::spawn(async move {
                consumer.start().await;
            });
            tracing::info!("Kafka consumer started for match.events");
        }

        // 构建 gRPC 服务器
        Server::builder()
            .add_service(reflection_service)
            .add_service(OrderServiceServer::new(order_service))
            .serve(addr)
            .await?;

        Ok(())
    }

    async fn init_tables(pool: &DBPool) -> Result<(), String> {
        let sqlite_pool = pool.sqlite_pool()
            .ok_or_else(|| "Not a SQLite pool".to_string())?;

        // 创建订单表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS orders (
                id TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                market_id INTEGER NOT NULL,
                outcome_id INTEGER NOT NULL,
                side TEXT NOT NULL,
                order_type TEXT NOT NULL,
                price TEXT NOT NULL,
                quantity TEXT NOT NULL,
                filled_quantity TEXT NOT NULL DEFAULT '0',
                filled_amount TEXT NOT NULL DEFAULT '0',
                status TEXT NOT NULL DEFAULT 'pending',
                client_order_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#
        )
        .execute(sqlite_pool)
        .await
        .map_err(|e| e.to_string())?;

        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id)")
            .execute(sqlite_pool)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_market_id ON orders(market_id)")
            .execute(sqlite_pool)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status)")
            .execute(sqlite_pool)
            .await
            .map_err(|e| e.to_string())?;

        tracing::info!("Order tables initialized");
        Ok(())
    }
}
