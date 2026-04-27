//! Position 模块 - 持仓管理
//!
//! 职责:
//! - 开仓/平仓
//! - 持仓查询
//! - 盈亏计算

use rust_decimal::Decimal;
use std::time::Duration;

use crate::errors::PortfolioError;
use crate::models::{Position, PositionSide};
use crate::repository::PortfolioRepository;

/// 持仓服务
pub struct PositionService {
    repo: PortfolioRepository,
}

impl PositionService {
    pub fn new(repo: PortfolioRepository) -> Self {
        Self { repo }
    }

    /// 开仓或加仓（加权平均价格），乐观锁重试
    pub async fn open_or_add_position(
        &self,
        user_id: &str,
        market_id: u64,
        outcome_id: u64,
        side: PositionSide,
        size: Decimal,
        price: Decimal,
    ) -> Result<Position, PortfolioError> {
        if size <= Decimal::ZERO {
            return Err(PortfolioError::InvalidOperation(
                "Position size must be positive".into(),
            ));
        }

        for attempt in 0..3 {
            let existing = self
                .repo
                .get_position(user_id, market_id as i64, outcome_id as i64, side.as_str())
                .await?;

            let mut pos = if let Some(mut p) = existing {
                // 加仓：加权平均价格
                let total_cost = p.entry_price * p.size + price * size;
                p.size += size;
                p.entry_price = total_cost / p.size;
                p.version += 1;
                p.updated_at = chrono::Utc::now();
                p
            } else {
                Position {
                    id: uuid::Uuid::new_v4().to_string(),
                    user_id: user_id.to_string(),
                    market_id,
                    outcome_id,
                    side,
                    size,
                    entry_price: price,
                    version: 0,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            };

            if self.repo.upsert_position_with_version(&mut pos).await? {
                return Ok(pos);
            }

            tracing::warn!(
                "Position version conflict on open_or_add, retrying (attempt {})",
                attempt + 1
            );
            tokio::time::sleep(Duration::from_millis(10 * (attempt + 1))).await;
        }

        Err(PortfolioError::OptimisticLockFailed(
            "open_or_add_position failed after retries".into(),
        ))
    }

    /// 平仓（减仓），乐观锁重试
    pub async fn close_position(
        &self,
        user_id: &str,
        market_id: u64,
        outcome_id: u64,
        side: PositionSide,
        size: Decimal,
    ) -> Result<Option<Position>, PortfolioError> {
        if size <= Decimal::ZERO {
            return Err(PortfolioError::InvalidOperation(
                "Close size must be positive".into(),
            ));
        }

        for attempt in 0..3 {
            let existing = self
                .repo
                .get_position(user_id, market_id as i64, outcome_id as i64, side.as_str())
                .await?;

            let mut pos = match existing {
                Some(p) => p,
                None => return Ok(None),
            };

            if pos.size < size {
                return Err(PortfolioError::InsufficientPosition {
                    available: pos.size.to_string(),
                    required: size.to_string(),
                });
            }

            pos.size -= size;
            pos.version += 1;
            pos.updated_at = chrono::Utc::now();

            if self.repo.upsert_position_with_version(&mut pos).await? {
                if pos.size > Decimal::ZERO {
                    return Ok(Some(pos));
                } else {
                    return Ok(None);
                }
            }

            tracing::warn!(
                "Position version conflict on close_position, retrying (attempt {})",
                attempt + 1
            );
            tokio::time::sleep(Duration::from_millis(10 * (attempt + 1))).await;
        }

        Err(PortfolioError::OptimisticLockFailed(
            "close_position failed after retries".into(),
        ))
    }

    /// 获取持仓
    pub async fn get_position(
        &self,
        user_id: &str,
        market_id: u64,
        outcome_id: u64,
        side: PositionSide,
    ) -> Result<Option<Position>, PortfolioError> {
        self.repo
            .get_position(user_id, market_id as i64, outcome_id as i64, side.as_str())
            .await
    }
}
