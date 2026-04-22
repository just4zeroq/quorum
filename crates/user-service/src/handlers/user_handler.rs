//! User Handler

use salvo::prelude::*;
use std::result::Result as StdResult;

#[handler]
pub async fn register(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> StdResult<(), salvo::Error> {
    // Parse and handle register request
    let _register_req = req.parse_json::<serde_json::Value>().await?;
    res.status_code(StatusCode::OK);
    res.render(Json(serde_json::json!({ "message": "register endpoint" })));
    Ok(())
}

#[handler]
pub async fn login(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> StdResult<(), salvo::Error> {
    // Parse and handle login request
    let _login_req = req.parse_json::<serde_json::Value>().await?;
    res.status_code(StatusCode::OK);
    res.render(Json(serde_json::json!({ "message": "login endpoint" })));
    Ok(())
}

#[handler]
pub async fn logout(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> StdResult<(), salvo::Error> {
    // Parse and handle logout request
    let _logout_req = req.parse_json::<serde_json::Value>().await?;
    res.status_code(StatusCode::OK);
    res.render(Json(serde_json::json!({ "message": "logout endpoint" })));
    Ok(())
}

#[handler]
pub async fn get_wallet_nonce(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> StdResult<(), salvo::Error> {
    // Parse and handle wallet nonce request
    let _nonce_req = req.parse_json::<serde_json::Value>().await?;
    res.status_code(StatusCode::OK);
    res.render(Json(serde_json::json!({ "message": "wallet nonce endpoint" })));
    Ok(())
}

#[handler]
pub async fn wallet_login(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> StdResult<(), salvo::Error> {
    // Parse and handle wallet login request
    let _login_req = req.parse_json::<serde_json::Value>().await?;
    res.status_code(StatusCode::OK);
    res.render(Json(serde_json::json!({ "message": "wallet login endpoint" })));
    Ok(())
}

#[handler]
pub async fn get_user(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> StdResult<(), salvo::Error> {
    // Handle get user request
    let _user_req = req.parse_json::<serde_json::Value>().await?;
    res.status_code(StatusCode::OK);
    res.render(Json(serde_json::json!({ "message": "get user endpoint" })));
    Ok(())
}