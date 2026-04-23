use serde::{Deserialize, Serialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

pub type UserId = u64;
pub type OrderId = u64;
pub type SymbolId = i32;
pub type Currency = i32;
pub type Price = i64;
pub type Size = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum OrderAction {
    Ask,
    Bid,
}

impl OrderAction {
    pub fn opposite(self) -> Self {
        match self {
            OrderAction::Ask => OrderAction::Bid,
            OrderAction::Bid => OrderAction::Ask,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum OrderType {
    Gtc,              // Good-Till-Cancel
    Ioc,              // Immediate-or-Cancel
    Fok,              // Fill-or-Kill
    FokBudget,        // FOK with budget
    IocBudget,        // IOC with budget
    PostOnly,         // 只做 Maker，不吃单
    StopLimit,        // 止损限价单
    StopMarket,       // 止损市价单
    Iceberg,          // 冰山单
    Day,              // 当日有效
    Gtd(i64),         // Good-Till-Date (时间戳)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum SymbolType {
    CurrencyExchangePair,  // 现货
    FuturesContract,       // 期货
    PerpetualSwap,         // 永续合约
    CallOption,            // 看涨期权
    PutOption,             // 看跌期权
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum CommandResultCode {
    New,
    ValidForMatchingEngine,
    Success,
    Accepted,
    
    // Auth
    AuthInvalidUser,
    
    // Risk
    RiskNsf,
    RiskInvalidReserveBidPrice,
    RiskAskPriceLowerThanFee,
    RiskMarginTradingDisabled,
    
    // Matching
    MatchingInvalidOrderBookId,
    MatchingUnknownOrderId,
    MatchingUnsupportedCommand,
    MatchingMoveFailedPriceOverRiskLimit,
    MatchingReduceFailedWrongSize,
    MatchingInvalidOrderSize,
    
    // State
    StatePersistRiskEngineFailed,
    StatePersistMatchingEngineFailed,
    
    // User
    UserMgmtUserAlreadyExists,
    
    // Other
    InvalidSymbol,
    UnsupportedSymbolType,
    BinaryCommandFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct CoreSymbolSpecification {
    pub symbol_id: SymbolId,
    pub symbol_type: SymbolType,
    pub base_currency: Currency,
    pub quote_currency: Currency,
    pub base_scale_k: i64,
    pub quote_scale_k: i64,
    pub taker_fee: i64,
    pub maker_fee: i64,
    pub margin_buy: i64,
    pub margin_sell: i64,
}

impl Default for CoreSymbolSpecification {
    fn default() -> Self {
        Self {
            symbol_id: 0,
            symbol_type: SymbolType::CurrencyExchangePair,
            base_currency: 0,
            quote_currency: 0,
            base_scale_k: 1,
            quote_scale_k: 1,
            taker_fee: 0,
            maker_fee: 0,
            margin_buy: 0,
            margin_sell: 0,
        }
    }
}

/// 市场类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketType {
    Binary,        // 二元市场 (Yes/No)
    MultiOutcome, // 多元市场 (2+ outcomes)
}

/// 预测市场规格
#[derive(Debug, Clone)]
pub struct PredictionMarketSpec {
    pub market_id: u64,
    pub market_type: MarketType,
    pub outcomes: Vec<OutcomeSpec>,
    pub stake_limit_per_market: u64,
    pub stake_limit_per_outcome: u64,
    pub settlement_price: Option<Price>,
    pub resolved_outcome: Option<u64>,
}

/// 结果规格
#[derive(Debug, Clone)]
pub struct OutcomeSpec {
    pub outcome_id: u64,
    pub outcome_name: String,
    pub asset: String,
}

impl Default for MarketType {
    fn default() -> Self {
        MarketType::Binary
    }
}

impl PredictionMarketSpec {
    pub fn new_binary(market_id: u64, yes_asset: &str, no_asset: &str) -> Self {
        Self {
            market_id,
            market_type: MarketType::Binary,
            outcomes: vec![
                OutcomeSpec {
                    outcome_id: 1,
                    outcome_name: "yes".to_string(),
                    asset: yes_asset.to_string(),
                },
                OutcomeSpec {
                    outcome_id: 2,
                    outcome_name: "no".to_string(),
                    asset: no_asset.to_string(),
                },
            ],
            stake_limit_per_market: u64::MAX,
            stake_limit_per_outcome: u64::MAX,
            settlement_price: None,
            resolved_outcome: None,
        }
    }

    pub fn new_multi_outcome(market_id: u64, outcomes: Vec<(&str, &str)>) -> Self {
        Self {
            market_id,
            market_type: MarketType::MultiOutcome,
            outcomes: outcomes
                .into_iter()
                .enumerate()
                .map(|(i, (name, asset))| OutcomeSpec {
                    outcome_id: (i + 1) as u64,
                    outcome_name: name.to_string(),
                    asset: asset.to_string(),
                })
                .collect(),
            stake_limit_per_market: u64::MAX,
            stake_limit_per_outcome: u64::MAX,
            settlement_price: None,
            resolved_outcome: None,
        }
    }
}
