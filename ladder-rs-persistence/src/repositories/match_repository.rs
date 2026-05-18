//! Match repository for match persistence operations
//!
//! The most complex repository. Records a complete match atomically: match header,
//! participants, rating computation, and rating snapshots. Also provides duplicate
//! detection and season write protection.

use crate::error::{PersistenceError, Result};
use crate::rating_engine_bridge::{MatchInput, RatingEngineBridge, RatingInput};
use crate::repositories::job_repository::JobRepository;
use crate::{Match, MatchParticipant, RatingSnapshot};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Result of recording a match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub match_id: String,
    pub snapshots: Vec<RatingSnapshot>,
}

/// A batch match entry for record_match_batch
#[derive(Debug, Clone)]
pub struct BatchEntry {
    pub participants: Vec<MatchParticipant>,
    pub score_metadata: Option<serde_json::Value>,
    pub recorded_at: DateTime<Utc>,
}

/// Result of a single batch entry
#[derive(Debug, Clone)]
pub struct BatchEntryResult {
    pub match_id: String,
    pub snapshots: Vec<RatingSnapshot>,
}

/// Filter options for listing matches
#[derive(Debug, Clone)]
pub struct MatchFilter {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub player_id: Option<String>,
}

/// Correction payload for correcting a match
#[derive(Debug, Clone)]
pub struct MatchCorrection {
    pub new_participants: Vec<MatchParticipant>,
    pub reason: String,
    pub score_metadata: Option<serde_json::Value>,
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// JSON payload stored in rating_snapshots.rating_json
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RatingJson {
    rating_value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    uncertainty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volatility: Option<f64>,
    rating_period: i32,
}

/// Row type for query_as deserialization from matches table
#[derive(sqlx::FromRow)]
struct MatchRow {
    id: String,
    season_id: String,
    recorded_at: String,
    score_metadata_json: Option<String>,
    is_corrected: bool,
    created_at: String,
    /// Computed via window function, not a real column
    #[sqlx(default)]
    match_number: Option<i64>,
}

/// Row type for query_as deserialization from match_participants
#[derive(sqlx::FromRow)]
struct ParticipantRow {
    player_id: String,
    placement: i32,
    rating_before: Option<String>,
    rating_after: Option<String>,
}

/// Row type for query_as deserialization from rating_snapshots
#[derive(sqlx::FromRow)]
struct SnapshotRow {
    id: String,
    season_id: String,
    player_id: String,
    match_id: String,
    conservative_rating: f64,
    rating_json: String,
    created_at: String,
}

impl SnapshotRow {
    fn into_snapshot(self) -> Result<RatingSnapshot> {
        let rj: RatingJson = serde_json::from_str(&self.rating_json).map_err(|e| {
            PersistenceError::DatabaseError(format!("Failed to parse rating_json: {}", e))
        })?;
        let created_at = DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| PersistenceError::DatabaseError(format!("Invalid created_at: {}", e)))?;
        Ok(RatingSnapshot {
            id: self.id,
            match_id: self.match_id,
            player_id: self.player_id,
            season_id: self.season_id,
            rating_value: rj.rating_value,
            uncertainty: rj.uncertainty,
            volatility: rj.volatility,
            conservative_rating: self.conservative_rating,
            rating_period: rj.rating_period,
            created_at,
        })
    }
}

impl MatchRow {
    fn into_match(self) -> Result<Match> {
        let created_at = DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                PersistenceError::DatabaseError(format!("Invalid created_at timestamp: {}", e))
            })?;

        Ok(Match {
            id: self.id,
            season_id: self.season_id,
            match_number: self.match_number.unwrap_or(0) as i32,
            is_corrected: self.is_corrected,
            convergence_quality: "converged".into(),
            created_at,
        })
    }
}

/// Default rating values per algorithm, used when no prior snapshot exists.
fn default_rating_input(algorithm: &str) -> RatingInput {
    match algorithm {
        "elo" => RatingInput {
            rating: 1500.0,
            uncertainty: None,
            volatility: None,
        },
        "glicko" => RatingInput {
            rating: 1500.0,
            uncertainty: Some(350.0),
            volatility: None,
        },
        "glicko2" => RatingInput {
            rating: 1500.0,
            uncertainty: Some(350.0),
            volatility: Some(0.06),
        },
        "trueskill" => RatingInput {
            rating: 25.0,
            uncertainty: Some(8.333),
            volatility: None,
        },
        _ => RatingInput {
            rating: 1500.0,
            uncertainty: None,
            volatility: None,
        },
    }
}

/// Standalone duplicate-check query for pool-based calls (where &Pool is Copy).
async fn is_duplicate_query(
    pool: &SqlitePool,
    season_id: &str,
    participants: &[MatchParticipant],
    recorded_at: &DateTime<Utc>,
) -> Result<bool> {
    let recorded_at_str = recorded_at.to_rfc3339();
    let n = participants.len() as i64;

    let candidate_ids: Vec<(String,)> = sqlx::query_as(
        "SELECT m.id FROM matches m \
         WHERE m.season_id = ? AND m.recorded_at = ? \
         AND (SELECT COUNT(*) FROM match_participants WHERE match_id = m.id) = ?",
    )
    .bind(season_id)
    .bind(&recorded_at_str)
    .bind(n)
    .fetch_all(pool)
    .await
    .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

    for (candidate_id,) in &candidate_ids {
        let rows = sqlx::query_as::<_, (String, i32)>(
            "SELECT player_id, placement FROM match_participants WHERE match_id = ? ORDER BY placement",
        )
        .bind(candidate_id)
        .fetch_all(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let mut incoming: Vec<(String, i32)> = participants
            .iter()
            .map(|p| (p.player_id.clone(), p.placement))
            .collect();
        incoming.sort();

        let mut existing = rows;
        existing.sort();

        if incoming == existing {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Repository for match operations
pub struct MatchRepository;

impl MatchRepository {
    /// Get a match by ID
    pub async fn get_by_id(pool: &SqlitePool, match_id: &str) -> Result<Option<Match>> {
        if match_id.is_empty() {
            return Ok(None);
        }

        // Compute match_number as the ordinal position within the season
        let row = sqlx::query_as::<_, MatchRow>(
            "SELECT m.id, m.season_id, m.recorded_at, m.score_metadata_json, m.is_corrected, m.created_at, \
             (SELECT COUNT(*) FROM matches m2 WHERE m2.season_id = m.season_id AND m2.recorded_at <= m.recorded_at) AS match_number \
             FROM matches m WHERE m.id = ?",
        )
        .bind(match_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.into_match()?)),
            None => Ok(None),
        }
    }

    /// Atomically record a match with participants, rating computation, and snapshots.
    ///
    /// Transaction steps:
    /// 1. Validate inputs
    /// 2. Check is_season_closed — reject if closed
    /// 3. Check is_duplicate — reject if duplicate
    /// 4. INSERT match header
    /// 5. INSERT match_participants
    /// 6. Compute ratings via Rating Engine Bridge
    /// 7. INSERT rating_snapshots
    pub async fn record_match(
        pool: &SqlitePool,
        season_id: &str,
        participants: Vec<MatchParticipant>,
        score_metadata: Option<serde_json::Value>,
        recorded_at: DateTime<Utc>,
    ) -> Result<MatchResult> {
        // ── Input validation ────────────────────────────────────────────
        if season_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "season_id cannot be empty".into(),
            ));
        }
        if participants.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "participants cannot be empty".into(),
            ));
        }
        // Check for duplicate player_ids in participants
        {
            let mut seen = std::collections::HashSet::new();
            for p in &participants {
                if !seen.insert(&p.player_id) {
                    return Err(PersistenceError::InvalidInput(format!(
                        "Duplicate player_id in participants: {}",
                        p.player_id
                    )));
                }
            }
        }

        // ── Transaction ─────────────────────────────────────────────────
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| PersistenceError::TransactionError(e.to_string()))?;

        // Step 1: Check season not closed
        let closed = Self::is_season_closed_tx(&mut tx, season_id).await?;
        if closed {
            return Err(PersistenceError::Conflict(
                "Season is closed — cannot record new matches".into(),
            ));
        }

        // Step 2: Check not duplicate
        let dup = Self::is_duplicate_tx(&mut tx, season_id, &participants, &recorded_at).await?;
        if dup {
            return Err(PersistenceError::Conflict(
                "Duplicate match detected".into(),
            ));
        }

        // Step 3: Get season algorithm
        let raw_algorithm = fetch_season_algorithm_tx(&mut tx, season_id).await?;
        // Glicko and Glicko-2 only support 1v1 matches. Fall back to Elo for larger matches.
        let algorithm = if (raw_algorithm == "glicko" || raw_algorithm == "glicko2")
            && participants.len() > 2
        {
            "elo"
        } else {
            &raw_algorithm
        };

        // Step 4: Fetch current ratings for each participant
        let mut pre_match_inputs: Vec<RatingInput> = Vec::with_capacity(participants.len());
        let mut pre_match_jsons: Vec<String> = Vec::with_capacity(participants.len());

        for p in &participants {
            let snapshot = fetch_latest_snapshot_tx(&mut tx, &p.player_id, season_id).await?;
            match snapshot {
                Some(s) => {
                    let ri = RatingInput {
                        rating: s.rating_value,
                        uncertainty: s.uncertainty,
                        volatility: s.volatility,
                    };
                    let rj = RatingJson {
                        rating_value: s.rating_value,
                        uncertainty: s.uncertainty,
                        volatility: s.volatility,
                        rating_period: s.rating_period,
                    };
                    pre_match_jsons.push(serde_json::to_string(&rj).map_err(|e| {
                        PersistenceError::InvalidInput(format!("Failed to serialize rating: {}", e))
                    })?);
                    pre_match_inputs.push(ri);
                }
                None => {
                    let ri = default_rating_input(algorithm);
                    let rj = RatingJson {
                        rating_value: ri.rating,
                        uncertainty: ri.uncertainty,
                        volatility: ri.volatility,
                        rating_period: 0,
                    };
                    pre_match_jsons.push(serde_json::to_string(&rj).map_err(|e| {
                        PersistenceError::InvalidInput(format!("Failed to serialize rating: {}", e))
                    })?);
                    pre_match_inputs.push(ri);
                }
            }
        }

        // Step 5: INSERT match header
        let match_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let recorded_at_str = recorded_at.to_rfc3339();
        let score_metadata_str = score_metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| {
                PersistenceError::InvalidInput(format!("Failed to serialize score_metadata: {}", e))
            })?;

        sqlx::query(
            "INSERT INTO matches (id, season_id, recorded_at, score_metadata_json, is_corrected, created_at) \
             VALUES (?, ?, ?, ?, 0, ?)",
        )
        .bind(&match_id)
        .bind(season_id)
        .bind(&recorded_at_str)
        .bind(&score_metadata_str)
        .bind(now.to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Step 6: INSERT match_participants
        let player_ids: Vec<String> = participants.iter().map(|p| p.player_id.clone()).collect();
        let placements: Vec<u32> = participants.iter().map(|p| p.placement as u32).collect();
        // For now, all draws are false (no tie support in basic match entry)
        let draws: Vec<bool> = vec![false; participants.len()];

        for (i, p) in participants.iter().enumerate() {
            let participant_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO match_participants (id, match_id, player_id, placement, rating_before, rating_after, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&participant_id)
            .bind(&match_id)
            .bind(&p.player_id)
            .bind(p.placement)
            .bind(&pre_match_jsons[i])
            .bind("") // rating_after is empty for now — will be filled post-computation
            .bind(now.to_rfc3339())
            .execute(&mut *tx)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        }

        // Step 7: Compute ratings
        let match_input = MatchInput {
            ratings: pre_match_inputs.clone(),
            placements,
            draws,
        };

        let bridge_result =
            RatingEngineBridge::compute(algorithm, &match_input, &player_ids, season_id, &match_id);

        // If the rating engine cannot handle this match (e.g. >2 participants
        // for algorithms that only support 1v1), fall back to passthrough:
        // keep existing ratings unchanged.
        let (snapshots, _convergence_quality) = match bridge_result {
            Ok(br) => {
                let rating_period = Self::compute_next_rating_period_tx(&mut tx, season_id).await?;
                let snaps =
                    RatingEngineBridge::to_snapshots(&br, &player_ids, season_id, rating_period)?;
                (snaps, br.convergence_quality)
            }
            Err(_) => {
                // Passthrough: keep existing ratings
                let rating_period = Self::compute_next_rating_period_tx(&mut tx, season_id).await?;
                let now = Utc::now();
                let snaps: Vec<RatingSnapshot> = pre_match_inputs
                    .iter()
                    .zip(player_ids.iter())
                    .map(|(ri, pid)| RatingSnapshot {
                        id: uuid::Uuid::new_v4().to_string(),
                        match_id: match_id.clone(),
                        player_id: pid.clone(),
                        season_id: season_id.to_string(),
                        rating_value: ri.rating,
                        uncertainty: ri.uncertainty,
                        volatility: ri.volatility,
                        conservative_rating: RatingEngineBridge::conservative_rating(
                            algorithm,
                            ri.rating,
                            ri.uncertainty,
                        ),
                        rating_period,
                        created_at: now,
                    })
                    .collect();
                (snaps, "degraded".to_string())
            }
        };

        // Step 8: INSERT rating snapshots
        for snapshot in &snapshots {
            let rj = RatingJson {
                rating_value: snapshot.rating_value,
                uncertainty: snapshot.uncertainty,
                volatility: snapshot.volatility,
                rating_period: snapshot.rating_period,
            };
            let rating_json_str = serde_json::to_string(&rj).map_err(|e| {
                PersistenceError::InvalidInput(format!("Failed to serialize rating_json: {}", e))
            })?;

            sqlx::query(
                "INSERT INTO rating_snapshots (id, season_id, player_id, match_id, conservative_rating, rating_json, timestamp, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&snapshot.id)
            .bind(&snapshot.season_id)
            .bind(&snapshot.player_id)
            .bind(&snapshot.match_id)
            .bind(snapshot.conservative_rating)
            .bind(&rating_json_str)
            .bind(snapshot.created_at.to_rfc3339())
            .bind(snapshot.created_at.to_rfc3339())
            .execute(&mut *tx)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        }

        // Step 9: Update match_participants with rating_after
        for snapshot in &snapshots {
            let rj = RatingJson {
                rating_value: snapshot.rating_value,
                uncertainty: snapshot.uncertainty,
                volatility: snapshot.volatility,
                rating_period: snapshot.rating_period,
            };
            let rating_after_str = serde_json::to_string(&rj).map_err(|e| {
                PersistenceError::InvalidInput(format!("Failed to serialize rating_after: {}", e))
            })?;

            sqlx::query(
                "UPDATE match_participants SET rating_after = ? WHERE match_id = ? AND player_id = ?",
            )
            .bind(&rating_after_str)
            .bind(&match_id)
            .bind(&snapshot.player_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        }

        // Step 10: Commit
        tx.commit()
            .await
            .map_err(|e| PersistenceError::TransactionError(e.to_string()))?;

        Ok(MatchResult {
            match_id,
            snapshots,
        })
    }

    /// Record multiple matches in batch
    pub async fn record_match_batch(
        pool: &SqlitePool,
        season_id: &str,
        entries: Vec<BatchEntry>,
    ) -> Result<Vec<BatchEntryResult>> {
        let mut results = Vec::with_capacity(entries.len());

        for entry in entries {
            let result = Self::record_match(
                pool,
                season_id,
                entry.participants,
                entry.score_metadata,
                entry.recorded_at,
            )
            .await?;

            results.push(BatchEntryResult {
                match_id: result.match_id,
                snapshots: result.snapshots,
            });
        }

        Ok(results)
    }

    /// List matches in a season with optional filtering
    pub async fn list_matches(
        pool: &SqlitePool,
        season_id: &str,
        filter: &MatchFilter,
    ) -> Result<Vec<Match>> {
        let limit = filter.limit.unwrap_or(100);
        let offset = filter.offset.unwrap_or(0);

        let rows = if let Some(ref player_id) = filter.player_id {
            // Filter by player_id: join with match_participants
            sqlx::query_as::<_, MatchRow>(
                "SELECT m.id, m.season_id, m.recorded_at, m.score_metadata_json, m.is_corrected, m.created_at, \
                 (SELECT COUNT(*) FROM matches m2 WHERE m2.season_id = m.season_id AND m2.recorded_at <= m.recorded_at) AS match_number \
                 FROM matches m \
                 INNER JOIN match_participants mp ON mp.match_id = m.id \
                 WHERE m.season_id = ? AND mp.player_id = ? \
                 ORDER BY m.recorded_at \
                 LIMIT ? OFFSET ?",
            )
            .bind(season_id)
            .bind(player_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query_as::<_, MatchRow>(
                "SELECT m.id, m.season_id, m.recorded_at, m.score_metadata_json, m.is_corrected, m.created_at, \
                 (SELECT COUNT(*) FROM matches m2 WHERE m2.season_id = m.season_id AND m2.recorded_at <= m.recorded_at) AS match_number \
                 FROM matches m \
                 WHERE m.season_id = ? \
                 ORDER BY m.recorded_at \
                 LIMIT ? OFFSET ?",
            )
            .bind(season_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?
        };

        let mut matches = Vec::with_capacity(rows.len());
        for row in rows {
            matches.push(row.into_match()?);
        }
        Ok(matches)
    }

    /// Correct a match: update participants, insert audit log, queue recalculation job.
    /// All within a single transaction.
    pub async fn correct_match(
        pool: &SqlitePool,
        match_id: &str,
        correction: &MatchCorrection,
        changed_by: &str,
    ) -> Result<String> {
        // ── Input validation ────────────────────────────────────────────
        if match_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "match_id cannot be empty".into(),
            ));
        }
        if correction.new_participants.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "new_participants cannot be empty".into(),
            ));
        }
        if correction.reason.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "reason cannot be empty".into(),
            ));
        }

        // Ensure the changed_by user exists (auto-create if needed for FK constraint)
        ensure_user_exists(pool, changed_by).await?;

        // ── Transaction ─────────────────────────────────────────────────
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| PersistenceError::TransactionError(e.to_string()))?;

        // Step 1: Verify match exists and capture before state
        let existing = sqlx::query_as::<_, (String,)>("SELECT id FROM matches WHERE id = ?")
            .bind(match_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let (_match_id,) = existing.ok_or_else(|| PersistenceError::NotFound {
            entity: "match".into(),
            id: match_id.to_string(),
        })?;

        // Fetch current participants for before_state
        let before_participants = sqlx::query_as::<_, ParticipantRow>(
            "SELECT player_id, placement, rating_before, rating_after \
             FROM match_participants WHERE match_id = ? ORDER BY placement",
        )
        .bind(match_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let before_state = serde_json::json!({
            "participants": before_participants.iter().map(|p| {
                serde_json::json!({
                    "player_id": p.player_id,
                    "placement": p.placement,
                    "rating_before": p.rating_before,
                    "rating_after": p.rating_after,
                })
            }).collect::<Vec<_>>()
        });

        // Build after_state
        let after_state = serde_json::json!({
            "participants": correction.new_participants.iter().map(|p| {
                serde_json::json!({
                    "player_id": p.player_id,
                    "placement": p.placement,
                })
            }).collect::<Vec<_>>(),
            "score_metadata": correction.score_metadata,
        });

        // Step 2: Mark match as corrected
        sqlx::query("UPDATE matches SET is_corrected = 1 WHERE id = ?")
            .bind(match_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Step 3: INSERT audit log
        let audit_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let before_state_str = serde_json::to_string(&before_state).map_err(|e| {
            PersistenceError::InvalidInput(format!("Failed to serialize before_state: {}", e))
        })?;
        let after_state_str = serde_json::to_string(&after_state).map_err(|e| {
            PersistenceError::InvalidInput(format!("Failed to serialize after_state: {}", e))
        })?;

        sqlx::query(
            "INSERT INTO match_audit_log (id, match_id, actor_user_id, original_data_json, corrected_data_json, reason, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&audit_id)
        .bind(match_id)
        .bind(changed_by)
        .bind(&before_state_str)
        .bind(&after_state_str)
        .bind(&correction.reason)
        .bind(now.to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Step 4: Insert recalculation job
        // Need the season_id — fetch from match
        let season_row: (String,) = sqlx::query_as("SELECT season_id FROM matches WHERE id = ?")
            .bind(match_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let season_id = season_row.0;

        let job_id = JobRepository::insert_job_tx(&mut tx, &season_id, changed_by).await?;

        // Step 5: Commit
        tx.commit()
            .await
            .map_err(|e| PersistenceError::TransactionError(e.to_string()))?;

        Ok(job_id)
    }

    /// Check if a match is a duplicate based on participants, placements, and timestamp
    pub async fn is_duplicate(
        pool: &SqlitePool,
        season_id: &str,
        participants: &[MatchParticipant],
        recorded_at: &DateTime<Utc>,
    ) -> Result<bool> {
        is_duplicate_query(pool, season_id, participants, recorded_at).await
    }

    /// Check if a season is closed for writing
    pub async fn is_season_closed(pool: &SqlitePool, season_id: &str) -> Result<bool> {
        if season_id.is_empty() {
            return Ok(false);
        }

        let row = sqlx::query("SELECT is_open, end_date FROM seasons WHERE id = ?")
            .bind(season_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let is_open: bool = sqlx::Row::get(&r, 0);
                if !is_open {
                    return Ok(true);
                }
                // Also check end_date
                let end_date: Option<String> = sqlx::Row::get(&r, 1);
                if let Some(ed) = end_date {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(&ed) {
                        if dt <= Utc::now() {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            }
            None => Ok(false),
        }
    }

    // ── Private helpers ─────────────────────────────────────────────────────
}

impl MatchRepository {
    /// Check if season is closed (transaction-aware).
    async fn is_season_closed_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        season_id: &str,
    ) -> Result<bool> {
        let row = sqlx::query("SELECT is_open, end_date FROM seasons WHERE id = ?")
            .bind(season_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let is_open: bool = sqlx::Row::get(&r, 0);
                if !is_open {
                    return Ok(true);
                }
                let end_date: Option<String> = sqlx::Row::get(&r, 1);
                if let Some(ed) = end_date {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(&ed) {
                        if dt <= Utc::now() {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            }
            None => Ok(false),
        }
    }

    /// Check if match is duplicate (transaction-aware).
    async fn is_duplicate_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        season_id: &str,
        participants: &[MatchParticipant],
        recorded_at: &DateTime<Utc>,
    ) -> Result<bool> {
        let recorded_at_str = recorded_at.to_rfc3339();
        let n = participants.len() as i64;

        // Find candidate matches: same season, same recorded_at, same participant count
        let candidate_ids: Vec<(String,)> = sqlx::query_as(
            "SELECT m.id FROM matches m \
             WHERE m.season_id = ? AND m.recorded_at = ? \
             AND (SELECT COUNT(*) FROM match_participants WHERE match_id = m.id) = ?",
        )
        .bind(season_id)
        .bind(&recorded_at_str)
        .bind(n)
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        for (candidate_id,) in &candidate_ids {
            let rows = sqlx::query_as::<_, (String, i32)>(
                "SELECT player_id, placement FROM match_participants WHERE match_id = ? ORDER BY placement",
            )
            .bind(candidate_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

            let mut incoming: Vec<(String, i32)> = participants
                .iter()
                .map(|p| (p.player_id.clone(), p.placement))
                .collect();
            incoming.sort();

            let mut existing = rows;
            existing.sort();

            if incoming == existing {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Compute the next rating_period for a season (count distinct matches with snapshots + 1).
    async fn compute_next_rating_period_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        season_id: &str,
    ) -> Result<i32> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT match_id) FROM rating_snapshots WHERE season_id = ?",
        )
        .bind(season_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok((row.0 + 1) as i32)
    }
}

// ── Transaction-level helpers (reuse across methods) ─────────────────────────

/// Ensure a user record exists for FK constraints. Auto-creates one if missing.
async fn ensure_user_exists(pool: &SqlitePool, user_id: &str) -> Result<()> {
    if user_id.is_empty() {
        return Ok(());
    }
    let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
    if exists.0 > 0 {
        return Ok(());
    }
    // Auto-create a minimal user record
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(format!("auto_{}", &user_id[..8.min(user_id.len())]))
    .bind(format!("{}@auto.local", &user_id[..8.min(user_id.len())]))
    .bind("auto_created")
    .bind("user")
    .execute(pool)
    .await
    .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
    Ok(())
}

async fn fetch_season_algorithm_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    season_id: &str,
) -> Result<String> {
    let row: (String,) = sqlx::query_as("SELECT algorithm FROM seasons WHERE id = ?")
        .bind(season_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?
        .ok_or_else(|| PersistenceError::NotFound {
            entity: "season".into(),
            id: season_id.to_string(),
        })?;

    Ok(row.0)
}

async fn fetch_latest_snapshot_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    player_id: &str,
    season_id: &str,
) -> Result<Option<RatingSnapshot>> {
    let row = sqlx::query_as::<_, SnapshotRow>(
        "SELECT id, season_id, player_id, match_id, conservative_rating, rating_json, created_at \
         FROM rating_snapshots \
         WHERE player_id = ? AND season_id = ? \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(player_id)
    .bind(season_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

    match row {
        Some(r) => Ok(Some(r.into_snapshot()?)),
        None => Ok(None),
    }
}
