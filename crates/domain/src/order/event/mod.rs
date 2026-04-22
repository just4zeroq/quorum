//! Order Event - 订单事件

use serde::{Deserialize, Serialize};

/// 订单事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum OrderEvent {
    /// 订单创建
    Created {
        order_id: String,
        user_id: i64,
        market_id: i64,
        outcome_id: i64,
    },
    /// 订单提交到撮合引擎
    Submitted { order_id: String },
    /// 部分成交
    PartiallyFilled {
        order_id: String,
        filled_quantity: String,
        filled_amount: String,
        price: String,
    },
    /// 完全成交
    Filled {
        order_id: String,
        filled_quantity: String,
        filled_amount: String,
    },
    /// 订单取消
    Cancelled {
        order_id: String,
        reason: Option<String>,
    },
    /// 订单拒单
    Rejected {
        order_id: String,
        reason: String,
    },
}

impl OrderEvent {
    pub fn order_id(&self) -> &str {
        match self {
            OrderEvent::Created { order_id, .. } => order_id,
            OrderEvent::Submitted { order_id } => order_id,
            OrderEvent::PartiallyFilled { order_id, .. } => order_id,
            OrderEvent::Filled { order_id, .. } => order_id,
            OrderEvent::Cancelled { order_id, .. } => order_id,
            OrderEvent::Rejected { order_id, .. } => order_id,
        }
    }
}