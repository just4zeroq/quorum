//! Matching Engine - 撮合引擎核心实现

use super::orderbook::BookOrder;
use super::types::*;
use crate::orderbook::OrderBook;
use common::*;
use parking_lot::RwLock;
use std::sync::Arc;

/// 撮合引擎
pub struct MatchingEngine {
    /// 交易对 -> 订单簿
    orderbooks: RwLock<std::collections::HashMap<String, Arc<OrderBook>>>,
    /// 统计信息
    stats: RwLock<std::collections::HashMap<String, EngineStats>>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            orderbooks: RwLock::new(std::collections::HashMap::new()),
            stats: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// 获取或创建订单簿
    fn get_or_create_orderbook(&self, symbol: &str) -> Arc<OrderBook> {
        let mut orderbooks = self.orderbooks.write();
        if let Some(ob) = orderbooks.get(symbol) {
            return Arc::clone(ob);
        }
        let ob = Arc::new(OrderBook::new(symbol.to_string()));
        orderbooks.insert(symbol.to_string(), Arc::clone(&ob));
        ob
    }

    /// 处理订单
    pub fn process_order(&self, input: OrderInput) -> Result<MatchResult, Error> {
        let symbol = &input.symbol;
        let orderbook = self.get_or_create_orderbook(symbol);

        // 根据订单类型处理
        match input.order_type {
            OrderType::Limit => self.process_limit_order(input, &orderbook),
            OrderType::Market => self.process_market_order(input, &orderbook),
            OrderType::IOC => self.process_ioc_order(input, &orderbook),
            OrderType::FOK => self.process_fok_order(input, &orderbook),
            OrderType::PostOnly => self.process_post_only_order(input, &orderbook),
        }
    }

    /// 处理限价单
    fn process_limit_order(&self, input: OrderInput, orderbook: &OrderBook) -> Result<MatchResult, Error> {
        let mut trades = Vec::new();
        let mut order_updates = Vec::new();
        let mut remaining = input.quantity;
        let order_price = input.price;

        // 尝试与对手方撮合
        let opposite_side = match input.side {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        };

        // 循环撮合直到不能匹配或全部成交
        while remaining > Decimal::ZERO {
            // 获取对手方最优价
            let (best_price, best_qty) = match orderbook.get_opposite_best(input.side) {
                Some(p) => p,
                None => break,
            };

            // 检查价格是否匹配
            let price_match = match input.side {
                Side::Buy => input.price >= best_price,
                Side::Sell => input.price <= best_price,
            };

            if !price_match {
                break;
            }

            // 计算成交数量
            let match_qty = std::cmp::min(remaining, best_qty);

            // 创建成交记录（使用被动方价格作为成交价）
            let trade = Trade::new(
                input.order_id.clone(),
                "counter_order_id".to_string(), // 简化处理
                input.symbol.clone(),
                match input.side {
                    Side::Buy => input.user_id.clone(),
                    Side::Sell => "maker_user".to_string(),
                },
                match input.side {
                    Side::Buy => "maker_user".to_string(),
                    Side::Sell => input.user_id.clone(),
                },
                best_price,
                match_qty,
            );
            trades.push(trade);

            remaining -= match_qty;

            // 更新统计
            self.update_stats(&input.symbol, match_qty, best_price * match_qty);
        }

        // 如果有剩余，挂在订单簿上
        let fully_filled = remaining == Decimal::ZERO;
        if !fully_filled {
            let book_order = BookOrder::new(
                input.order_id.clone(),
                input.user_id.clone(),
                input.side,
                input.price,
                remaining,
                input.order_type,
            );
            orderbook.add_order(book_order)?;
        }

        // 更新订单状态
        order_updates.push(OrderUpdate {
            order_id: input.order_id,
            filled_quantity: input.quantity - remaining,
            status: if fully_filled { OrderStatus::Filled } else { OrderStatus::PartiallyFilled },
        });

        Ok(MatchResult {
            trades,
            order_updates,
            fully_filled,
            remaining_quantity: remaining,
        })
    }

    /// 处理市价单
    fn process_market_order(&self, input: OrderInput, orderbook: &OrderBook) -> Result<MatchResult, Error> {
        let mut trades = Vec::new();
        let mut order_updates = Vec::new();
        let mut remaining = input.quantity;

        // 市价单会以对手方最优价逐档成交
        while remaining > Decimal::ZERO {
            let (best_price, best_qty) = match orderbook.get_opposite_best(input.side) {
                Some(p) => p,
                None => break,
            };

            let match_qty = std::cmp::min(remaining, best_qty);

            let trade = Trade::new(
                input.order_id.clone(),
                "counter_order_id".to_string(),
                input.symbol.clone(),
                match input.side {
                    Side::Buy => input.user_id.clone(),
                    Side::Sell => "maker_user".to_string(),
                },
                match input.side {
                    Side::Buy => "maker_user".to_string(),
                    Side::Sell => input.user_id.clone(),
                },
                best_price,
                match_qty,
            );
            trades.push(trade);

            remaining -= match_qty;
            self.update_stats(&input.symbol, match_qty, best_price * match_qty);
        }

        let fully_filled = remaining == Decimal::ZERO;

        order_updates.push(OrderUpdate {
            order_id: input.order_id,
            filled_quantity: input.quantity - remaining,
            status: if fully_filled { OrderStatus::Filled } else { OrderStatus::Cancelled },
        });

        Ok(MatchResult {
            trades,
            order_updates,
            fully_filled,
            remaining_quantity: remaining,
        })
    }

    /// 处理 IOC 订单
    fn process_ioc_order(&self, input: OrderInput, orderbook: &OrderBook) -> Result<MatchResult, Error> {
        // IOC = 立即匹配能成交的部分，剩余取消
        // 和限价单类似，但不挂剩余部分
        let mut result = self.process_limit_order(input, orderbook)?;

        // 如果有剩余，取消剩余订单
        if !result.fully_filled {
            result.remaining_quantity = Decimal::ZERO;
            result.order_updates[0].status = OrderStatus::Cancelled;
        }

        Ok(result)
    }

    /// 处理 FOK 订单
    fn process_fok_order(&self, input: OrderInput, orderbook: &OrderBook) -> Result<MatchResult, Error> {
        // FOK = 全部成交或全部取消
        // 先检查能否全部成交

        let mut total_available = Decimal::ZERO;
        let opposite_side = match input.side {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        };

        // 计算对手方全部挂单量
        match opposite_side {
            Side::Sell => {
                let asks = orderbook.asks.read();
                for (price, level) in asks.iter() {
                    if match input.side {
                        Side::Buy => input.price >= *price,
                        _ => false,
                    } {
                        total_available += level.total_quantity();
                    }
                }
            }
            Side::Buy => {
                let bids = orderbook.bids.read();
                for (price, level) in bids.iter().rev() {
                    if match input.side {
                        Side::Sell => input.price <= *price,
                        _ => false,
                    } {
                        total_available += level.total_quantity();
                    }
                }
            }
        }

        // 如果对手方总量不足，返回拒绝
        if total_available < input.quantity {
            return Err(Error::MatchingError("Insufficient liquidity for FOK order".to_string()));
        }

        // 尝试全部成交
        self.process_limit_order(input, orderbook)
    }

    /// 处理 PostOnly 订单
    fn process_post_only_order(&self, input: OrderInput, orderbook: &OrderBook) -> Result<MatchResult, Error> {
        // PostOnly = 如果会立即成交则拒绝，否则挂单

        // 检查是否立即成交
        if orderbook.is_price_match(input.side, input.price) {
            return Err(Error::MatchingError("PostOnly order would execute immediately".to_string()));
        }

        // 不成交，直接挂单
        let book_order = BookOrder::new(
            input.order_id.clone(),
            input.user_id.clone(),
            input.side,
            input.price,
            input.quantity,
            input.order_type,
        );
        orderbook.add_order(book_order)?;

        Ok(MatchResult {
            trades: vec![],
            order_updates: vec![OrderUpdate {
                order_id: input.order_id,
                filled_quantity: Decimal::ZERO,
                status: OrderStatus::Submitted,
            }],
            fully_filled: false,
            remaining_quantity: input.quantity,
        })
    }

    /// 撤销订单
    pub fn cancel_order(&self, input: CancelInput) -> Result<Option<BookOrder>, Error> {
        let orderbook = self.get_or_create_orderbook(&input.symbol);
        orderbook.cancel_order(&input.order_id)
    }

    /// 获取深度
    pub fn get_depth(&self, symbol: &str, limit: usize) -> Orderbook {
        let orderbook = self.get_or_create_orderbook(symbol);
        orderbook.get_depth(limit)
    }

    /// 获取订单簿统计
    pub fn get_stats(&self, symbol: &str) -> Option<EngineStats> {
        let stats = self.stats.read();
        stats.get(symbol).cloned()
    }

    /// 更新统计
    fn update_stats(&self, symbol: &str, volume: Decimal, quote_volume: Decimal) {
        let mut stats = self.stats.write();
        let entry = stats.entry(symbol.to_string()).or_insert_with(|| EngineStats {
            symbol: symbol.to_string(),
            ..Default::default()
        });
        entry.total_trades += 1;
        entry.total_volume += volume;
        entry.total_quote_volume += quote_volume;
        entry.last_price = Some(quote_volume / volume);
        entry.updated_at = chrono::Utc::now().timestamp_millis();
    }
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_order_match() {
        let engine = MatchingEngine::new();

        // 先下一个卖单
        let sell_order = OrderInput {
            order_id: "order1".to_string(),
            user_id: "user1".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: Side::Sell,
            order_type: OrderType::Limit,
            price: Decimal::new(50000, 0),
            quantity: Decimal::new(1, 0),
        };
        let result = engine.process_order(sell_order).unwrap();
        assert!(result.trades.is_empty()); // 没有对手方，挂单

        // 再下一个买单
        let buy_order = OrderInput {
            order_id: "order2".to_string(),
            user_id: "user2".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Decimal::new(50000, 0),
            quantity: Decimal::new(1, 0),
        };
        let result = engine.process_order(buy_order).unwrap();
        assert!(!result.trades.is_empty());
        assert!(result.fully_filled);
    }
}