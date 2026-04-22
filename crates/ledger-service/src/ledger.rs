//! 账本管理器

use common::*;
use parking_lot::RwLock;
use rust_decimal::Decimal;
use std::collections::VecDeque;

/// 账本管理器
pub struct LedgerManager {
    /// 账本条目（append-only）
    entries: RwLock<VecDeque<LedgerEntry>>,
    /// 自增ID
    next_id: RwLock<u64>,
}

impl LedgerManager {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(VecDeque::new()),
            next_id: RwLock::new(1),
        }
    }

    /// 写入账本条目（复式记账）
    pub fn write_entry(&self, biz_type: BizType, ref_id: String, items: Vec<LedgerItem>) -> Result<LedgerEntry, Error> {
        // 验证：DEBIT 总和 == CREDIT 总和
        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;

        for item in &items {
            match item.direction {
                Direction::Debit => total_debit += item.amount,
                Direction::Credit => total_credit += item.amount,
            }
        }

        if total_debit != total_credit {
            return Err(Error::LedgerError(format!(
                "Debit {} != Credit {} - double-entry constraint violated",
                total_debit, total_credit
            )));
        }

        let id = {
            let mut next = self.next_id.write();
            let id = *next;
            *next += 1;
            id
        };

        let entry = LedgerEntry {
            id,
            biz_type,
            ref_id,
            entries: items,
            timestamp: chrono::Utc::now(),
            status: EntryStatus::Confirmed,
        };

        let mut entries = self.entries.write();
        entries.push_back(entry.clone());

        tracing::info!("Ledger entry written: id={}, biz_type={:?}", entry.id, entry.biz_type);
        Ok(entry)
    }

    /// 获取账户余额（通过重算）
    pub fn get_balance(&self, account: &str, asset: &str) -> Decimal {
        let entries = self.entries.read();
        let mut balance = Decimal::ZERO;

        for entry in entries.iter() {
            for item in &entry.entries {
                if item.account == account && item.asset == asset {
                    match item.direction {
                        Direction::Credit => balance += item.amount,
                        Direction::Debit => balance -= item.amount,
                    }
                }
            }
        }

        balance
    }

    /// 获取账户历史
    pub fn get_history(&self, account: &str, limit: usize) -> Vec<LedgerEntry> {
        let entries = self.entries.read();
        entries
            .iter()
            .filter(|e| e.entries.iter().any(|i| i.account == account))
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// 冲正
    pub fn reverse_entry(&self, original_id: u64, reason: String) -> Result<LedgerEntry, Error> {
        let entries = self.entries.read();
        let original = entries
            .iter()
            .find(|e| e.id == original_id)
            .ok_or_else(|| Error::LedgerError("Original entry not found".to_string()))?;

        // 创建冲正条目
        let reverse_items: Vec<LedgerItem> = original
            .entries
            .iter()
            .map(|item| LedgerItem {
                account: item.account.clone(),
                asset: item.asset.clone(),
                direction: match item.direction {
                    Direction::Debit => Direction::Credit,
                    Direction::Credit => Direction::Debit,
                },
                amount: item.amount,
            })
            .collect();

        drop(entries);

        self.write_entry(
            BizType::Transfer,
            format!("reverse:{}:{}", original_id, reason),
            reverse_items,
        )
    }
}

impl Default for LedgerManager {
    fn default() -> Self {
        Self::new()
    }
}