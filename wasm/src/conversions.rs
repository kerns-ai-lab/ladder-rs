//! Task 1.2.2: Conversion Implementations
//!
//! This module provides comprehensive type conversions between Rust types
//! and JavaScript/WASM boundary types for all rating systems.

pub mod core;
pub mod elo;
pub mod glicko;
pub mod trueskill;

// Re-export common conversion utilities
pub use core::*;