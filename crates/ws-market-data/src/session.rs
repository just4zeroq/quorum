//! WebSocket Session Management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tokio_tungstenite::tungstenite::Message;

/// 会话类型别名
type SharedSession = Arc<RwLock<ClientSession>>;

/// 客户端订阅的 Channel 类型
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Channel {
    OrderBook,
    Kline,
    Trade,
    Ticker,
}

impl Channel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "orderbook" => Some(Channel::OrderBook),
            "kline" => Some(Channel::Kline),
            "trade" => Some(Channel::Trade),
            "ticker" => Some(Channel::Ticker),
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
        market_id: Option<u64>,
        #[serde(rename = "market_ids")]
        market_ids: Option<Vec<u64>>,
        #[allow(dead_code)]
        params: Option<SubscriptionParams>,
    },
    #[serde(rename = "unsubscribe")]
    Unsubscribe {
        channel: String,
        #[serde(rename = "market_id")]
        market_id: Option<u64>,
        #[serde(rename = "market_ids")]
        market_ids: Option<Vec<u64>>,
    },
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SubscriptionParams {
    #[allow(dead_code)]
    pub interval: Option<String>,
    #[allow(dead_code)]
    pub depth: Option<usize>,
}

/// 客户端会话
pub struct ClientSession {
    pub id: Uuid,
    pub sender: tokio::sync::mpsc::Sender<Message>,
    pub subscriptions: HashMap<Channel, Vec<u64>>,
    #[allow(dead_code)]
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

    pub fn subscribe(&mut self, channel: Channel, market_ids: Vec<u64>) {
        self.subscriptions
            .entry(channel)
            .or_insert_with(Vec::new)
            .extend(market_ids);
    }

    pub fn unsubscribe(&mut self, channel: &Channel, market_ids: &[u64]) {
        if let Some(ids) = self.subscriptions.get_mut(channel) {
            for market_id in market_ids {
                ids.retain(|&id| id != *market_id);
            }
            if ids.is_empty() {
                self.subscriptions.remove(channel);
            }
        }
    }

    pub fn is_subscribed(&self, channel: &Channel, market_id: u64) -> bool {
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

    pub async fn broadcast_to_market(&self, channel: &Channel, market_id: u64, msg: &str) {
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

    #[allow(dead_code)]
    pub async fn len(&self) -> usize {
        self.sessions.read().await.len()
    }

    #[allow(dead_code)]
    pub async fn is_empty(&self) -> bool {
        self.sessions.read().await.is_empty()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    fn create_test_session() -> ClientSession {
        let (tx, _rx) = mpsc::channel(100);
        ClientSession::new(tx)
    }

    #[tokio::test]
    async fn test_session_manager_add_and_remove() {
        let manager = SessionManager::new();
        let session = create_test_session();
        let id = manager.add(session).await;

        assert!(manager.get(&id).await.is_some());
        manager.remove(&id).await;
        assert!(manager.get(&id).await.is_none());
    }

    #[tokio::test]
    async fn test_session_manager_get_all() {
        let manager = SessionManager::new();
        let session1 = create_test_session();
        let session2 = create_test_session();
        manager.add(session1).await;
        manager.add(session2).await;

        let all = manager.get_all().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_session_subscribe_unsubscribe() {
        let mut session = create_test_session();
        session.subscribe(Channel::OrderBook, vec![1, 2, 3]);

        assert!(session.is_subscribed(&Channel::OrderBook, 1));
        assert!(session.is_subscribed(&Channel::OrderBook, 3));
        assert!(!session.is_subscribed(&Channel::OrderBook, 4));
        assert!(!session.is_subscribed(&Channel::Kline, 1));

        session.unsubscribe(&Channel::OrderBook, &[1]);
        assert!(!session.is_subscribed(&Channel::OrderBook, 1));
        assert!(session.is_subscribed(&Channel::OrderBook, 2));
    }

    #[tokio::test]
    async fn test_session_subscribe_multiple_channels() {
        let mut session = create_test_session();
        session.subscribe(Channel::Kline, vec![1]);
        session.subscribe(Channel::Trade, vec![1, 2]);

        assert!(session.is_subscribed(&Channel::Kline, 1));
        assert!(session.is_subscribed(&Channel::Trade, 2));
        assert!(!session.is_subscribed(&Channel::OrderBook, 1));
    }

    #[tokio::test]
    async fn test_channel_from_str() {
        assert_eq!(Channel::from_str("orderbook"), Some(Channel::OrderBook));
        assert_eq!(Channel::from_str("kline"), Some(Channel::Kline));
        assert_eq!(Channel::from_str("trade"), Some(Channel::Trade));
        assert_eq!(Channel::from_str("ticker"), Some(Channel::Ticker));
        assert_eq!(Channel::from_str("unknown"), None);
        assert_eq!(Channel::from_str("ORDERBOOK"), Some(Channel::OrderBook));
    }

    #[tokio::test]
    async fn test_broadcast_to_subscribed_market() {
        let manager = SessionManager::new();
        let (tx, mut rx) = mpsc::channel(100);

        let mut session = ClientSession::new(tx);
        session.subscribe(Channel::Trade, vec![42]);
        manager.add(session).await;

        let msg = "{\"price\": \"100\"}";
        manager.broadcast_to_market(&Channel::Trade, 42, msg).await;

        // Give broadcast time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Should have received the message
        let received = rx.try_recv();
        assert!(received.is_ok());
    }

    #[tokio::test]
    async fn test_no_broadcast_to_unsubscribed_market() {
        let manager = SessionManager::new();
        let (tx, mut rx) = mpsc::channel(100);

        let mut session = ClientSession::new(tx);
        session.subscribe(Channel::Trade, vec![42]);
        manager.add(session).await;

        // Broadcast to a different market
        manager.broadcast_to_market(&Channel::Trade, 99, "test").await;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Should NOT have received the message
        let received = rx.try_recv();
        assert!(received.is_err());
    }

    #[test]
    fn test_session_send() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (tx, mut rx) = mpsc::channel(100);
            let session = ClientSession::new(tx);

            session.send("hello").await.unwrap();
            let received = rx.recv().await.unwrap();
            assert_eq!(received.to_string(), "hello");
        });
    }
}
