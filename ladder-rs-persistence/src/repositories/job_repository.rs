//! Job repository for recalculation job lifecycle management.
//!
//! Manages the `recalculation_jobs` table: inserting new jobs (with deduplication),
//! atomically claiming one job for execution, updating job status,
//! and recovering stuck jobs on startup.

use crate::{PersistenceError, RecalculationJob, Result};
use sqlx::SqlitePool;

/// Repository for recalculation job operations
pub struct JobRepository;

impl JobRepository {
    /// Insert a new recalculation job or return existing queued job.
    ///
    /// Deduplication: if a `queued` or `in_progress` job already exists for this season,
    /// return the existing job ID instead of inserting. Only inserts a new job if no
    /// pending job exists.
    ///
    /// Returns the job ID (either newly created or existing).
    pub async fn insert_job(
        _pool: &SqlitePool,
        _season_id: &str,
        _triggered_by: &str,
    ) -> Result<String> {
        Err(PersistenceError::Unknown(
            "insert_job not yet implemented".into(),
        ))
    }

    /// Atomically claim the next queued job for execution.
    ///
    /// Uses a single SQL statement:
    /// ```sql
    /// UPDATE recalculation_jobs
    /// SET status = 'in_progress', started_at = CURRENT_TIMESTAMP
    /// WHERE id = (
    ///     SELECT id FROM recalculation_jobs
    ///     WHERE status = 'queued'
    ///     ORDER BY triggered_at ASC LIMIT 1
    /// )
    /// RETURNING *;
    /// ```
    ///
    /// Returns the claimed job, or None if no queued jobs exist.
    /// Safe under SQLite's serialized write model.
    pub async fn claim_next_job(_pool: &SqlitePool) -> Result<Option<RecalculationJob>> {
        Err(PersistenceError::Unknown(
            "claim_next_job not yet implemented".into(),
        ))
    }

    /// Mark a job as successfully completed.
    pub async fn mark_completed(_pool: &SqlitePool, _job_id: &str) -> Result<()> {
        Err(PersistenceError::Unknown(
            "mark_completed not yet implemented".into(),
        ))
    }

    /// Mark a job as failed with an error message.
    pub async fn mark_failed(
        _pool: &SqlitePool,
        _job_id: &str,
        _error_message: &str,
    ) -> Result<()> {
        Err(PersistenceError::Unknown(
            "mark_failed not yet implemented".into(),
        ))
    }

    /// Get a job by ID.
    pub async fn get_job(_pool: &SqlitePool, _job_id: &str) -> Result<Option<RecalculationJob>> {
        Err(PersistenceError::Unknown(
            "get_job not yet implemented".into(),
        ))
    }

    /// Reset all in_progress jobs back to queued.
    ///
    /// Called on startup before the poller loop begins, to recover jobs
    /// that were stuck from a previous process termination.
    ///
    /// ```sql
    /// UPDATE recalculation_jobs
    /// SET status = 'queued', started_at = NULL
    /// WHERE status = 'in_progress';
    /// ```
    ///
    /// Returns the count of jobs reset.
    pub async fn reset_stuck_jobs(_pool: &SqlitePool) -> Result<u32> {
        Err(PersistenceError::Unknown(
            "reset_stuck_jobs not yet implemented".into(),
        ))
    }

    /// Check if a season has a pending (queued or in_progress) recalculation job.
    pub async fn is_pending_for_season(_pool: &SqlitePool, _season_id: &str) -> Result<bool> {
        Err(PersistenceError::Unknown(
            "is_pending_for_season not yet implemented".into(),
        ))
    }
}
