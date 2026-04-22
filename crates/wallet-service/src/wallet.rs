//! 钱包管理器

use common::*;
use parking_lot::RwLock;
use std::collections::HashMap;

/// 钱包管理器
pub struct WalletManager {
    /// 充值地址（简化：内存存储）
    deposit_addresses: RwLock<HashMap<String, String>>,
    /// 充值记录
    deposits: RwLock<HashMap<String, Deposit>>,
    /// 提现记录
    withdrawals: RwLock<HashMap<String, Withdrawal>>,
}

impl WalletManager {
    pub fn new() -> Self {
        Self {
            deposit_addresses: RwLock::new(HashMap::new()),
            deposits: RwLock::new(HashMap::new()),
            withdrawals: RwLock::new(HashMap::new()),
        }
    }

    /// 生成充值地址（简化版）
    pub fn generate_deposit_address(&self, user_id: &str, asset: &str) -> String {
        let key = format!("{}:{}", user_id, asset);
        let address = format!("0x{:016x}", uuid::Uuid::new_v4().as_u128());
        let mut addresses = self.deposit_addresses.write();
        addresses.insert(key, address.clone());
        address
    }

    /// 获取充值地址
    pub fn get_deposit_address(&self, user_id: &str, asset: &str) -> Option<String> {
        let key = format!("{}:{}", user_id, asset);
        let addresses = self.deposit_addresses.read();
        addresses.get(&key).cloned()
    }

    /// 记录充值
    pub fn record_deposit(&self, user_id: String, asset: String, amount: Decimal, tx_hash: String) -> Deposit {
        let deposit = Deposit {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            asset,
            amount,
            address: "".to_string(),
            tx_hash,
            confirmations: 0,
            status: DepositStatus::Pending,
            created_at: chrono::Utc::now(),
        };
        let mut deposits = self.deposits.write();
        deposits.insert(deposit.id.clone(), deposit.clone());
        deposit
    }

    /// 确认充值
    pub fn confirm_deposit(&self, deposit_id: &str, confirmations: u32) -> Result<Deposit, Error> {
        let mut deposits = self.deposits.write();
        if let Some(deposit) = deposits.get_mut(deposit_id) {
            let threshold = match deposit.asset.as_str() {
                "BTC" => 1,
                "ETH" => 6,
                _ => 1,
            };
            if confirmations >= threshold {
                deposit.confirmations = confirmations;
                deposit.status = DepositStatus::Credited;
            }
            Ok(deposit.clone())
        } else {
            Err(Error::WalletError("Deposit not found".to_string()))
        }
    }

    /// 记录提现
    pub fn record_withdrawal(&self, user_id: String, asset: String, amount: Decimal, address: String) -> Withdrawal {
        let withdrawal = Withdrawal {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            asset,
            amount,
            address,
            status: WithdrawalStatus::Pending,
            created_at: chrono::Utc::now(),
        };
        let mut withdrawals = self.withdrawals.write();
        withdrawals.insert(withdrawal.id.clone(), withdrawal.clone());
        withdrawal
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}