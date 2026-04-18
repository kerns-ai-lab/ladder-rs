//! Player alias endpoint handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::middleware::UserContext;
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
    State(_state): State<()>,
    user: UserContext,
    Path((_league_id, _player_id)): Path<(i64, i64)>,
    Json(_req): Json<CreateAliasRequest>,
) -> Result<impl IntoResponse, ServerError> {
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
    State(_state): State<()>,
    user: UserContext,
    Path((_league_id, _player_id, _alias_player_id)): Path<(i64, i64, i64)>,
) -> Result<impl IntoResponse, ServerError> {
    let job_id = Uuid::new_v4().to_string();
    Ok((
        StatusCode::ACCEPTED,
        Json(AliasResponse {
            job_id,
            status: "queued".to_string(),
        }),
    ))
}
