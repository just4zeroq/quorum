//! 公共类型集成测试

use common::*;

#[test]
fn test_asset_from_str() {
    assert_eq!(Asset::USDT, Asset::from("USDT"));
    assert_eq!(Asset::BTC, Asset::from("btc"));
    assert_eq!(Asset::ETH, Asset::from("Eth"));
    assert_eq!(Asset::BNB, Asset::from("BNB"));
    assert_eq!(Asset::SOL, Asset::from("Sol"));
    match Asset::from("DOGE") {
        Asset::Other(s) => assert_eq!(s, "DOGE"),
        _ => panic!("Expected Other"),
    }
}

#[test]
fn test_asset_as_str() {
    assert_eq!(Asset::USDT.as_str(), "USDT");
    assert_eq!(Asset::BTC.as_str(), "BTC");
    assert_eq!(Asset::Other("DOGE".to_string()).as_str(), "DOGE");
}

#[test]
fn test_account_balance() {
    let balance = AccountBalance::new();
    assert_eq!(balance.available, rust_decimal::Decimal::ZERO);
    assert_eq!(balance.total(), rust_decimal::Decimal::ZERO);

    let mut bal = AccountBalance::new();
    bal.available = rust_decimal::Decimal::new(100, 0);
    bal.frozen = rust_decimal::Decimal::new(50, 0);
    assert_eq!(bal.total(), rust_decimal::Decimal::new(150, 0));
}

#[test]
fn test_account_balance_default() {
    let balance = AccountBalance::default();
    assert_eq!(balance.available, rust_decimal::Decimal::ZERO);
}

#[test]
fn test_side_is_buy_sell() {
    assert!(Side::Buy.is_buy());
    assert!(!Side::Buy.is_sell());
    assert!(Side::Sell.is_sell());
    assert!(!Side::Sell.is_buy());
}

#[test]
fn test_order_new_and_methods() {
    let order = Order::new(
        "usr_1".to_string(),
        "BTCUSDT".to_string(),
        Side::Buy,
        OrderType::Limit,
        rust_decimal::Decimal::new(50000, 0),
        rust_decimal::Decimal::new(1, 0),
    );

    assert_eq!(order.user_id, "usr_1");
    assert_eq!(order.symbol, "BTCUSDT");
    assert!(order.side.is_buy());
    assert_eq!(order.filled_quantity, rust_decimal::Decimal::ZERO);
    assert_eq!(order.remaining_quantity(), rust_decimal::Decimal::new(1, 0));
    assert!(!order.is_completed());
    assert_eq!(order.status, OrderStatus::Pending);
}

#[test]
fn test_order_completed_status() {
    let mut order = Order::new(
        "usr_1".to_string(),
        "BTCUSDT".to_string(),
        Side::Sell,
        OrderType::Market,
        rust_decimal::Decimal::new(50000, 0),
        rust_decimal::Decimal::new(1, 0),
    );

    assert!(!order.is_completed());

    order.status = OrderStatus::Filled;
    assert!(order.is_completed());

    order.status = OrderStatus::Cancelled;
    assert!(order.is_completed());

    order.status = OrderStatus::Rejected;
    assert!(order.is_completed());

    order.status = OrderStatus::PartiallyFilled;
    assert!(!order.is_completed());
}

#[test]
fn test_order_remaining_quantity() {
    let mut order = Order::new(
        "usr_1".to_string(),
        "ETHUSDT".to_string(),
        Side::Buy,
        OrderType::Limit,
        rust_decimal::Decimal::new(3000, 0),
        rust_decimal::Decimal::new(10, 0),
    );

    assert_eq!(order.remaining_quantity(), rust_decimal::Decimal::new(10, 0));
    order.filled_quantity = rust_decimal::Decimal::new(4, 0);
    assert_eq!(order.remaining_quantity(), rust_decimal::Decimal::new(6, 0));
    order.filled_quantity = rust_decimal::Decimal::new(10, 0);
    assert_eq!(order.remaining_quantity(), rust_decimal::Decimal::ZERO);
}

#[test]
fn test_trade_new() {
    let trade = Trade::new(
        "ord_1".to_string(),
        "ord_2".to_string(),
        "BTCUSDT".to_string(),
        "buyer_1".to_string(),
        "seller_1".to_string(),
        rust_decimal::Decimal::new(50000, 0),
        rust_decimal::Decimal::new(2, 0),
    );

    assert_eq!(trade.order_id, "ord_1");
    assert_eq!(trade.counter_order_id, "ord_2");
    assert_eq!(trade.buyer_id, "buyer_1");
    assert_eq!(trade.seller_id, "seller_1");
    assert_eq!(trade.price, rust_decimal::Decimal::new(50000, 0));
    assert_eq!(trade.quantity, rust_decimal::Decimal::new(2, 0));
    assert_eq!(trade.amount, rust_decimal::Decimal::new(100000, 0));
}

#[test]
fn test_position_new() {
    let position = Position::new(
        "usr_1".to_string(),
        "BTCUSDT".to_string(),
        PositionSide::Long,
        rust_decimal::Decimal::new(1, 0),
        rust_decimal::Decimal::new(50000, 0),
        rust_decimal::Decimal::new(10000, 0),
        10,
    );

    assert_eq!(position.user_id, "usr_1");
    assert_eq!(position.symbol, "BTCUSDT");
    assert_eq!(position.leverage, 10);
    assert_eq!(position.maintenance_margin, rust_decimal::Decimal::ZERO);
}

#[test]
fn test_orderbook_new_and_methods() {
    let mut ob = Orderbook::new("BTCUSDT".to_string());
    assert_eq!(ob.symbol, "BTCUSDT");
    assert!(ob.asks.is_empty());
    assert!(ob.bids.is_empty());
    assert!(ob.best_ask().is_none());
    assert!(ob.best_bid().is_none());
    assert!(ob.spread().is_none());

    ob.asks.push(PriceLevel {
        price: rust_decimal::Decimal::new(50001, 0),
        quantity: rust_decimal::Decimal::new(1, 0),
    });
    ob.bids.push(PriceLevel {
        price: rust_decimal::Decimal::new(49999, 0),
        quantity: rust_decimal::Decimal::new(1, 0),
    });

    assert!(ob.best_ask().is_some());
    assert!(ob.best_bid().is_some());
    assert_eq!(ob.best_ask().unwrap().price, rust_decimal::Decimal::new(50001, 0));
    assert_eq!(ob.spread().unwrap(), rust_decimal::Decimal::new(2, 0));
}

#[test]
fn test_entry_status() {
    assert_eq!(EntryStatus::Confirmed as u8, 0);
    assert_eq!(EntryStatus::Pending as u8, 1);
}

#[test]
fn test_biz_type_variants() {
    assert_eq!(BizType::Trade as u8, 0);
    assert_eq!(BizType::Fee as u8, 1);
    assert_eq!(BizType::Deposit as u8, 2);
    assert_eq!(BizType::Withdraw as u8, 3);
    assert_eq!(BizType::Transfer as u8, 6);
}

#[test]
fn test_direction() {
    assert_eq!(Direction::Debit as u8, 0);
    assert_eq!(Direction::Credit as u8, 1);
}

#[test]
fn test_kyc_status() {
    assert_eq!(KycStatus::None as u8, 0);
    assert_eq!(KycStatus::Pending as u8, 1);
    assert_eq!(KycStatus::Verified as u8, 2);
    assert_eq!(KycStatus::Rejected as u8, 3);
}

#[test]
fn test_deposit_status() {
    assert_eq!(DepositStatus::Pending as u8, 0);
    assert_eq!(DepositStatus::Credited as u8, 1);
    assert_eq!(DepositStatus::Rejected as u8, 3);
}

#[test]
fn test_withdrawal_status() {
    assert_eq!(WithdrawalStatus::Pending as u8, 0);
    assert_eq!(WithdrawalStatus::Approved as u8, 1);
    assert_eq!(WithdrawalStatus::Confirmed as u8, 3);
    assert_eq!(WithdrawalStatus::Failed as u8, 5);
}

#[test]
fn test_risk_signal_type() {
    assert_eq!(RiskSignalType::Liquidation as u8, 0);
    assert_eq!(RiskSignalType::PositionLimit as u8, 3);
}

#[test]
fn test_position_side() {
    assert_eq!(PositionSide::Long as u8, 0);
    assert_eq!(PositionSide::Short as u8, 1);
    assert_eq!(PositionSide::Both as u8, 2);
}

#[test]
fn test_kline_struct() {
    let kline = Kline {
        symbol: "BTCUSDT".to_string(),
        interval: "1h".to_string(),
        open: rust_decimal::Decimal::new(50000, 0),
        high: rust_decimal::Decimal::new(51000, 0),
        low: rust_decimal::Decimal::new(49000, 0),
        close: rust_decimal::Decimal::new(50500, 0),
        volume: rust_decimal::Decimal::new(1000, 0),
        quote_volume: rust_decimal::Decimal::new(50000000, 0),
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(kline.symbol, "BTCUSDT");
    assert_eq!(kline.interval, "1h");
}
