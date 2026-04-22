//! 账户管理器

use common::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// 账户管理器
pub struct AccountManager {
    /// 用户账户存储 (user_id -> account_type -> asset -> balance)
    accounts: RwLock<HashMap<String, HashMap<AccountType, HashMap<String, AccountBalance>>>>,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(HashMap::new()),
        }
    }

    /// 获取账户余额
    pub fn get_balance(
        &self,
        user_id: &str,
        account_type: AccountType,
        asset: &str,
    ) -> Result<AccountBalance> {
        let accounts = self.accounts.read();
        if let Some(account_map) = accounts.get(user_id) {
            if let Some(balance_map) = account_map.get(&account_type) {
                if let Some(balance) = balance_map.get(asset) {
                    return Ok(balance.clone());
                }
            }
        }
        Ok(AccountBalance::new())
    }

    /// 冻结资金（用于下单）
    pub fn freeze(
        &self,
        user_id: &str,
        account_type: AccountType,
        asset: &str,
        amount: Decimal,
    ) -> Result<()> {
        let mut accounts = self.accounts.write();

        // 获取或创建账户
        let account_map = accounts
            .entry(user_id.to_string())
            .or_insert_with(|| HashMap::new());
        let balance_map = account_map
            .entry(account_type)
            .or_insert_with(|| HashMap::new());

        let balance = balance_map.entry(asset.to_string()).or_insert_with(AccountBalance::new);

        // 检查可用余额
        if balance.available < amount {
            return Err(Error::InsufficientBalance(format!(
                "Available balance {} is less than {}",
                balance.available, amount
            )));
        }

        // 冻结资金
        balance.available -= amount;
        balance.frozen += amount;

        tracing::info!(
            "Freeze: user={}, asset={}, amount={}, available={}, frozen={}",
            user_id,
            asset,
            amount,
            balance.available,
            balance.frozen
        );

        Ok(())
    }

    /// 解冻资金（用于撤单或部分成交）
    pub fn unfreeze(
        &self,
        user_id: &str,
        account_type: AccountType,
        asset: &str,
        amount: Decimal,
    ) -> Result<()> {
        let mut accounts = self.accounts.write();

        let account_map = accounts
            .entry(user_id.to_string())
            .or_insert_with(|| HashMap::new());
        let balance_map = account_map
            .entry(account_type)
            .or_insert_with(|| HashMap::new());

        let balance = balance_map.entry(asset.to_string()).or_insert_with(AccountBalance::new);

        // 解冻资金
        if balance.frozen >= amount {
            balance.frozen -= amount;
            balance.available += amount;

            tracing::info!(
                "Unfreeze: user={}, asset={}, amount={}, available={}, frozen={}",
                user_id,
                asset,
                amount,
                balance.available,
                balance.frozen
            );
            Ok(())
        } else {
            Err(Error::FreezeFailed(format!(
                "Frozen balance {} is less than {}",
                balance.frozen, amount
            )))
        }
    }

    /// 扣款（用于成交）
    pub fn deduct(
        &self,
        user_id: &str,
        account_type: AccountType,
        asset: &str,
        amount: Decimal,
    ) -> Result<()> {
        let mut accounts = self.accounts.write();

        let account_map = accounts
            .entry(user_id.to_string())
            .or_insert_with(|| HashMap::new());
        let balance_map = account_map
            .entry(account_type)
            .or_insert_with(|| HashMap::new());

        let balance = balance_map.entry(asset.to_string()).or_insert_with(AccountBalance::new);

        // 从冻结金额中扣除
        if balance.frozen >= amount {
            balance.frozen -= amount;

            tracing::info!(
                "Deduct: user={}, asset={}, amount={}, frozen={}",
                user_id,
                asset,
                amount,
                balance.frozen
            );
            Ok(())
        } else {
            Err(Error::InsufficientBalance(format!(
                "Frozen balance {} is less than {}",
                balance.frozen, amount
            )))
        }
    }

    /// 入账（用于充值或成交收款）
    pub fn credit(
        &self,
        user_id: &str,
        account_type: AccountType,
        asset: &str,
        amount: Decimal,
    ) -> Result<()> {
        let mut accounts = self.accounts.write();

        let account_map = accounts
            .entry(user_id.to_string())
            .or_insert_with(|| HashMap::new());
        let balance_map = account_map
            .entry(account_type)
            .or_insert_with(|| HashMap::new());

        let balance = balance_map.entry(asset.to_string()).or_insert_with(AccountBalance::new);

        balance.available += amount;

        // 如果是合约账户，同时更新 equity
        if account_type == AccountType::Futures {
            balance.equity += amount;
        }

        tracing::info!(
            "Credit: user={}, asset={}, amount={}, available={}",
            user_id,
            asset,
            amount,
            balance.available
        );

        Ok(())
    }

    /// 内部划转
    pub fn transfer(
        &self,
        user_id: &str,
        from_account: AccountType,
        to_account: AccountType,
        asset: &str,
        amount: Decimal,
    ) -> Result<()> {
        // 先从源账户扣款
        {
            let mut accounts = self.accounts.write();
            let account_map = accounts
                .entry(user_id.to_string())
                .or_insert_with(|| HashMap::new());
            let balance_map = account_map
                .entry(from_account)
                .or_insert_with(|| HashMap::new());
            let balance = balance_map
                .entry(asset.to_string())
                .or_insert_with(AccountBalance::new);

            if balance.available < amount {
                return Err(Error::InsufficientBalance(format!(
                    "Available balance {} is less than {}",
                    balance.available, amount
                )));
            }

            balance.available -= amount;
            // 如果是合约账户，减少 equity
            if from_account == AccountType::Futures {
                balance.equity -= amount;
            }
        }

        // 再向目标账户入账
        {
            let mut accounts = self.accounts.write();
            let account_map = accounts
                .entry(user_id.to_string())
                .or_insert_with(|| HashMap::new());
            let balance_map = account_map
                .entry(to_account)
                .or_insert_with(|| HashMap::new());
            let balance = balance_map
                .entry(asset.to_string())
                .or_insert_with(AccountBalance::new);

            balance.available += amount;
            // 如果是合约账户，增加 equity
            if to_account == AccountType::Futures {
                balance.equity += amount;
            }
        }

        tracing::info!(
            "Transfer: user={}, from={:?}, to={:?}, asset={}, amount={}",
            user_id,
            from_account,
            to_account,
            asset,
            amount
        );

        Ok(())
    }

    /// 更新合约账户权益
    pub fn update_equity(&self, user_id: &str, asset: &str, unrealized_pnl: Decimal) -> Result<()> {
        let mut accounts = self.accounts.write();

        let account_map = accounts
            .entry(user_id.to_string())
            .or_insert_with(|| HashMap::new());
        let balance_map = account_map
            .entry(AccountType::Futures)
            .or_insert_with(|| HashMap::new());

        let balance = balance_map
            .entry(asset.to_string())
            .or_insert_with(AccountBalance::new);

        // Equity = Balance + Unrealized PnL
        // 这里简化处理：直接更新 equity
        balance.equity = balance.available + balance.frozen + unrealized_pnl;

        Ok(())
    }

    /// 创建测试账户
    pub fn create_test_account(&self, user_id: &str, asset: &str, initial_balance: Decimal) {
        let mut accounts = self.accounts.write();
        let account_map = accounts
            .entry(user_id.to_string())
            .or_insert_with(|| HashMap::new());
        let balance_map = account_map
            .entry(AccountType::Spot)
            .or_insert_with(|| HashMap::new());

        let balance = balance_map
            .entry(asset.to_string())
            .or_insert_with(AccountBalance::new);

        balance.available = initial_balance;
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局账户管理器
pub type GlobalAccountManager = Arc<AccountManager>;

pub fn create_account_manager() -> GlobalAccountManager {
    Arc::new(AccountManager::new())
}