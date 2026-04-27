//! WebSocket Proxy Handlers
//!
//! 转发 WebSocket 连接到后端 WebSocket 服务

use salvo::prelude::*;
use salvo::websocket::WebSocketUpgrade;
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};

/// 将 tungstenite::Message 转换为 salvo::websocket::Message
fn to_salvo_msg(msg: tokio_tungstenite::tungstenite::Message) -> salvo::websocket::Message {
    match msg {
        tokio_tungstenite::tungstenite::Message::Text(s) => salvo::websocket::Message::text(s),
        tokio_tungstenite::tungstenite::Message::Binary(v) => salvo::websocket::Message::binary(v),
        tokio_tungstenite::tungstenite::Message::Ping(v) => salvo::websocket::Message::ping(v),
        tokio_tungstenite::tungstenite::Message::Pong(v) => salvo::websocket::Message::pong(v),
        tokio_tungstenite::tungstenite::Message::Close(_) => salvo::websocket::Message::close(),
        tokio_tungstenite::tungstenite::Message::Frame(_) => salvo::websocket::Message::close(),
    }
}

/// 将 salvo::websocket::Message 转换为 tungstenite::Message
fn to_tungstenite_msg(msg: salvo::websocket::Message) -> Option<tokio_tungstenite::tungstenite::Message> {
    if msg.is_text() {
        msg.to_str().ok().map(|s| tokio_tungstenite::tungstenite::Message::Text(s.to_string()))
    } else if msg.is_binary() {
        Some(tokio_tungstenite::tungstenite::Message::Binary(msg.as_bytes().to_vec()))
    } else if msg.is_ping() {
        Some(tokio_tungstenite::tungstenite::Message::Ping(msg.as_bytes().to_vec()))
    } else if msg.is_pong() {
        Some(tokio_tungstenite::tungstenite::Message::Pong(msg.as_bytes().to_vec()))
    } else if msg.is_close() {
        Some(tokio_tungstenite::tungstenite::Message::Close(None))
    } else {
        None
    }
}

/// 在 salvo 和 tungstenite WebSocket 之间双向转发消息
async fn proxy_websocket(
    salvo_ws: salvo::websocket::WebSocket,
    backend_url: &str,
) {
    match connect_async(backend_url).await {
        Ok((backend_ws, _)) => {
            let (mut sw, mut sr) = salvo_ws.split();
            let (mut bw, mut br) = backend_ws.split();

            // 客户端 → 后端
            let c2b = tokio::spawn(async move {
                while let Some(Ok(msg)) = sr.next().await {
                    if let Some(tmsg) = to_tungstenite_msg(msg) {
                        if bw.send(tmsg).await.is_err() {
                            break;
                        }
                    }
                }
            });

            // 后端 → 客户端
            let b2c = tokio::spawn(async move {
                while let Some(Ok(msg)) = br.next().await {
                    let smsg = to_salvo_msg(msg);
                    if sw.send(smsg).await.is_err() {
                        break;
                    }
                }
            });

            let _ = tokio::join!(c2b, b2c);
        }
        Err(e) => {
            tracing::error!("Failed to connect to backend {}: {}", backend_url, e);
        }
    }
}

/// WebSocket 市场数据代理 (-> ws://127.0.0.1:50016)
#[handler]
pub async fn ws_market_data_proxy(req: &mut Request, res: &mut Response) {
    tracing::info!("WebSocket upgrade request for /ws/market-data");
    let _ = WebSocketUpgrade::new()
        .upgrade(req, res, |client_ws| async move {
            proxy_websocket(client_ws, "ws://127.0.0.1:50016").await;
        })
        .await;
}

/// WebSocket 订单代理 (-> ws://127.0.0.1:50017)
#[handler]
pub async fn ws_order_proxy(req: &mut Request, res: &mut Response) {
    tracing::info!("WebSocket upgrade request for /ws/order");
    let _ = WebSocketUpgrade::new()
        .upgrade(req, res, |client_ws| async move {
            proxy_websocket(client_ws, "ws://127.0.0.1:50017").await;
        })
        .await;
}

/// WebSocket 预测市场代理 (-> ws://127.0.0.1:50018)
#[handler]
pub async fn ws_prediction_proxy(req: &mut Request, res: &mut Response) {
    tracing::info!("WebSocket upgrade request for /ws/prediction");
    let _ = WebSocketUpgrade::new()
        .upgrade(req, res, |client_ws| async move {
            proxy_websocket(client_ws, "ws://127.0.0.1:50018").await;
        })
        .await;
}
