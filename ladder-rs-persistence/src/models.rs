//! Domain models for persistence layer

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a player in the system.
///
/// Players are global entities (not scoped to a single league).
/// League membership is managed via the `league_players` join table.
/// `league_id` is an optional context field populated when querying
/// within a league context (e.g., listing players in a league).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    /// Optional league context — set when querying within a league scope,
    /// None for global player lookups.
    pub league_id: Option<String>,
    pub name: String,
    pub nickname: Option<String>,
    pub is_active: bool,
    pub player_type: String,
    pub created_at: DateTime<Utc>,
}

/// Represents a match in a season
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub season_id: String,
    pub match_number: i32,
    pub is_corrected: bool,
    pub convergence_quality: String,
    pub created_at: DateTime<Utc>,
}

/// Participant in a match with placement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchParticipant {
    pub player_id: String,
    pub placement: i32,
}

/// Represents a rating snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingSnapshot {
    pub id: String,
    pub match_id: String,
    pub player_id: String,
    pub season_id: String,
    pub rating_value: f64,
    pub uncertainty: Option<f64>,
    pub volatility: Option<f64>,
    pub conservative_rating: f64,
    pub rating_period: i32,
    pub created_at: DateTime<Utc>,
}

/// Result of a match correction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectMatchResult {
    pub job_id: String,
}

/// Represents an audit log entry for match corrections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub match_id: String,
    pub actor_user_id: String,
    pub before_state: serde_json::Value,
    pub after_state: serde_json::Value,
    pub changed_at: DateTime<Utc>,
}

/// Represents a recalculation job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecalculationJob {
    pub id: String,
    pub season_id: String,
    pub status: JobStatus,
    pub triggered_by: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Status of a recalculation job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    InProgress,
    Completed,
    Failed,
    PermanentlyFailed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "queued"),
            JobStatus::InProgress => write!(f, "in_progress"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::PermanentlyFailed => write!(f, "permanently_failed"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "queued" => Ok(JobStatus::Queued),
            "in_progress" => Ok(JobStatus::InProgress),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            "permanently_failed" => Ok(JobStatus::PermanentlyFailed),
            _ => Err(format!("Invalid job status: {}", s)),
        }
    }
}

/// Represents a season
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub id: String,
    pub league_id: String,
    pub number: i32,
    pub algorithm: String,
    pub is_open: bool,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
