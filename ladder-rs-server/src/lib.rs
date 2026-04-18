//! Axum HTTP server for ladder-rs

pub mod handlers;
pub mod middleware;
pub mod error;

pub use error::{ServerError, Result};
