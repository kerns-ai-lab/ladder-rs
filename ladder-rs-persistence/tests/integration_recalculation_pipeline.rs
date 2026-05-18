//! Integration tests for the recalculation pipeline.
//!
//! Tests the end-to-end flow: alias trigger → job creation → claim job →
//! season replay → snapshot replace → leaderboard update. Also covers
//! alias removal, full-season replay determinism, and job lifecycle.
//!
//! ## Coverage Map
//!
//! | Flow                    | Tests |
//! |-------------------------|-------|
//! | Alias trigger → jobs    | 1     |
//! | Full pipeline replay    | 1     |
//! | Season replay determinism| 1    |
//! | Snapshot replace        | 1     |
//! | Alias removal           | 1     |
//! | Job deduplication       | 1     |
//! | Job lifecycle           | 1     |
//! | Bridge determinism      | 1     |
//!
//! Total: 8 tests

use chrono::Utc;
use ladder_rs_persistence::{
    create_pool, AlgorithmParams, AliasRepository, JobRepository, LeagueRepository, MatchInput,
    MatchParticipant, MatchRepository, PlayerRepository, RatingEngineBridge, RatingInput,
    SeasonRepository, SeedingChoice,
};
use sqlx::SqlitePool;
use std::collections::HashSet;

// ============================================================================
// Test Fixtures
// ============================================================================

/// Creates an in-memory SQLite pool with full migrations applied.
async fn setup_test_db() -> SqlitePool {
    use sqlx::migrate::Migrator;
    use std::path::Path;

    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite pool for testing");

    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    if migrations_path.exists() {
        let migrator = Migrator::new(migrations_path)
            .await
            .expect("Failed to create migrator");
        migrator.run(&pool).await.expect("Failed to run migrations");
    }

    pool
}

/// Fixed timestamp for deterministic test matches.
fn test_timestamp(offset_minutes: i64) -> chrono::DateTime<Utc> {
    let base = chrono::DateTime::parse_from_rfc3339("2025-06-15T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    base + chrono::Duration::minutes(offset_minutes)
}

/// Build a list of MatchParticipant from player IDs with sequential placements.
fn make_participants(player_ids: &[String]) -> Vec<MatchParticipant> {
    player_ids
        .iter()
        .enumerate()
        .map(|(i, pid)| MatchParticipant {
            player_id: pid.clone(),
            placement: (i + 1) as i32,
        })
        .collect()
}

/// Seeds a user record (needed for alias FK constraints).
async fn seed_user(pool: &SqlitePool, id: &str) {
    sqlx::query("INSERT INTO users (id, username, email, password_hash, role) VALUES (?, ?, ?, 'hash', 'user')")
        .bind(id)
        .bind(format!("user_{}", id))
        .bind(format!("{}@test.local", id))
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to insert user {}: {}", id, e));
}

/// Creates a league, season, and players. Returns (league_id, season_id, player_ids).
async fn seed_league_season_players(
    pool: &SqlitePool,
    algorithm: &str,
    player_count: usize,
) -> (String, String, Vec<String>) {
    // Create league with unique name (leagues.name has unique constraint)
    let league_name = format!("Test League {}", uuid::Uuid::new_v4());
    let league =
        LeagueRepository::create_league(pool, &league_name, "", algorithm, "public", "admin")
            .await
            .expect("Failed to create league");

    // Create season
    let params = AlgorithmParams {
        initial_rating: if algorithm == "trueskill" {
            25.0
        } else {
            1500.0
        },
        initial_deviation: if algorithm == "elo" {
            None
        } else {
            Some(350.0)
        },
        extra: None,
    };
    let season =
        SeasonRepository::create_season(pool, &league.id, algorithm, &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    // Create players with unique names (players.name has unique constraint)
    let mut player_ids = Vec::with_capacity(player_count);
    for i in 0..player_count {
        let player_name = format!("Player {} [{}]", i + 1, uuid::Uuid::new_v4());
        let player = PlayerRepository::create_player(pool, &player_name, "human")
            .await
            .expect("Failed to create player");
        player_ids.push(player.id);
    }

    (league.id, season.id, player_ids)
}

/// Reads match participants directly from the database.
async fn read_match_participants(pool: &SqlitePool, match_id: &str) -> Vec<(String, i32)> {
    #[derive(sqlx::FromRow)]
    struct Row {
        player_id: String,
        placement: i32,
    }
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT player_id, placement FROM match_participants WHERE match_id = ? ORDER BY placement",
    )
    .bind(match_id)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to read match participants: {}", e));
    rows.into_iter()
        .map(|r| (r.player_id, r.placement))
        .collect()
}

/// Reads all rating snapshots for a season, ordered by created_at.
async fn read_season_snapshots(
    pool: &SqlitePool,
    season_id: &str,
) -> Vec<ladder_rs_persistence::RatingSnapshot> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        match_id: String,
        player_id: String,
        season_id: String,
        conservative_rating: f64,
        rating_json: String,
        created_at: String,
    }

    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, season_id, player_id, match_id, conservative_rating, rating_json, created_at \
         FROM rating_snapshots WHERE season_id = ? ORDER BY created_at ASC",
    )
    .bind(season_id)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to read snapshots: {}", e));

    rows.into_iter()
        .map(|r| {
            #[derive(serde::Deserialize)]
            struct RatingJson {
                rating_value: f64,
                uncertainty: Option<f64>,
                volatility: Option<f64>,
                rating_period: i32,
            }
            let rj: RatingJson =
                serde_json::from_str(&r.rating_json).expect("Failed to parse rating_json");
            let created_at = chrono::DateTime::parse_from_rfc3339(&r.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .expect("Failed to parse created_at");
            ladder_rs_persistence::RatingSnapshot {
                id: r.id,
                match_id: r.match_id,
                player_id: r.player_id,
                season_id: r.season_id,
                rating_value: rj.rating_value,
                uncertainty: rj.uncertainty,
                volatility: rj.volatility,
                conservative_rating: r.conservative_rating,
                rating_period: rj.rating_period,
                created_at,
            }
        })
        .collect()
}

/// Reads the latest rating snapshot for each player in a season.
async fn read_latest_snapshots(
    pool: &SqlitePool,
    season_id: &str,
    player_ids: &[String],
) -> Vec<(String, ladder_rs_persistence::RatingSnapshot)> {
    let all_snapshots = read_season_snapshots(pool, season_id).await;
    let mut latest: std::collections::HashMap<String, ladder_rs_persistence::RatingSnapshot> =
        std::collections::HashMap::new();

    for snap in all_snapshots {
        if player_ids.contains(&snap.player_id) {
            latest
                .entry(snap.player_id.clone())
                .and_modify(|existing| {
                    if snap.created_at > existing.created_at {
                        *existing = snap.clone();
                    }
                })
                .or_insert(snap);
        }
    }

    let mut result: Vec<_> = latest.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Replaces all snapshots in a season with new ones (delete old, insert new).
async fn replace_season_snapshots(
    pool: &SqlitePool,
    season_id: &str,
    new_snapshots: &[ladder_rs_persistence::RatingSnapshot],
) {
    sqlx::query("DELETE FROM rating_snapshots WHERE season_id = ?")
        .bind(season_id)
        .execute(pool)
        .await
        .expect("Failed to delete old snapshots");

    for snap in new_snapshots {
        #[derive(serde::Serialize)]
        struct RatingJson {
            rating_value: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            uncertainty: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            volatility: Option<f64>,
            rating_period: i32,
        }
        let rj = RatingJson {
            rating_value: snap.rating_value,
            uncertainty: snap.uncertainty,
            volatility: snap.volatility,
            rating_period: snap.rating_period,
        };
        let rating_json_str = serde_json::to_string(&rj).expect("Failed to serialize rating_json");

        sqlx::query(
            "INSERT INTO rating_snapshots (id, season_id, player_id, match_id, conservative_rating, rating_json, created_at, timestamp) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&snap.id)
        .bind(&snap.season_id)
        .bind(&snap.player_id)
        .bind(&snap.match_id)
        .bind(snap.conservative_rating)
        .bind(&rating_json_str)
        .bind(snap.created_at.to_rfc3339())
        .bind(snap.created_at.to_rfc3339())
        .execute(pool)
        .await
        .expect("Failed to insert new snapshot");
    }
}

// ============================================================================
// Test 1: Alias trigger creates recalculation jobs
// ============================================================================

#[tokio::test]
async fn test_alias_trigger_creates_recalculation_jobs() {
    let pool = setup_test_db().await;
    seed_user(&pool, "admin").await;

    let (_league_id, season_id, player_ids) = seed_league_season_players(&pool, "elo", 3).await;

    // Record matches so the season has player history
    MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&player_ids[..2]),
        None,
        test_timestamp(0),
    )
    .await
    .expect("Failed to record match");

    MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&[player_ids[0].clone(), player_ids[2].clone()]),
        None,
        test_timestamp(10),
    )
    .await
    .expect("Failed to record match");

    // Create alias between player 0 and player 1
    let job_ids = AliasRepository::create_alias(&pool, &player_ids[0], &player_ids[1], "admin")
        .await
        .expect("create_alias should succeed");

    // Verify job IDs returned
    assert!(
        !job_ids.is_empty(),
        "Alias creation must trigger at least one recalculation job"
    );
    for id in &job_ids {
        assert!(!id.is_empty(), "Job ID must not be empty");
        assert!(
            uuid::Uuid::parse_str(id).is_ok(),
            "Job ID must be a valid UUID: {}",
            id
        );
    }

    // Verify jobs exist in the database
    for job_id in &job_ids {
        let job = JobRepository::get_job(&pool, job_id)
            .await
            .expect("get_job should succeed")
            .expect("Job should exist in the database");

        assert_eq!(job.season_id, season_id, "Job season_id should match");
        assert!(
            matches!(job.status, ladder_rs_persistence::JobStatus::Queued),
            "Job should be in Queued status, got {:?}",
            job.status
        );
    }

    // Verify no duplicate jobs created for the same season (deduplication)
    let alias2_job_ids =
        AliasRepository::create_alias(&pool, &player_ids[0], &player_ids[1], "admin").await;

    // The second create_alias should fail because the unique constraint
    // on (primary_player_id, alias_player_id) prevents re-inserting the same alias.
    // If it succeeds, the returned job IDs should match existing ones.
    match alias2_job_ids {
        Ok(ids) => {
            // Deduplication: should return the existing job IDs
            let original_set: HashSet<_> = job_ids.iter().collect();
            let second_set: HashSet<_> = ids.iter().collect();
            assert_eq!(
                original_set, second_set,
                "Second create_alias (duplicate) should return the same job IDs"
            );
        }
        Err(e) => {
            // Conflict on unique constraint is also valid
            let msg = format!("{}", e).to_lowercase();
            assert!(
                msg.contains("unique") || msg.contains("conflict") || msg.contains("already"),
                "Expected conflict or dedup on duplicate alias. Got: {}",
                msg
            );
        }
    }
}

// ============================================================================
// Test 2: Full recalculation pipeline — alias trigger → claim → replay → replace
// ============================================================================

#[tokio::test]
async fn test_full_recalculation_pipeline_alias_trigger_to_leaderboard() {
    let pool = setup_test_db().await;
    seed_user(&pool, "admin").await;

    let (_league_id, season_id, player_ids) = seed_league_season_players(&pool, "elo", 3).await;

    // Record several matches
    let _match1 = MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&player_ids[..2]),
        None,
        test_timestamp(0),
    )
    .await
    .expect("Failed to record match 1");

    let _match2 = MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&[player_ids[1].clone(), player_ids[2].clone()]),
        None,
        test_timestamp(10),
    )
    .await
    .expect("Failed to record match 2");

    // Capture snapshots before alias
    let snapshots_before = read_season_snapshots(&pool, &season_id).await;
    assert!(
        !snapshots_before.is_empty(),
        "Should have snapshots before alias trigger"
    );

    // Create alias
    let job_ids = AliasRepository::create_alias(&pool, &player_ids[0], &player_ids[1], "admin")
        .await
        .expect("create_alias should succeed");

    assert!(!job_ids.is_empty(), "Alias creation must create jobs");

    // Claim the next queued job (simulating the worker)
    let claimed_job = JobRepository::claim_next_job(&pool)
        .await
        .expect("claim_next_job should succeed")
        .expect("Should have a job to claim");

    assert_eq!(
        claimed_job.season_id, season_id,
        "Claimed job should be for the correct season"
    );
    assert!(
        matches!(
            claimed_job.status,
            ladder_rs_persistence::JobStatus::InProgress
        ),
        "Claimed job should be InProgress, got {:?}",
        claimed_job.status
    );

    // Simulate the recalculation worker: re-read matches and recompute ratings
    // Read all matches in the season
    let filter = ladder_rs_persistence::MatchFilter {
        limit: Some(100),
        offset: None,
        player_id: None,
    };
    let matches = MatchRepository::list_matches(&pool, &season_id, &filter)
        .await
        .expect("list_matches should succeed");

    assert!(
        matches.len() >= 2,
        "Should have at least 2 matches, got {}",
        matches.len()
    );

    // Recompute snapshots: for each match, gather participants, compute via bridge
    let mut new_snapshots: Vec<ladder_rs_persistence::RatingSnapshot> = Vec::new();
    let mut current_ratings: std::collections::HashMap<String, RatingInput> =
        std::collections::HashMap::new();

    for m in &matches {
        let participants = read_match_participants(&pool, &m.id).await;
        let player_ids_m: Vec<String> = participants.iter().map(|(pid, _)| pid.clone()).collect();
        let placements: Vec<u32> = participants.iter().map(|(_, p)| *p as u32).collect();
        let draws: Vec<bool> = vec![false; participants.len()];

        // Get pre-match ratings (use current_ratings or defaults)
        let ratings: Vec<RatingInput> = player_ids_m
            .iter()
            .map(|pid| {
                current_ratings
                    .get(pid)
                    .cloned()
                    .unwrap_or_else(|| RatingInput {
                        rating: 1500.0,
                        uncertainty: None,
                        volatility: None,
                    })
            })
            .collect();

        let match_input = MatchInput {
            ratings,
            placements,
            draws,
        };

        let bridge_result =
            RatingEngineBridge::compute("elo", &match_input, &player_ids_m, &season_id, &m.id)
                .expect("RatingEngineBridge::compute should succeed");

        let snaps = RatingEngineBridge::to_snapshots(
            &bridge_result,
            &player_ids_m,
            &season_id,
            (new_snapshots.len() as i32 / player_ids_m.len() as i32) + 1,
        )
        .expect("to_snapshots should succeed");

        // Update current_ratings for the next match
        for (pid, snap) in player_ids_m.iter().zip(snaps.iter()) {
            current_ratings.insert(
                pid.clone(),
                RatingInput {
                    rating: snap.rating_value,
                    uncertainty: snap.uncertainty,
                    volatility: snap.volatility,
                },
            );
        }

        new_snapshots.extend(snaps);
    }

    assert!(
        !new_snapshots.is_empty(),
        "Recalculation should produce new snapshots"
    );

    // Replace old snapshots with new ones
    replace_season_snapshots(&pool, &season_id, &new_snapshots).await;

    // Mark job as completed
    JobRepository::mark_completed(&pool, &claimed_job.id)
        .await
        .expect("mark_completed should succeed");

    // Verify job status is now Completed
    let updated_job = JobRepository::get_job(&pool, &claimed_job.id)
        .await
        .expect("get_job should succeed")
        .expect("Job should still exist");

    assert!(
        matches!(
            updated_job.status,
            ladder_rs_persistence::JobStatus::Completed
        ),
        "Job should be Completed, got {:?}",
        updated_job.status
    );

    // Verify snapshots are now the new ones (not the originals)
    let snapshots_after = read_season_snapshots(&pool, &season_id).await;
    assert_eq!(
        snapshots_after.len(),
        new_snapshots.len(),
        "Snapshot count should match new snapshots after replace"
    );
}

// ============================================================================
// Test 3: Full season replay determinism
// ============================================================================

#[tokio::test]
async fn test_full_season_replay_is_deterministic() {
    let pool = setup_test_db().await;

    let (_league_id, season_id, player_ids) = seed_league_season_players(&pool, "elo", 4).await;

    // Record several matches
    for offset in &[0, 10, 20, 30, 40] {
        // Rotate which players play
        let idx = (*offset / 10) as usize % 3;
        let p1 = &player_ids[idx];
        let p2 = &player_ids[(idx + 1) % player_ids.len()];
        MatchRepository::record_match(
            &pool,
            &season_id,
            make_participants(&[p1.clone(), p2.clone()]),
            None,
            test_timestamp(*offset),
        )
        .await
        .expect("Failed to record match");
    }

    // Collect all match results (match_id + participants in order)
    let filter = ladder_rs_persistence::MatchFilter {
        limit: Some(100),
        offset: None,
        player_id: None,
    };
    let matches = MatchRepository::list_matches(&pool, &season_id, &filter)
        .await
        .expect("list_matches should succeed");

    assert!(
        matches.len() >= 5,
        "Should have at least 5 matches for season replay"
    );

    // Helper: replay the season from scratch and return the final ratings per player
    async fn replay_season(
        pool: &SqlitePool,
        matches: &[ladder_rs_persistence::Match],
        season_id: &str,
    ) -> std::collections::HashMap<String, f64> {
        let mut current_ratings: std::collections::HashMap<String, RatingInput> =
            std::collections::HashMap::new();

        for m in matches {
            let participants = read_match_participants(pool, &m.id).await;
            let pids: Vec<String> = participants.iter().map(|(pid, _)| pid.clone()).collect();
            let placements: Vec<u32> = participants.iter().map(|(_, p)| *p as u32).collect();
            let draws: Vec<bool> = vec![false; participants.len()];

            let ratings: Vec<RatingInput> = pids
                .iter()
                .map(|pid| {
                    current_ratings.get(pid).cloned().unwrap_or(RatingInput {
                        rating: 1500.0,
                        uncertainty: None,
                        volatility: None,
                    })
                })
                .collect();

            let match_input = MatchInput {
                ratings,
                placements,
                draws,
            };

            let result = RatingEngineBridge::compute("elo", &match_input, &pids, season_id, &m.id)
                .expect("compute should succeed");

            for (pid, output) in pids.iter().zip(result.outputs.iter()) {
                current_ratings.insert(
                    pid.clone(),
                    RatingInput {
                        rating: output.rating,
                        uncertainty: output.uncertainty,
                        volatility: output.volatility,
                    },
                );
            }
        }

        current_ratings
            .into_iter()
            .map(|(k, v)| (k, v.rating))
            .collect()
    }

    // Run the replay twice and compare
    let first_replay = replay_season(&pool, &matches, &season_id).await;
    let second_replay = replay_season(&pool, &matches, &season_id).await;

    // Same number of players should have final ratings
    assert_eq!(
        first_replay.len(),
        second_replay.len(),
        "Both replays should produce ratings for the same set of players"
    );

    // Each player should have exactly the same final rating in both replays
    for (pid, rating1) in &first_replay {
        let rating2 = second_replay
            .get(pid)
            .unwrap_or_else(|| panic!("Player {} missing from second replay", pid));
        assert!(
            (rating1 - rating2).abs() < 1e-10,
            "Replay ratings for player {} must be deterministic: {} vs {}",
            pid,
            rating1,
            rating2
        );
    }
}

// ============================================================================
// Test 4: Snapshot replace determinism
// ============================================================================

#[tokio::test]
async fn test_snapshot_replace_preserves_deterministic_output() {
    let pool = setup_test_db().await;

    let (_league_id, season_id, player_ids) = seed_league_season_players(&pool, "glicko2", 2).await;

    // Record a match
    let match_result = MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&player_ids[..2]),
        None,
        test_timestamp(0),
    )
    .await
    .expect("Failed to record match");

    let original_snapshots = match_result.snapshots;
    assert_eq!(
        original_snapshots.len(),
        2,
        "Should have 2 snapshots for 2 participants"
    );

    // Read match participants and re-compute via the bridge with the same input
    let participants = read_match_participants(&pool, &match_result.match_id).await;
    let pids: Vec<String> = participants.iter().map(|(pid, _)| pid.clone()).collect();
    let placements: Vec<u32> = participants.iter().map(|(_, p)| *p as u32).collect();
    let draws: Vec<bool> = vec![false; participants.len()];

    // Use default ratings (same as what record_match used for new players)
    let ratings: Vec<RatingInput> = pids
        .iter()
        .map(|_| RatingInput {
            rating: 1500.0,
            uncertainty: Some(350.0),
            volatility: Some(0.06),
        })
        .collect();

    let match_input = MatchInput {
        ratings,
        placements,
        draws,
    };

    let bridge_result = RatingEngineBridge::compute(
        "glicko2",
        &match_input,
        &pids,
        &season_id,
        &match_result.match_id,
    )
    .expect("compute should succeed");

    let recomputed_snapshots =
        RatingEngineBridge::to_snapshots(&bridge_result, &pids, &season_id, 1)
            .expect("to_snapshots should succeed");

    assert_eq!(
        recomputed_snapshots.len(),
        original_snapshots.len(),
        "Recomputed snapshot count should match original"
    );

    // Compare ratings: recomputed should match original (determinism)
    // Sort by player_id for comparison
    let mut orig_sorted = original_snapshots.clone();
    orig_sorted.sort_by(|a, b| a.player_id.cmp(&b.player_id));
    let mut recomputed_sorted = recomputed_snapshots.clone();
    recomputed_sorted.sort_by(|a, b| a.player_id.cmp(&b.player_id));

    for (orig, recomputed) in orig_sorted.iter().zip(recomputed_sorted.iter()) {
        assert_eq!(
            orig.player_id, recomputed.player_id,
            "Player IDs should match in sorted order"
        );
        assert!(
            (orig.rating_value - recomputed.rating_value).abs() < 1e-10,
            "Rating for player {} should be deterministic: {} vs {}",
            orig.player_id,
            orig.rating_value,
            recomputed.rating_value
        );
        if let (Some(ou), Some(ru)) = (orig.uncertainty, recomputed.uncertainty) {
            assert!(
                (ou - ru).abs() < 1e-10,
                "Uncertainty for player {} should be deterministic: {} vs {}",
                orig.player_id,
                ou,
                ru
            );
        }
    }

    // Now replace old snapshots with recomputed ones (same values)
    replace_season_snapshots(&pool, &season_id, &recomputed_snapshots).await;

    // Verify the snapshots in the database match
    let db_snapshots = read_season_snapshots(&pool, &season_id).await;
    assert_eq!(
        db_snapshots.len(),
        recomputed_snapshots.len(),
        "DB snapshot count after replace should match"
    );

    let mut db_sorted = db_snapshots.clone();
    db_sorted.sort_by(|a, b| a.player_id.cmp(&b.player_id));

    for (db, recomp) in db_sorted.iter().zip(recomputed_sorted.iter()) {
        assert_eq!(db.player_id, recomp.player_id);
        assert!(
            (db.rating_value - recomp.rating_value).abs() < 1e-10,
            "DB rating should match recomputed for player {}",
            db.player_id
        );
    }
}

// ============================================================================
// Test 5: Alias removal creates jobs and changes ratings
// ============================================================================

#[tokio::test]
async fn test_alias_removal_creates_jobs_and_enables_independent_replay() {
    let pool = setup_test_db().await;
    seed_user(&pool, "admin").await;

    let (_league_id, season_id, player_ids) = seed_league_season_players(&pool, "elo", 3).await;

    // Record matches before alias
    MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&player_ids[..2]),
        None,
        test_timestamp(0),
    )
    .await
    .expect("Failed to record match 1");

    // Create alias between player 0 and player 1
    let _alias_job_ids =
        AliasRepository::create_alias(&pool, &player_ids[0], &player_ids[1], "admin")
            .await
            .expect("create_alias should succeed");

    // Record another match
    MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&[player_ids[0].clone(), player_ids[2].clone()]),
        None,
        test_timestamp(10),
    )
    .await
    .expect("Failed to record match 2");

    // Capture the latest snapshots for all players
    let snapshots_with_alias = read_latest_snapshots(&pool, &season_id, &player_ids).await;
    assert_eq!(
        snapshots_with_alias.len(),
        3,
        "Should have latest snapshots for all 3 players"
    );

    // Consume (claim) the alias-link job so the subsequent removal creates
    // a fresh job instead of deduplicating to the existing queued one.
    {
        let claimed = JobRepository::claim_next_job(&pool)
            .await
            .expect("claim_next_job should succeed")
            .expect("Should have a job from create_alias");
        JobRepository::mark_completed(&pool, &claimed.id)
            .await
            .expect("mark_completed should succeed");
    }

    // Now remove the alias
    let removal_job_ids = AliasRepository::remove_alias(&pool, &player_ids[0], &player_ids[1])
        .await
        .expect("remove_alias should succeed");

    assert!(
        !removal_job_ids.is_empty(),
        "Alias removal must trigger recalculation jobs"
    );

    // Verify jobs are queued
    for job_id in &removal_job_ids {
        let job = JobRepository::get_job(&pool, job_id)
            .await
            .expect("get_job should succeed")
            .expect("Job should exist");
        assert_eq!(
            job.triggered_by, "alias_unlink",
            "Job should be triggered by 'alias_unlink'"
        );
        assert!(
            matches!(job.status, ladder_rs_persistence::JobStatus::Queued),
            "Job should be Queued"
        );
    }

    // Verify the alias was actually removed from the database
    let aliases = AliasRepository::get_aliases(&pool, &player_ids[0])
        .await
        .expect("get_aliases should succeed");
    let still_linked = aliases.iter().any(|a| {
        (a.primary_player_id == player_ids[0] && a.alias_player_id == player_ids[1])
            || (a.primary_player_id == player_ids[1] && a.alias_player_id == player_ids[0])
    });
    assert!(!still_linked, "Alias should be removed from the database");

    // Replay the season without the alias (players are independent)
    let filter = ladder_rs_persistence::MatchFilter {
        limit: Some(100),
        offset: None,
        player_id: None,
    };
    let matches = MatchRepository::list_matches(&pool, &season_id, &filter)
        .await
        .expect("list_matches should succeed");

    let mut current_ratings: std::collections::HashMap<String, RatingInput> =
        std::collections::HashMap::new();

    for m in &matches {
        let participants = read_match_participants(&pool, &m.id).await;
        let pids: Vec<String> = participants.iter().map(|(pid, _)| pid.clone()).collect();
        let placements: Vec<u32> = participants.iter().map(|(_, p)| *p as u32).collect();
        let draws: Vec<bool> = vec![false; participants.len()];

        let ratings: Vec<RatingInput> = pids
            .iter()
            .map(|pid| {
                current_ratings.get(pid).cloned().unwrap_or(RatingInput {
                    rating: 1500.0,
                    uncertainty: None,
                    volatility: None,
                })
            })
            .collect();

        let match_input = MatchInput {
            ratings,
            placements,
            draws,
        };

        let result = RatingEngineBridge::compute("elo", &match_input, &pids, &season_id, &m.id)
            .expect("compute should succeed");

        for (pid, output) in pids.iter().zip(result.outputs.iter()) {
            current_ratings.insert(
                pid.clone(),
                RatingInput {
                    rating: output.rating,
                    uncertainty: output.uncertainty,
                    volatility: output.volatility,
                },
            );
        }
    }

    // We should have final ratings for all players after independent replay
    assert!(
        current_ratings.len() >= 2,
        "Replay without alias should produce ratings for at least 2 players"
    );

    // At minimum, verify that the two previously-aliased players have
    // independent ratings (they won't necessarily be different, but the
    // replay system treats them as separate)
    let rating0 = current_ratings.get(&player_ids[0]);
    let rating1 = current_ratings.get(&player_ids[1]);
    assert!(
        rating0.is_some(),
        "Player 0 should have a rating after replay"
    );
    assert!(
        rating1.is_some(),
        "Player 1 should have a rating after replay"
    );
}

// ============================================================================
// Test 6: Job deduplication — alias triggers only one job per season
// ============================================================================

#[tokio::test]
async fn test_multiple_alias_triggers_no_duplicate_jobs() {
    let pool = setup_test_db().await;
    seed_user(&pool, "admin").await;

    let (_league_id, season_id, player_ids) = seed_league_season_players(&pool, "elo", 4).await;

    // Record matches to give all players history in the season
    MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&player_ids[..2]),
        None,
        test_timestamp(0),
    )
    .await
    .expect("Failed to record match 1");

    MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&player_ids[2..4]),
        None,
        test_timestamp(10),
    )
    .await
    .expect("Failed to record match 2");

    // Create first alias
    let job_ids_1 = AliasRepository::create_alias(&pool, &player_ids[0], &player_ids[1], "admin")
        .await
        .expect("First create_alias should succeed");

    assert!(!job_ids_1.is_empty(), "Should create at least one job");

    // Count queued jobs for this season
    #[derive(sqlx::FromRow)]
    struct CountRow {
        cnt: i64,
    }
    let count_before: CountRow = sqlx::query_as(
        "SELECT COUNT(*) as cnt FROM recalculation_jobs WHERE season_id = ? AND status = 'queued'",
    )
    .bind(&season_id)
    .fetch_one(&pool)
    .await
    .expect("Count query should succeed");

    let queued_before = count_before.cnt;

    // Create second alias with different players — this should create
    // additional jobs for the same season, but the first alias's job already
    // covers it. The insert_job method deduplicates: if a queued or in_progress
    // job already exists for the season, it returns the existing job ID.
    let job_ids_2 = AliasRepository::create_alias(&pool, &player_ids[2], &player_ids[3], "admin")
        .await
        .expect("Second create_alias should succeed");

    // Since there's already a queued job for this season, insert_job
    // should deduplicate and return the existing job ID.
    // The returned job_ids should be the same as before (the existing queued job).
    let count_after: CountRow = sqlx::query_as(
        "SELECT COUNT(*) as cnt FROM recalculation_jobs WHERE season_id = ? AND status = 'queued'",
    )
    .bind(&season_id)
    .fetch_one(&pool)
    .await
    .expect("Count query should succeed");

    assert_eq!(
        count_after.cnt, queued_before,
        "Number of queued jobs should not increase due to deduplication. Before: {}, After: {}",
        queued_before, count_after.cnt
    );

    // The returned IDs should match the existing job
    let set1: HashSet<_> = job_ids_1.iter().collect();
    let set2: HashSet<_> = job_ids_2.iter().collect();
    assert_eq!(
        set1, set2,
        "Deduplication should return the same existing job IDs"
    );
}

// ============================================================================
// Test 7: Recalculation job lifecycle (claim → complete → verify)
// ============================================================================

#[tokio::test]
async fn test_recalculation_job_lifecycle_claim_and_complete() {
    let pool = setup_test_db().await;
    seed_user(&pool, "admin").await;

    let (_league_id, season_id, player_ids) = seed_league_season_players(&pool, "elo", 2).await;

    // Record a match
    MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&player_ids[..2]),
        None,
        test_timestamp(0),
    )
    .await
    .expect("Failed to record match");

    // Trigger a recalculation via alias
    let job_ids = AliasRepository::create_alias(&pool, &player_ids[0], &player_ids[1], "admin")
        .await
        .expect("create_alias should succeed");

    assert_eq!(job_ids.len(), 1, "Should create exactly one job");

    // Claim the job
    let claimed = JobRepository::claim_next_job(&pool)
        .await
        .expect("claim_next_job should succeed")
        .expect("Should have a job to claim");

    assert_eq!(claimed.id, job_ids[0], "Should claim the created job");
    assert!(
        matches!(claimed.status, ladder_rs_persistence::JobStatus::InProgress),
        "Job should be InProgress after claim"
    );

    // Verify no more queued jobs
    let next = JobRepository::claim_next_job(&pool)
        .await
        .expect("claim_next_job should succeed");
    assert!(next.is_none(), "No more queued jobs should exist");

    // Mark the job as completed
    JobRepository::mark_completed(&pool, &claimed.id)
        .await
        .expect("mark_completed should succeed");

    // Verify job status
    let completed = JobRepository::get_job(&pool, &claimed.id)
        .await
        .expect("get_job should succeed")
        .expect("Job should exist");

    assert!(
        matches!(
            completed.status,
            ladder_rs_persistence::JobStatus::Completed
        ),
        "Job should be Completed"
    );

    // Marking again should succeed (idempotent update)
    JobRepository::mark_completed(&pool, &claimed.id)
        .await
        .expect("Second mark_completed should succeed (idempotent)");

    // Test failure path with a new job
    // Close the season so we can test with a fresh job
    let (_league_id2, season_id2, player_ids2) = seed_league_season_players(&pool, "elo", 2).await;

    MatchRepository::record_match(
        &pool,
        &season_id2,
        make_participants(&player_ids2[..2]),
        None,
        test_timestamp(0),
    )
    .await
    .expect("Failed to record match");

    let _job_ids2 = AliasRepository::create_alias(&pool, &player_ids2[0], &player_ids2[1], "admin")
        .await
        .expect("create_alias should succeed");

    // Claim and then fail
    let claimed2 = JobRepository::claim_next_job(&pool)
        .await
        .expect("claim should succeed")
        .expect("Should have job to claim");

    JobRepository::mark_failed(&pool, &claimed2.id, "Simulated worker crash")
        .await
        .expect("mark_failed should succeed");

    let failed_job = JobRepository::get_job(&pool, &claimed2.id)
        .await
        .expect("get_job should succeed")
        .expect("Job should exist");

    assert!(
        matches!(failed_job.status, ladder_rs_persistence::JobStatus::Failed),
        "Job should be Failed, got {:?}",
        failed_job.status
    );
    assert!(
        failed_job
            .error_message
            .as_deref()
            .unwrap_or("")
            .contains("Simulated"),
        "Job should have error message"
    );
}

// ============================================================================
// Test 8: RatingEngineBridge determinism across algorithms
// ============================================================================

#[tokio::test]
async fn test_rating_engine_bridge_determinism_across_algorithms() {
    // Test that the bridge produces identical results for identical inputs
    // across all supported algorithms
    let algorithms = ["elo", "glicko", "glicko2", "trueskill"];

    for algo in &algorithms {
        let ratings = vec![
            RatingInput {
                rating: if *algo == "trueskill" { 25.0 } else { 1500.0 },
                uncertainty: if *algo == "elo" { None } else { Some(350.0) },
                volatility: if *algo == "glicko2" { Some(0.06) } else { None },
            },
            RatingInput {
                rating: if *algo == "trueskill" { 25.0 } else { 1500.0 },
                uncertainty: if *algo == "elo" { None } else { Some(350.0) },
                volatility: if *algo == "glicko2" { Some(0.06) } else { None },
            },
        ];
        let placements = vec![1u32, 2u32];
        let draws = vec![false, false];
        let player_ids = vec!["alice".to_string(), "bob".to_string()];

        let input1 = MatchInput {
            ratings: ratings.clone(),
            placements: placements.clone(),
            draws: draws.clone(),
        };

        let input2 = MatchInput {
            ratings,
            placements,
            draws,
        };

        let result1 =
            RatingEngineBridge::compute(algo, &input1, &player_ids, "season-test", "match-test")
                .unwrap_or_else(|e| panic!("compute should succeed for {}: {}", algo, e));

        let result2 =
            RatingEngineBridge::compute(algo, &input2, &player_ids, "season-test", "match-test")
                .unwrap_or_else(|e| panic!("compute should succeed for {}: {}", algo, e));

        assert_eq!(
            result1.outputs.len(),
            result2.outputs.len(),
            "Same output count for {}",
            algo
        );

        for (i, (o1, o2)) in result1
            .outputs
            .iter()
            .zip(result2.outputs.iter())
            .enumerate()
        {
            assert!(
                (o1.rating - o2.rating).abs() < 1e-10,
                "Algorithm {}: rating for player {} should be deterministic: {} vs {}",
                algo,
                i,
                o1.rating,
                o2.rating
            );
            assert_eq!(
                o1.conservative_rating, o2.conservative_rating,
                "Algorithm {}: conservative_rating should be deterministic",
                algo
            );
        }
    }

    // Test single-participant degenerate case
    let single_rating = vec![RatingInput {
        rating: 1500.0,
        uncertainty: None,
        volatility: None,
    }];
    let single_input = MatchInput {
        ratings: single_rating.clone(),
        placements: vec![1],
        draws: vec![false],
    };
    let single_pids = vec!["solo".to_string()];

    let result_a = RatingEngineBridge::compute("elo", &single_input, &single_pids, "s", "m")
        .expect("Single player compute should succeed");

    let result_b = RatingEngineBridge::compute("elo", &single_input, &single_pids, "s", "m")
        .expect("Single player compute should succeed");

    assert_eq!(result_a.outputs.len(), 1);
    assert_eq!(result_b.outputs.len(), 1);
    assert!(
        (result_a.outputs[0].rating - result_b.outputs[0].rating).abs() < 1e-10,
        "Single-player degenerate case should be deterministic"
    );
    assert_eq!(
        result_a.convergence_quality, "degraded",
        "Single-player result should be marked as degraded"
    );
}

// ============================================================================
// Additional edge-case test: Error paths in bridge compute
// ============================================================================

#[tokio::test]
async fn test_rating_engine_bridge_error_paths() {
    // Empty ratings
    let result = RatingEngineBridge::compute(
        "elo",
        &MatchInput {
            ratings: vec![],
            placements: vec![],
            draws: vec![],
        },
        &[],
        "s",
        "m",
    );
    assert!(result.is_err(), "Empty ratings should produce an error");

    // Mismatched lengths
    let result = RatingEngineBridge::compute(
        "elo",
        &MatchInput {
            ratings: vec![RatingInput {
                rating: 1500.0,
                uncertainty: None,
                volatility: None,
            }],
            placements: vec![1, 2], // different length
            draws: vec![false],
        },
        &["a".to_string()],
        "s",
        "m",
    );
    assert!(
        result.is_err(),
        "Mismatched placements length should produce an error"
    );

    // Mismatched draws length
    let result = RatingEngineBridge::compute(
        "elo",
        &MatchInput {
            ratings: vec![RatingInput {
                rating: 1500.0,
                uncertainty: None,
                volatility: None,
            }],
            placements: vec![1],
            draws: vec![false, false], // different length
        },
        &["a".to_string()],
        "s",
        "m",
    );
    assert!(
        result.is_err(),
        "Mismatched draws length should produce an error"
    );

    // Mismatched player_ids length
    let result = RatingEngineBridge::compute(
        "elo",
        &MatchInput {
            ratings: vec![RatingInput {
                rating: 1500.0,
                uncertainty: None,
                volatility: None,
            }],
            placements: vec![1],
            draws: vec![false],
        },
        &["a".to_string(), "b".to_string()], // different length
        "s",
        "m",
    );
    assert!(
        result.is_err(),
        "Mismatched player_ids length should produce an error"
    );

    // Unknown algorithm — need at least 2 participants to reach the
    // algorithm dispatch (single-participant is a short-circuit degenerate case)
    let result = RatingEngineBridge::compute(
        "nonsense_algo",
        &MatchInput {
            ratings: vec![
                RatingInput {
                    rating: 1500.0,
                    uncertainty: None,
                    volatility: None,
                },
                RatingInput {
                    rating: 1500.0,
                    uncertainty: None,
                    volatility: None,
                },
            ],
            placements: vec![1, 2],
            draws: vec![false, false],
        },
        &["a".to_string(), "b".to_string()],
        "s",
        "m",
    );
    assert!(result.is_err(), "Unknown algorithm should produce an error");
    if let Err(e) = result {
        let msg = format!("{}", e).to_lowercase();
        assert!(
            msg.contains("unknown") || msg.contains("algorithm"),
            "Error should mention unknown algorithm, got: {}",
            msg
        );
    }
}

// ============================================================================
// Additional edge-case test: CorrectMatch creates recalculation jobs
// ============================================================================

#[tokio::test]
async fn test_correct_match_creates_recalculation_job() {
    let pool = setup_test_db().await;

    let (_league_id, season_id, player_ids) = seed_league_season_players(&pool, "elo", 4).await;

    // Record a match
    let match_result = MatchRepository::record_match(
        &pool,
        &season_id,
        make_participants(&player_ids[..2]),
        None,
        test_timestamp(0),
    )
    .await
    .expect("Failed to record match");

    // Correct the match
    let correction = ladder_rs_persistence::MatchCorrection {
        new_participants: vec![
            MatchParticipant {
                player_id: player_ids[2].clone(),
                placement: 1,
            },
            MatchParticipant {
                player_id: player_ids[3].clone(),
                placement: 2,
            },
        ],
        reason: "Wrong players".to_string(),
        score_metadata: None,
    };

    let job_id =
        MatchRepository::correct_match(&pool, &match_result.match_id, &correction, "admin_user")
            .await
            .expect("correct_match should succeed");

    assert!(!job_id.is_empty(), "Correction should return a job_id");
    assert!(
        uuid::Uuid::parse_str(&job_id).is_ok(),
        "job_id should be a valid UUID"
    );

    // Verify the job exists and is queued
    let job = JobRepository::get_job(&pool, &job_id)
        .await
        .expect("get_job should succeed")
        .expect("Job should exist");

    assert_eq!(job.season_id, season_id);
    assert!(
        matches!(job.status, ladder_rs_persistence::JobStatus::Queued),
        "Correction job should be Queued"
    );

    // Verify the match is marked as corrected
    let m = MatchRepository::get_by_id(&pool, &match_result.match_id)
        .await
        .expect("get_by_id should succeed")
        .expect("Match should exist");

    assert!(m.is_corrected, "Match should be marked as corrected");
}
