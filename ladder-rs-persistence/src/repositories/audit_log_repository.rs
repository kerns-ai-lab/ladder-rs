//! Audit log repository for audit log entry persistence

use crate::{AuditLogEntry, Result};
use sqlx::SqlitePool;

/// Repository for audit log operations
pub struct AuditLogRepository;

impl AuditLogRepository {
    /// Insert a new audit log entry
    pub async fn insert(
        _pool: &SqlitePool,
        _match_id: &str,
        _actor_user_id: &str,
        _before_state: serde_json::Value,
        _after_state: serde_json::Value,
    ) -> Result<AuditLogEntry> {
        todo!("SQL not yet implemented: AuditLogRepository::insert")
    }

    /// Get audit log entries for a match
    pub async fn get_for_match(_pool: &SqlitePool, _match_id: &str) -> Result<Vec<AuditLogEntry>> {
        todo!("SQL not yet implemented: AuditLogRepository::get_for_match")
    }
}
