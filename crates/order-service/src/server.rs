//! Order Service gRPC Server

use std::net::SocketAddr;
use tonic::transport::Server;
use registry::ServiceRegistry;

use crate::config::Config;
use crate::services::OrderServiceImpl;
use api::order::order_service_server::OrderServiceServer;
use crate::queue_consumer::MatchEventConsumer;
use crate::queue_producer::OrderCommandProducer;
use db::{DBPool, Config as DBConfig};
use queue::{ConsumerManager, ProducerManager, Config as QueueConfig};
use crate::repository::OrderRepository;

pub struct OrderServer {
    config: Config,
    pool: DBPool,
    queue_consumer: Option<MatchEventConsumer>,
    queue_producer: Option<OrderCommandProducer>,
}

impl OrderServer {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // 创建数据库连接池
        let db_config: DBConfig = config.database.clone().into();
        let pool = DBPool::new(&db_config.merge()).await
            .map_err(|e| format!("Failed to create DB pool: {}", e))?;

        // 初始化表
        Self::init_tables(&pool).await?;

        // 初始化队列
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

        // 创建 Queue Producer
        let producer = ProducerManager::new(merged_config.clone());
        producer.init().await.map_err(|e| format!("Failed to init producer: {}", e))?;
        let queue_producer = OrderCommandProducer::new(producer);

        // 创建 Queue Consumer (for match.events)
        let consumer = ConsumerManager::new(merged_config.clone(), vec!["match.events".to_string()]);
        consumer.init().await.map_err(|e| format!("Failed to init consumer: {}", e))?;
        let order_repo = OrderRepository::new(pool.clone());

        // 创建 order_events producer
        let event_producer = ProducerManager::new(merged_config);
        event_producer.init().await.map_err(|e| format!("Failed to init event producer: {}", e))?;

        let queue_consumer = MatchEventConsumer::new(consumer, order_repo)
            .with_event_producer(event_producer);

        Ok(Self {
            config,
            pool,
            queue_consumer: Some(queue_consumer),
            queue_producer: Some(queue_producer),
        })
    }

    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 创建服务，注入 Queue Producer
        let order_service = if let Some(producer) = self.queue_producer.take() {
            OrderServiceImpl::new(self.pool.clone()).with_queue_producer(producer)
        } else {
            OrderServiceImpl::new(self.pool.clone())
        };

        let addr = format!("{}:{}", self.config.service.host, self.config.service.port)
            .parse::<SocketAddr>()?;

        tracing::info!("Starting Order Service on {}", addr);

        // 注册到 etcd
        let registry = ServiceRegistry::new(
            "order-service",
            &format!("http://{}", addr),
            &self.config.etcd_endpoints,
        ).await?;

        registry.register(30).await?;
        let heartbeat_handle = registry.clone().start_heartbeat(30, 10);

        tracing::info!("Order service registered to etcd");

        // 启动 Queue 消费者
        if let Some(mut consumer) = self.queue_consumer.take() {
            let _handle = tokio::spawn(async move {
                consumer.start().await;
            });
            tracing::info!("Queue consumer started for match.events");
        }

        // 构建 gRPC 服务器
        Server::builder()
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
