//! Queue Consumer - 消费撮合事件
//!
//! 从 match.events 主题消费撮合结果，更新订单状态并调用 Portfolio.SettleTrade 完成清算

use queue::{ConsumerManager, ProducerManager, Message};
use crate::repository::OrderRepository;
use crate::models::OrderStatus;
use rust_decimal::Decimal;
use tonic::transport::Channel;
use tracing::{info, error};

/// ID 前缀常量
const ORDER_ID_PREFIX: &str = "ord_";
const USER_ID_PREFIX: &str = "usr_";

/// 撮合事件（从 matching-engine 接收，包含 taker/maker 上下文）
#[derive(serde::Deserialize, Debug)]
struct MatchEvent {
    event_type: String,
    size: i64,
    price: i64,
    matched_order_id: u64,
    matched_order_uid: u64,
    #[allow(dead_code)]
    bidder_hold_price: i64,
    taker_order_id: Option<u64>,
    taker_uid: Option<u64>,
    symbol: Option<i32>,
    taker_action: Option<String>,
}

/// 撮合事件消费者
pub struct MatchEventConsumer {
    consumer: ConsumerManager,
    order_repo: OrderRepository,
    event_producer: Option<ProducerManager>,
    /// Portfolio Service gRPC 客户端
    portfolio_client: Option<api::portfolio::portfolio_service_client::PortfolioServiceClient<Channel>>,
}

impl MatchEventConsumer {
    pub fn new(consumer: ConsumerManager, order_repo: OrderRepository) -> Self {
        Self {
            consumer,
            order_repo,
            event_producer: None,
            portfolio_client: None,
        }
    }

    /// 设置事件生产者
    pub fn with_event_producer(mut self, producer: ProducerManager) -> Self {
        self.event_producer = Some(producer);
        self
    }

    /// 设置 Portfolio 客户端
    pub fn with_portfolio_client(
        mut self,
        client: api::portfolio::portfolio_service_client::PortfolioServiceClient<Channel>,
    ) -> Self {
        self.portfolio_client = Some(client);
        self
    }

    /// 启动消费
    pub async fn start(&mut self) {
        info!("MatchEventConsumer starting...");

        loop {
            match self.consumer.recv().await {
                Ok(Some(msg)) => {
                    if let Err(e) = self.process_message(&msg.value).await {
                        error!("Failed to process message: {}", e);
                    }
                }
                Ok(None) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Err(e) => {
                    error!("Failed to receive message: {}", e);
                }
            }
        }
    }

    /// 处理消息
    async fn process_message(&mut self, data: &str) -> Result<(), String> {
        let event: MatchEvent = serde_json::from_str(data)
            .map_err(|e| format!("Failed to parse event: {}", e))?;

        match event.event_type.as_str() {
            "Trade" => {
                info!("Processing trade event: matched_order_id={}, taker_uid={:?}, symbol={:?}",
                    event.matched_order_id, event.taker_uid, event.symbol);

                // 1. 更新 maker 订单状态
                let filled_quantity = event.size.to_string();
                let filled_amount = (event.size * event.price).to_string();
                let db_order_id = format!("{}{}", ORDER_ID_PREFIX, event.matched_order_id);
                self.order_repo.update_status(
                    &db_order_id,
                    &OrderStatus::Filled.to_string(),
                    &filled_quantity,
                    &filled_amount,
                ).await.map_err(|e| e.to_string())?;

                // 2. 发布 order_events
                self.publish_order_event(
                    &db_order_id,
                    event.matched_order_uid as i64,
                    "filled",
                    &filled_quantity,
                    &filled_amount,
                    &event.price.to_string(),
                ).await;

                // 3. 调用 Portfolio.SettleTrade 完成清算
                if let Some(ref mut client) = self.portfolio_client {
                    if let Err(e) = Self::settle_trade(client, &event).await {
                        error!("Portfolio.SettleTrade failed: {}", e);
                    }
                } else {
                    info!("No portfolio client configured, skipping SettleTrade");
                }
            }
            "Reject" => {
                info!("Processing reject event for order {}", event.matched_order_id);
                let db_order_id = format!("{}{}", ORDER_ID_PREFIX, event.matched_order_id);
                self.order_repo.update_status(
                    &db_order_id,
                    &OrderStatus::Rejected.to_string(),
                    "0",
                    "0",
                ).await.map_err(|e| e.to_string())?;

                self.publish_order_event(
                    &db_order_id,
                    event.matched_order_uid as i64,
                    "rejected",
                    "0",
                    "0",
                    &event.price.to_string(),
                ).await;
            }
            "Reduce" => {
                info!("Processing reduce event for order {}", event.matched_order_id);
                let filled_quantity = event.size.to_string();
                let filled_amount = (event.size * event.price).to_string();
                let db_order_id = format!("{}{}", ORDER_ID_PREFIX, event.matched_order_id);
                self.order_repo.update_status(
                    &db_order_id,
                    &OrderStatus::PartiallyFilled.to_string(),
                    &filled_quantity,
                    &filled_amount,
                ).await.map_err(|e| e.to_string())?;

                self.publish_order_event(
                    &db_order_id,
                    event.matched_order_uid as i64,
                    "partially_filled",
                    &filled_quantity,
                    &filled_amount,
                    &event.price.to_string(),
                ).await;
            }
            _ => {
                info!("Unknown event type: {}", event.event_type);
            }
        }

        Ok(())
    }

    /// 调用 Portfolio.SettleTrade 完成一笔成交的清算
    async fn settle_trade(
        client: &mut api::portfolio::portfolio_service_client::PortfolioServiceClient<Channel>,
        event: &MatchEvent,
    ) -> Result<(), String> {
        // 解出 market_id / outcome_id
        let symbol = event.symbol.unwrap_or(0) as i64;
        let market_id = symbol / utils::constants::OUTCOME_MULTIPLIER;
        let outcome_id = symbol % utils::constants::OUTCOME_MULTIPLIER;

        if market_id == 0 {
            return Err("Invalid market_id (symbol=0)".to_string());
        }

        // 转换价格和数量（i64 scaled → Decimal string）
        let price_dec = Decimal::new(event.price, 0)
            / Decimal::new(utils::constants::PRICE_SCALE, 0);
        let size_dec = Decimal::new(event.size, 0);

        let taker_uid = event.taker_uid.unwrap_or(0);
        let maker_uid = event.matched_order_uid;

        // 根据吃单方向确定买卖双方（用户 ID 格式："usr_xxx"）
        let is_taker_buyer = matches!(event.taker_action.as_deref(), Some("bid" | "buy"));
        let (buyer_id, seller_id) = if is_taker_buyer {
            (format!("{}{}", USER_ID_PREFIX, taker_uid), format!("{}{}", USER_ID_PREFIX, maker_uid))
        } else {
            (format!("{}{}", USER_ID_PREFIX, maker_uid), format!("{}{}", USER_ID_PREFIX, taker_uid))
        };

        let trade_id = format!("t_{}_{}", event.matched_order_id, event.taker_order_id.unwrap_or(0));

        info!(
            "Settling trade: trade_id={}, market={}, outcome={}, buyer={}, seller={}, price={}, size={}",
            trade_id, market_id, outcome_id, buyer_id, seller_id, price_dec, size_dec
        );

        let request = tonic::Request::new(api::portfolio::SettleTradeRequest {
            trade_id,
            market_id: market_id as u64,
            outcome_id: outcome_id as u64,
            buyer_id,
            seller_id,
            price: price_dec.to_string(),
            size: size_dec.to_string(),
            taker_fee_rate: "0.001".to_string(),
            maker_fee_rate: "0.001".to_string(),
        });

        client.settle_trade(request).await
            .map(|_resp| {
                info!("Portfolio.SettleTrade succeeded for trade");
            })
            .map_err(|e| format!("SettleTrade gRPC failed: {}", e))
    }

    /// 发布订单事件到 order_events 主题
    async fn publish_order_event(
        &self,
        order_id: &str,
        user_id: i64,
        status: &str,
        filled_quantity: &str,
        filled_amount: &str,
        price: &str,
    ) {
        if let Some(ref producer) = self.event_producer {
            let now = chrono::Utc::now().timestamp_millis();
            let payload = serde_json::json!({
                "type": "order_update",
                "user_id": user_id,
                "data": {
                    "order_id": order_id,
                    "status": status,
                    "filled_quantity": filled_quantity,
                    "filled_amount": filled_amount,
                    "price": price,
                    "updated_at": now
                }
            });

            let json_str = serde_json::to_string(&payload).unwrap_or_default();
            let msg = Message {
                key: Some(order_id.to_string()),
                value: json_str,
            };

            if let Err(e) = producer.send("order_events", msg).await {
                error!("Failed to publish order event: {}", e);
            } else {
                info!("Published order_event for order {}", order_id);
            }
        }
    }
}
