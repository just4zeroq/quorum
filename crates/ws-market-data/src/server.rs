//! WebSocket Server

use std::net::SocketAddr;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn, error};

use crate::session::{SessionManager, SubscribeMessage, Channel, ClientSession};

/// 创建 SessionManager
pub fn create_session_manager() -> SessionManager {
    SessionManager::new()
}

/// 启动 WebSocket 服务
pub async fn start(addr: &str, session_manager: Arc<SessionManager>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("WebSocket server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let manager = session_manager.clone();
                tokio::spawn(handle_connection(stream, addr, manager));
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

/// 处理 WebSocket 连接
async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
    session_manager: Arc<SessionManager>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            warn!("WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };

    info!("New WebSocket connection from {}", addr);

    let (write, mut read) = ws_stream.split();
    let (tx, rx) = tokio::sync::mpsc::channel::<Message>(100);

    // 把 rx forward 到 write
    let write_handle = tokio::spawn(async move {
        let mut rx = rx;
        let mut write = write;
        while let Some(msg) = rx.recv().await {
            if let Err(e) = write.send(msg).await {
                error!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // 创建会话
    let session = ClientSession::new(tx);
    let session_id = session_manager.add(session).await;

    // 启动心跳任务
    let heartbeat_tx = session_manager.get(&session_id).await;
    if let Some(session) = heartbeat_tx {
        let tx = session.read().await.sender.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                if tx.send(Message::Text(r#"{"type":"pong","timestamp":0}"#.into())).await.is_err() {
                    break;
                }
            }
        });
    }

    // 处理消息
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let text_str = text.to_string();
                if let Err(e) = handle_message(&session_manager, &session_id, &text_str).await {
                    warn!("Failed to handle message from {}: {}", addr, e);
                }
            }
            Ok(Message::Ping(data)) => {
                if let Some(session) = session_manager.get(&session_id).await {
                    let tx = session.read().await.sender.clone();
                    let _ = tx.send(Message::Pong(data)).await;
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client {} closed connection", addr);
                break;
            }
            Err(e) => {
                warn!("WebSocket error from {}: {}", addr, e);
                break;
            }
            _ => {}
        }
    }

    // 清理
    session_manager.remove(&session_id).await;
    write_handle.abort();

    info!("Connection from {} closed", addr);
}

/// 处理客户端消息
async fn handle_message(
    session_manager: &Arc<SessionManager>,
    session_id: &uuid::Uuid,
    text: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg: SubscribeMessage = match serde_json::from_str(text) {
        Ok(msg) => msg,
        Err(e) => {
            let error_resp = serde_json::json!({
                "type": "error",
                "code": 400,
                "message": "Invalid message format",
                "details": e.to_string()
            });
            send_to_session(session_manager, session_id, &serde_json::to_string(&error_resp)?).await?;
            return Ok(());
        }
    };

    match msg {
        SubscribeMessage::Subscribe { channel, market_id, market_ids, .. } => {
            let channel = match Channel::from_str(&channel) {
                Some(c) => c,
                None => {
                    let error_resp = serde_json::json!({
                        "type": "error",
                        "code": 400,
                        "message": "Unknown channel",
                        "details": channel
                    });
                    send_to_session(session_manager, session_id, &serde_json::to_string(&error_resp)?).await?;
                    return Ok(());
                }
            };

            let market_ids = market_ids.unwrap_or_else(|| market_id.map(|id| vec![id]).unwrap_or_default());

            if let Some(session) = session_manager.get(session_id).await {
                let mut session = session.write().await;
                session.subscribe(channel.clone(), market_ids.clone());
                info!("Session {} subscribed to {:?} for markets {:?}", session_id, channel, market_ids);
            }

            let resp = serde_json::json!({
                "type": "subscribed",
                "channel": channel,
                "market_ids": market_ids
            });
            send_to_session(session_manager, session_id, &serde_json::to_string(&resp)?).await?;
        }

        SubscribeMessage::Unsubscribe { channel, market_id, market_ids } => {
            let channel = match Channel::from_str(&channel) {
                Some(c) => c,
                None => {
                    let error_resp = serde_json::json!({
                        "type": "error",
                        "code": 400,
                        "message": "Unknown channel",
                        "details": channel
                    });
                    send_to_session(session_manager, session_id, &serde_json::to_string(&error_resp)?).await?;
                    return Ok(());
                }
            };

            let market_ids = market_ids.unwrap_or_else(|| market_id.map(|id| vec![id]).unwrap_or_default());

            if let Some(session) = session_manager.get(session_id).await {
                let mut session = session.write().await;
                session.unsubscribe(&channel, &market_ids);
                info!("Session {} unsubscribed from {:?} for markets {:?}", session_id, channel, market_ids);
            }

            let resp = serde_json::json!({
                "type": "unsubscribed",
                "channel": channel,
                "market_ids": market_ids
            });
            send_to_session(session_manager, session_id, &serde_json::to_string(&resp)?).await?;
        }

        SubscribeMessage::Ping => {
            let resp = serde_json::json!({
                "type": "pong",
                "timestamp": chrono::Utc::now().timestamp()
            });
            send_to_session(session_manager, session_id, &serde_json::to_string(&resp)?).await?;
        }
    }

    Ok(())
}

/// 发送消息到指定会话
async fn send_to_session(
    session_manager: &Arc<SessionManager>,
    session_id: &uuid::Uuid,
    msg: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(session) = session_manager.get(session_id).await {
        let session = session.read().await;
        session.send(msg).await?;
    }
    Ok(())
}
