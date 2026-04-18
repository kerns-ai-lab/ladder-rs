//! Player alias endpoint handlers

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

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
    _user: UserContext,
    Path((_league_id, _player_id)): Path<(i64, i64)>,
    Json(_req): Json<CreateAliasRequest>,
) -> Result<Response, ServerError> {
    todo!("player alias creation not yet implemented")
}

/// DELETE /api/leagues/{league_id}/players/{player_id}/aliases/{alias_player_id}
pub async fn remove_alias(
    State(_state): State<()>,
    _user: UserContext,
    Path((_league_id, _player_id, _alias_player_id)): Path<(i64, i64, i64)>,
) -> Result<Response, ServerError> {
    todo!("player alias removal not yet implemented")
}
