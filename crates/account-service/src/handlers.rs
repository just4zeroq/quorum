//! Account Service HTTP 处理器

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use common::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AccountManager;

pub type AppState = Arc<AccountManager>;

/// 获取账户余额
#[derive(Debug, Deserialize)]
pub struct GetBalanceParams {
    pub user_id: String,
    pub account_type: Option<AccountType>,
    pub asset: String,
}

pub async fn get_balance(
    State(state): State<AppState>,
    Query(params): Query<GetBalanceParams>,
) -> Result<Json<AccountBalance>, Error> {
    let account_type = params.account_type.unwrap_or(AccountType::Spot);
    let balance = state.get_balance(&params.user_id, account_type, &params.asset)?;
    Ok(Json(balance))
}

/// 冻结资金
#[derive(Debug, Deserialize)]
pub struct FreezeRequest {
    pub user_id: String,
    pub account_type: AccountType,
    pub asset: String,
    pub amount: Decimal,
}

pub async fn freeze(
    State(state): State<AppState>,
    Json(req): Json<FreezeRequest>,
) -> Result<Json<serde_json::Value>, Error> {
    state.freeze(&req.user_id, req.account_type, &req.asset, req.amount)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Funds frozen successfully"
    })))
}

/// 解冻资金
#[derive(Debug, Deserialize)]
pub struct UnfreezeRequest {
    pub user_id: String,
    pub account_type: AccountType,
    pub asset: String,
    pub amount: Decimal,
}

pub async fn unfreeze(
    State(state): State<AppState>,
    Json(req): Json<UnfreezeRequest>,
) -> Result<Json<serde_json::Value>, Error> {
    state.unfreeze(&req.user_id, req.account_type, &req.asset, req.amount)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Funds unfrozen successfully"
    })))
}

/// 扣款
#[derive(Debug, Deserialize)]
pub struct DeductRequest {
    pub user_id: String,
    pub account_type: AccountType,
    pub asset: String,
    pub amount: Decimal,
}

pub async fn deduct(
    State(state): State<AppState>,
    Json(req): Json<DeductRequest>,
) -> Result<Json<serde_json::Value>, Error> {
    state.deduct(&req.user_id, req.account_type, &req.asset, req.amount)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Funds deducted successfully"
    })))
}

/// 入账
#[derive(Debug, Deserialize)]
pub struct CreditRequest {
    pub user_id: String,
    pub account_type: AccountType,
    pub asset: String,
    pub amount: Decimal,
}

pub async fn credit(
    State(state): State<AppState>,
    Json(req): Json<CreditRequest>,
) -> Result<Json<serde_json::Value>, Error> {
    state.credit(&req.user_id, req.account_type, &req.asset, req.amount)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Funds credited successfully"
    })))
}

/// 内部划转
#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub user_id: String,
    pub from_account: AccountType,
    pub to_account: AccountType,
    pub asset: String,
    pub amount: Decimal,
}

pub async fn transfer(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<serde_json::Value>, Error> {
    state.transfer(
        &req.user_id,
        req.from_account,
        req.to_account,
        &req.asset,
        req.amount,
    )?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Transfer successful"
    })))
}

/// 更新权益
#[derive(Debug, Deserialize)]
pub struct UpdateEquityRequest {
    pub user_id: String,
    pub asset: String,
    pub unrealized_pnl: Decimal,
}

pub async fn update_equity(
    State(state): State<AppState>,
    Json(req): Json<UpdateEquityRequest>,
) -> Result<Json<serde_json::Value>, Error> {
    state.update_equity(&req.user_id, &req.asset, req.unrealized_pnl)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Equity updated successfully"
    })))
}