//! Trade Module - 成交领域

pub mod model;
pub mod event;
pub mod shared;

pub use model::{Trade, TradeSide, TradeQuery};
pub use event::TradeEvent;
