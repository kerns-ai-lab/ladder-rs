//! Job repository for recalculation job persistence

use crate::{JobStatus, RecalculationJob, Result};
use sqlx::SqlitePool;

/// Repository for recalculation job operations
pub struct JobRepository;

impl JobRepository {
    /// Insert a new recalculation job or return existing queued job
    pub async fn insert_job(
        _pool: &SqlitePool,
        _season_id: &str,
        _triggered_by: &str,
    ) -> Result<String> {
        todo!("SQL not yet implemented: JobRepository::insert_job")
    }

    /// Get a job by ID
    pub async fn get_by_id(_pool: &SqlitePool, _job_id: &str) -> Result<Option<RecalculationJob>> {
        todo!("SQL not yet implemented: JobRepository::get_by_id")
    }

    /// Update job status
    pub async fn update_status(
        _pool: &SqlitePool,
        _job_id: &str,
        _status: JobStatus,
    ) -> Result<()> {
        todo!("SQL not yet implemented: JobRepository::update_status")
    }

    /// Get the latest queued job for a season
    pub async fn get_latest_queued(
        _pool: &SqlitePool,
        _season_id: &str,
    ) -> Result<Option<RecalculationJob>> {
        todo!("SQL not yet implemented: JobRepository::get_latest_queued")
    }
}
