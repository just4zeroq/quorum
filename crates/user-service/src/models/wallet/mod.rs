//! Wallet Module

pub mod model;
pub mod event;
pub mod shared;

pub use model::{WalletAddress, WalletType, ChainType, WalletInfo, WalletLoginRequest, WalletLoginResponse, GetWalletNonceRequest, GetWalletNonceResponse, WalletBindRequest, WalletBindResponse};
