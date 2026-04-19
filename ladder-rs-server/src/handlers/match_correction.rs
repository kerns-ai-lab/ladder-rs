//! Handler for PATCH /api/matches/{id} - admin match correction endpoint

use crate::Result;
use axum::{extract::Path, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

/// Request body for match correction
#[derive(Debug, Deserialize)]
pub struct CorrectMatchRequest {
    /// New participants with placements
    pub participants: Vec<CorrectionParticipant>,
    /// Reason for the correction
    pub reason: Option<String>,
}

/// Participant in a correction request
#[derive(Debug, Deserialize, Serialize)]
pub struct CorrectionParticipant {
    pub player_id: String,
    pub placement: i32,
}

/// Response body for match correction
#[derive(Debug, Serialize)]
pub struct CorrectMatchResponse {
    pub job_id: String,
    pub status: String,
    pub message: String,
}

/// PATCH /api/matches/{id} - Correct a match (Admin only)
///
/// This endpoint is restricted to authenticated users with Admin role.
/// Returns 202 Accepted with a job ID for async recalculation.
pub async fn correct_match(
    Path(_match_id): Path<String>,
    Json(_payload): Json<CorrectMatchRequest>,
) -> Result<(StatusCode, Json<CorrectMatchResponse>)> {
    // Implementation will be added in task 900.1.2
    // For now, return a placeholder response
    Ok((
        StatusCode::ACCEPTED,
        Json(CorrectMatchResponse {
            job_id: "job-placeholder".to_string(),
            status: "queued".to_string(),
            message: "Match correction queued. Recalculation in progress.".to_string(),
        }),
    ))
}
