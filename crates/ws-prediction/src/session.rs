//! WebSocket Session Management for Prediction Market Events

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tokio_tungstenite::tungstenite::Message;

/// 会话类型别名
type SharedSession = Arc<RwLock<ClientSession>>;

/// 市场事件订阅频道
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize)]
pub enum Channel {
    MarketStatus,
    Settlement,
}

impl Channel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "market_status" | "market" => Some(Channel::MarketStatus),
            "settlement" | "settle" => Some(Channel::Settlement),
            _ => None,
        }
    }
}

/// 客户端订阅消息
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "action")]
pub enum SubscribeMessage {
    #[serde(rename = "subscribe")]
    Subscribe {
        channel: String,
        #[serde(rename = "market_id")]
        market_id: Option<i64>,
        #[serde(rename = "market_ids")]
        market_ids: Option<Vec<i64>>,
    },
    #[serde(rename = "unsubscribe")]
    Unsubscribe {
        channel: String,
        #[serde(rename = "market_id")]
        market_id: Option<i64>,
        #[serde(rename = "market_ids")]
        market_ids: Option<Vec<i64>>,
    },
    #[serde(rename = "ping")]
    Ping,
}

/// 客户端会话
pub struct ClientSession {
    pub id: Uuid,
    pub sender: tokio::sync::mpsc::Sender<Message>,
    pub subscriptions: HashMap<Channel, Vec<i64>>, // channel -> [market_ids]
    pub subscribed_at: chrono::DateTime<chrono::Utc>,
}

impl ClientSession {
    pub fn new(sender: tokio::sync::mpsc::Sender<Message>) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            subscriptions: HashMap::new(),
            subscribed_at: chrono::Utc::now(),
        }
    }

    pub fn subscribe(&mut self, channel: Channel, market_ids: Vec<i64>) {
        self.subscriptions
            .entry(channel)
            .or_insert_with(Vec::new)
            .extend(market_ids);
    }

    pub fn unsubscribe(&mut self, channel: &Channel, market_ids: &[i64]) {
        if let Some(ids) = self.subscriptions.get_mut(channel) {
            for market_id in market_ids {
                ids.retain(|&id| id != *market_id);
            }
            if ids.is_empty() {
                self.subscriptions.remove(channel);
            }
        }
    }

    pub fn is_subscribed(&self, channel: &Channel, market_id: i64) -> bool {
        self.subscriptions
            .get(channel)
            .map(|ids| ids.contains(&market_id))
            .unwrap_or(false)
    }

    pub async fn send(&self, msg: &str) -> Result<(), tokio::sync::mpsc::error::SendError<Message>> {
        self.sender.send(Message::Text(msg.into())).await
    }
}

/// Session 管理器
pub struct SessionManager {
    sessions: RwLock<HashMap<Uuid, SharedSession>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add(&self, session: ClientSession) -> Uuid {
        let id = session.id;
        self.sessions.write().await.insert(id, Arc::new(RwLock::new(session)));
        id
    }

    pub async fn remove(&self, id: &Uuid) {
        self.sessions.write().await.remove(id);
    }

    pub async fn get(&self, id: &Uuid) -> Option<SharedSession> {
        self.sessions.read().await.get(id).cloned()
    }

    pub async fn get_all(&self) -> Vec<SharedSession> {
        self.sessions.read().await.values().cloned().collect()
    }

    pub async fn broadcast_to_market(&self, channel: &Channel, market_id: i64, msg: &str) {
        let sessions = self.get_all().await;
        for session in sessions {
            let is_subscribed = session.read().await.is_subscribed(channel, market_id);
            if is_subscribed {
                if let Err(e) = session.read().await.send(msg).await {
                    tracing::warn!("Failed to send message: {}", e);
                }
            }
        }
    }

    pub async fn len(&self) -> usize {
        self.sessions.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.sessions.read().await.is_empty()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
