//! Persistence layer for ladder-rs
//!
//! This crate provides database access functions for the ladder-rs rating system.
//! All database operations go through this layer. Both the Axum backend and
//! swarm operators consume this library for DB access.

pub mod error;
pub mod models;
pub mod pool;
pub mod repositories;

pub use error::{PersistenceError, Result};
pub use models::*;
pub use pool::{acquire_connection, create_pool, create_pool_with_config, PoolConfig};

// Re-export commonly used items
pub use repositories::{
    audit_log_repository::AuditLogRepository,
    job_repository::JobRepository,
    match_repository::MatchRepository,
    rating_history_repository::{
        RatingHistoryEntry, RatingHistoryRepository, RatingHistoryResponse, SeasonOverviewEntry,
        SeasonOverviewResponse,
    },
};
