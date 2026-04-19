//! Handler for GET /api/jobs/{id} - job status endpoint

use crate::{error::ServerError, Result};
use axum::{extract::Path, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use ladder_rs_persistence::JobStatus;
use serde::Serialize;
use uuid::Uuid;

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
/// Validates that `job_id` is a well-formed UUID and returns 400 if not.
///
/// TODO(900.1.3): Replace with real repository lookup; return 404 for unknown
/// IDs and 401 when no authenticated session is present.
pub async fn get_job_status(
    Path(job_id): Path<String>,
) -> Result<(StatusCode, Json<JobStatusResponse>)> {
    // Validate UUID format before touching the database layer.
    Uuid::parse_str(&job_id)
        .map_err(|_| ServerError::InvalidInput(format!("Invalid job_id format: '{job_id}'")))?;

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
