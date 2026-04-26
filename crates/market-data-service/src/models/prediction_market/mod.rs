//! Prediction Market Module - 预测市场领域

pub mod model;
pub mod event;
pub mod shared;

pub use model::{PredictionMarket, MarketOutcome, MarketStatus, Resolution};
pub use event::PredictionMarketEvent;
pub use shared::*;