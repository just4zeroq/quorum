//! Domain - 共享领域模型
//!
//! 服务间共享的数据模型、事件定义
//!
//! 目录结构:
//! ```
//! domain/
//! ├── order/          # 订单领域
//! │   ├── model/      # 订单数据模型
//! │   ├── event/      # 订单事件
//! │   └── shared/     # 共享模型
//! ├── trade/          # 成交领域
//! │   ├── model/
//! │   ├── event/
//! │   └── shared/
//! ├── user/           # 用户领域
//! ├── market_data/    # 行情领域
//! └── prediction_market/  # 预测市场领域
//! ```

pub mod order;
pub mod trade;
pub mod user;
pub mod market_data;
pub mod prediction_market;

// Re-export commonly used types
pub use order::model::{Order, OrderStatus, OrderType, OrderSide, OrderQuery};
pub use order::event::OrderEvent;

pub use trade::model::{Trade, TradeSide, TradeQuery};
pub use trade::event::TradeEvent;

pub use user::model::User;

pub use market_data::model::{Market, Outcome, OrderBook, Kline};

pub use prediction_market::model::{PredictionMarket, MarketOutcome, MarketStatus};