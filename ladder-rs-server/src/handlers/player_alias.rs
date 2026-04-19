//! Player alias endpoint handlers

use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::middleware::auth::UserContext;
use crate::ServerError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAliasRequest {
    pub alias_player_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasResponse {
    pub job_id: String,
    pub status: String,
}

/// POST /api/leagues/{league_id}/players/{player_id}/aliases
pub async fn create_alias(
    _user: UserContext,
    Path((_league_id, _player_id)): Path<(i64, i64)>,
    Json(_req): Json<CreateAliasRequest>,
) -> Result<(StatusCode, Json<AliasResponse>), ServerError> {
    // TODO(900.2): implement real alias creation and job dispatch
    let job_id = Uuid::new_v4().to_string();
    Ok((
        StatusCode::ACCEPTED,
        Json(AliasResponse {
            job_id,
            status: "queued".to_string(),
        }),
    ))
}

/// DELETE /api/leagues/{league_id}/players/{player_id}/aliases/{alias_player_id}
pub async fn remove_alias(
    _user: UserContext,
    Path((_league_id, _player_id, _alias_player_id)): Path<(i64, i64, i64)>,
) -> Result<StatusCode, ServerError> {
    // TODO(900.3): implement real alias removal
    Ok(StatusCode::NO_CONTENT)
}
