//! 用户设置模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 用户设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub user_id: i64,
    pub language: String,
    pub theme: String,
    pub timezone: String,
    pub notifications: Notifications,
    pub trading_preferences: TradingPreferences,
    pub show_balance: bool,
    pub show_pnl: bool,
    pub compact_view: bool,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// 通知设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notifications {
    pub email: bool,
    pub sms: bool,
    pub push: bool,
    pub order_trade: bool,
    pub price_alert: bool,
    pub deposit_withdraw: bool,
    pub system: bool,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            email: true,
            sms: false,
            push: true,
            order_trade: true,
            price_alert: true,
            deposit_withdraw: true,
            system: true,
        }
    }
}

/// 交易偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPreferences {
    pub confirm_order: bool,
    pub confirm_cancel: bool,
    pub default_order_type: String,
    pub default_time_in_force: String,
}

impl Default for TradingPreferences {
    fn default() -> Self {
        Self {
            confirm_order: true,
            confirm_cancel: false,
            default_order_type: "limit".to_string(),
            default_time_in_force: "gtc".to_string(),
        }
    }
}

/// 更新设置请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSettingsRequest {
    pub language: Option<String>,
    pub theme: Option<String>,
    pub timezone: Option<String>,
    pub notifications: Option<Notifications>,
    pub trading_preferences: Option<TradingPreferences>,
    pub show_balance: Option<bool>,
    pub show_pnl: Option<bool>,
    pub compact_view: Option<bool>,
}