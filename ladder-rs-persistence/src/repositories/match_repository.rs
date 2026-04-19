//! Match repository for match persistence operations

use crate::{Match, MatchParticipant, Result};
use sqlx::SqlitePool;

/// Repository for match operations
pub struct MatchRepository;

impl MatchRepository {
    /// Get a match by ID
    pub async fn get_by_id(_pool: &SqlitePool, _match_id: &str) -> Result<Option<Match>> {
        todo!("SQL not yet implemented: MatchRepository::get_by_id")
    }

    /// Update a match record
    pub async fn update(
        _pool: &SqlitePool,
        _match_id: &str,
        _participants: Vec<MatchParticipant>,
    ) -> Result<()> {
        todo!("SQL not yet implemented: MatchRepository::update")
    }

    /// Mark a match as corrected
    pub async fn mark_corrected(_pool: &SqlitePool, _match_id: &str) -> Result<()> {
        todo!("SQL not yet implemented: MatchRepository::mark_corrected")
    }
}
