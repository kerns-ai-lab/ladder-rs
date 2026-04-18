//! Handlers for rating history endpoints

use crate::Result;
use axum::{extract::Path, http::StatusCode, Json};
use ladder_rs_persistence::{RatingHistoryResponse, SeasonOverviewResponse};

/// GET /api/players/{player_id}/seasons/{season_id}/history
pub async fn get_rating_history(
    Path((player_id, season_id)): Path<(i64, i64)>,
) -> Result<(StatusCode, Json<RatingHistoryResponse>)> {
    Ok((
        StatusCode::OK,
        Json(RatingHistoryResponse { entries: vec![] }),
    ))
}

/// GET /api/seasons/{season_id}/players/{player_id}/history
pub async fn get_rating_history_season_centric(
    Path((season_id, player_id)): Path<(i64, i64)>,
) -> Result<(StatusCode, Json<RatingHistoryResponse>)> {
    get_rating_history(Path((player_id, season_id))).await
}

/// GET /api/players/{player_id}/seasons
pub async fn get_season_overview(
    Path(player_id): Path<i64>,
) -> Result<(StatusCode, Json<SeasonOverviewResponse>)> {
    Ok((
        StatusCode::OK,
        Json(SeasonOverviewResponse { seasons: vec![] }),
    ))
}
