//! Order Service gRPC Server

use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic_reflection::server::Builder;

use crate::config::Config;
use crate::services::OrderServiceImpl;
use crate::pb::order_service_server::OrderServiceServer;

pub struct OrderServer {
    config: Config,
}

impl OrderServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // 连接数据库
        let pool = SqlitePoolOptions::new()
            .max_connections(self.config.database.max_connections as u32)
            .connect(&self.config.database.url)
            .await?;

        // 初始化表
        Self::init_tables(&pool).await?;

        // 创建服务
        let order_service = OrderServiceImpl::new(pool);
        let addr = format!("{}:{}", self.config.service.host, self.config.service.port)
            .parse::<SocketAddr>()?;

        tracing::info!("Starting Order Service on {}", addr);

        // 添加反射服务
        let reflection_service = Builder::configure()
            .register_encoded_file_descriptor_set(include_bytes!("pb/order_service.desc"))
            .build_v1()?;

        // 构建 gRPC 服务器
        Server::builder()
            .add_service(reflection_service)
            .add_service(OrderServiceServer::new(order_service))
            .serve(addr)
            .await?;

        Ok(())
    }

    async fn init_tables(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
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
        .execute(pool)
        .await?;

        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id)")
            .execute(pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_market_id ON orders(market_id)")
            .execute(pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status)")
            .execute(pool)
            .await?;

        // 订单事件表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS order_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                old_status TEXT,
                new_status TEXT NOT NULL,
                filled_quantity TEXT,
                filled_amount TEXT,
                price TEXT,
                reason TEXT,
                created_at INTEGER NOT NULL
            )
            "#
        )
        .execute(pool)
        .await?;

        // 订单事件索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_order_events_order_id ON order_events(order_id)")
            .execute(pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_order_events_created_at ON order_events(created_at)")
            .execute(pool)
            .await?;

        tracing::info!("Order tables initialized");
        Ok(())
    }
}