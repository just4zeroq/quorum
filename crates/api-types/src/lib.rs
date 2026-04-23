//! API Types - 统一接口定义包
//!
//! 包含所有微服务的 gRPC 接口定义和数据类型
//! 服务实现和 API Gateway 都应依赖此包
//!
//! ## 设计原则
//! - 契约优先 (Contract First)
//! - 接口定义与实现分离
//! - 单一职责 - 只包含类型定义，不包含业务逻辑
//!
//! ## 模块结构
//! - `user` - 用户服务接口 (注册、登录、用户信息)
//! - `order` - 订单服务接口 (下单、撤单、查询)
//! - `auth` - 鉴权服务接口 (Token 验证、刷新)
//! - `market_data` - 行情服务接口 (订单簿、Ticker、K线)
//! - `prediction_market` - 预测市场服务接口 (市场管理、结算)
//! - `matching` - 撮合引擎接口 (订单簿操作)

pub mod user {
    include!("pb/user.rs");
}

pub mod order {
    include!("pb/order.rs");
}

pub mod auth {
    include!("pb/auth.rs");
}
