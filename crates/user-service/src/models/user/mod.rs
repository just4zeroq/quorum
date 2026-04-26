//! User Module

pub mod event;
pub mod shared;

pub use super::user_domain::model::{User, UserStatus, UserSession};
pub use event::UserEvent;
pub use shared::*;
