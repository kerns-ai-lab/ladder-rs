//! Error handling for the server

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, ServerError>;

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ServerError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ServerError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            ServerError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServerError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ServerError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            ServerError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ServerError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}
