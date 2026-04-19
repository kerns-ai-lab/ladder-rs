//! Axum HTTP server for ladder-rs

pub mod error;
pub mod handlers;
pub mod middleware;

pub use error::{Result, ServerError};

use axum::{routing::get, Router};

/// Build the application router with all registered routes.
pub fn create_router() -> Router {
    Router::new()
        .route(
            "/api/players/{player_id}/seasons/{season_id}/history",
            get(handlers::get_rating_history),
        )
        .route(
            "/api/seasons/{season_id}/players/{player_id}/history",
            get(handlers::get_rating_history_season_centric),
        )
        .route(
            "/api/players/{player_id}/seasons",
            get(handlers::get_season_overview),
        )
}
