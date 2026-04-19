//! Handlers for rating history endpoints
//!
//! These handlers will call RatingHistoryRepository once the persistence layer
//! is wired via AppState. Currently the repository methods are todo!() stubs
//! pending database schema implementation.

use crate::{Result, ServerError};
use axum::{extract::Path, http::StatusCode, Json};
use ladder_rs_persistence::{RatingHistoryResponse, SeasonOverviewResponse};

/// GET /api/players/{player_id}/seasons/{season_id}/history
///
/// Returns chronologically ordered rating history for a player in a specific season.
/// Returns 404 if the player does not exist.
/// Returns 200 with empty entries if the player exists but has no matches in the season.
pub async fn get_rating_history(
    Path((_player_id, _season_id)): Path<(i64, i64)>,
) -> Result<(StatusCode, Json<RatingHistoryResponse>)> {
    // TODO(900.3.1): Wire AppState with RatingHistoryRepository pool once DB layer is ready.
    // Implementation flow:
    //   1. Call repo.player_exists(player_id) → 404 if false
    //   2. Call repo.get_per_season_history(player_id, season_id) → return entries
    Err(ServerError::InternalError(
        "Rating history repository not yet wired — pending task 900.3.1".to_string(),
    ))
}

/// GET /api/seasons/{season_id}/players/{player_id}/history
///
/// Season-centric alias for the same data as the player-centric route.
/// Note: path extracts (season_id, player_id) in that order.
pub async fn get_rating_history_season_centric(
    Path((season_id, player_id)): Path<(i64, i64)>,
) -> Result<(StatusCode, Json<RatingHistoryResponse>)> {
    // Delegate to the canonical player-centric handler, swapping arg order.
    get_rating_history(Path((player_id, season_id))).await
}

/// GET /api/players/{player_id}/seasons
///
/// Returns a season-level overview (final rating per season) for a player.
/// Returns 404 if the player does not exist.
pub async fn get_season_overview(
    Path(_player_id): Path<i64>,
) -> Result<(StatusCode, Json<SeasonOverviewResponse>)> {
    // TODO(900.3.1): Wire AppState with RatingHistoryRepository pool once DB layer is ready.
    Err(ServerError::InternalError(
        "Rating history repository not yet wired — pending task 900.3.1".to_string(),
    ))
}
