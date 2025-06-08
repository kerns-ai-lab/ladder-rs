pub mod core;
pub mod elo;
pub mod error;
pub mod glicko;
pub mod trueskill;

pub use crate::core::{Outcome, Rating, RatingSystem, TeamRating};
pub use crate::error::Error;
