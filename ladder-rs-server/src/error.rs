//! Error handling for the server

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use ladder_rs_persistence::PersistenceError;
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

impl From<PersistenceError> for ServerError {
    fn from(err: PersistenceError) -> Self {
        match err {
            PersistenceError::NotFound { entity, id } => {
                ServerError::NotFound(format!("{} with id {} not found", entity, id))
            }
            PersistenceError::Conflict(msg) => ServerError::Conflict(msg),
            PersistenceError::InvalidInput(msg) => ServerError::InvalidInput(msg),
            PersistenceError::DatabaseError(msg) => ServerError::DatabaseError(msg),
            PersistenceError::TransactionError(msg) => ServerError::InternalError(msg),
            PersistenceError::Unknown(msg) => ServerError::InternalError(msg),
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            ServerError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Unauthorized".to_string(),
            ),
            ServerError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", "Forbidden".to_string()),
            ServerError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ServerError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            ServerError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            ServerError::DatabaseError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg)
            }
            ServerError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg)
            }
        };

        let body = Json(json!({
            "error": message,
            "error_code": error_code,
        }));

        (status, body).into_response()
    }
}
