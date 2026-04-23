//! Protobuf 生成代码
//!
//! 引入 tonic-build 生成的代码

pub mod account {
    include!("pb/account.rs");
}

pub mod account_service_server {
    pub use super::account::account_service_server::*;
}