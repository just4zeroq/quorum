//! 行情数据服务

use common::*;
use parking_lot::RwLock;
use std::collections::HashMap;

/// 行情数据服务
pub struct MarketDataService {
    /// 行情数据
    tickers: RwLock<HashMap<String, Ticker>>,
    /// K线数据
    klines: RwLock<HashMap<String, Vec<Kline>>>,
    /// 实时成交
    trades: RwLock<HashMap<String, Vec<Trade>>>,
}

impl MarketDataService {
    pub fn new() -> Self {
        Self {
            tickers: RwLock::new(HashMap::new()),
            klines: RwLock::new(HashMap::new()),
            trades: RwLock::new(HashMap::new()),
        }
    }

    /// 更新行情
    pub fn update_ticker(&self, ticker: Ticker) {
        let mut tickers = self.tickers.write();
        tickers.insert(ticker.symbol.clone(), ticker);
    }

    /// 获取行情
    pub fn get_ticker(&self, symbol: &str) -> Option<Ticker> {
        let tickers = self.tickers.read();
        tickers.get(symbol).cloned()
    }

    /// 添加成交记录
    pub fn add_trade(&self, trade: Trade) {
        let mut trades = self.trades.write();
        let symbol = trade.symbol.clone();
        let entry = trades.entry(symbol).or_insert_with(Vec::new);
        entry.push(trade);
        // 只保留最近1000条
        if entry.len() > 1000 {
            entry.remove(0);
        }
    }

    /// 获取最近成交
    pub fn get_recent_trades(&self, symbol: &str, limit: usize) -> Vec<Trade> {
        let trades = self.trades.read();
        trades
            .get(symbol)
            .map(|t| t.iter().rev().take(limit).cloned().collect())
            .unwrap_or_default()
    }

    /// 更新K线
    pub fn update_kline(&self, kline: Kline) {
        let mut klines = self.klines.write();
        let key = format!("{}:{}", kline.symbol, kline.interval);
        let entry = klines.entry(key).or_insert_with(Vec::new);
        entry.push(kline);
    }
}

impl Default for MarketDataService {
    fn default() -> Self {
        Self::new()
    }
}