//! Handler for GET /api/jobs/{id} - job status endpoint

use crate::Result;
use axum::{extract::Path, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use ladder_rs_persistence::JobStatus;
use serde::Serialize;

/// Response body for job status
#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: JobStatus,
    pub season_id: String,
    pub triggered_by: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// GET /api/jobs/{id} - Get recalculation job status
///
/// Returns the status, retry count, and other details of a recalculation job.
pub async fn get_job_status(
    Path(job_id): Path<String>,
) -> Result<(StatusCode, Json<JobStatusResponse>)> {
    // Implementation will be added in task 900.1.3
    // For now, return a placeholder response
    Ok((
        StatusCode::OK,
        Json(JobStatusResponse {
            job_id,
            status: JobStatus::Queued,
            season_id: "season-placeholder".to_string(),
            triggered_by: "match_correction".to_string(),
            retry_count: 0,
            max_retries: 3,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }),
    ))
}
