//! Matching Engine Server
//!
//! 通过消息队列消费订单命令，调用撮合引擎处理
//!
//! 架构: async Queue consumer -> mpsc channel -> sync worker thread -> ExchangeCore
//!
//! ## Kafka 消息格式
//!
//! **消费 `order.commands` topic:**
//! ```json
//! {
//!   "command": "place",          // place / cancel
//!   "order_id": 12345,
//!   "uid": 1,
//!   "symbol": 1001,              // market_id * 1000 + outcome_id
//!   "price": 65000,              // scaled: 0.65 * 100000
//!   "size": 100,
//!   "action": "bid",             // bid / ask
//!   "order_type": "gtc"          // gtc / ioc / fok / post_only
//! }
//! ```
//!
//! **发布 `match.events` topic (每笔成交):**
//! ```json
//! {
//!   "event_type": "Trade",
//!   "size": 100,
//!   "price": 65000,
//!   "matched_order_id": 42,
//!   "matched_order_uid": 2,
//!   "taker_order_id": 99,
//!   "taker_uid": 1,
//!   "symbol": 1001,
//!   "bidder_hold_price": 65000
//! }
//! ```
//!
//! **发布 `market.trade` topic (行情推送):**
//! ```json
//! {
//!   "trade": {
//!     "market_id": 1,
//!     "outcome_id": 1,
//!     "price": 65000,
//!     "size": 100,
//!     "taker_side": "buy",
//!     "timestamp": 1705312200000
//!   }
//! }
//! ```

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use queue::{ConsumerManagerWithHandler, MessageHandler, Config as QueueConfig, ConsumeMessage, ProducerManager, Message};
use crate::api::{OrderCommand, OrderCommandType, OrderAction, OrderType, MatcherTradeEvent};
use crate::core::exchange::{ExchangeCore, ExchangeConfig};

/// 编码 symbol: market_id * OUTCOME_MULTIPLIER + outcome_id
const OUTCOME_MULTIPLIER: i32 = 1000;

/// 事件通道，用于将撮合结果发送到消息队列
type EventSender = tokio::sync::mpsc::Sender<EnrichedTradeEvent>;

/// 携带上下文的成交事件
#[derive(Clone, serde::Serialize)]
struct EnrichedTradeEvent {
    #[serde(flatten)]
    inner: MatcherTradeEvent,
    taker_order_id: u64,
    taker_uid: u64,
    symbol: i32,
    /// 吃单方方向: "bid"=买, "ask"=卖
    taker_action: String,
}

/// 订单命令 JSON 结构
#[derive(serde::Deserialize, Clone)]
struct OrderCommandJson {
    command: String,
    order_id: Option<u64>,
    uid: Option<u64>,
    symbol: Option<i32>,
    price: Option<i64>,
    size: Option<i64>,
    action: Option<String>,
    order_type: Option<String>,
}

/// 启动服务
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "0.0.0.0:50051".parse()?;
    tracing::info!("Matching Engine listening on {}", addr);

    // 初始化队列配置
    let queue_config = QueueConfig::default();
    let merged_config = queue_config.merge();

    // 创建 command channel（async handler -> sync worker）
    let (cmd_tx, cmd_rx): (Sender<OrderCommand>, Receiver<OrderCommand>) = mpsc::channel();

    // 创建事件通道（撮合结果 -> Queue producer）
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<EnrichedTradeEvent>(10000);

    // 初始化 Queue producer（match.events & market.* events）
    let producer = Arc::new(ProducerManager::new(merged_config.clone()));
    producer.init().await.map_err(|e| format!("Failed to init producer: {}", e))?;

    // 启动事件发送任务（async）
    let producer_sender = producer.clone();
    let producer_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            // 1. 发送 match.events（给 Order Service 进行清算）
            let match_json = serde_json::to_string(&event.inner).unwrap_or_default();
            let match_msg = Message {
                key: Some(event.inner.matched_order_id.to_string()),
                value: match_json,
            };
            if let Err(e) = producer_sender.send("match.events", match_msg).await {
                tracing::error!("Failed to send match event: {}", e);
            }

            // 2. 发送 market.trade（给 ws-market-data 推送）
            let symbol = event.symbol;
            let market_id = symbol / OUTCOME_MULTIPLIER;
            let outcome_id = symbol % OUTCOME_MULTIPLIER;
            let trade_json = serde_json::json!({
                "trade": {
                    "market_id": market_id,
                    "outcome_id": outcome_id,
                    "price": event.inner.price,
                    "size": event.inner.size,
                    "taker_side": if event.inner.taker_uid == event.taker_uid { "buy" } else { "sell" },
                    "timestamp": chrono::Utc::now().timestamp_millis()
                }
            });
            let trade_msg = Message {
                key: Some(format!("m{}o{}", market_id, outcome_id)),
                value: trade_json.to_string(),
            };
            if let Err(e) = producer_sender.send("market.trade", trade_msg).await {
                tracing::error!("Failed to send market.trade event: {}", e);
            }

            tracing::debug!(
                "Emitted events: order={}, uid={}, symbol={}, price={}, size={}",
                event.inner.matched_order_id, event.inner.matched_order_uid,
                symbol, event.inner.price, event.inner.size
            );
        }
        tracing::info!("Event sender task stopped");
    });

    // 创建同步 worker thread（持有 ExchangeCore）
    let worker_handle = thread::spawn(move || {
        run_sync_worker(cmd_rx, event_tx);
    });

    // 创建消费者管理器，消费 order.commands 主题
    let consumer = ConsumerManagerWithHandler::new(merged_config.clone(), vec!["order.commands".to_string()]);
    consumer.init().await?;

    tracing::info!("Consumer initialized for order.commands topic");

    // 创建 command sender 的 Arc 包装（用于在闭包中共享）
    let cmd_tx = Arc::new(Mutex::new(Some(cmd_tx)));
    let cmd_tx_for_handler = cmd_tx.clone();

    // 定义消息处理器
    let handler: MessageHandler = Arc::new(move |msg: ConsumeMessage| {
        tracing::debug!("Processing order command: key={:?}", msg.key);

        let order_cmd = match parse_and_validate_order(&msg.value) {
            Ok(cmd) => cmd,
            Err(e) => {
                tracing::error!("Failed to parse order: {}", e);
                return Err(e);
            }
        };

        tracing::info!(
            "Order parsed: cmd={:?}, order_id={}, uid={}, symbol={}, price={}, size={}",
            order_cmd.command, order_cmd.order_id, order_cmd.uid,
            order_cmd.symbol, order_cmd.price, order_cmd.size
        );

        let tx_guard = cmd_tx_for_handler.lock().unwrap();
        if let Some(ref tx) = *tx_guard {
            if let Err(e) = tx.send(order_cmd) {
                tracing::error!("Failed to send order to worker: {}", e);
                Err(format!("Channel send error: {}", e))
            } else {
                Ok(())
            }
        } else {
            Err("Worker channel closed".to_string())
        }
    });

    // 启动消费循环
    tokio::spawn(async move {
        if let Err(e) = consumer.start(handler).await {
            tracing::error!("Consumer error: {}", e);
        }
    });

    tracing::info!("Matching Engine Service started");

    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down...");

    // 关闭 command channel（通知 worker）
    {
        let mut tx_guard = cmd_tx.lock().unwrap();
        tx_guard.take();
    }

    worker_handle.join().unwrap_or(());
    producer_handle.abort();

    Ok(())
}

/// 同步 worker 运行函数
fn run_sync_worker(cmd_rx: Receiver<OrderCommand>, event_tx: EventSender) {
    tracing::info!("Starting sync worker thread");

    let mut exchange = ExchangeCore::new(ExchangeConfig::default());
    exchange.startup();

    // 懒注册符号表，避免重复注册
    let mut registered_symbols = std::collections::HashSet::<i32>::new();

    // 设置结果消费者回调：事件富化 + 发送到 async channel
    let event_tx_clone = event_tx.clone();
    let consumer = Arc::new(move |cmd: &OrderCommand| {
        let taker_action = match cmd.action {
            crate::api::OrderAction::Bid => "bid",
            crate::api::OrderAction::Ask => "ask",
        };
        for event in &cmd.matcher_events {
            let enriched_event = EnrichedTradeEvent {
                inner: event.clone(),
                taker_order_id: cmd.order_id,
                taker_uid: cmd.uid,
                symbol: cmd.symbol,
                taker_action: taker_action.to_string(),
            };
            if let Err(e) = event_tx_clone.try_send(enriched_event) {
                tracing::error!("Failed to send event to async channel: {}", e);
            }
        }
    });
    exchange.set_result_consumer(consumer);

    // Worker loop
    loop {
        match cmd_rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(cmd) => {
                tracing::debug!("Worker received command: order_id={}, symbol={}", cmd.order_id, cmd.symbol);

                // 懒注册：PlaceOrder 遇到未知符号时自动注册
                if cmd.command == crate::api::OrderCommandType::PlaceOrder {
                    if registered_symbols.insert(cmd.symbol) {
                        let spec = crate::api::CoreSymbolSpecification {
                            symbol_id: cmd.symbol,
                            symbol_type: crate::api::SymbolType::CurrencyExchangePair,
                            base_currency: cmd.symbol,    // outcome token
                            quote_currency: 0,            // USDC
                            base_scale_k: 1,              // 数量不缩放
                            quote_scale_k: 100000,        // PRICE_SCALE
                            ..Default::default()
                        };
                        tracing::info!("Auto-registered symbol {}", cmd.symbol);
                        exchange.add_symbol(spec);
                    }
                }

                exchange.submit_command(cmd);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                tracing::info!("Worker channel disconnected, shutting down");
                break;
            }
        }
    }

    tracing::info!("Sync worker thread stopped");
}

/// 解析并验证订单消息
fn parse_and_validate_order(json: &str) -> Result<OrderCommand, String> {
    let cmd_json: OrderCommandJson = serde_json::from_str(json)
        .map_err(|e| format!("Parse error: {}", e))?;

    build_order_command(&cmd_json)
}

/// 构建 OrderCommand
fn build_order_command(json: &OrderCommandJson) -> Result<OrderCommand, String> {
    let command = match json.command.to_lowercase().as_str() {
        "place" | "place_order" | "new" => OrderCommandType::PlaceOrder,
        "cancel" | "cancel_order" => OrderCommandType::CancelOrder,
        "move" | "move_order" => OrderCommandType::MoveOrder,
        "reduce" | "reduce_order" => OrderCommandType::ReduceOrder,
        _ => return Err(format!("Unknown command: {}", json.command)),
    };

    let uid = json.uid.ok_or_else(|| "uid is required".to_string())?;
    let order_id = json.order_id.ok_or_else(|| "order_id is required".to_string())?;
    let symbol = json.symbol.ok_or_else(|| "symbol is required".to_string())?;

    let (price, size) = match command {
        OrderCommandType::PlaceOrder => {
            let price = json.price.ok_or_else(|| "price is required for place order".to_string())?;
            let size = json.size.ok_or_else(|| "size is required for place order".to_string())?;
            if price <= 0 {
                return Err("price must be positive".to_string());
            }
            if size <= 0 {
                return Err("size must be positive".to_string());
            }
            (price, size)
        }
        OrderCommandType::CancelOrder | OrderCommandType::ReduceOrder => {
            (json.price.unwrap_or(0), json.size.unwrap_or(0))
        }
        _ => (json.price.unwrap_or(0), json.size.unwrap_or(0)),
    };

    let action = if let Some(a) = &json.action {
        match a.to_lowercase().as_str() {
            "bid" | "buy" => OrderAction::Bid,
            "ask" | "sell" => OrderAction::Ask,
            _ => return Err(format!("Invalid action: {}", a)),
        }
    } else {
        OrderAction::Bid
    };

    let order_type = json.order_type.as_ref()
        .map(|t| match t.to_lowercase().as_str() {
            "gtc" => OrderType::Gtc,
            "ioc" => OrderType::Ioc,
            "fok" => OrderType::Fok,
            "fok_budget" => OrderType::FokBudget,
            "post_only" => OrderType::PostOnly,
            _ => OrderType::Gtc,
        })
        .unwrap_or(OrderType::Gtc);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    Ok(OrderCommand {
        command,
        result_code: crate::api::CommandResultCode::New,
        uid,
        order_id,
        symbol,
        price,
        reserve_price: price,
        size,
        action,
        order_type,
        timestamp,
        events_group: 0,
        service_flags: 0,
        stop_price: None,
        visible_size: None,
        expire_time: None,
        matcher_events: Vec::with_capacity(8),
    })
}

/// 从 symbol 中解码 market_id
fn _symbol_to_market_id(symbol: i32) -> i64 {
    (symbol / OUTCOME_MULTIPLIER) as i64
}

/// 从 symbol 中解码 outcome_id
fn _symbol_to_outcome_id(symbol: i32) -> i64 {
    (symbol % OUTCOME_MULTIPLIER) as i64
}
