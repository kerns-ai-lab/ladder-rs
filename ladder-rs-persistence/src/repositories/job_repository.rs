//! Job repository for recalculation job lifecycle management.
//!
//! Manages the `recalculation_jobs` table: inserting new jobs (with deduplication),
//! atomically claiming one job for execution, updating job status,
//! and recovering stuck jobs on startup.

use crate::{JobStatus, PersistenceError, RecalculationJob, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use std::str::FromStr;

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
        pool: &SqlitePool,
        season_id: &str,
        triggered_by: &str,
    ) -> Result<String> {
        if season_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "season_id cannot be empty".into(),
            ));
        }
        if triggered_by.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "triggered_by cannot be empty".into(),
            ));
        }

        // Check for existing queued or in_progress job
        let existing: Option<(String,)> =
            sqlx::query_as("SELECT id FROM recalculation_jobs WHERE season_id = ? AND status IN ('queued', 'in_progress') LIMIT 1")
                .bind(season_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        if let Some((existing_id,)) = existing {
            return Ok(existing_id);
        }

        let job_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO recalculation_jobs (id, season_id, triggered_by, status, created_at, updated_at) VALUES (?, ?, ?, 'queued', ?, ?)",
        )
        .bind(&job_id)
        .bind(season_id)
        .bind(triggered_by)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(job_id)
    }

    /// Atomically claim the next queued job for execution.
    ///
    /// Uses a two-step approach (SQLite doesn't support RETURNING):
    /// 1. SELECT the next queued job ID
    /// 2. UPDATE it to in_progress
    /// 3. SELECT the updated row
    ///
    /// Returns the claimed job, or None if no queued jobs exist.
    pub async fn claim_next_job(pool: &SqlitePool) -> Result<Option<RecalculationJob>> {
        // Step 1: Find the next queued job
        let queued: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM recalculation_jobs WHERE status = 'queued' ORDER BY created_at ASC LIMIT 1",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let (job_id,) = match queued {
            Some(id) => id,
            None => return Ok(None),
        };

        // Step 2: Atomically update to in_progress
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE recalculation_jobs SET status = 'in_progress', started_at = ?, updated_at = ? WHERE id = ? AND status = 'queued'",
        )
        .bind(&now)
        .bind(&now)
        .bind(&job_id)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Step 3: Fetch the claimed job
        Self::get_job(pool, &job_id).await
    }

    /// Mark a job as successfully completed.
    pub async fn mark_completed(pool: &SqlitePool, job_id: &str) -> Result<()> {
        if job_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "job_id cannot be empty".into(),
            ));
        }

        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE recalculation_jobs SET status = 'completed', completed_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(&now)
        .bind(job_id)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(PersistenceError::NotFound {
                entity: "recalculation_job".into(),
                id: job_id.to_string(),
            });
        }

        Ok(())
    }

    /// Mark a job as failed with an error message.
    pub async fn mark_failed(pool: &SqlitePool, job_id: &str, error_message: &str) -> Result<()> {
        if job_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "job_id cannot be empty".into(),
            ));
        }

        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE recalculation_jobs SET status = 'failed', error_message = ?, completed_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(error_message)
        .bind(&now)
        .bind(&now)
        .bind(job_id)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(PersistenceError::NotFound {
                entity: "recalculation_job".into(),
                id: job_id.to_string(),
            });
        }

        Ok(())
    }

    /// Get a job by ID.
    pub async fn get_job(pool: &SqlitePool, job_id: &str) -> Result<Option<RecalculationJob>> {
        if job_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "job_id cannot be empty".into(),
            ));
        }

        let row = sqlx::query_as::<_, JobRow>(
            "SELECT id, season_id, status, triggered_by, retry_count, max_retries, error_message, created_at, updated_at FROM recalculation_jobs WHERE id = ?",
        )
        .bind(job_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.into_job()?)),
            None => Ok(None),
        }
    }

    /// Reset all in_progress jobs back to queued.
    ///
    /// Called on startup before the poller loop begins, to recover jobs
    /// that were stuck from a previous process termination.
    ///
    /// Returns the count of jobs reset.
    pub async fn reset_stuck_jobs(pool: &SqlitePool) -> Result<u32> {
        let result = sqlx::query(
            "UPDATE recalculation_jobs SET status = 'queued', started_at = NULL, updated_at = ? WHERE status = 'in_progress'",
        )
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }

    /// Check if a season has a pending (queued or in_progress) recalculation job.
    pub async fn is_pending_for_season(pool: &SqlitePool, season_id: &str) -> Result<bool> {
        if season_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "season_id cannot be empty".into(),
            ));
        }

        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM recalculation_jobs WHERE season_id = ? AND status IN ('queued', 'in_progress')",
        )
        .bind(season_id)
        .fetch_one(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(result.0 > 0)
    }
}

// ── Query Row Type ──────────────────────────────────────────────────────────

/// Row type for query_as deserialization from recalculation_jobs table.
#[derive(sqlx::FromRow)]
struct JobRow {
    id: String,
    season_id: String,
    status: String,
    triggered_by: String,
    retry_count: i32,
    max_retries: i32,
    error_message: Option<String>,
    created_at: String,
    updated_at: String,
}

impl JobRow {
    fn into_job(self) -> Result<RecalculationJob> {
        let status = JobStatus::from_str(&self.status).map_err(|e| {
            PersistenceError::DatabaseError(format!("Invalid job status '{}': {}", self.status, e))
        })?;

        Ok(RecalculationJob {
            id: self.id,
            season_id: self.season_id,
            status,
            triggered_by: self.triggered_by,
            retry_count: self.retry_count,
            max_retries: self.max_retries,
            error_message: self.error_message,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| {
                    PersistenceError::DatabaseError(format!("Invalid created_at timestamp: {}", e))
                })?,
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| {
                    PersistenceError::DatabaseError(format!("Invalid updated_at timestamp: {}", e))
                })?,
        })
    }
}
