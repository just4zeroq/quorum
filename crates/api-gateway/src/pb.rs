//! Proto 生成模块
//!
//! 此模块由 build.rs 自动生成，包含 gRPC 服务接口定义
//!
//! 运行 `cargo build` 时会自动编译 .proto 文件

pub mod user {
    include!("pb/user.rs");
}

pub mod order {
    include!("pb/order.rs");
}

pub mod auth {
    include!("pb/auth.rs");
}
