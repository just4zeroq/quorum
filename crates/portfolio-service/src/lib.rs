//! Portfolio Service
//!
//! 账户余额、持仓管理、结算清算、账本流水
//!
//! 合并自:
//! - Account Service (账户余额管理)
//! - Position Service (持仓管理)
//! - Clearing Service (结算清算)
//! - Ledger Service (账本流水)

pub mod account;      // 账户余额管理
pub mod position;     // 持仓管理
pub mod clearing;     // 结算清算
pub mod ledger;       // 账本流水
pub mod models;       // 数据模型
pub mod errors;       // 错误定义
