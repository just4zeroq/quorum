//! OrderBook - 订单簿实现
//!
//! 使用红黑树思想的价格索引 + 每个价格档位内的 FIFO 队列

use super::types::BookOrder;
use common::*;
use std::collections::{BTreeMap, VecDeque};
use parking_lot::RwLock;

/// 价格档位
#[derive(Debug, Clone)]
pub struct PriceLevel {
    /// 价格
    pub price: Decimal,
    /// 订单队列（FIFO）
    pub orders: VecDeque<BookOrder>,
}

impl PriceLevel {
    pub fn new(price: Decimal) -> Self {
        Self {
            price,
            orders: VecDeque::new(),
        }
    }

    pub fn total_quantity(&self) -> Decimal {
        self.orders
            .iter()
            .map(|o| o.remaining_quantity())
            .sum()
    }

    pub fn add_order(&mut self, order: BookOrder) {
        self.orders.push_back(order);
    }

    /// 尝试成交，返回成交数量
    pub fn match_quantity(&mut self, quantity: Decimal) -> Decimal {
        let mut remaining = quantity;
        let mut total_matched = Decimal::ZERO;

        while remaining > Decimal::ZERO {
            if let Some(order) = self.orders.front_mut() {
                let order_remaining = order.remaining_quantity();
                if order_remaining <= remaining {
                    // 当前订单完全成交
                    order.filled_quantity = order.quantity;
                    total_matched += order_remaining;
                    remaining -= order_remaining;
                    self.orders.pop_front();
                } else {
                    // 部分成交
                    order.filled_quantity += remaining;
                    total_matched += remaining;
                    remaining = Decimal::ZERO;
                }
            } else {
                break;
            }
        }

        total_matched
    }
}

/// 订单簿
pub struct OrderBook {
    /// 交易对
    symbol: String,
    /// 卖盘（价格从低到高）
    asks: RwLock<BTreeMap<Decimal, PriceLevel>>,
    /// 买盘（价格从高到低）
    bids: RwLock<BTreeMap<Decimal, PriceLevel>>,
    /// 订单索引（order_id -> (side, price)）
    order_index: RwLock<std::collections::HashMap<String, (Side, Decimal)>>,
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            asks: RwLock::new(BTreeMap::new()),
            bids: RwLock::new(BTreeMap::new()),
            order_index: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// 添加订单
    pub fn add_order(&self, order: BookOrder) -> Result<(), Error> {
        let price = order.price;
        let side = order.side;

        match side {
            Side::Buy => {
                let mut bids = self.bids.write();
                let level = bids.entry(price).or_insert_with(|| PriceLevel::new(price));
                level.add_order(order);
            }
            Side::Sell => {
                let mut asks = self.asks.write();
                let level = asks.entry(price).or_insert_with(|| PriceLevel::new(price));
                level.add_order(order);
            }
        }

        // 添加索引
        let mut index = self.order_index.write();
        index.insert(order.id.clone(), (side, price));

        Ok(())
    }

    /// 撤销订单
    pub fn cancel_order(&self, order_id: &str) -> Result<Option<BookOrder>, Error> {
        let mut index = self.order_index.write();
        if let Some((side, price)) = index.remove(order_id) {
            match side {
                Side::Buy => {
                    let mut bids = self.bids.write();
                    if let Some(level) = bids.get_mut(price) {
                        if let Some(pos) = level.orders.iter().position(|o| o.id == order_id) {
                            let order = level.orders.remove(pos);
                            // 如果档位为空，删除该档位
                            if level.orders.is_empty() {
                                drop(bids);
                                let mut bids = self.bids.write();
                                bids.remove(price);
                            }
                            return Ok(Some(order));
                        }
                    }
                }
                Side::Sell => {
                    let mut asks = self.asks.write();
                    if let Some(level) = asks.get_mut(price) {
                        if let Some(pos) = level.orders.iter().position(|o| o.id == order_id) {
                            let order = level.orders.remove(pos);
                            if level.orders.is_empty() {
                                drop(asks);
                                let mut asks = self.asks.write();
                                asks.remove(price);
                            }
                            return Ok(Some(order));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// 获取最佳卖价（最低价）
    pub fn best_ask(&self) -> Option<(Decimal, Decimal)> {
        let asks = self.asks.read();
        if let Some((price, level)) = asks.first_key_value() {
            Some((*price, level.total_quantity()))
        } else {
            None
        }
    }

    /// 获取最佳买价（最高价）
    pub fn best_bid(&self) -> Option<(Decimal, Decimal)> {
        let bids = self.bids.read();
        if let Some((price, level)) = bids.last_key_value() {
            Some((*price, level.total_quantity()))
        } else {
            None
        }
    }

    /// 获取深度
    pub fn get_depth(&self, limit: usize) -> Orderbook {
        let asks = self.asks.read();
        let bids = self.bids.read();

        let ask_levels: Vec<PriceLevel> = asks
            .values()
            .take(limit)
            .cloned()
            .collect();

        let bid_levels: Vec<PriceLevel> = bids
            .values()
            .rev()
            .take(limit)
            .cloned()
            .collect();

        Orderbook {
            symbol: self.symbol.clone(),
            asks: ask_levels
                .into_iter()
                .map(|l| PriceLevelData {
                    price: l.price,
                    quantity: l.total_quantity(),
                })
                .collect(),
            bids: bid_levels
                .into_iter()
                .map(|l| PriceLevelData {
                    price: l.price,
                    quantity: l.total_quantity(),
                })
                .collect(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// 获取对手方最优价
    pub fn get_opposite_best(&self, side: Side) -> Option<(Decimal, Decimal)> {
        match side {
            Side::Buy => self.best_ask(),
            Side::Sell => self.best_bid(),
        }
    }

    /// 价格是否匹配
    pub fn is_price_match(&self, side: Side, price: Decimal) -> bool {
        match side {
            Side::Buy => {
                // 买单：价格 >= 卖一价
                if let Some((best_ask, _)) = self.best_ask() {
                    price >= best_ask
                } else {
                    false
                }
            }
            Side::Sell => {
                // 卖单：价格 <= 买一价
                if let Some((best_bid, _)) = self.best_bid() {
                    price <= best_bid
                } else {
                    false
                }
            }
        }
    }

    /// 尝试与对手方撮合
    pub fn match_with_counterparty(
        &self,
        side: Side,
        price: Decimal,
        quantity: Decimal,
    ) -> Vec<(BookOrder, Decimal)> {
        // 买单找卖盘，卖单找买盘
        let (counter_orders, matched) = match side {
            Side::Buy => {
                let mut asks = self.asks.write();
                let mut matched_trades = Vec::new();
                let mut remaining = quantity;

                // 从最低卖价开始
                for (_price, level) in asks.iter_mut() {
                    if *_price > price {
                        break; // 价格不匹配了
                    }

                    let matched_qty = level.match_quantity(remaining);
                    if matched_qty > Decimal::ZERO {
                        // 收集已成交的订单
                        for order in level.orders.iter() {
                            if order.filled_quantity > Decimal::ZERO {
                                matched_trades.push((order.clone(), matched_qty));
                            }
                        }
                        remaining -= matched_qty;
                        if remaining <= Decimal::ZERO {
                            break;
                        }
                    }
                }

                // 清理空档位
                asks.retain(|_, v| !v.orders.is_empty());

                (vec![], matched)
            }
            Side::Sell => {
                let mut bids = self.bids.write();
                let mut matched_trades = Vec::new();
                let mut remaining = quantity;

                // 从最高买价开始
                for (_price, level) in bids.iter_mut().rev() {
                    if *_price < price {
                        break;
                    }

                    let matched_qty = level.match_quantity(remaining);
                    if matched_qty > Decimal::ZERO {
                        for order in level.orders.iter() {
                            if order.filled_quantity > Decimal::ZERO {
                                matched_trades.push((order.clone(), matched_qty));
                            }
                        }
                        remaining -= matched_qty;
                        if remaining <= Decimal::ZERO {
                            break;
                        }
                    }
                }

                bids.retain(|_, v| !v.orders.is_empty());

                (vec![], matched)
            }
        };

        matched
    }

    /// 清空订单簿
    pub fn clear(&self) {
        self.asks.write().clear();
        self.bids.write().clear();
        self.order_index.write().clear();
    }

    /// 获取订单数量
    pub fn order_count(&self) -> usize {
        let asks = self.asks.read();
        let bids = self.bids.read();
        asks.values().map(|l| l.orders.len()).sum::<usize>()
            + bids.values().map(|l| l.orders.len()).sum::<usize>()
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new("BTC/USDT".to_string())
    }
}

/// 用于序列化的简单结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PriceLevelData {
    pub price: Decimal,
    pub quantity: Decimal,
}