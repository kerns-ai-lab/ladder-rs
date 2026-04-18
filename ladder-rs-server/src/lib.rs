//! Axum HTTP server for ladder-rs

pub mod error;
pub mod handlers;
pub mod middleware;

pub use error::{Result, ServerError};
