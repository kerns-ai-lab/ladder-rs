pub mod core;
pub mod error;

// Conditionally compile algorithm modules based on features
#[cfg(any(feature = "elo-only", feature = "all-algorithms"))]
pub mod elo;

#[cfg(any(feature = "glicko-only", feature = "all-algorithms"))]
pub mod glicko;

#[cfg(any(feature = "trueskill-only", feature = "all-algorithms"))]
pub mod trueskill;

pub use crate::core::{Outcome, Rating, RatingSystem, TeamRating};
pub use crate::error::Error;

// Conditionally re-export algorithm modules
#[cfg(any(feature = "elo-only", feature = "all-algorithms"))]
pub use crate::elo::{EloRating, EloSystem, EloTeamRating};

#[cfg(any(feature = "glicko-only", feature = "all-algorithms"))]
pub use crate::glicko::{Glicko, GlickoRating, GlickoTeamRating};

#[cfg(any(feature = "trueskill-only", feature = "all-algorithms"))]
pub use crate::trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam};
