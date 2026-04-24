//! API 路由定义

use salvo::prelude::*;

use crate::handlers::*;
use crate::middleware::{auth, cors_handler, log_request, rate_limit};
use crate::ws_proxy::{ws_market_data_proxy, ws_order_proxy, ws_prediction_proxy};

/// 创建路由器
pub fn create_router() -> Router {
    Router::new()
        // 全局中间件
        .hoop(cors_handler())
        .hoop(log_request)
        .hoop(rate_limit)

        // 健康检查
        .push(Router::with_path("/health").get(health_check))
        .push(Router::with_path("/ready").get(ready_check))

        // 用户相关
        .push(Router::with_path("/api/v1/users")
            .push(Router::with_path("/register").post(register))
            .push(Router::with_path("/login").post(login))
            .push(Router::with_path("/refresh").post(refresh_token))
            .push(Router::with_path("/logout").post(logout).hoop(auth))
            .push(Router::with_path("/me").get(get_current_user).hoop(auth))
        )

        // 订单相关（需要认证）
        .push(Router::with_path("/api/v1/orders")
            .hoop(auth)
            .push(Router::with_path("").post(create_order).get(get_orders))
            .push(Router::with_path("/<order_id>").get(get_order).delete(cancel_order))
        )

        // 账户相关
        .push(Router::with_path("/api/v1/accounts")
            .hoop(auth)
            .push(Router::with_path("/balance").get(get_balance))
        )

        // 持仓相关
        .push(Router::with_path("/api/v1/positions")
            .hoop(auth)
            .push(Router::with_path("").get(get_positions))
        )

        // 预测市场相关（公开）
        .push(Router::with_path("/api/v1/markets")
            .push(Router::with_path("").get(list_markets))
            .push(Router::with_path("/<market_id>").get(get_market))
            .push(Router::with_path("/<market_id>/resolve").post(resolve_market).hoop(auth))
            .push(Router::with_path("/<market_id>/outcomes").get(get_market_outcomes))
            .push(Router::with_path("/<market_id>/price").get(get_market_price))
        )

        // 行情相关（公开）
        .push(Router::with_path("/api/v1/market")
            .push(Router::with_path("/depth").get(get_depth))
            .push(Router::with_path("/ticker").get(get_ticker))
            .push(Router::with_path("/kline").get(get_kline))
            .push(Router::with_path("/trades").get(get_recent_trades))
        )

        // 充值提现
        .push(Router::with_path("/api/v1/wallet")
            .hoop(auth)
            .push(Router::with_path("/deposit/address").get(get_deposit_address))
            .push(Router::with_path("/withdraw").post(withdraw))
            .push(Router::with_path("/history").get(get_wallet_history))
        )

        // WebSocket 代理
        .push(Router::with_path("/ws/market-data").get(ws_market_data_proxy))
        .push(Router::with_path("/ws/order").get(ws_order_proxy))
        .push(Router::with_path("/ws/prediction").get(ws_prediction_proxy))
}
