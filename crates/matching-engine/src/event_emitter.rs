//! Event Emitter - Queue 事件发布

use std::sync::Arc;
use queue::{ProducerManager, ProducerError, Message};
use crate::api::events::MatcherTradeEvent;
use tracing::info;

#[derive(Clone)]
pub struct EventEmitter {
    producer: Arc<ProducerManager>,
}

impl EventEmitter {
    pub fn new(producer: ProducerManager) -> Self {
        Self {
            producer: Arc::new(producer),
        }
    }

    /// 发布成交事件
    pub async fn emit_trade(&self, event: &MatcherTradeEvent) -> Result<(), ProducerError> {
        let json = serde_json::to_string(event).map_err(|e| {
            ProducerError::Serialization(e)
        })?;

        let msg = Message {
            key: Some(event.matched_order_id.to_string()),
            value: json,
        };

        self.producer.send("match.events", msg).await?;
        info!("Emitted trade event: order_id={}", event.matched_order_id);
        Ok(())
    }

    /// 发布拒绝事件
    pub async fn emit_reject(&self, event: &MatcherTradeEvent) -> Result<(), ProducerError> {
        let json = serde_json::to_string(event).map_err(|e| {
            ProducerError::Serialization(e)
        })?;

        let msg = Message {
            key: None,
            value: json,
        };

        self.producer.send("match.events", msg).await?;
        info!("Emitted reject event: price={}, size={}", event.price, event.size);
        Ok(())
    }

    /// 发布减少事件
    pub async fn emit_reduce(&self, event: &MatcherTradeEvent) -> Result<(), ProducerError> {
        let json = serde_json::to_string(event).map_err(|e| {
            ProducerError::Serialization(e)
        })?;

        let msg = Message {
            key: Some(event.matched_order_id.to_string()),
            value: json,
        };

        self.producer.send("match.events", msg).await?;
        Ok(())
    }
}
