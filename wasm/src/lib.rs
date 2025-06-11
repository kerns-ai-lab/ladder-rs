//! Optimized WebAssembly bindings for the ladder-rs rating systems
//!
//! This crate provides JavaScript interfaces for rating calculations,
//! with implementations for Elo, Glicko, and TrueSkill systems.

// Required for WASM bindings
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;

// Use `wee_alloc` as the global allocator for smaller WASM binary size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Set up panic hook for better error messages in the browser
#[cfg(feature = "console_error_panic_hook")]
fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

// Module declarations - minimal set for Elo
pub mod types;
pub mod elo_wasm;

// Re-export only Elo system
pub use elo_wasm::{EloSystem, EloRating, EloUtils, MatchOutcome, MatchResult};