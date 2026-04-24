//! WebSocket Session Management for Order Service

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tokio_tungstenite::tungstenite::Message;

/// 会话类型别名
type SharedSession = Arc<RwLock<ClientSession>>;

/// 认证状态
#[derive(Debug, Clone, PartialEq)]
pub enum AuthStatus {
    Pending,
    Authenticated { user_id: i64 },
}

/// 客户端会话
pub struct ClientSession {
    pub id: Uuid,
    pub sender: tokio::sync::mpsc::Sender<Message>,
    pub auth: AuthStatus,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

impl ClientSession {
    pub fn new(sender: tokio::sync::mpsc::Sender<Message>) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            auth: AuthStatus::Pending,
            connected_at: chrono::Utc::now(),
        }
    }

    /// 设置认证
    pub fn authenticate(&mut self, user_id: i64) {
        self.auth = AuthStatus::Authenticated { user_id };
    }

    /// 获取用户ID（如果已认证）
    pub fn user_id(&self) -> Option<i64> {
        match self.auth {
            AuthStatus::Authenticated { user_id } => Some(user_id),
            _ => None,
        }
    }

    /// 是否已认证
    pub fn is_authenticated(&self) -> bool {
        matches!(self.auth, AuthStatus::Authenticated { .. })
    }

    pub async fn send(&self, msg: &str) -> Result<(), tokio::sync::mpsc::error::SendError<Message>> {
        self.sender.send(Message::Text(msg.into())).await
    }
}

/// 认证消息
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "action")]
pub enum ClientMessage {
    #[serde(rename = "auth")]
    Auth {
        token: String,
    },
    #[serde(rename = "ping")]
    Ping,
}

/// Session 管理器
pub struct SessionManager {
    sessions: RwLock<HashMap<Uuid, SharedSession>>,
    user_sessions: RwLock<HashMap<i64, Vec<Uuid>>>, // user_id -> session_ids
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            user_sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add(&self, session: ClientSession) -> Uuid {
        let id = session.id;
        self.sessions.write().await.insert(id, Arc::new(RwLock::new(session)));
        id
    }

    pub async fn remove(&self, id: &Uuid) {
        // 从user_sessions索引中移除
        if let Some(session) = self.sessions.read().await.get(id) {
            let session = session.read().await;
            if let Some(user_id) = session.user_id() {
                let mut user_sessions = self.user_sessions.write().await;
                if let Some(sessions) = user_sessions.get_mut(&user_id) {
                    sessions.retain(|sid| sid != id);
                    if sessions.is_empty() {
                        user_sessions.remove(&user_id);
                    }
                }
            }
        }
        self.sessions.write().await.remove(id);
    }

    pub async fn get(&self, id: &Uuid) -> Option<SharedSession> {
        self.sessions.read().await.get(id).cloned()
    }

    /// 更新认证状态并建立user_id索引
    pub async fn authenticate(&self, session_id: &Uuid, user_id: i64) -> bool {
        if let Some(session) = self.sessions.read().await.get(session_id) {
            let mut session = session.write().await;
            session.authenticate(user_id);
            let mut user_sessions = self.user_sessions.write().await;
            user_sessions.entry(user_id).or_insert_with(Vec::new).push(*session_id);
            true
        } else {
            false
        }
    }

    /// 获取特定用户的所有会话
    pub async fn get_user_sessions(&self, user_id: i64) -> Vec<SharedSession> {
        let user_sessions = self.user_sessions.read().await;
        let mut result = Vec::new();
        if let Some(session_ids) = user_sessions.get(&user_id) {
            let sessions = self.sessions.read().await;
            for id in session_ids {
                if let Some(session) = sessions.get(id) {
                    result.push(session.clone());
                }
            }
        }
        result
    }

    /// 广播消息给特定用户的所有会话
    pub async fn send_to_user(&self, user_id: i64, msg: &str) {
        let sessions = self.get_user_sessions(user_id).await;
        for session in sessions {
            if let Err(e) = session.read().await.send(msg).await {
                tracing::warn!("Failed to send to user {}: {}", user_id, e);
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
