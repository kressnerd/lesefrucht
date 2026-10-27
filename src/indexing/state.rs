//! Index state and freshness.

use crate::config::Config;
use crate::index::{now_ms, StoreStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreState {
    Ready,
    Indexing,
    Empty,
    Unavailable,
}

pub fn freshness_ok(status: &StoreStatus, config: &Config) -> bool {
    let Some(last_success_at) = status.last_success_at else {
        return false;
    };
    let bound_ms = config.index.freshness_bound_hours.saturating_mul(3_600_000);
    now_ms().saturating_sub(last_success_at) <= bound_ms
}
