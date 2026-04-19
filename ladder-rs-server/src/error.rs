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

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            ServerError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "Unauthorized".to_string())
            }
            ServerError::Forbidden => {
                (StatusCode::FORBIDDEN, "FORBIDDEN", "Forbidden".to_string())
            }
            err @ ServerError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND", err.to_string()),
            err @ ServerError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", err.to_string()),
            err @ ServerError::InvalidInput(_) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", err.to_string())
            }
            ServerError::DatabaseError(msg) => {
                eprintln!("Database error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An internal server error occurred".to_string(),
                )
            }
            ServerError::InternalError(msg) => {
                eprintln!("Internal server error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An internal server error occurred".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": message,
            "error_code": error_code,
        }));

        (status, body).into_response()
    }
}

impl From<PersistenceError> for ServerError {
    fn from(e: PersistenceError) -> Self {
        match e {
            PersistenceError::NotFound { entity, id } => {
                ServerError::NotFound(format!("{entity} with id {id} not found"))
            }
            PersistenceError::Conflict(msg) => ServerError::Conflict(msg),
            PersistenceError::DatabaseError(msg) => ServerError::InternalError(msg),
            PersistenceError::InvalidInput(msg) => ServerError::InvalidInput(msg),
            PersistenceError::TransactionError(msg) => ServerError::InternalError(msg),
            PersistenceError::Unknown(msg) => ServerError::InternalError(msg),
            PersistenceError::QueryFailed(msg) => ServerError::InternalError(msg),
        }
    }
}
