//! Match repository for match persistence operations
//!
//! The most complex repository. Records a complete match atomically: match header,
//! participants, rating computation, and rating snapshots. Also provides duplicate
//! detection and season write protection.

use crate::error::{PersistenceError, Result};
use crate::{Match, MatchParticipant, RatingSnapshot};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Result of recording a match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub match_id: String,
    pub snapshots: Vec<RatingSnapshot>,
}

/// A batch match entry for record_match_batch
#[derive(Debug, Clone)]
pub struct BatchEntry {
    pub participants: Vec<MatchParticipant>,
    pub score_metadata: Option<serde_json::Value>,
    pub recorded_at: DateTime<Utc>,
}

/// Result of a single batch entry
#[derive(Debug, Clone)]
pub struct BatchEntryResult {
    pub match_id: String,
    pub snapshots: Vec<RatingSnapshot>,
}

/// Filter options for listing matches
#[derive(Debug, Clone)]
pub struct MatchFilter {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub player_id: Option<String>,
}

/// Correction payload for correcting a match
#[derive(Debug, Clone)]
pub struct MatchCorrection {
    pub new_participants: Vec<MatchParticipant>,
    pub reason: String,
    pub score_metadata: Option<serde_json::Value>,
}

/// Repository for match operations
pub struct MatchRepository;

impl MatchRepository {
    /// Get a match by ID
    pub async fn get_by_id(_pool: &SqlitePool, _match_id: &str) -> Result<Option<Match>> {
        Err(PersistenceError::Unknown(
            "get_by_id not yet implemented".into(),
        ))
    }

    /// Atomically record a match with participants, rating computation, and snapshots.
    ///
    /// Transaction steps:
    /// 1. Check is_season_closed — reject if closed
    /// 2. Check is_duplicate — reject if duplicate
    /// 3. INSERT match header
    /// 4. INSERT match_participants
    /// 5. Compute ratings via Rating Engine Bridge
    /// 6. INSERT rating_snapshots
    pub async fn record_match(
        _pool: &SqlitePool,
        _season_id: &str,
        _participants: Vec<MatchParticipant>,
        _score_metadata: Option<serde_json::Value>,
        _recorded_at: DateTime<Utc>,
    ) -> Result<MatchResult> {
        Err(PersistenceError::Unknown(
            "record_match not yet implemented".into(),
        ))
    }

    /// Record multiple matches in batch
    pub async fn record_match_batch(
        _pool: &SqlitePool,
        _season_id: &str,
        _entries: Vec<BatchEntry>,
    ) -> Result<Vec<BatchEntryResult>> {
        Err(PersistenceError::Unknown(
            "record_match_batch not yet implemented".into(),
        ))
    }

    /// List matches in a season with optional filtering
    pub async fn list_matches(
        _pool: &SqlitePool,
        _season_id: &str,
        _filter: &MatchFilter,
    ) -> Result<Vec<Match>> {
        Err(PersistenceError::Unknown(
            "list_matches not yet implemented".into(),
        ))
    }

    /// Correct a match: update participants, insert audit log, queue recalculation job.
    /// All within a single transaction.
    pub async fn correct_match(
        _pool: &SqlitePool,
        _match_id: &str,
        _correction: &MatchCorrection,
        _changed_by: &str,
    ) -> Result<String> {
        // Returns job_id
        Err(PersistenceError::Unknown(
            "correct_match not yet implemented".into(),
        ))
    }

    /// Check if a match is a duplicate based on participants, placements, and timestamp
    pub async fn is_duplicate(
        _pool: &SqlitePool,
        _season_id: &str,
        _participants: &[MatchParticipant],
        _recorded_at: &DateTime<Utc>,
    ) -> Result<bool> {
        Err(PersistenceError::Unknown(
            "is_duplicate not yet implemented".into(),
        ))
    }

    /// Check if a season is closed for writing
    pub async fn is_season_closed(_pool: &SqlitePool, _season_id: &str) -> Result<bool> {
        Err(PersistenceError::Unknown(
            "is_season_closed not yet implemented".into(),
        ))
    }
}
