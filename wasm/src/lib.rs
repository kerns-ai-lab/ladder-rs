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

// Module declarations
pub mod types;
pub mod elo_wasm;
// pub mod glicko_wasm;
pub mod trueskill_wasm;

// Re-export Elo system
pub use elo_wasm::{EloSystem, EloRating, EloUtils, MatchOutcome, MatchResult};

// Re-export Glicko system
// pub use glicko_wasm::{GlickoSystem, GlickoRating, GlickoMatchResult, GlickoUtils};

// Re-export TrueSkill system
pub use trueskill_wasm::{TrueSkillSystem, TrueSkillRating, TrueSkillTeam, TrueSkillUtils};