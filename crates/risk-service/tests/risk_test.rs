//! Risk Service 单元测试
//!
//! Risk Service 是无数据库的纯规则引擎服务，因此测试均为纯单元测试。

#[cfg(test)]
mod rules_test {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    use risk_service::rules::{self, RiskCheckContext};

    fn make_ctx(
        user_id: &str,
        side: &str,
        order_type: &str,
        price: &str,
        quantity: &str,
    ) -> RiskCheckContext {
        RiskCheckContext {
            user_id: user_id.to_string(),
            market_id: 1,
            outcome_id: 1,
            side: side.to_string(),
            order_type: order_type.to_string(),
            price: Decimal::from_str(price).unwrap(),
            quantity: Decimal::from_str(quantity).unwrap(),
        }
    }

    // ==================== Basic Validation Tests ====================

    #[test]
    fn test_empty_user_id_rejected() {
        let ctx = make_ctx("", "YES", "limit", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("user_id"));
    }

    #[test]
    fn test_empty_side_rejected() {
        let ctx = make_ctx("usr_001", "", "limit", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("side"));
    }

    #[test]
    fn test_invalid_side_rejected() {
        let ctx = make_ctx("usr_001", "INVALID", "limit", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("side"));
    }

    #[test]
    fn test_case_insensitive_side() {
        let ctx = make_ctx("usr_001", "yes", "limit", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);

        let ctx = make_ctx("usr_001", "YeS", "limit", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);

        let ctx = make_ctx("usr_001", "NO", "limit", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);
    }

    #[test]
    fn test_empty_order_type_rejected() {
        let ctx = make_ctx("usr_001", "YES", "", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("order type"));
    }

    #[test]
    fn test_invalid_order_type_rejected() {
        let ctx = make_ctx("usr_001", "YES", "stop_loss", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("order type"));
    }

    #[test]
    fn test_case_insensitive_order_type() {
        let ctx = make_ctx("usr_001", "YES", "LIMIT", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);

        let ctx = make_ctx("usr_001", "NO", "Market", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);
    }

    // ==================== Price Validation Tests ====================

    #[test]
    fn test_price_zero_rejected() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0", "100");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("Price"));
        assert!(result.reason.contains("positive"));
    }

    #[test]
    fn test_price_negative_rejected() {
        let ctx = make_ctx("usr_001", "YES", "limit", "-0.1", "100");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("Price"));
    }

    #[test]
    fn test_price_equal_one_rejected() {
        let ctx = make_ctx("usr_001", "YES", "limit", "1.0", "100");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("1.0"));
    }

    #[test]
    fn test_price_above_one_rejected() {
        let ctx = make_ctx("usr_001", "YES", "limit", "1.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("1.0"));
    }

    #[test]
    fn test_price_boundary_accepted() {
        // price 0.001 (just above 0)
        let ctx = make_ctx("usr_001", "YES", "limit", "0.001", "100");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);

        // price 0.999 (just below 1)
        let ctx = make_ctx("usr_001", "YES", "limit", "0.999", "100");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);
    }

    // ==================== Quantity Validation Tests ====================

    #[test]
    fn test_quantity_zero_rejected() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0.5", "0");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("Quantity"));
    }

    #[test]
    fn test_quantity_negative_rejected() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0.5", "-10");
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("Quantity"));
    }

    // ==================== Happy Path Tests ====================

    #[test]
    fn test_valid_yes_limit_order() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0.5", "100");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);
        assert!(result.reason.is_empty());
    }

    #[test]
    fn test_valid_no_limit_order() {
        let ctx = make_ctx("usr_001", "NO", "limit", "0.3", "50");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);
    }

    #[test]
    fn test_valid_yes_market_order() {
        let ctx = make_ctx("usr_001", "YES", "market", "0.6", "200");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);
    }

    #[test]
    fn test_valid_no_market_order() {
        let ctx = make_ctx("usr_001", "NO", "market", "0.4", "75");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);
    }

    #[test]
    fn test_small_price_accepted() {
        let ctx = make_ctx("usr_001", "YES", "limit", "0.01", "1000");
        let result = rules::evaluate(&ctx);
        assert!(result.accepted);
    }

    // ==================== First-fail Semantics ====================

    #[test]
    fn test_first_error_returned() {
        // Multiple errors: empty user_id AND invalid price
        // Should report user_id first (order of checks)
        let ctx = RiskCheckContext {
            user_id: String::new(),
            market_id: 1,
            outcome_id: 1,
            side: "YES".to_string(),
            order_type: "limit".to_string(),
            price: Decimal::from_str("2.0").unwrap(),
            quantity: Decimal::from_str("100").unwrap(),
        };
        let result = rules::evaluate(&ctx);
        assert!(!result.accepted);
        assert!(result.reason.contains("user_id"));
    }
}

#[cfg(test)]
mod risk_manager_test {
    use risk_service::RiskServiceImpl;
    use api::risk::CheckRiskRequest;
    use api::RiskService;

    // ==================== gRPC Service Tests ====================

    #[tokio::test]
    async fn test_check_risk_accepted() {
        let service = RiskServiceImpl::default();
        let request = tonic::Request::new(CheckRiskRequest {
            user_id: "usr_001".to_string(),
            market_id: 1,
            outcome_id: 1,
            side: "YES".to_string(),
            order_type: "limit".to_string(),
            price: "0.5".to_string(),
            quantity: "100".to_string(),
        });

        let response = service.check_risk(request).await.unwrap();
        let result = response.into_inner();
        assert!(result.accepted);
        assert!(result.reason.is_empty());
    }

    #[tokio::test]
    async fn test_check_risk_invalid_price() {
        let service = RiskServiceImpl::default();
        let request = tonic::Request::new(CheckRiskRequest {
            user_id: "usr_001".to_string(),
            market_id: 1,
            outcome_id: 1,
            side: "YES".to_string(),
            order_type: "limit".to_string(),
            price: "invalid".to_string(),
            quantity: "100".to_string(),
        });

        let response = service.check_risk(request).await;
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_check_risk_rejected() {
        let service = RiskServiceImpl::default();
        let request = tonic::Request::new(CheckRiskRequest {
            user_id: "".to_string(),
            market_id: 1,
            outcome_id: 1,
            side: "YES".to_string(),
            order_type: "limit".to_string(),
            price: "0.5".to_string(),
            quantity: "100".to_string(),
        });

        let response = service.check_risk(request).await.unwrap();
        let result = response.into_inner();
        assert!(!result.accepted);
        assert!(!result.reason.is_empty());
    }

    #[tokio::test]
    async fn test_check_risk_price_out_of_range() {
        let service = RiskServiceImpl::default();
        let request = tonic::Request::new(CheckRiskRequest {
            user_id: "usr_001".to_string(),
            market_id: 1,
            outcome_id: 1,
            side: "YES".to_string(),
            order_type: "limit".to_string(),
            price: "0".to_string(),
            quantity: "100".to_string(),
        });

        let response = service.check_risk(request).await.unwrap();
        let result = response.into_inner();
        assert!(!result.accepted);
        assert!(result.reason.contains("positive"));
    }
}
