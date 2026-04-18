//! Axum HTTP server for ladder-rs

pub mod error;
pub mod handlers;
pub mod middleware;

pub use error::{Result, ServerError};

use axum::{
    routing::{delete, get, post},
    Router,
};
use handlers::swarm::{
    agents, dashboard_summary, match_volume, rating_distribution, top_bottom_agents, AppState,
};

/// Build the Axum router with all routes registered.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/leagues/{league_id}/dashboard", get(dashboard_summary))
        .route(
            "/api/leagues/{league_id}/dashboard/rating-distribution",
            get(rating_distribution),
        )
        .route(
            "/api/leagues/{league_id}/dashboard/match-volume",
            get(match_volume),
        )
        .route(
            "/api/leagues/{league_id}/dashboard/top-bottom",
            get(top_bottom_agents),
        )
        .route("/api/leagues/{league_id}/dashboard/agents", get(agents))
        .route(
            "/api/leagues/{league_id}/players/{player_id}/aliases",
            post(handlers::create_alias),
        )
        .route(
            "/api/leagues/{league_id}/players/{player_id}/aliases/{alias_player_id}",
            delete(handlers::remove_alias),
        )
        .with_state(state)
}
