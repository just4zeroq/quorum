//! 仓位管理器

use common::*;
use parking_lot::RwLock;
use std::collections::HashMap;

/// 仓位管理器
pub struct PositionManager {
    /// 持仓存储 (position_id -> Position)
    positions: RwLock<HashMap<String, Position>>,
    /// 用户索引 (user_id:symbol -> position_id)
    user_index: RwLock<HashMap<String, String>>,
}

impl PositionManager {
    pub fn new() -> Self {
        Self {
            positions: RwLock::new(HashMap::new()),
            user_index: RwLock::new(HashMap::new()),
        }
    }

    /// 开仓
    pub fn open_position(
        &self,
        user_id: String,
        symbol: String,
        side: PositionSide,
        quantity: Decimal,
        entry_price: Decimal,
        margin: Decimal,
        leverage: u32,
    ) -> Result<Position, Error> {
        let position = Position::new(user_id.clone(), symbol.clone(), side, quantity, entry_price, margin, leverage);

        let key = format!("{}:{}", user_id, symbol);
        let mut index = self.user_index.write();
        index.insert(key, position.id.clone());

        let mut positions = self.positions.write();
        positions.insert(position.id.clone(), position.clone());

        tracing::info!("Position opened: {}", position.id);
        Ok(position)
    }

    /// 更新持仓（成交后）
    pub fn update_position(&self, position_id: &str, filled_quantity: Decimal, price: Decimal) -> Result<Position, Error> {
        let mut positions = self.positions.write();
        if let Some(pos) = positions.get_mut(position_id) {
            pos.quantity += filled_quantity;
            pos.updated_at = chrono::Utc::now();
            Ok(pos.clone())
        } else {
            Err(Error::PositionNotFound(position_id.to_string()))
        }
    }

    /// 平仓
    pub fn close_position(&self, position_id: &str, quantity: Decimal) -> Result<Position, Error> {
        let mut positions = self.positions.write();
        if let Some(pos) = positions.get_mut(position_id) {
            if pos.quantity < quantity {
                return Err(Error::Business("Insufficient position quantity".to_string()));
            }
            pos.quantity -= quantity;
            pos.updated_at = chrono::Utc::now();

            if pos.quantity == Decimal::ZERO {
                let user_id = pos.user_id.clone();
                let symbol = pos.symbol.clone();
                positions.remove(position_id);

                // 清理索引
                let mut index = self.user_index.write();
                index.remove(&format!("{}:{}", user_id, symbol));
            }
            Ok(pos.clone())
        } else {
            Err(Error::PositionNotFound(position_id.to_string()))
        }
    }

    /// 获取持仓
    pub fn get_position(&self, position_id: &str) -> Result<Position, Error> {
        let positions = self.positions.read();
        positions
            .get(position_id)
            .cloned()
            .ok_or_else(|| Error::PositionNotFound(position_id.to_string()))
    }

    /// 获取用户持仓
    pub fn get_user_position(&self, user_id: &str, symbol: &str) -> Option<Position> {
        let key = format!("{}:{}", user_id, symbol);
        let index = self.user_index.read();
        if let Some(position_id) = index.get(&key) {
            let positions = self.positions.read();
            positions.get(position_id).cloned()
        } else {
            None
        }
    }

    /// 获取用户所有持仓
    pub fn get_user_positions(&self, user_id: &str) -> Vec<Position> {
        let positions = self.positions.read();
        positions
            .values()
            .filter(|p| p.user_id == user_id)
            .cloned()
            .collect()
    }
}

impl Default for PositionManager {
    fn default() -> Self {
        Self::new()
    }
}