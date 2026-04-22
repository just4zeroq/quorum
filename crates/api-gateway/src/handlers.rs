//! API 处理器

use salvo::prelude::*;
use common::*;
use serde::{Deserialize, Serialize};

/// 健康检查
#[handler]
pub async fn health_check(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "ok",
        "service": "api-gateway",
    })));
}

/// 就绪检查
#[handler]
pub async fn ready_check(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "ready",
        "service": "api-gateway",
    })));
}

// ========== 用户相关 ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub message: String,
}

#[handler]
pub async fn register(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let payload = req.parse_json::<RegisterRequest>().await?;
    // TODO: 调用 User Service
    res.render(Json(RegisterResponse {
        user_id: uuid::Uuid::new_v4().to_string(),
        message: "Registration successful".to_string(),
    }));
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
}

#[handler]
pub async fn login(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let payload = req.parse_json::<LoginRequest>().await?;
    // TODO: 调用 User Service 验证
    res.render(Json(LoginResponse {
        token: "mock_token".to_string(),
        user_id: uuid::Uuid::new_v4().to_string(),
    }));
    Ok(())
}

#[handler]
pub async fn get_current_user(_req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    // TODO: 从 depot 获取 user_id
    res.render(Json(User {
        id: "mock_user_id".to_string(),
        username: "mock_user".to_string(),
        email: Some("mock@example.com".to_string()),
        phone: None,
        kyc_status: KycStatus::Verified,
        two_factor_enabled: false,
        created_at: chrono::Utc::now(),
    }));
    Ok(())
}

// ========== 订单相关 ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateOrderResponse {
    pub order: Order,
}

#[handler]
pub async fn create_order(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let payload = req.parse_json::<CreateOrderRequest>().await?;
    let order = Order::new(
        "mock_user_id".to_string(),
        payload.symbol,
        payload.side,
        payload.order_type,
        payload.price,
        payload.quantity,
    );
    res.render(Json(CreateOrderResponse { order }));
    Ok(())
}

#[handler]
pub async fn get_order(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let order_id = req.param::<String>("order_id").unwrap_or_default();
    res.render(Json(Order::new(
        "mock_user_id".to_string(),
        "BTC/USDT".to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::new(50000, 0),
        Decimal::new(1, 0),
    )));
    Ok(())
}

#[handler]
pub async fn cancel_order(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let order_id = req.param::<String>("order_id").unwrap_or_default();
    res.render(Json(serde_json::json!({
        "order_id": order_id,
        "status": "cancelled",
    })));
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct GetOrdersQuery {
    pub symbol: Option<String>,
    pub status: Option<OrderStatus>,
    pub limit: Option<usize>,
}

#[handler]
pub async fn get_orders(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let _query = req.parse_queries::<GetOrdersQuery>().unwrap_or_default();
    res.render(Json(vec![]));
    Ok(())
}

// ========== 账户相关 ==========

#[derive(Debug, Deserialize)]
pub struct GetBalanceQuery {
    pub account_type: Option<AccountType>,
}

#[handler]
pub async fn get_balance(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let _query = req.parse_queries::<GetBalanceQuery>().unwrap_or_default();
    res.render(Json(AccountBalance {
        available: Decimal::new(10000, 0),
        frozen: Decimal::new(1000, 0),
        equity: Decimal::new(11000, 0),
    }));
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransferRequest {
    pub from_account: AccountType,
    pub to_account: AccountType,
    pub asset: String,
    pub amount: Decimal,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransferResponse {
    pub success: bool,
    pub message: String,
}

#[handler]
pub async fn transfer(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let _payload = req.parse_json::<TransferRequest>().await?;
    res.render(Json(TransferResponse {
        success: true,
        message: "Transfer successful".to_string(),
    }));
    Ok(())
}

// ========== 持仓相关 ==========

#[handler]
pub async fn get_positions(_req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    res.render(Json(vec![]));
    Ok(())
}

#[handler]
pub async fn get_position(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let _symbol = req.param::<String>("symbol").unwrap_or_default();
    res.status_code(StatusCode::NOT_FOUND);
    res.render(Text::Plain("Position not found"));
    Ok(())
}

// ========== 行情相关 ==========

#[derive(Debug, Deserialize)]
pub struct DepthQuery {
    pub symbol: String,
    pub limit: Option<usize>,
}

#[handler]
pub async fn get_depth(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let query = req.parse_queries::<DepthQuery>().unwrap_or_default();
    let symbol = query.symbol.unwrap_or_else(|| "BTC/USDT".to_string());

    let mut orderbook = Orderbook::new(symbol);
    orderbook.asks.push(PriceLevel {
        price: Decimal::new(50050, 0),
        quantity: Decimal::new(10, 0),
    });
    orderbook.asks.push(PriceLevel {
        price: Decimal::new(50051, 0),
        quantity: Decimal::new(20, 0),
    });
    orderbook.bids.push(PriceLevel {
        price: Decimal::new(50049, 0),
        quantity: Decimal::new(15, 0),
    });
    orderbook.bids.push(PriceLevel {
        price: Decimal::new(50048, 0),
        quantity: Decimal::new(25, 0),
    });
    res.render(Json(orderbook));
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct TickerQuery {
    pub symbol: String,
}

#[handler]
pub async fn get_ticker(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let query = req.parse_queries::<TickerQuery>().unwrap_or_default();
    let symbol = query.symbol.unwrap_or_else(|| "BTC/USDT".to_string());
    res.render(Json(Ticker {
        symbol,
        last_price: Decimal::new(50000, 0),
        price_change: Decimal::new(1000, 0),
        price_change_percent: Decimal::new(2, 1),
        high_price: Decimal::new(51000, 0),
        low_price: Decimal::new(48000, 0),
        volume: Decimal::new(10000, 0),
        quote_volume: Decimal::new(500000000, 0),
        timestamp: chrono::Utc::now(),
    }));
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct KlineQuery {
    pub symbol: String,
    pub interval: String,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<usize>,
}

#[handler]
pub async fn get_kline(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let _query = req.parse_queries::<KlineQuery>().unwrap_or_default();
    res.render(Json(vec![]));
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct TradesQuery {
    pub symbol: String,
    pub limit: Option<usize>,
}

#[handler]
pub async fn get_recent_trades(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let _query = req.parse_queries::<TradesQuery>().unwrap_or_default();
    res.render(Json(vec![]));
    Ok(())
}

// ========== 钱包相关 ==========

#[derive(Debug, Deserialize)]
pub struct DepositAddressQuery {
    pub asset: String,
}

#[handler]
pub async fn get_deposit_address(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let query = req.parse_queries::<DepositAddressQuery>().unwrap_or_default();
    let asset = query.asset.unwrap_or_else(|| "BTC".to_string());
    res.render(Json(serde_json::json!({
        "address": "0x1234567890abcdef",
        "asset": asset,
    })));
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WithdrawRequest {
    pub asset: String,
    pub amount: Decimal,
    pub address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WithdrawResponse {
    pub withdrawal_id: String,
    pub status: String,
}

#[handler]
pub async fn withdraw(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let _payload = req.parse_json::<WithdrawRequest>().await?;
    res.render(Json(WithdrawResponse {
        withdrawal_id: uuid::Uuid::new_v4().to_string(),
        status: "pending".to_string(),
    }));
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct WalletHistoryQuery {
    pub asset: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<usize>,
}

#[handler]
pub async fn get_wallet_history(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<()> {
    let _query = req.parse_queries::<WalletHistoryQuery>().unwrap_or_default();
    res.render(Json(serde_json::json!({
        "deposits": [],
        "withdrawals": [],
    })));
    Ok(())
}