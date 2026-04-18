//! Axum HTTP server for ladder-rs

pub mod error;
pub mod handlers;
pub mod middleware;

pub use error::{Result, ServerError};

use axum::{routing::get, Router};
use handlers::swarm::{
    agents, dashboard_summary, match_volume, rating_distribution, top_bottom_agents, AppState,
};

/// Build the Axum router with all swarm dashboard routes registered.
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
        .with_state(state)
}
