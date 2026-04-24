//! Queue Consumer for Market Data
//!
//! 使用 common/queue 包消费市场数据

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

use queue::{ConsumerManager, MergedConfig, ConsumeMessage};
use crate::session::{Channel, SessionManager};

/// 市场数据消息处理器
pub struct MarketDataHandler {
    session_manager: Arc<SessionManager>,
    consumer: Arc<RwLock<Option<ConsumerManager>>>,
    topics: Vec<String>,
}

impl MarketDataHandler {
    /// 创建处理器
    pub fn new(session_manager: Arc<SessionManager>, topics: Vec<String>) -> Self {
        Self {
            session_manager,
            consumer: Arc::new(RwLock::new(None)),
            topics,
        }
    }

    /// 初始化消费者
    pub async fn init(&self, config: MergedConfig) -> Result<(), queue::ConsumerError> {
        let manager = ConsumerManager::new(config, self.topics.clone());
        manager.init().await?;
        let mut guard = self.consumer.write().await;
        *guard = Some(manager);
        Ok(())
    }

    /// 启动消费循环
    pub async fn start(&self) {
        loop {
            let msg = {
                let guard = self.consumer.read().await;
                if let Some(ref consumer) = *guard {
                    match consumer.recv().await {
                        Ok(Some(msg)) => msg,
                        Ok(None) => {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            continue;
                        }
                        Err(e) => {
                            tracing::error!("Consumer error: {}", e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    }
                } else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            };

            // 处理消息
            if let Err(e) = self.process_message(&msg).await {
                tracing::warn!("Failed to process message: {}", e);
            }
        }
    }

    /// 处理消息并广播
    async fn process_message(&self, msg: &ConsumeMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json: serde_json::Value = serde_json::from_str(&msg.value)?;

        // 确定消息类型和推送目标
        let (channel, market_id, message_str) = if let Some(orderbook) = json.get("orderbook") {
            let market_id = orderbook.get("market_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let channel = Channel::OrderBook;
            let msg_str = serde_json::json!({
                "type": "orderbook",
                "market_id": market_id,
                "data": orderbook
            }).to_string();
            (channel, market_id, msg_str)
        } else if let Some(kline) = json.get("kline") {
            let market_id = kline.get("market_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let interval = kline.get("interval").and_then(|v| v.as_str()).unwrap_or("1m");
            let channel = Channel::Kline;
            let msg_str = serde_json::json!({
                "type": "kline",
                "market_id": market_id,
                "interval": interval,
                "data": kline
            }).to_string();
            (channel, market_id, msg_str)
        } else if let Some(trade) = json.get("trade") {
            let market_id = trade.get("market_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let channel = Channel::Trade;
            let msg_str = serde_json::json!({
                "type": "trade",
                "market_id": market_id,
                "data": trade
            }).to_string();
            (channel, market_id, msg_str)
        } else if let Some(ticker) = json.get("ticker") {
            let market_id = ticker.get("market_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let channel = Channel::Ticker;
            let msg_str = serde_json::json!({
                "type": "ticker",
                "market_id": market_id,
                "data": ticker
            }).to_string();
            (channel, market_id, msg_str)
        } else {
            return Ok(());
        };

        // 广播到订阅者
        self.session_manager.broadcast_to_market(&channel, market_id, &message_str).await;

        Ok(())
    }
}
