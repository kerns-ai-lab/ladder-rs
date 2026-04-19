//! HTTP request handlers

pub mod job_status;
pub mod match_correction;
pub mod player_alias;
pub mod rating_history;
pub mod swarm;

pub use job_status::get_job_status;
pub use match_correction::correct_match;
pub use player_alias::{create_alias, remove_alias};
pub use rating_history::*;
pub use swarm::*;
