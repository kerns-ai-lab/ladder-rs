//! Optimized WebAssembly bindings for the ladder-rs rating systems
//!
//! This crate provides JavaScript interfaces for rating calculations,
//! with implementations for Elo, Glicko, and TrueSkill systems.

use wasm_bindgen::prelude::*;

// Use `wee_alloc` as the global allocator for smaller WASM binary size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Module declarations - minimal set for Elo
pub mod types;
pub mod elo_wasm;

// Re-export only Elo system
pub use elo_wasm::{EloSystem, EloRating, EloUtils, MatchOutcome, MatchResult};