//! Wallet Service Repositories

pub mod deposit_repo;
pub mod withdraw_repo;
pub mod whitelist_repo;
pub mod payment_password_repo;

pub use deposit_repo::DepositRepository;
pub use withdraw_repo::WithdrawRepository;
pub use whitelist_repo::WhitelistRepository;
pub use payment_password_repo::PaymentPasswordRepository;
