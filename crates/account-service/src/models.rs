//! 数据模型定义
//!
//! 定义 Account Service 的核心数据模型

use serde::{Deserialize, Serialize};

/// 账户
///
/// 代表用户在某一资产上的余额账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// 账户ID
    pub id: i64,
    /// 用户ID
    pub user_id: i64,
    /// 资产标识: "USDT" 或 "{market_id}_{outcome}" (如 "12345_yes")
    pub asset: String,
    /// 资产精度 (小数位数)
    pub precision: u8,
    /// 可用余额 (最小单位整数)
    pub available: i64,
    /// 冻结余额 (最小单位整数)
    pub frozen: i64,
    /// 锁定余额 (最小单位整数)
    pub locked: i64,
    /// 创建时间 (unix timestamp ms)
    pub created_at: i64,
    /// 更新时间 (unix timestamp ms)
    pub updated_at: i64,
}

impl Account {
    /// 创建新账户
    pub fn new(user_id: i64, asset: String, precision: u8) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: 0, // 数据库自增
            user_id,
            asset,
            precision,
            available: 0,
            frozen: 0,
            locked: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// 检查是否有足够的可用余额
    pub fn has_sufficient_available(&self, amount: i64) -> bool {
        self.available >= amount
    }

    /// 检查是否有足够的冻结余额
    pub fn has_sufficient_frozen(&self, amount: i64) -> bool {
        self.frozen >= amount
    }

    /// 检查是否有足够的锁定余额
    pub fn has_sufficient_locked(&self, amount: i64) -> bool {
        self.locked >= amount
    }
}

/// 余额操作记录
///
/// 记录所有余额变更操作，用于审计和对账
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceOperation {
    /// 操作记录ID
    pub id: i64,
    /// 账户ID
    pub account_id: i64,
    /// 用户ID
    pub user_id: i64,
    /// 资产标识
    pub asset: String,
    /// 操作类型
    pub operation_type: BalanceOperationType,
    /// 操作金额 (最小单位整数)
    pub amount: i64,
    /// 操作前可用余额
    pub balance_before: i64,
    /// 操作后可用余额
    pub balance_after: i64,
    /// 操作前冻结余额
    pub frozen_before: i64,
    /// 操作后冻结余额
    pub frozen_after: i64,
    /// 原因说明
    pub reason: Option<String>,
    /// 关联ID (order_id, trade_id 等)
    pub ref_id: Option<String>,
    /// 创建时间
    pub created_at: i64,
}

impl BalanceOperation {
    /// 创建新的操作记录
    pub fn new(
        user_id: i64,
        asset: &str,
        operation_type: BalanceOperationType,
        amount: i64,
        balance_before: i64,
        balance_after: i64,
        frozen_before: i64,
        frozen_after: i64,
        ref_id: &str,
    ) -> Self {
        Self {
            id: 0, // 数据库自增
            account_id: 0,
            user_id,
            asset: asset.to_string(),
            operation_type,
            amount,
            balance_before,
            balance_after,
            frozen_before,
            frozen_after,
            reason: None,
            ref_id: Some(ref_id.to_string()),
            created_at: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// 设置原因
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

/// 余额操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BalanceOperationType {
    /// 充值
    Deposit,
    /// 提现
    Withdraw,
    /// 冻结 (下单)
    Freeze,
    /// 解冻 (撤单)
    Unfreeze,
    /// 扣减 (成交消耗)
    Deduct,
    /// 转入
    TransferIn,
    /// 转出
    TransferOut,
    /// 手续费
    Fee,
    /// 风控锁定
    Lock,
    /// 风控解锁
    Unlock,
    /// 结算派彩 (结果代币 -> 基础资产)
    Settlement,
}

impl BalanceOperationType {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Deposit => "deposit",
            Self::Withdraw => "withdraw",
            Self::Freeze => "freeze",
            Self::Unfreeze => "unfreeze",
            Self::Deduct => "deduct",
            Self::TransferIn => "transfer_in",
            Self::TransferOut => "transfer_out",
            Self::Fee => "fee",
            Self::Lock => "lock",
            Self::Unlock => "unlock",
            Self::Settlement => "settlement",
        }
    }

    /// 从字符串转换
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "deposit" => Some(Self::Deposit),
            "withdraw" => Some(Self::Withdraw),
            "freeze" => Some(Self::Freeze),
            "unfreeze" => Some(Self::Unfreeze),
            "deduct" => Some(Self::Deduct),
            "transfer_in" => Some(Self::TransferIn),
            "transfer_out" => Some(Self::TransferOut),
            "fee" => Some(Self::Fee),
            "lock" => Some(Self::Lock),
            "unlock" => Some(Self::Unlock),
            "settlement" => Some(Self::Settlement),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new(1, "USDT".to_string(), 6);
        assert_eq!(account.user_id, 1);
        assert_eq!(account.asset, "USDT");
        assert_eq!(account.precision, 6);
        assert_eq!(account.available, 0);
        assert_eq!(account.frozen, 0);
    }

    #[test]
    fn test_balance_operation_type() {
        assert_eq!(BalanceOperationType::Freeze.as_str(), "freeze");
        assert_eq!(BalanceOperationType::from_str("freeze"), Some(BalanceOperationType::Freeze));
    }
}