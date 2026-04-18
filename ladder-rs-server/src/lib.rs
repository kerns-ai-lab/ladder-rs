//! Axum HTTP server for ladder-rs

pub mod error;
pub mod handlers;
pub mod middleware;

pub use error::{Result, ServerError};

use axum::{
    routing::{delete, post},
    Router,
};

/// Build the Axum router with all registered routes.
pub fn build_router() -> Router {
    Router::new()
        .route(
            "/api/leagues/:league_id/players/:player_id/aliases",
            post(handlers::create_alias),
        )
        .route(
            "/api/leagues/:league_id/players/:player_id/aliases/:alias_player_id",
            delete(handlers::remove_alias),
        )
}
