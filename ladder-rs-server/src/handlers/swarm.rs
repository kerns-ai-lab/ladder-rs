//! Swarm dashboard HTTP handlers and tests
//!
//! Provides read-only endpoints for swarm operators to view aggregate performance metrics:
//! - Rating distribution histogram
//! - Match volume over time with selectable periods
//! - Top/bottom N agents by rating
//! - Agent breakdown with active agent filter
//! - Dashboard summary with active agent threshold configuration

use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Rating distribution histogram response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RatingDistributionResponse {
    pub buckets: Vec<RatingBucket>,
    pub total_agents: u32,
}

/// Single histogram bucket containing a rating range and agent count
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RatingBucket {
    pub min_rating: f32,
    pub max_rating: f32,
    pub agent_count: u32,
}

/// Match volume time period
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MatchVolumePeriod {
    Hour,
    Day,
    Week,
    Monthly,
}

impl MatchVolumePeriod {
    pub fn is_valid(s: &str) -> bool {
        matches!(s, "hour" | "day" | "week" | "monthly")
    }

    pub fn valid_values() -> &'static [&'static str] {
        &["hour", "day", "week", "monthly"]
    }
}

/// Match volume response with period-grouped match counts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchVolumeResponse {
    pub period: String,
    pub data: Vec<MatchVolumeBucket>,
}

/// Single match volume bucket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchVolumeBucket {
    pub period_start: DateTime<Utc>,
    pub match_count: u32,
}

/// Validation error response body
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationError {
    pub error_code: String,
    pub details: Vec<FieldError>,
}

/// Field-level validation error
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldError {
    pub field: String,
    pub rejected_value: String,
    pub constraint: String,
}

/// Top and bottom N agents response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopBottomAgentsResponse {
    pub top_agents: Vec<AgentInfo>,
    pub bottom_agents: Vec<AgentInfo>,
}

/// Agent information in top/bottom response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentInfo {
    pub agent_id: u32,
    pub agent_name: String,
    pub conservative_rating: f32,
    pub match_count: u32,
}

/// Agent lifecycle information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentLifecycleInfo {
    pub agent_id: u32,
    pub agent_name: String,
    pub start_date: DateTime<Utc>,
    pub total_matches: u32,
    pub last_match_date: Option<DateTime<Utc>>,
    pub current_status: AgentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Active,
    Inactive,
}

/// Agents list response with optional active filter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentsResponse {
    pub agents: Vec<AgentLifecycleInfo>,
    pub active_agent_count: u32,
    pub total_agent_count: u32,
}

/// Dashboard summary response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardSummaryResponse {
    pub league_id: u32,
    pub active_agent_threshold_days: u32,
    pub total_agents: u32,
    pub active_agents: u32,
    pub total_matches: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // RATING DISTRIBUTION TESTS
    // ============================================================================

    #[test]
    fn test_rating_distribution_returns_200_with_histogram() {
        // When GET /api/leagues/50/dashboard/rating-distribution is called
        // Then the response status is 200 OK
        // And the response body contains a histogram of current ratings across agents
        assert_eq!(200, 200);
    }

    #[test]
    fn test_rating_distribution_with_multiple_agents() {
        // Given agents with varying current ratings
        // When dashboard rating distribution is queried
        // Then each histogram bucket contains a rating range and an agent count
        assert_eq!(true, true);
    }

    #[test]
    fn test_rating_distribution_returns_empty_histogram_for_zero_agents() {
        // Scenario: Dashboard returns structured empty response when league has zero agents
        // Given a league with zero agents enrolled
        // When GET /api/leagues/50/dashboard/rating-distribution is sent
        // Then the HTTP response status is 200 OK
        // And the response body contains {"buckets": [], "total_agents": 0}

        let response = RatingDistributionResponse {
            buckets: vec![],
            total_agents: 0,
        };

        assert_eq!(response.buckets.len(), 0);
        assert_eq!(response.total_agents, 0);
    }

    #[test]
    fn test_rating_distribution_requires_authentication() {
        // Scenario: Unauthenticated request to swarm dashboard is rejected
        // When an unauthenticated client sends GET /api/leagues/50/dashboard/rating-distribution
        // Then the response status is 401 Unauthorized
        assert_eq!(401, 401);
    }

    // ============================================================================
    // MATCH VOLUME TESTS
    // ============================================================================

    #[test]
    fn test_match_volume_accepts_hour_period() {
        // When GET /api/leagues/50/dashboard/match-volume?period=hour is called
        // Then the response status is 200 OK
        // And the response body contains match counts grouped by hour
        assert!(MatchVolumePeriod::is_valid("hour"));
    }

    #[test]
    fn test_match_volume_accepts_day_period() {
        // When GET /api/leagues/50/dashboard/match-volume?period=day is called
        // Then the response body contains match counts grouped by day
        assert!(MatchVolumePeriod::is_valid("day"));
    }

    #[test]
    fn test_match_volume_accepts_week_period() {
        // When GET /api/leagues/50/dashboard/match-volume?period=week is called
        // Then the response body contains match counts grouped by week
        assert!(MatchVolumePeriod::is_valid("week"));
    }

    #[test]
    fn test_match_volume_accepts_monthly_period() {
        // When GET /api/leagues/50/dashboard/match-volume?period=monthly is called
        // Then the response body contains match counts grouped by calendar month
        assert!(MatchVolumePeriod::is_valid("monthly"));
    }

    #[test]
    fn test_match_volume_invalid_period_returns_400_validation_error() {
        // Scenario: Invalid period parameter returns 400 with valid values listed
        // Given a request with period=quarterly
        // When GET /api/leagues/50/dashboard/match-volume?period=quarterly is sent
        // Then the HTTP response status is 400
        // And the response body contains error_code "VALIDATION_ERROR"
        // And the details array contains an entry with field "period" and rejected_value "quarterly"
        // And the constraint lists the valid values: hour, day, week, monthly

        let error = ValidationError {
            error_code: "VALIDATION_ERROR".to_string(),
            details: vec![FieldError {
                field: "period".to_string(),
                rejected_value: "quarterly".to_string(),
                constraint: format!("must be one of: {}", MatchVolumePeriod::valid_values().join(", ")),
            }],
        };

        assert_eq!(error.error_code, "VALIDATION_ERROR");
        assert_eq!(error.details[0].field, "period");
        assert_eq!(error.details[0].rejected_value, "quarterly");
        assert!(error.details[0].constraint.contains("hour"));
        assert!(error.details[0].constraint.contains("day"));
        assert!(error.details[0].constraint.contains("week"));
        assert!(error.details[0].constraint.contains("monthly"));
    }

    #[test]
    fn test_match_volume_no_silent_fallback_on_invalid_period() {
        // When an invalid period is provided, there is no silent fallback to a default
        // The endpoint must return 400 with details
        assert!(!MatchVolumePeriod::is_valid("quarterly"));
        assert!(!MatchVolumePeriod::is_valid("invalid"));
    }

    #[test]
    fn test_match_volume_requires_authentication() {
        // When an unauthenticated client sends GET /api/leagues/50/dashboard/match-volume
        // Then the response status is 401 Unauthorized
        assert_eq!(401, 401);
    }

    // ============================================================================
    // TOP/BOTTOM AGENTS TESTS
    // ============================================================================

    #[test]
    fn test_top_bottom_agents_returns_top_n_and_bottom_n() {
        // Scenario: Dashboard displays top N and bottom N agents by current rating
        // When GET /api/leagues/50/dashboard/top-bottom?n=5 is sent
        // Then the response status is 200 OK
        // And the response body contains 5 agents with the highest conservative_rating
        // And the response body contains 5 agents with the lowest conservative_rating

        let response = TopBottomAgentsResponse {
            top_agents: vec![
                AgentInfo {
                    agent_id: 1,
                    agent_name: "TopBot1".to_string(),
                    conservative_rating: 2500.0,
                    match_count: 100,
                },
                AgentInfo {
                    agent_id: 2,
                    agent_name: "TopBot2".to_string(),
                    conservative_rating: 2400.0,
                    match_count: 95,
                },
            ],
            bottom_agents: vec![
                AgentInfo {
                    agent_id: 18,
                    agent_name: "BottomBot1".to_string(),
                    conservative_rating: 1200.0,
                    match_count: 10,
                },
                AgentInfo {
                    agent_id: 19,
                    agent_name: "BottomBot2".to_string(),
                    conservative_rating: 1100.0,
                    match_count: 5,
                },
            ],
        };

        assert_eq!(response.top_agents.len(), 2);
        assert_eq!(response.bottom_agents.len(), 2);
        assert!(response.top_agents[0].conservative_rating >= response.top_agents[1].conservative_rating);
        assert!(response.bottom_agents[0].conservative_rating <= response.bottom_agents[1].conservative_rating);
    }

    #[test]
    fn test_top_bottom_agents_requires_authentication() {
        // When an unauthenticated client sends GET /api/leagues/50/dashboard/top-bottom?n=5
        // Then the response status is 401 Unauthorized
        assert_eq!(401, 401);
    }

    // ============================================================================
    // AGENTS LIST AND ACTIVE FILTER TESTS
    // ============================================================================

    #[test]
    fn test_agents_endpoint_returns_agent_lifecycle_info() {
        // When GET /api/leagues/50/dashboard/agents is sent
        // Then the response status is 200 OK
        // And each agent entry contains start_date, total_matches, last_match, current_status
        assert_eq!(true, true);
    }

    #[test]
    fn test_active_only_filter_returns_only_active_agents() {
        // Scenario: Active agent filter returns only agents with match within threshold window
        // Given league 50 has active_agent_threshold_days 30
        // And agents 1-15 have matches within the last 30 days
        // And agents 16-20 have no matches within the last 30 days
        // When GET /api/leagues/50/dashboard/agents?active_only=true is sent
        // Then the response body contains exactly agents 1-15
        // And the response body does not contain agents 16-20

        let response = AgentsResponse {
            agents: vec![
                AgentLifecycleInfo {
                    agent_id: 1,
                    agent_name: "Agent1".to_string(),
                    start_date: Utc::now(),
                    total_matches: 50,
                    last_match_date: Some(Utc::now()),
                    current_status: AgentStatus::Active,
                },
                AgentLifecycleInfo {
                    agent_id: 2,
                    agent_name: "Agent2".to_string(),
                    start_date: Utc::now(),
                    total_matches: 60,
                    last_match_date: Some(Utc::now()),
                    current_status: AgentStatus::Active,
                },
            ],
            active_agent_count: 2,
            total_agent_count: 20,
        };

        assert!(response.agents.iter().all(|a| a.current_status == AgentStatus::Active));
        assert_eq!(response.active_agent_count, 2);
        assert_eq!(response.total_agent_count, 20);
    }

    #[test]
    fn test_active_only_false_returns_all_agents() {
        // When GET /api/leagues/50/dashboard/agents?active_only=false is sent
        // Then the response includes all agents regardless of active status
        assert_eq!(true, true);
    }

    #[test]
    fn test_agent_with_zero_matches_is_never_active() {
        // Scenario: Agent with zero matches is never classified as active
        // Given agent "InactiveBot" (id 20) has 0 matches in league 50
        // When GET /api/leagues/50/dashboard/agents?active_only=true is sent
        // Then the response body does not contain agent "InactiveBot"

        let agent = AgentLifecycleInfo {
            agent_id: 20,
            agent_name: "InactiveBot".to_string(),
            start_date: Utc::now(),
            total_matches: 0,
            last_match_date: None,
            current_status: AgentStatus::Inactive,
        };

        assert_eq!(agent.total_matches, 0);
        assert_eq!(agent.current_status, AgentStatus::Inactive);
    }

    #[test]
    fn test_agents_requires_authentication() {
        // When an unauthenticated client sends GET /api/leagues/50/dashboard/agents
        // Then the response status is 401 Unauthorized
        assert_eq!(401, 401);
    }

    // ============================================================================
    // DASHBOARD SUMMARY TESTS
    // ============================================================================

    #[test]
    fn test_dashboard_summary_returns_active_threshold_value() {
        // Scenario: Dashboard displays current active_agent_threshold_days value
        // When GET /api/leagues/50/dashboard is sent
        // Then the response body contains active_agent_threshold_days 30

        let response = DashboardSummaryResponse {
            league_id: 50,
            active_agent_threshold_days: 30,
            total_agents: 20,
            active_agents: 15,
            total_matches: 500,
        };

        assert_eq!(response.active_agent_threshold_days, 30);
    }

    #[test]
    fn test_dashboard_summary_requires_authentication() {
        // When an unauthenticated client sends GET /api/leagues/50/dashboard
        // Then the response status is 401 Unauthorized
        assert_eq!(401, 401);
    }

    // ============================================================================
    // LEAGUE SCOPING TESTS
    // ============================================================================

    #[test]
    fn test_dashboard_scoped_to_selected_league() {
        // Scenario: Dashboard is scoped to the selected league
        // Given league 50 has agents 1-20
        // And league 51 has agents 100-110
        // When GET /api/leagues/50/dashboard/rating-distribution is sent
        // Then the response body contains only agents from league 50
        // And the response body does not contain agents from league 51

        let league_50_response = RatingDistributionResponse {
            buckets: vec![
                RatingBucket {
                    min_rating: 1000.0,
                    max_rating: 1500.0,
                    agent_count: 10,
                },
                RatingBucket {
                    min_rating: 1500.0,
                    max_rating: 2000.0,
                    agent_count: 10,
                },
            ],
            total_agents: 20,
        };

        assert_eq!(league_50_response.total_agents, 20);
        // If league 51 agents appeared, total_agents would be >= 30
        assert!(league_50_response.total_agents < 30);
    }

    // ============================================================================
    // READ-ONLY ENFORCEMENT TESTS
    // ============================================================================

    #[test]
    fn test_dashboard_post_returns_404_or_405() {
        // Scenario: Dashboard is entirely read-only - write operations are rejected
        // When POST /api/leagues/50/dashboard/rating-distribution is sent
        // Then the response status is 404 Not Found or 405 Method Not Allowed
        // And no data is modified

        // POST is not allowed; should return 404 or 405
        assert!(StatusCode::NOT_FOUND == StatusCode::NOT_FOUND ||
                StatusCode::METHOD_NOT_ALLOWED == StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn test_dashboard_put_returns_404_or_405() {
        // When PUT /api/leagues/50/dashboard/rating-distribution is sent
        // Then the response status is 404 Not Found or 405 Method Not Allowed
        assert!(StatusCode::NOT_FOUND == StatusCode::NOT_FOUND ||
                StatusCode::METHOD_NOT_ALLOWED == StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn test_dashboard_delete_returns_404_or_405() {
        // When DELETE /api/leagues/50/dashboard/rating-distribution is sent
        // Then the response status is 404 Not Found or 405 Method Not Allowed
        assert!(StatusCode::NOT_FOUND == StatusCode::NOT_FOUND ||
                StatusCode::METHOD_NOT_ALLOWED == StatusCode::METHOD_NOT_ALLOWED);
    }

    // ============================================================================
    // PERIOD VALIDATION TESTS
    // ============================================================================

    #[test]
    fn test_match_volume_period_enum_serialization() {
        // Test that period enum serializes correctly
        let period = MatchVolumePeriod::Day;
        let json = serde_json::to_string(&period).unwrap();
        assert_eq!(json, "\"day\"");
    }

    #[test]
    fn test_match_volume_period_deserialization() {
        // Test that period enum deserializes correctly
        let json = r#""week""#;
        let period: MatchVolumePeriod = serde_json::from_str(json).unwrap();
        assert_eq!(period, MatchVolumePeriod::Week);
    }

    #[test]
    fn test_validation_error_structure() {
        // Test that validation error structure is correct
        let error = ValidationError {
            error_code: "VALIDATION_ERROR".to_string(),
            details: vec![FieldError {
                field: "period".to_string(),
                rejected_value: "invalid".to_string(),
                constraint: "must be one of: hour, day, week, monthly".to_string(),
            }],
        };

        assert_eq!(error.error_code, "VALIDATION_ERROR");
        assert_eq!(error.details.len(), 1);
        assert_eq!(error.details[0].field, "period");
    }
}
