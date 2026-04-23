//! Prediction Market Order Book - 预测市场订单簿 (方案1: 单一 YES 订单簿)
//!
//! ## 设计原理
//! - 只维护 YES 订单簿，NO 价格通过互补计算展示
//! - 每个订单记录 `original_asset` 标识原始下单品种 (YES/NO)
//! - 撤单时通过 order_id 定位原始订单
//!
//! ## 价格转换 (前端展示)
//! - YES 价格: 直接展示
//! - NO 价格: SCALE - YES 价格
//!
//! ## 下单转换 (内部处理)
//! - 买 YES → YES 簿 Bid
//! - 卖 YES → YES 簿 Ask
//! - 买 NO  → YES 簿 Ask (价格取反)
//! - 卖 NO  → YES 簿 Bid (价格取反)
//!
//! ## 二元市场 (Binary Market)
//! - YES 和 NO 是互补的，价格之和 ≈ 1.0
//! - 买 YES = 卖 NO, 卖 YES = 买 NO

use std::collections::HashMap;
use ahash::AHashMap;
use crate::api::types::{MarketType, OutcomeSpec, Price, Size, OrderId, UserId, OrderAction};

pub const SCALE_PRICE: i64 = 100_000_000; // 8 位小数精度

/// 单个订单
#[derive(Debug, Clone)]
pub struct OrderEntry {
    pub order_id: OrderId,
    pub uid: UserId,                    // 用户ID - 必须
    pub price: Price,                   // YES 簿价格 (内部统一用 YES 价格)
    pub volume: Size,
    pub action: OrderAction,            // YES 簿方向
    pub original_asset: String,         // 原始资产: "1_yes" 或 "1_no"
    pub original_action: OrderAction,   // 原始下单方向
}

impl OrderEntry {
    /// 判断是否为 YES 订单
    pub fn is_yes_order(&self) -> bool {
        self.original_asset.to_lowercase().contains("yes")
    }
}

/// 价格档位 (聚合多个订单)
#[derive(Debug, Clone, Default)]
pub struct PriceLevel {
    pub orders: Vec<OrderEntry>,  // 该价格的所有订单
    pub total_volume: Size,
}

impl PriceLevel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_order(&mut self, order: OrderEntry) {
        self.total_volume += order.volume;
        self.orders.push(order);
    }

    pub fn remove_order(&mut self, order_id: OrderId) -> Option<Size> {
        if let Some(pos) = self.orders.iter().position(|o| o.order_id == order_id) {
            let removed = self.orders.remove(pos);
            self.total_volume -= removed.volume;
            Some(removed.volume)
        } else {
            None
        }
    }
}

/// 单个结果的订单簿
/// 注意: Plan 1 只存储 YES 订单簿，NO 订单通过价格取反展示
#[derive(Debug, Clone)]
pub struct OutcomeOrderBook {
    pub outcome_id: u64,
    pub outcome_name: String,
    pub asset: String,                   // YES 资产标识
    pub bids: AHashMap<Price, PriceLevel>,  // 买单 (价格降序)
    pub asks: AHashMap<Price, PriceLevel>,  // 卖单 (价格升序)
    pub orders_by_id: AHashMap<OrderId, OrderEntry>,  // order_id -> order (快速查找)
}

impl OutcomeOrderBook {
    pub fn new(outcome: &OutcomeSpec) -> Self {
        Self {
            outcome_id: outcome.outcome_id,
            outcome_name: outcome.outcome_name.clone(),
            asset: outcome.asset.clone(),
            bids: AHashMap::new(),
            asks: AHashMap::new(),
            orders_by_id: AHashMap::new(),
        }
    }

    /// 添加买单
    pub fn add_bid(&mut self, order_id: OrderId, uid: UserId, price: Price, volume: Size,
                   original_asset: String, original_action: OrderAction) {
        let entry = OrderEntry {
            order_id,
            uid,
            price,
            volume,
            action: OrderAction::Bid,
            original_asset,
            original_action,
        };

        self.orders_by_id.insert(order_id, entry.clone());
        let level = self.bids.entry(price).or_insert_with(PriceLevel::new);
        level.add_order(entry);
    }

    /// 添加卖单
    pub fn add_ask(&mut self, order_id: OrderId, uid: UserId, price: Price, volume: Size,
                   original_asset: String, original_action: OrderAction) {
        let entry = OrderEntry {
            order_id,
            uid,
            price,
            volume,
            action: OrderAction::Ask,
            original_asset,
            original_action,
        };

        self.orders_by_id.insert(order_id, entry.clone());
        let level = self.asks.entry(price).or_insert_with(PriceLevel::new);
        level.add_order(entry);
    }

    /// 取消订单 (验证所有者)
    pub fn cancel_order(&mut self, order_id: OrderId, uid: UserId) -> Option<CancelResult> {
        if let Some(order) = self.orders_by_id.remove(&order_id) {
            // 验证订单所有者
            if order.uid != uid {
                self.orders_by_id.insert(order_id, order);
                return None;
            }

            let level = match order.action {
                OrderAction::Bid => self.bids.get_mut(&order.price),
                OrderAction::Ask => self.asks.get_mut(&order.price),
            };

            if let Some(level) = level {
                level.remove_order(order_id);
                if level.orders.is_empty() {
                    match order.action {
                        OrderAction::Bid => { self.bids.remove(&order.price); }
                        OrderAction::Ask => { self.asks.remove(&order.price); }
                    }
                }
            }

            Some(CancelResult {
                order_id,
                uid: order.uid,
                price: order.price,
                volume: order.volume,
                action: order.action,
                original_asset: order.original_asset,
                original_action: order.original_action,
            })
        } else {
            None
        }
    }

    /// 获取订单所有者
    pub fn get_order_owner(&self, order_id: OrderId) -> Option<UserId> {
        self.orders_by_id.get(&order_id).map(|o| o.uid)
    }

    /// 获取最佳买价 (最高买价)
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().max().copied()
    }

    /// 获取最佳卖价 (最低卖价)
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().min().copied()
    }

    /// 获取当前价格 (中间价)
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid as f64 + ask as f64) / 2.0),
            (Some(bid), None) => Some(bid as f64),
            (None, Some(ask)) => Some(ask as f64),
            (None, None) => None,
        }
    }

    /// 获取概率价格 (转换为 0-1 范围)
    pub fn probability(&self) -> Option<f64> {
        self.mid_price().map(|p| p as f64 / SCALE_PRICE as f64)
    }

    /// 检查订单是否存在
    pub fn has_order(&self, order_id: OrderId) -> bool {
        self.orders_by_id.contains_key(&order_id)
    }
}

/// 预测市场订单簿 (方案1)
/// 只存储 YES 订单簿，NO 通过价格取反展示
#[derive(Debug, Clone)]
pub struct PredictionOrderBook {
    pub market_id: u64,
    pub market_type: MarketType,
    pub outcomes: AHashMap<String, OutcomeOrderBook>,    // asset -> orderbook
    pub outcomes_by_id: AHashMap<u64, OutcomeOrderBook>, // outcome_id -> orderbook
    // 二元市场特有
    pub yes_outcome_id: Option<u64>,  // YES 结果的 ID
    pub no_outcome_id: Option<u64>,   // NO 结果的 ID
}

impl PredictionOrderBook {
    pub fn new(market_id: u64, market_type: MarketType) -> Self {
        Self {
            market_id,
            market_type,
            outcomes: AHashMap::new(),
            outcomes_by_id: AHashMap::new(),
            yes_outcome_id: None,
            no_outcome_id: None,
        }
    }

    /// 从市场规格创建订单簿
    pub fn from_market_spec(market_id: u64, market_type: MarketType, outcomes: &[OutcomeSpec]) -> Self {
        let mut book = Self::new(market_id, market_type);

        for outcome in outcomes {
            let outcome_book = OutcomeOrderBook::new(outcome);
            book.outcomes.insert(outcome.asset.clone(), outcome_book.clone());
            book.outcomes_by_id.insert(outcome.outcome_id, outcome_book);

            // 识别 YES/NO
            let name_lower = outcome.outcome_name.to_lowercase();
            if name_lower == "yes" || name_lower == "y" {
                book.yes_outcome_id = Some(outcome.outcome_id);
            } else if name_lower == "no" || name_lower == "n" {
                book.no_outcome_id = Some(outcome.outcome_id);
            }
        }

        book
    }

    /// 判断是否为二元市场
    pub fn is_binary(&self) -> bool {
        self.market_type == MarketType::Binary
    }

    /// 获取指定结果的订单簿
    pub fn get_outcome(&self, asset: &str) -> Option<&OutcomeOrderBook> {
        self.outcomes.get(asset)
    }

    /// 获取指定结果ID的订单簿
    pub fn get_outcome_by_id(&self, outcome_id: u64) -> Option<&OutcomeOrderBook> {
        self.outcomes_by_id.get(&outcome_id)
    }

    /// 获取指定结果ID的订单簿 (可变)
    pub fn get_outcome_mut(&mut self, outcome_id: u64) -> Option<&mut OutcomeOrderBook> {
        self.outcomes_by_id.get_mut(&outcome_id)
    }

    /// 获取指定结果的当前价格 (概率)
    pub fn get_price(&self, outcome_id: u64) -> Option<f64> {
        self.outcomes_by_id.get(&outcome_id).and_then(|b| b.probability())
    }

    /// 获取所有结果的当前价格
    pub fn get_all_prices(&self) -> HashMap<u64, f64> {
        self.outcomes_by_id
            .iter()
            .filter_map(|(id, book)| book.probability().map(|p| (*id, p)))
            .collect()
    }

    /// 计算互补价格 (SCALE - price)
    pub fn complement_price(&self, price: Price) -> Price {
        SCALE_PRICE - price
    }

    /// 获取对手结果 ID (用于二元市场)
    pub fn get_opposite_outcome_id(&self, outcome_id: u64) -> Option<u64> {
        if !self.is_binary() {
            return None;
        }
        if Some(outcome_id) == self.yes_outcome_id {
            return self.no_outcome_id;
        } else if Some(outcome_id) == self.no_outcome_id {
            return self.yes_outcome_id;
        }
        None
    }

    /// 获取 NO 的展示价格 (前端用)
    /// YES 价格 → NO 价格 = SCALE - YES 价格
    pub fn get_no_display_price(&self, yes_price: Price) -> Price {
        self.complement_price(yes_price)
    }

    /// 将订单转换为 YES 簿格式
    /// 返回 (yes_price, yes_action, original_asset)
    pub fn normalize_to_yes_order(
        &self,
        outcome_id: u64,
        action: OrderAction,
        price: Price,
    ) -> Option<(Price, OrderAction, String)> {
        let is_yes = Some(outcome_id) == self.yes_outcome_id;

        if is_yes {
            // YES 订单: 直接使用
            let asset = self.yes_outcome_id
                .and_then(|id| self.outcomes_by_id.get(&id))
                .map(|b| b.asset.clone())
                .unwrap_or_default();
            Some((price, action, asset))
        } else if Some(outcome_id) == self.no_outcome_id {
            // NO 订单: 转换
            let yes_price = self.complement_price(price);
            let yes_action = action.opposite(); // 买NO = 卖YES = Ask
            let asset = self.no_outcome_id
                .and_then(|id| self.outcomes_by_id.get(&id))
                .map(|b| b.asset.clone())
                .unwrap_or_default();
            Some((yes_price, yes_action, asset))
        } else {
            None
        }
    }

    /// 添加订单 (方案1: 自动转换为 YES 簿格式)
    pub fn add_order(
        &mut self,
        outcome_id: u64,
        order_id: OrderId,
        uid: UserId,
        action: OrderAction,
        price: Price,
        volume: Size,
    ) -> Result<(), String> {
        // 转换为 YES 簿格式
        let (yes_price, yes_action, original_asset) = self
            .normalize_to_yes_order(outcome_id, action, price)
            .ok_or_else(|| format!("Unknown outcome_id: {}", outcome_id))?;

        // 添加到 YES 订单簿 (outcome_id 1 = YES)
        if let Some(yes_id) = self.yes_outcome_id {
            let book = self.outcomes_by_id.get_mut(&yes_id)
                .ok_or("YES outcome not found")?;

            match yes_action {
                OrderAction::Bid => book.add_bid(order_id, uid, yes_price, volume, original_asset, action),
                OrderAction::Ask => book.add_ask(order_id, uid, yes_price, volume, original_asset, action),
            }
        }

        Ok(())
    }

    /// 取消订单 (验证所有者)
    /// Plan 1: 只在 YES 订单簿中查找和移除
    pub fn cancel_order(
        &mut self,
        _outcome_id: u64,
        order_id: OrderId,
        uid: UserId,
    ) -> Result<CancelResult, String> {
        // Plan 1: 所有订单都在 YES 簿中，用 order_id 直接查找
        if let Some(yes_id) = self.yes_outcome_id {
            let book = self.outcomes_by_id.get_mut(&yes_id)
                .ok_or("YES outcome not found")?;

            book.cancel_order(order_id, uid)
                .ok_or_else(|| format!("Order {} not found", order_id))
        } else {
            Err("Binary market must have YES outcome".to_string())
        }
    }

    /// 获取 YES 订单簿的 L2 数据 (用于展示)
    pub fn get_yes_l2_data(&self, depth: usize) -> Option<L2Data> {
        let yes_id = self.yes_outcome_id?;
        let book = self.outcomes_by_id.get(&yes_id)?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        // Bids: 按价格降序
        let mut bid_prices: Vec<_> = book.bids.keys().collect();
        bid_prices.sort_by(|a, b| b.cmp(a));
        for price in bid_prices.iter().take(depth) {
            if let Some(level) = book.bids.get(price) {
                bids.push(L2Level {
                    price: **price,
                    volume: level.total_volume,
                });
            }
        }

        // Asks: 按价格升序
        let mut ask_prices: Vec<_> = book.asks.keys().collect();
        ask_prices.sort();
        for price in ask_prices.iter().take(depth) {
            if let Some(level) = book.asks.get(price) {
                asks.push(L2Level {
                    price: **price,
                    volume: level.total_volume,
                });
            }
        }

        Some(L2Data { bids, asks })
    }

    /// 获取 NO 的展示 L2 数据 (前端用，价格取反)
    pub fn get_no_l2_data(&self, depth: usize) -> Option<L2Data> {
        let yes_l2 = self.get_yes_l2_data(depth)?;

        let mut bids = Vec::new(); // NO Bids = YES Asks 取反
        let mut asks = Vec::new(); // NO Asks = YES Bids 取反

        // NO Bids: YES Asks 的价格取反
        for level in yes_l2.asks.iter().take(depth) {
            bids.push(L2Level {
                price: self.complement_price(level.price),
                volume: level.volume,
            });
        }

        // NO Asks: YES Bids 的价格取反
        for level in yes_l2.bids.iter().take(depth) {
            asks.push(L2Level {
                price: self.complement_price(level.price),
                volume: level.volume,
            });
        }

        Some(L2Data { bids, asks })
    }
}

/// L2 行情数据
#[derive(Debug, Clone)]
pub struct L2Level {
    pub price: Price,
    pub volume: Size,
}

#[derive(Debug, Clone)]
pub struct L2Data {
    pub bids: Vec<L2Level>,
    pub asks: Vec<L2Level>,
}

/// 取消结果
#[derive(Debug, Clone)]
pub struct CancelResult {
    pub order_id: OrderId,
    pub uid: UserId,
    pub price: Price,
    pub volume: Size,
    pub action: OrderAction,
    pub original_asset: String,
    pub original_action: OrderAction,
}

/// 市场订单簿管理器
#[derive(Debug, Clone)]
pub struct MarketOrderBookManager {
    markets: AHashMap<u64, PredictionOrderBook>,
}

impl MarketOrderBookManager {
    pub fn new() -> Self {
        Self {
            markets: AHashMap::new(),
        }
    }

    /// 创建市场订单簿
    pub fn create_market(&mut self, market_id: u64, market_type: MarketType, outcomes: &[OutcomeSpec]) {
        let book = PredictionOrderBook::from_market_spec(market_id, market_type, outcomes);
        self.markets.insert(market_id, book);
    }

    /// 获取市场订单簿
    pub fn get_market(&self, market_id: u64) -> Option<&PredictionOrderBook> {
        self.markets.get(&market_id)
    }

    /// 获取市场订单簿 (可变)
    pub fn get_market_mut(&mut self, market_id: u64) -> Option<&mut PredictionOrderBook> {
        self.markets.get_mut(&market_id)
    }

    /// 删除市场订单簿
    pub fn remove_market(&mut self, market_id: u64) {
        self.markets.remove(&market_id);
    }

    /// 在指定市场的指定结果上添加订单 (方案1)
    pub fn add_order(
        &mut self,
        market_id: u64,
        outcome_id: u64,
        order_id: OrderId,
        uid: UserId,
        action: OrderAction,
        price: Price,
        volume: Size,
    ) -> Result<(), String> {
        let market = self.markets.get_mut(&market_id)
            .ok_or_else(|| format!("Market {} not found", market_id))?;

        market.add_order(outcome_id, order_id, uid, action, price, volume)
    }

    /// 取消订单 (验证所有者)
    pub fn cancel_order(
        &mut self,
        market_id: u64,
        outcome_id: u64,
        order_id: OrderId,
        uid: UserId,
    ) -> Result<CancelResult, String> {
        let market = self.markets.get_mut(&market_id)
            .ok_or_else(|| format!("Market {} not found", market_id))?;

        market.cancel_order(outcome_id, order_id, uid)
    }

    /// 获取指定市场的价格信息
    pub fn get_prices(&self, market_id: u64) -> Option<HashMap<u64, f64>> {
        self.markets.get(&market_id).map(|m| m.get_all_prices())
    }

    /// 获取 YES 订单簿 L2 数据
    pub fn get_yes_l2(&self, market_id: u64, depth: usize) -> Option<L2Data> {
        self.markets.get(&market_id)?.get_yes_l2_data(depth)
    }

    /// 获取 NO 订单簿 L2 数据 (展示用)
    pub fn get_no_l2(&self, market_id: u64, depth: usize) -> Option<L2Data> {
        self.markets.get(&market_id)?.get_no_l2_data(depth)
    }
}

impl Default for MarketOrderBookManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_market_add_and_cancel() {
        // 创建二元市场
        let outcomes = vec![
            OutcomeSpec { outcome_id: 1, outcome_name: "yes".to_string(), asset: "1_yes".to_string() },
            OutcomeSpec { outcome_id: 2, outcome_name: "no".to_string(), asset: "1_no".to_string() },
        ];

        let mut manager = MarketOrderBookManager::new();
        manager.create_market(1, MarketType::Binary, &outcomes);

        // 添加 YES 买单: 价格 0.65, order_id = 100, uid = 1
        manager.add_order(1, 1, 100, 1, OrderAction::Bid, 65_000_000, 100).unwrap();

        // 检查 YES 订单簿有订单
        let yes_book = manager.get_market(1).unwrap().get_outcome_by_id(1).unwrap();
        assert!(yes_book.has_order(100), "YES order should exist");
        assert_eq!(yes_book.get_order_owner(100), Some(1));

        // 取消 YES 订单 (uid = 1)
        let result = manager.cancel_order(1, 1, 100, 1).unwrap();
        assert_eq!(result.order_id, 100);
        assert_eq!(result.uid, 1);
        assert_eq!(result.price, 65_000_000);
        assert_eq!(result.original_asset, "1_yes");
        assert_eq!(result.original_action, OrderAction::Bid);

        // 检查订单已移除
        let yes_book = manager.get_market(1).unwrap().get_outcome_by_id(1).unwrap();
        assert!(!yes_book.has_order(100), "YES order should be removed");
    }

    #[test]
    fn test_no_order_conversion() {
        // 创建二元市场
        let outcomes = vec![
            OutcomeSpec { outcome_id: 1, outcome_name: "yes".to_string(), asset: "1_yes".to_string() },
            OutcomeSpec { outcome_id: 2, outcome_name: "no".to_string(), asset: "1_no".to_string() },
        ];

        let mut manager = MarketOrderBookManager::new();
        manager.create_market(1, MarketType::Binary, &outcomes);

        // 添加 NO 买单 (买 NO = 卖 YES): 价格 0.35
        // 应该转换为 YES 簿 Ask at 0.65
        manager.add_order(1, 2, 200, 1, OrderAction::Bid, 35_000_000, 100).unwrap();

        // 检查 YES 订单簿: 订单应该在 Ask 侧
        let yes_book = manager.get_market(1).unwrap().get_outcome_by_id(1).unwrap();
        assert!(yes_book.has_order(200), "NO order should be in YES book");
        assert!(yes_book.best_ask().is_some());

        // 取消 NO 订单
        let result = manager.cancel_order(1, 2, 200, 1).unwrap();
        assert_eq!(result.order_id, 200);
        assert_eq!(result.original_asset, "1_no");
        assert_eq!(result.original_action, OrderAction::Bid); // 用户下的是 Bid
    }

    #[test]
    fn test_cancel_wrong_user() {
        let outcomes = vec![
            OutcomeSpec { outcome_id: 1, outcome_name: "yes".to_string(), asset: "1_yes".to_string() },
            OutcomeSpec { outcome_id: 2, outcome_name: "no".to_string(), asset: "1_no".to_string() },
        ];

        let mut manager = MarketOrderBookManager::new();
        manager.create_market(1, MarketType::Binary, &outcomes);

        // 用户1下单
        manager.add_order(1, 1, 100, 1, OrderAction::Bid, 65_000_000, 100).unwrap();

        // 用户2尝试取消用户1的订单，应该失败
        let result = manager.cancel_order(1, 1, 100, 2); // uid = 2 (wrong user)
        assert!(result.is_err(), "Order should NOT be cancelled by wrong user");

        // 订单应该还在
        let yes_book = manager.get_market(1).unwrap().get_outcome_by_id(1).unwrap();
        assert!(yes_book.has_order(100));
    }

    #[test]
    fn test_complement_price() {
        let outcomes = vec![
            OutcomeSpec { outcome_id: 1, outcome_name: "yes".to_string(), asset: "1_yes".to_string() },
            OutcomeSpec { outcome_id: 2, outcome_name: "no".to_string(), asset: "1_no".to_string() },
        ];

        let book = PredictionOrderBook::from_market_spec(1, MarketType::Binary, &outcomes);

        // YES 价格 0.65 (65_000_000) 的互补价格应该是 0.35 (35_000_000)
        assert_eq!(book.complement_price(65_000_000), 35_000_000);
        // YES 价格 0.50 的互补价格应该还是 0.50
        assert_eq!(book.complement_price(50_000_000), 50_000_000);
    }

    #[test]
    fn test_no_l2_data_display() {
        let outcomes = vec![
            OutcomeSpec { outcome_id: 1, outcome_name: "yes".to_string(), asset: "1_yes".to_string() },
            OutcomeSpec { outcome_id: 2, outcome_name: "no".to_string(), asset: "1_no".to_string() },
        ];

        let mut manager = MarketOrderBookManager::new();
        manager.create_market(1, MarketType::Binary, &outcomes);

        // 添加 YES 买单和卖单
        manager.add_order(1, 1, 100, 1, OrderAction::Bid, 60_000_000, 100).unwrap(); // 买 YES @0.60
        manager.add_order(1, 1, 101, 2, OrderAction::Ask, 70_000_000, 100).unwrap();  // 卖 YES @0.70

        // 获取 YES L2
        let yes_l2 = manager.get_yes_l2(1, 10).unwrap();
        assert_eq!(yes_l2.bids.len(), 1);
        assert_eq!(yes_l2.asks.len(), 1);
        assert_eq!(yes_l2.bids[0].price, 60_000_000);
        assert_eq!(yes_l2.asks[0].price, 70_000_000);

        // 获取 NO L2 (展示用 - 价格取反)
        let no_l2 = manager.get_no_l2(1, 10).unwrap();
        assert_eq!(no_l2.bids.len(), 1); // NO Bid = YES Ask 取反
        assert_eq!(no_l2.asks.len(), 1); // NO Ask = YES Bid 取反
        assert_eq!(no_l2.bids[0].price, 30_000_000); // 1.0 - 0.70 = 0.30
        assert_eq!(no_l2.asks[0].price, 40_000_000); // 1.0 - 0.60 = 0.40
    }

    #[test]
    fn test_normalize_to_yes_order() {
        let outcomes = vec![
            OutcomeSpec { outcome_id: 1, outcome_name: "yes".to_string(), asset: "1_yes".to_string() },
            OutcomeSpec { outcome_id: 2, outcome_name: "no".to_string(), asset: "1_no".to_string() },
        ];

        let book = PredictionOrderBook::from_market_spec(1, MarketType::Binary, &outcomes);

        // YES 买单保持不变
        let (price, action, asset) = book.normalize_to_yes_order(1, OrderAction::Bid, 65_000_000).unwrap();
        assert_eq!(price, 65_000_000);
        assert_eq!(action, OrderAction::Bid);
        assert_eq!(asset, "1_yes");

        // NO 买单转换为 YES 卖单 (价格取反，方向取反)
        let (price, action, asset) = book.normalize_to_yes_order(2, OrderAction::Bid, 35_000_000).unwrap();
        assert_eq!(price, 65_000_000); // 1.0 - 0.35 = 0.65
        assert_eq!(action, OrderAction::Ask); // Bid -> Ask
        assert_eq!(asset, "1_no");
    }
}