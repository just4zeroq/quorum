//! Matching Engine Server
//!
//! 通过消息队列消费订单命令，调用撮合引擎处理
//!
//! 架构: async Queue consumer -> mpsc channel -> sync worker thread -> ExchangeCore

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use queue::{ConsumerManagerWithHandler, MessageHandler, Config as QueueConfig, ConsumeMessage, ProducerManager, Message};
use crate::api::{OrderCommand, OrderCommandType, OrderAction, OrderType, MatcherTradeEvent};
use crate::core::exchange::{ExchangeCore, ExchangeConfig};

/// 事件通道，用于将撮合结果发送到消息队列
type EventSender = tokio::sync::mpsc::Sender<MatcherTradeEvent>;

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
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<MatcherTradeEvent>(10000);

    // 初始化 Queue producer
    let producer = ProducerManager::new(merged_config.clone());
    producer.init().await.map_err(|e| format!("Failed to init producer: {}", e))?;

    // 启动事件发送任务（async）
    let producer_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let topic = "match.events";
            let json = serde_json::to_string(&event).unwrap_or_default();
            let msg = Message {
                key: Some(event.matched_order_id.to_string()),
                value: json,
            };
            if let Err(e) = producer.send(topic, msg).await {
                tracing::error!("Failed to send event: {}", e);
            }
        }
        tracing::info!("Event sender task stopped");
    });

    // 创建同步 worker thread
    let worker_handle = thread::spawn(move || {
        run_sync_worker(cmd_rx, event_tx);
    });

    // 创建消费者管理器
    let consumer = ConsumerManagerWithHandler::new(merged_config.clone(), vec!["order.commands".to_string()]);
    consumer.init().await?;

    tracing::info!("Consumer initialized for order.commands topic");

    // 创建 command sender 的 Arc 包装（用于在闭包中共享）
    let cmd_tx = Arc::new(Mutex::new(Some(cmd_tx)));
    let cmd_tx_for_handler = cmd_tx.clone();

    // 定义消息处理器
    let handler: MessageHandler = Arc::new(move |msg: ConsumeMessage| {
        tracing::debug!("Processing order command: key={:?}", msg.key);

        // 解析订单消息
        let order_cmd = match parse_and_validate_order(&msg.value) {
            Ok(cmd) => cmd,
            Err(e) => {
                tracing::error!("Failed to parse order: {}", e);
                return Err(e);
            }
        };

        tracing::info!(
            "Order parsed: cmd={:?}, order_id={}, uid={}, price={}, size={}",
            order_cmd.command, order_cmd.order_id, order_cmd.uid, order_cmd.price, order_cmd.size
        );

        // 发送到同步 worker 处理
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

    // 等待 Ctrl-C 信号
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down...");

    // 关闭 command channel（通知 worker）
    {
        let mut tx_guard = cmd_tx.lock().unwrap();
        tx_guard.take();
    }

    // 等待 worker 结束
    worker_handle.join().unwrap_or(());
    producer_handle.abort();

    Ok(())
}

/// 同步 worker 运行函数
fn run_sync_worker(cmd_rx: Receiver<OrderCommand>, event_tx: EventSender) {
    tracing::info!("Starting sync worker thread");

    // 初始化 ExchangeCore
    let mut exchange = ExchangeCore::new(ExchangeConfig::default());
    exchange.startup();

    // 设置结果消费者回调
    let event_tx_clone = event_tx.clone();
    let consumer = Arc::new(move |cmd: &OrderCommand| {
        for event in &cmd.matcher_events {
            let event_clone = event.clone();
            if let Err(e) = event_tx_clone.try_send(event_clone) {
                tracing::error!("Failed to send event to async channel: {}", e);
            }
        }
    });
    exchange.set_result_consumer(consumer);

    // Worker loop
    loop {
        match cmd_rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(cmd) => {
                tracing::debug!("Worker received command: order_id={}", cmd.order_id);
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

    // 必填字段验证
    let uid = json.uid.ok_or_else(|| "uid is required".to_string())?;
    let order_id = json.order_id.ok_or_else(|| "order_id is required".to_string())?;
    let symbol = json.symbol.ok_or_else(|| "symbol is required".to_string())?;

    // 下单命令需要验证价格和数量
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
            let price = json.price.unwrap_or(0);
            let size = json.size.unwrap_or(0);
            (price, size)
        }
        _ => {
            let price = json.price.unwrap_or(0);
            let size = json.size.unwrap_or(0);
            (price, size)
        }
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