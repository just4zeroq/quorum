//! WebSocket Server for Order Updates

use std::net::SocketAddr;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn, error};

use crate::session::{SessionManager, ClientMessage, ClientSession};

/// 创建 SessionManager
pub fn create_session_manager() -> SessionManager {
    SessionManager::new()
}

/// 启动 WebSocket 服务
pub async fn start(addr: &str, session_manager: Arc<SessionManager>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("WebSocket Order server listening on {}", addr);

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

    // 创建会话（未认证）
    let session = ClientSession::new(tx);
    let session_id = session_manager.add(session).await;

    // 发送认证要求
    {
        if let Some(session) = session_manager.get(&session_id).await {
            let auth_request = serde_json::json!({
                "type": "auth_required",
                "message": "Please authenticate with your token"
            });
            if let Err(e) = session.read().await.send(&serde_json::to_string(&auth_request).unwrap()).await {
                warn!("Failed to send auth request: {}", e);
            }
        }
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
    let msg: ClientMessage = match serde_json::from_str(text) {
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
        ClientMessage::Auth { token } => {
            // 认证处理：由于ws-order独立运行，这里简化处理
            // 实际生产环境应调用 Auth Service 验证token并获取user_id
            // 简化实现：从token中解析basic auth格式 "user:{user_id}"
            let user_id = if token.starts_with("user:") {
                token[5..].parse::<i64>().ok()
            } else {
                None
            };

            match user_id {
                Some(uid) => {
                    session_manager.authenticate(session_id, uid).await;
                    let resp = serde_json::json!({
                        "type": "auth_success",
                        "user_id": uid
                    });
                    send_to_session(session_manager, session_id, &serde_json::to_string(&resp)?).await?;
                    info!("Session {} authenticated as user {}", session_id, uid);
                }
                None => {
                    let resp = serde_json::json!({
                        "type": "auth_error",
                        "message": "Invalid token"
                    });
                    send_to_session(session_manager, session_id, &serde_json::to_string(&resp)?).await?;
                    warn!("Session {} auth failed: invalid token", session_id);
                }
            }
        }
        ClientMessage::Ping => {
            let resp = serde_json::json!({
                "type": "pong",
                "timestamp": chrono::Utc::now().timestamp_millis()
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
