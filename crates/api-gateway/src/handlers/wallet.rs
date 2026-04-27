//! 钱包相关处理器

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use crate::grpc::{create_wallet_client, GrpcConfig};
use crate::handlers::{parse_json, get_user_id_or_unknown};

// ========== 请求/响应类型 ==========

#[derive(Debug, Deserialize, Default)]
pub struct DepositAddressQuery {
    pub asset: String,
    pub network: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DepositAddressResponse {
    pub address: String,
    pub asset: String,
    pub network: String,
    pub memo: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WithdrawRequest {
    pub asset: String,
    pub amount: String,
    pub address: String,
    pub network: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WithdrawResponse {
    pub success: bool,
    pub withdrawal_id: String,
    pub asset: String,
    pub amount: String,
    pub fee: String,
    pub net_amount: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct WalletHistoryQuery {
    pub asset: Option<String>,
    pub r#type: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct WalletHistoryResponse {
    pub deposits: Vec<DepositRecord>,
    pub withdrawals: Vec<WithdrawRecord>,
}

#[derive(Debug, Serialize)]
pub struct DepositRecord {
    pub id: String,
    pub asset: String,
    pub amount: String,
    pub address: String,
    pub tx_hash: String,
    pub confirmations: u32,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct WithdrawRecord {
    pub id: String,
    pub asset: String,
    pub amount: String,
    pub fee: String,
    pub address: String,
    pub tx_hash: Option<String>,
    pub status: String,
    pub created_at: String,
}

// ========== 处理器 ==========

/// 获取充值地址
#[handler]
pub async fn get_deposit_address(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let query = req.parse_queries::<DepositAddressQuery>().unwrap_or_default();
    let asset = query.asset.to_uppercase();
    let network = query.network.unwrap_or_else(|| {
        match asset.as_str() {
            "USDT" => "TRC20".to_string(),
            "TRX" => "TRC20".to_string(),
            "BTC" => "BTC".to_string(),
            _ => "ERC20".to_string(),
        }
    });

    // TODO: 调用 Wallet Service gRPC 获取真实地址
    let address = match asset.as_str() {
        "USDT" | "TRX" => format!("TN{}W", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(32).collect::<String>()),
        "BTC" => format!("bc1q{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(38).collect::<String>()),
        _ => format!("0x{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(40).collect::<String>()),
    };

    let memo = if network == "TRC20" {
        Some(uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(8).collect())
    } else {
        None
    };

    res.render(Json(DepositAddressResponse {
        address,
        asset,
        network,
        memo,
    }));

    Ok(())
}

/// 提现
#[handler]
pub async fn withdraw(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id = get_user_id_or_unknown(depot);
    let payload = parse_json::<WithdrawRequest>(req).await?;

    let config = GrpcConfig::default();
    match create_wallet_client(config.wallet_service_addr).await {
        Ok(mut client) => {
            let grpc_req = api::wallet::CreateWithdrawRequest {
                user_id: user_id.parse::<i64>().unwrap_or(0),
                asset: payload.asset.clone(),
                amount: payload.amount.clone(),
                to_address: payload.address.clone(),
                chain: payload.network.clone().unwrap_or_else(|| "ETH".to_string()),
                payment_password: String::new(),
            };

            match client.create_withdraw(grpc_req).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    let amt = payload.amount.clone();
                    res.render(Json(WithdrawResponse {
                        success: data.success,
                        withdrawal_id: data.withdraw_id,
                        asset: payload.asset.to_uppercase(),
                        amount: payload.amount,
                        fee: "0.001".to_string(),
                        net_amount: amt,
                        status: "pending".to_string(),
                    }));
                }
                Err(e) => {
                    tracing::error!("Wallet withdraw failed: {:?}", e);
                    if e.code() == tonic::Code::FailedPrecondition {
                        res.status_code(StatusCode::BAD_REQUEST);
                        res.render(Json(serde_json::json!({"error": "Insufficient balance"})));
                    } else {
                        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                        res.render(Json(serde_json::json!({"error": format!("{:?}", e)})));
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to wallet service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Wallet service unavailable"})));
        }
    }

    Ok(())
}

/// 获取钱包历史（充值 + 提现）
#[handler]
pub async fn get_wallet_history(_req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusCode> {
    let user_id_str = get_user_id_or_unknown(depot);
    let user_id = user_id_str.parse::<i64>().unwrap_or(0);

    if user_id == 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({"error": "Invalid user"})));
        return Ok(());
    }

    let config = GrpcConfig::default();
    match create_wallet_client(config.wallet_service_addr).await {
        Ok(mut client) => {
            // 充值记录
            let deposit_req = api::wallet::GetDepositHistoryRequest {
                user_id,
                chain: String::new(),
                page: 1,
                page_size: 20,
            };
            let deposits = match client.get_deposit_history(deposit_req).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    data.deposits.into_iter().map(|d| DepositRecord {
                        id: d.tx_id.clone(),
                        asset: d.chain.clone(),
                        amount: d.amount,
                        address: d.address,
                        tx_hash: d.tx_id,
                        confirmations: 0,
                        status: "completed".to_string(),
                        created_at: d.created_at.to_string(),
                    }).collect()
                }
                Err(e) => {
                    tracing::error!("Failed to get deposit history: {:?}", e);
                    Vec::new()
                }
            };

            // 提现记录
            let withdraw_req = api::wallet::GetWithdrawHistoryRequest {
                user_id,
                page: 1,
                page_size: 20,
            };
            let withdrawals = match client.get_withdraw_history(withdraw_req).await {
                Ok(resp) => {
                    let data = resp.into_inner();
                    data.withdrawals.into_iter().map(|w| WithdrawRecord {
                        id: w.withdraw_id,
                        asset: w.asset,
                        amount: w.amount,
                        fee: w.fee,
                        address: w.to_address,
                        tx_hash: if w.tx_id.is_empty() { None } else { Some(w.tx_id) },
                        status: w.status,
                        created_at: w.created_at.to_string(),
                    }).collect()
                }
                Err(e) => {
                    tracing::error!("Failed to get withdraw history: {:?}", e);
                    Vec::new()
                }
            };

            res.render(Json(WalletHistoryResponse {
                deposits,
                withdrawals,
            }));
        }
        Err(e) => {
            tracing::error!("Failed to connect to wallet service: {:?}", e);
            res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            res.render(Json(serde_json::json!({"error": "Wallet service unavailable"})));
        }
    }

    Ok(())
}
