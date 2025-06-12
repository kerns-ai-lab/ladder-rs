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
#[cfg(any(feature = "trueskill-only", feature = "all-algorithms"))]
pub mod trueskill_wasm;
pub mod browser_compat;
pub mod performance_tracking;

// Re-export Elo system
pub use elo_wasm::{EloSystem, EloRating, EloUtils, MatchOutcome, MatchResult};

// Re-export Glicko system
// pub use glicko_wasm::{GlickoSystem, GlickoRating, GlickoMatchResult, GlickoUtils};

// Re-export TrueSkill system (if available)
#[cfg(any(feature = "trueskill-only", feature = "all-algorithms"))]
pub use trueskill_wasm::{TrueSkillSystem, TrueSkillRating, TrueSkillTeam, TrueSkillUtils};

// Re-export browser compatibility utilities
pub use browser_compat::{CrossBrowserCompat, EventCompat, PerformanceCompat};

// Initialize browser compatibility on module load
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();
    
    CrossBrowserCompat::init();
}