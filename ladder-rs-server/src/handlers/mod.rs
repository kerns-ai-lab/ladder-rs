//! HTTP request handlers

pub mod job_status;
pub mod match_correction;
pub mod player_alias;

pub use job_status::get_job_status;
pub use match_correction::correct_match;
pub use player_alias::{create_alias, remove_alias};
