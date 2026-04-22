//! Matching Engine - 撮合引擎（裁决中心）
//!
//! 负责订单撮合、Orderbook 维护
//!
//! 核心职责：
//! - 维护 Orderbook（买卖盘）
//! - 执行撮合匹配（Price-Time Priority）
//! - 输出事件（Trade、Orderbook 变更）

pub mod orderbook;
pub mod engine;
pub mod types;

pub use orderbook::OrderBook;
pub use engine::MatchingEngine;
pub use types::*;