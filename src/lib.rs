pub mod core;
pub mod error;
pub mod trueskill;
pub mod elo;
pub mod glicko;
pub mod mcp;

pub use crate::core::{Rating, RatingSystem, TeamRating, Outcome};
pub use crate::error::Error;