//! End-to-end integration tests: full match-record-to-leaderboard flow
//!
//! These tests exercise the REAL repository implementations (not stubs)
//! against an in-memory SQLite database, covering:
//!
//! 1. Full flow: league → season → players → league membership → match → snapshots → leaderboard
//! 2. Multiple matches produce progressive rating changes
//! 3. Match correction flow (correct_match → job created → match marked corrected)
//! 4. Batch match entry
//! 5. Season close prevents new matches
//! 6. Archived leagues still show historical data
//! 7. Leaderboard ordering by conservative rating
//! 8. Input validation (empty inputs, duplicate players)
//! 9. Duplicate match detection
//! 10. Matches with >2 participants

use chrono::{DateTime, Utc};
use ladder_rs_persistence::{
    create_pool as do_create_pool, AlgorithmParams, BatchEntry, JobRepository, LeagueFilter,
    LeagueRepository, MatchCorrection, MatchFilter, MatchParticipant, MatchRepository,
    PersistenceError, PlayerFilter, PlayerRepository, SeasonRepository, SeedingChoice,
};
use sqlx::{migrate::Migrator, Row, SqlitePool};
use std::path::Path;

// ── Test Fixtures ────────────────────────────────────────────────────────────

/// Creates a fully migrated in-memory SQLite pool for each test.
async fn setup_pool() -> SqlitePool {
    let pool = do_create_pool("sqlite::memory:")
        .await
        .expect("Failed to create in-memory pool");

    let migrator = Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
        .await
        .expect("Failed to create migrator");
    migrator.run(&pool).await.expect("Failed to run migrations");
    pool
}

/// Returns a fixed test timestamp.
fn t0() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2025-06-15T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}

/// Returns a different test timestamp.
fn t1() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2025-06-16T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}

/// Returns yet another test timestamp.
fn t2() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2025-06-17T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}

/// Builds 1v1 participants where player_0 beats player_1 (placement 1 = winner).
fn make_1v1(player_a: &str, player_b: &str) -> Vec<MatchParticipant> {
    vec![
        MatchParticipant {
            player_id: player_a.to_string(),
            placement: 1,
        },
        MatchParticipant {
            player_id: player_b.to_string(),
            placement: 2,
        },
    ]
}

/// Builds participants with explicit positional placements (1-indexed, lower is better).
fn make_participants(pairs: &[(usize, &str)]) -> Vec<MatchParticipant> {
    pairs
        .iter()
        .map(|(placement, pid)| MatchParticipant {
            player_id: pid.to_string(),
            placement: *placement as i32,
        })
        .collect()
}

/// Queries the latest rating snapshot for a player in a season.
async fn get_latest_snapshot(
    pool: &SqlitePool,
    player_id: &str,
    season_id: &str,
) -> (f64, f64, Option<f64>) {
    let row = sqlx::query(
        "SELECT rating_json, conservative_rating FROM rating_snapshots \
         WHERE player_id = ? AND season_id = ? \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(player_id)
    .bind(season_id)
    .fetch_one(pool)
    .await
    .expect("Should find a snapshot");

    let rating_json: String = row.get("rating_json");
    let rj: serde_json::Value = serde_json::from_str(&rating_json).unwrap();
    let rating_value: f64 = rj["rating_value"].as_f64().unwrap_or(f64::NAN);
    let uncertainty: Option<f64> = rj.get("uncertainty").and_then(|v| v.as_f64());
    let conservative: f64 = row.get("conservative_rating");
    (rating_value, conservative, uncertainty)
}

/// Queries the leaderboard for a season: (player_id, rating_value, conservative_rating)
/// ordered by conservative_rating DESC.
async fn get_leaderboard(pool: &SqlitePool, season_id: &str) -> Vec<(String, f64, f64)> {
    let rows = sqlx::query(
        "SELECT rs.player_id, rs.rating_json, rs.conservative_rating \
         FROM rating_snapshots rs \
         INNER JOIN ( \
           SELECT player_id, MAX(created_at) AS max_ts \
           FROM rating_snapshots \
           WHERE season_id = ? \
           GROUP BY player_id \
         ) latest ON rs.player_id = latest.player_id AND rs.created_at = latest.max_ts \
         WHERE rs.season_id = ? \
         ORDER BY rs.conservative_rating DESC",
    )
    .bind(season_id)
    .bind(season_id)
    .fetch_all(pool)
    .await
    .expect("Should query leaderboard");

    rows.iter()
        .map(|row| {
            let pid: String = row.get("player_id");
            let rating_json: String = row.get("rating_json");
            let rj: serde_json::Value = serde_json::from_str(&rating_json).unwrap();
            let rating_value: f64 = rj["rating_value"].as_f64().unwrap_or(f64::NAN);
            let conservative: f64 = row.get("conservative_rating");
            (pid, rating_value, conservative)
        })
        .collect()
}

// ============================================================================
// Test 1: Full flow — league → season → players → match → snapshots → leaderboard
// ============================================================================

#[tokio::test]
async fn test_full_flow_league_to_leaderboard() {
    let pool = setup_pool().await;

    // 1. Create a league
    let league = LeagueRepository::create_league(
        &pool,
        "Test League",
        "A test league for integration",
        "glicko2",
        "public",
        "test_user",
    )
    .await
    .expect("Should create league");
    assert!(!league.id.is_empty());
    assert_eq!(league.name, "Test League");
    assert!(league.is_active);

    // 2. Create a season
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: Some(350.0),
        extra: None,
    };
    let season = SeasonRepository::create_season(
        &pool,
        &league.id,
        "glicko2",
        &params,
        SeedingChoice::Reset,
    )
    .await
    .expect("Should create season");
    assert!(!season.id.is_empty());
    assert_eq!(season.league_id, league.id);
    assert!(season.is_open);
    assert_eq!(season.number, 1);

    // 3. Create players
    let alice = PlayerRepository::create_player(&pool, "Alice", "human")
        .await
        .expect("Should create Alice");
    let bob = PlayerRepository::create_player(&pool, "Bob", "human")
        .await
        .expect("Should create Bob");

    // 4. Add players to league
    PlayerRepository::add_to_league(&pool, &league.id, &alice.id)
        .await
        .expect("Should add Alice to league");
    PlayerRepository::add_to_league(&pool, &league.id, &bob.id)
        .await
        .expect("Should add Bob to league");

    // 5. Record a match — Alice beats Bob
    let participants = make_1v1(&alice.id, &bob.id);
    let result = MatchRepository::record_match(&pool, &season.id, participants, None, t0())
        .await
        .expect("Should record match");

    assert!(!result.match_id.is_empty());
    assert_eq!(result.snapshots.len(), 2);

    // 6. Verify rating snapshots exist for both players
    let alice_snap = get_latest_snapshot(&pool, &alice.id, &season.id).await;
    let bob_snap = get_latest_snapshot(&pool, &bob.id, &season.id).await;

    // Alice won, so her rating should be higher than Bob's
    assert!(
        alice_snap.0 > bob_snap.0,
        "Winner's rating ({}) should exceed loser's ({})",
        alice_snap.0,
        bob_snap.0
    );

    // 7. Get leaderboard — verify players are ordered by conservative rating
    let lb = get_leaderboard(&pool, &season.id).await;
    assert_eq!(lb.len(), 2, "Leaderboard should have 2 players");
    assert_eq!(
        lb[0].0, alice.id,
        "Alice (winner) should be first on leaderboard"
    );
    assert_eq!(
        lb[1].0, bob.id,
        "Bob (loser) should be second on leaderboard"
    );
    assert!(
        lb[0].2 > lb[1].2,
        "Leaderboard should be ordered by conservative_rating DESC"
    );
}

// ============================================================================
// Test 2: Multiple matches produce progressive rating changes
// ============================================================================

#[tokio::test]
async fn test_multiple_matches_progressive_ratings() {
    let pool = setup_pool().await;

    // Setup
    let league =
        LeagueRepository::create_league(&pool, "Prog League", "", "elo", "public", "test_user")
            .await
            .unwrap();
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: None,
        extra: None,
    };
    let season =
        SeasonRepository::create_season(&pool, &league.id, "elo", &params, SeedingChoice::Reset)
            .await
            .unwrap();

    let a = PlayerRepository::create_player(&pool, "Alpha", "human")
        .await
        .unwrap();
    let b = PlayerRepository::create_player(&pool, "Bravo", "human")
        .await
        .unwrap();
    let c = PlayerRepository::create_player(&pool, "Charlie", "human")
        .await
        .unwrap();

    for pid in &[&a.id, &b.id, &c.id] {
        PlayerRepository::add_to_league(&pool, &league.id, pid)
            .await
            .unwrap();
    }

    // Match 1: Alpha beats Bravo
    MatchRepository::record_match(&pool, &season.id, make_1v1(&a.id, &b.id), None, t0())
        .await
        .unwrap();

    let _a_after_m1 = get_latest_snapshot(&pool, &a.id, &season.id).await;
    let b_after_m1 = get_latest_snapshot(&pool, &b.id, &season.id).await;

    // Match 2: Bravo beats Charlie
    MatchRepository::record_match(&pool, &season.id, make_1v1(&b.id, &c.id), None, t1())
        .await
        .unwrap();

    let b_after_m2 = get_latest_snapshot(&pool, &b.id, &season.id).await;
    let _c_after_m2 = get_latest_snapshot(&pool, &c.id, &season.id).await;

    // Match 3: Alpha beats Charlie
    MatchRepository::record_match(&pool, &season.id, make_1v1(&a.id, &c.id), None, t2())
        .await
        .unwrap();

    let a_final = get_latest_snapshot(&pool, &a.id, &season.id).await;
    let b_final = get_latest_snapshot(&pool, &b.id, &season.id).await;
    let c_final = get_latest_snapshot(&pool, &c.id, &season.id).await;

    // Bravo's rating should have changed between matches
    assert!(
        (b_after_m2.0 - b_after_m1.0).abs() > 0.01,
        "Bravo's rating should change after playing match 2"
    );

    // Final leaderboard: Alpha should be highest (2 wins), Charlie lowest (2 losses)
    let lb = get_leaderboard(&pool, &season.id).await;
    assert_eq!(lb.len(), 3);
    assert_eq!(lb[0].0, a.id, "Alpha should lead the leaderboard");
    assert_eq!(lb[2].0, c.id, "Charlie should be last on leaderboard");
    assert!(a_final.0 > b_final.0, "Alpha should outrank Bravo");
    assert!(b_final.0 > c_final.0, "Bravo should outrank Charlie");
}

// ============================================================================
// Test 3: Match correction flow (correct_match → job created → match state)
// ============================================================================

#[tokio::test]
async fn test_match_correction_creates_job_and_marks_corrected() {
    let pool = setup_pool().await;

    // Setup
    let league = LeagueRepository::create_league(
        &pool,
        "Correction League",
        "",
        "elo",
        "public",
        "test_user",
    )
    .await
    .unwrap();
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: None,
        extra: None,
    };
    let season =
        SeasonRepository::create_season(&pool, &league.id, "elo", &params, SeedingChoice::Reset)
            .await
            .unwrap();

    let x = PlayerRepository::create_player(&pool, "Xavier", "human")
        .await
        .unwrap();
    let y = PlayerRepository::create_player(&pool, "Yara", "human")
        .await
        .unwrap();
    PlayerRepository::add_to_league(&pool, &league.id, &x.id)
        .await
        .unwrap();
    PlayerRepository::add_to_league(&pool, &league.id, &y.id)
        .await
        .unwrap();

    // Record initial match: Xavier beats Yara
    let match_result =
        MatchRepository::record_match(&pool, &season.id, make_1v1(&x.id, &y.id), None, t0())
            .await
            .unwrap();

    // Correct the match: swap the winner (Yara should have won)
    let correction = MatchCorrection {
        new_participants: vec![
            MatchParticipant {
                player_id: y.id.clone(),
                placement: 1, // Yara in 1st
            },
            MatchParticipant {
                player_id: x.id.clone(),
                placement: 2, // Xavier in 2nd
            },
        ],
        reason: "Incorrect placement — Yara actually won".into(),
        score_metadata: None,
    };

    let job_id =
        MatchRepository::correct_match(&pool, &match_result.match_id, &correction, "admin_user")
            .await
            .expect("Should correct match");

    assert!(
        !job_id.is_empty(),
        "Correcting a match should return a job ID"
    );

    // Verify the job exists
    let job = JobRepository::get_job(&pool, &job_id)
        .await
        .expect("Should fetch job")
        .expect("Job should exist");

    assert_eq!(job.season_id, season.id);
    assert!(matches!(
        job.status,
        ladder_rs_persistence::JobStatus::Queued
    ));

    // Verify the match is marked as corrected
    let m = MatchRepository::get_by_id(&pool, &match_result.match_id)
        .await
        .expect("Should fetch match")
        .expect("Match should exist");
    assert!(m.is_corrected, "Match should be marked as corrected");
}

// ============================================================================
// Test 4: Batch match entry
// ============================================================================

#[tokio::test]
async fn test_batch_match_entry_records_multiple_matches() {
    let pool = setup_pool().await;

    // Setup
    let league =
        LeagueRepository::create_league(&pool, "Batch League", "", "elo", "public", "test_user")
            .await
            .unwrap();
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: None,
        extra: None,
    };
    let season =
        SeasonRepository::create_season(&pool, &league.id, "elo", &params, SeedingChoice::Reset)
            .await
            .unwrap();

    let p1 = PlayerRepository::create_player(&pool, "P1", "human")
        .await
        .unwrap();
    let p2 = PlayerRepository::create_player(&pool, "P2", "human")
        .await
        .unwrap();
    let p3 = PlayerRepository::create_player(&pool, "P3", "human")
        .await
        .unwrap();
    let p4 = PlayerRepository::create_player(&pool, "P4", "human")
        .await
        .unwrap();

    for pid in &[&p1.id, &p2.id, &p3.id, &p4.id] {
        PlayerRepository::add_to_league(&pool, &league.id, pid)
            .await
            .unwrap();
    }

    let entries = vec![
        BatchEntry {
            participants: make_1v1(&p1.id, &p2.id),
            score_metadata: Some(serde_json::json!({"game": "round 1"})),
            recorded_at: t0(),
        },
        BatchEntry {
            participants: make_1v1(&p3.id, &p4.id),
            score_metadata: Some(serde_json::json!({"game": "round 1"})),
            recorded_at: t1(),
        },
        BatchEntry {
            participants: make_1v1(&p1.id, &p3.id),
            score_metadata: None,
            recorded_at: t2(),
        },
    ];

    let results = MatchRepository::record_match_batch(&pool, &season.id, entries)
        .await
        .expect("Should record batch of matches");

    assert_eq!(results.len(), 3, "Should have 3 batch results");
    for r in &results {
        assert!(!r.match_id.is_empty());
        assert_eq!(r.snapshots.len(), 2, "Each 1v1 match has 2 snapshots");
    }

    // Verify matches are in the database
    let filter = MatchFilter {
        limit: Some(10),
        offset: None,
        player_id: None,
    };
    let matches = MatchRepository::list_matches(&pool, &season.id, &filter)
        .await
        .expect("Should list matches");
    assert_eq!(matches.len(), 3);

    // Verify all 4 players have snapshots
    for pid in &[&p1.id, &p2.id, &p3.id, &p4.id] {
        let snap = get_latest_snapshot(&pool, pid, &season.id).await;
        assert!(snap.0.is_finite(), "{} should have a rating", pid);
    }
}

// ============================================================================
// Test 5: Season close prevents new matches
// ============================================================================

#[tokio::test]
async fn test_season_close_prevents_new_matches() {
    let pool = setup_pool().await;

    // Setup
    let league =
        LeagueRepository::create_league(&pool, "Closed League", "", "elo", "public", "test_user")
            .await
            .unwrap();
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: None,
        extra: None,
    };
    let season =
        SeasonRepository::create_season(&pool, &league.id, "elo", &params, SeedingChoice::Reset)
            .await
            .unwrap();

    let p1 = PlayerRepository::create_player(&pool, "Player1", "human")
        .await
        .unwrap();
    let p2 = PlayerRepository::create_player(&pool, "Player2", "human")
        .await
        .unwrap();
    PlayerRepository::add_to_league(&pool, &league.id, &p1.id)
        .await
        .unwrap();
    PlayerRepository::add_to_league(&pool, &league.id, &p2.id)
        .await
        .unwrap();

    // Record a match while season is open — should succeed
    MatchRepository::record_match(&pool, &season.id, make_1v1(&p1.id, &p2.id), None, t0())
        .await
        .expect("Should record match in open season");

    // Close the season
    SeasonRepository::close_season(&pool, &season.id)
        .await
        .expect("Should close season");

    // Verify season is closed
    let is_closed = MatchRepository::is_season_closed(&pool, &season.id)
        .await
        .expect("Should check season status");
    assert!(is_closed, "Season should be closed");

    // Try to record another match — should fail
    let result =
        MatchRepository::record_match(&pool, &season.id, make_1v1(&p2.id, &p1.id), None, t1())
            .await;

    match result {
        Err(PersistenceError::Conflict(msg)) => {
            assert!(
                msg.contains("closed"),
                "Error should mention season is closed: {}",
                msg
            );
        }
        other => panic!("Expected Conflict error, got: {:?}", other),
    }
}

// ============================================================================
// Test 6: Archived leagues still show historical data
// ============================================================================

#[tokio::test]
async fn test_archived_league_retains_historical_data() {
    let pool = setup_pool().await;

    // Setup: create league with matches
    let league =
        LeagueRepository::create_league(&pool, "Archive Me", "", "elo", "public", "test_user")
            .await
            .unwrap();
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: None,
        extra: None,
    };
    let season =
        SeasonRepository::create_season(&pool, &league.id, "elo", &params, SeedingChoice::Reset)
            .await
            .unwrap();

    let a = PlayerRepository::create_player(&pool, "ArchAlice", "human")
        .await
        .unwrap();
    let b = PlayerRepository::create_player(&pool, "ArchBob", "human")
        .await
        .unwrap();
    PlayerRepository::add_to_league(&pool, &league.id, &a.id)
        .await
        .unwrap();
    PlayerRepository::add_to_league(&pool, &league.id, &b.id)
        .await
        .unwrap();

    let result =
        MatchRepository::record_match(&pool, &season.id, make_1v1(&a.id, &b.id), None, t0())
            .await
            .unwrap();

    // Archive the league
    LeagueRepository::archive_league(&pool, &league.id)
        .await
        .expect("Should archive league");

    // Verify league is archived
    let archived = LeagueRepository::get_league(&pool, &league.id)
        .await
        .expect("Should fetch league")
        .expect("League should exist");
    assert!(archived.is_archived, "League should be archived");

    // Historical data should still be accessible:
    // - Match still exists
    let m = MatchRepository::get_by_id(&pool, &result.match_id)
        .await
        .expect("Match fetch should succeed even after archive");
    assert!(m.is_some(), "Match should still exist after archive");

    // - Snapshots still exist
    let lb = get_leaderboard(&pool, &season.id).await;
    assert_eq!(lb.len(), 2, "Leaderboard data should still be queryable");

    // - League listing with is_archived filter
    let archived_leagues = LeagueRepository::list_leagues(
        &pool,
        &LeagueFilter {
            is_active: None,
            is_archived: Some(true),
            limit: Some(10),
            offset: Some(0),
        },
    )
    .await
    .unwrap();
    assert!(
        archived_leagues.iter().any(|l| l.id == league.id),
        "Archived league should appear in archived list"
    );
}

// ============================================================================
// Test 7: Leaderboard ordering by conservative rating
// ============================================================================

#[tokio::test]
async fn test_leaderboard_ordering_by_conservative_rating() {
    let pool = setup_pool().await;

    // Setup with glicko2 (which has uncertainty, so conservative_rating != rating_value)
    let league = LeagueRepository::create_league(
        &pool,
        "Glicko Leaderboard",
        "",
        "glicko2",
        "public",
        "test_user",
    )
    .await
    .unwrap();
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: Some(350.0),
        extra: None,
    };
    let season = SeasonRepository::create_season(
        &pool,
        &league.id,
        "glicko2",
        &params,
        SeedingChoice::Reset,
    )
    .await
    .unwrap();

    let w = PlayerRepository::create_player(&pool, "Winner", "human")
        .await
        .unwrap();
    let l = PlayerRepository::create_player(&pool, "Loser", "human")
        .await
        .unwrap();

    PlayerRepository::add_to_league(&pool, &league.id, &w.id)
        .await
        .unwrap();
    PlayerRepository::add_to_league(&pool, &league.id, &l.id)
        .await
        .unwrap();

    // Record a match: Winner beats Loser
    MatchRepository::record_match(&pool, &season.id, make_1v1(&w.id, &l.id), None, t0())
        .await
        .unwrap();

    let w_snap = get_latest_snapshot(&pool, &w.id, &season.id).await;
    let l_snap = get_latest_snapshot(&pool, &l.id, &season.id).await;

    // With glicko2, uncertainty should be present
    assert!(
        w_snap.2.is_some(),
        "Glicko2 snapshots should have uncertainty"
    );
    assert!(
        l_snap.2.is_some(),
        "Glicko2 snapshots should have uncertainty"
    );

    // Conservative rating = rating - 2 * RD (for glicko2)
    // Winner should have higher conservative rating
    assert!(
        w_snap.1 > l_snap.1,
        "Winner's conservative rating ({}) should exceed loser's ({})",
        w_snap.1,
        l_snap.1
    );

    // Leaderboard should be ordered by conservative_rating DESC
    let lb = get_leaderboard(&pool, &season.id).await;
    assert_eq!(lb.len(), 2);
    assert_eq!(lb[0].0, w.id);
    assert_eq!(lb[1].0, l.id);
}

// ============================================================================
// Test 8: Input validation — empty inputs, duplicate players
// ============================================================================

#[tokio::test]
async fn test_record_match_rejects_empty_season_id() {
    let pool = setup_pool().await;
    let result = MatchRepository::record_match(&pool, "", make_1v1("a", "b"), None, t0()).await;
    assert!(matches!(result, Err(PersistenceError::InvalidInput(_))));
}

#[tokio::test]
async fn test_record_match_rejects_empty_participants() {
    let pool = setup_pool().await;
    let result = MatchRepository::record_match(&pool, "some-season", vec![], None, t0()).await;
    assert!(matches!(result, Err(PersistenceError::InvalidInput(_))));
}

#[tokio::test]
async fn test_record_match_rejects_duplicate_players() {
    let pool = setup_pool().await;
    let participants = vec![
        MatchParticipant {
            player_id: "dup-player".into(),
            placement: 1,
        },
        MatchParticipant {
            player_id: "dup-player".into(),
            placement: 2,
        },
    ];
    let result =
        MatchRepository::record_match(&pool, "some-season", participants, None, t0()).await;
    assert!(matches!(
        result,
        Err(PersistenceError::InvalidInput(ref msg))
        if msg.contains("Duplicate")
    ));
}

#[tokio::test]
async fn test_correct_match_rejects_empty_match_id() {
    let pool = setup_pool().await;
    let correction = MatchCorrection {
        new_participants: vec![MatchParticipant {
            player_id: "x".into(),
            placement: 1,
        }],
        reason: "test".into(),
        score_metadata: None,
    };
    let result = MatchRepository::correct_match(&pool, "", &correction, "admin").await;
    assert!(matches!(result, Err(PersistenceError::InvalidInput(_))));
}

#[tokio::test]
async fn test_correct_match_rejects_empty_reason() {
    let pool = setup_pool().await;
    let correction = MatchCorrection {
        new_participants: vec![MatchParticipant {
            player_id: "x".into(),
            placement: 1,
        }],
        reason: "".into(),
        score_metadata: None,
    };
    let result = MatchRepository::correct_match(&pool, "some-match", &correction, "admin").await;
    assert!(matches!(result, Err(PersistenceError::InvalidInput(_))));
}

// ============================================================================
// Test 9: Duplicate match detection
// ============================================================================

#[tokio::test]
async fn test_record_match_rejects_duplicate() {
    let pool = setup_pool().await;

    let league =
        LeagueRepository::create_league(&pool, "Dup League", "", "elo", "public", "test_user")
            .await
            .unwrap();
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: None,
        extra: None,
    };
    let season =
        SeasonRepository::create_season(&pool, &league.id, "elo", &params, SeedingChoice::Reset)
            .await
            .unwrap();

    let a = PlayerRepository::create_player(&pool, "DupA", "human")
        .await
        .unwrap();
    let b = PlayerRepository::create_player(&pool, "DupB", "human")
        .await
        .unwrap();
    PlayerRepository::add_to_league(&pool, &league.id, &a.id)
        .await
        .unwrap();
    PlayerRepository::add_to_league(&pool, &league.id, &b.id)
        .await
        .unwrap();

    let participants = make_1v1(&a.id, &b.id);
    let ts = t0();

    // First record — should succeed
    MatchRepository::record_match(&pool, &season.id, participants.clone(), None, ts)
        .await
        .expect("First match should succeed");

    // Second record with same participants and timestamp — should be rejected as duplicate
    let result =
        MatchRepository::record_match(&pool, &season.id, participants.clone(), None, ts).await;
    match result {
        Err(PersistenceError::Conflict(msg)) => {
            assert!(
                msg.contains("Duplicate"),
                "Error should mention duplicate: {}",
                msg
            );
        }
        other => panic!("Expected Conflict (duplicate), got: {:?}", other),
    }

    // Also test is_duplicate directly
    let is_dup = MatchRepository::is_duplicate(&pool, &season.id, &participants, &ts)
        .await
        .expect("Should check duplicate");
    assert!(
        is_dup,
        "is_duplicate should return true for duplicate match"
    );
}

// ============================================================================
// Test 10: Matches with >2 participants (multi-player free-for-all)
// ============================================================================

#[tokio::test]
async fn test_multi_player_match_records_all_snapshots() {
    let pool = setup_pool().await;

    let league =
        LeagueRepository::create_league(&pool, "FFA League", "", "elo", "public", "test_user")
            .await
            .unwrap();
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: None,
        extra: None,
    };
    let season =
        SeasonRepository::create_season(&pool, &league.id, "elo", &params, SeedingChoice::Reset)
            .await
            .unwrap();

    let players: Vec<_> = (0..4)
        .map(|i| {
            let name = format!("FFA{}", i);
            tokio::task::LocalSet::new();
            name
        })
        .collect();

    let mut player_ids = Vec::new();
    for name in &players {
        let p = PlayerRepository::create_player(&pool, name, "human")
            .await
            .unwrap();
        PlayerRepository::add_to_league(&pool, &league.id, &p.id)
            .await
            .unwrap();
        player_ids.push(p.id);
    }

    let participants = make_participants(&[
        (1, &player_ids[0]),
        (2, &player_ids[1]),
        (3, &player_ids[2]),
        (4, &player_ids[3]),
    ]);

    let result = MatchRepository::record_match(&pool, &season.id, participants, None, t0())
        .await
        .expect("Should record 4-player match");

    assert_eq!(
        result.snapshots.len(),
        4,
        "Should create 4 snapshots for 4-player match"
    );

    // All 4 players should have snapshot entries
    let lb = get_leaderboard(&pool, &season.id).await;
    assert_eq!(lb.len(), 4);
    // Player 0 placed 1st → should be at top
    assert_eq!(
        lb[0].0, player_ids[0],
        "1st place should top the leaderboard"
    );
}

// ============================================================================
// Test 11: Match listing with player_id filter
// ============================================================================

#[tokio::test]
async fn test_list_matches_filters_by_player() {
    let pool = setup_pool().await;

    // Setup
    let league =
        LeagueRepository::create_league(&pool, "Filter League", "", "elo", "public", "test_user")
            .await
            .unwrap();
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: None,
        extra: None,
    };
    let season =
        SeasonRepository::create_season(&pool, &league.id, "elo", &params, SeedingChoice::Reset)
            .await
            .unwrap();

    let a = PlayerRepository::create_player(&pool, "FilterA", "human")
        .await
        .unwrap();
    let b = PlayerRepository::create_player(&pool, "FilterB", "human")
        .await
        .unwrap();
    let c = PlayerRepository::create_player(&pool, "FilterC", "human")
        .await
        .unwrap();

    for pid in &[&a.id, &b.id, &c.id] {
        PlayerRepository::add_to_league(&pool, &league.id, pid)
            .await
            .unwrap();
    }

    // A vs B
    MatchRepository::record_match(&pool, &season.id, make_1v1(&a.id, &b.id), None, t0())
        .await
        .unwrap();
    // B vs C
    MatchRepository::record_match(&pool, &season.id, make_1v1(&b.id, &c.id), None, t1())
        .await
        .unwrap();
    // A vs C
    MatchRepository::record_match(&pool, &season.id, make_1v1(&a.id, &c.id), None, t2())
        .await
        .unwrap();

    // All matches
    let all = MatchRepository::list_matches(
        &pool,
        &season.id,
        &MatchFilter {
            limit: Some(10),
            offset: None,
            player_id: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(all.len(), 3);

    // Filter by player A — should appear in 2 matches
    let a_matches = MatchRepository::list_matches(
        &pool,
        &season.id,
        &MatchFilter {
            limit: Some(10),
            offset: None,
            player_id: Some(a.id.clone()),
        },
    )
    .await
    .unwrap();
    assert_eq!(a_matches.len(), 2, "A played in 2 matches");

    // Filter by player C — should appear in 2 matches
    let c_matches = MatchRepository::list_matches(
        &pool,
        &season.id,
        &MatchFilter {
            limit: Some(10),
            offset: None,
            player_id: Some(c.id.clone()),
        },
    )
    .await
    .unwrap();
    assert_eq!(c_matches.len(), 2, "C played in 2 matches");
}

// ============================================================================
// Test 12: Players in league listing works correctly
// ============================================================================

#[tokio::test]
async fn test_players_list_in_league() {
    let pool = setup_pool().await;

    let league =
        LeagueRepository::create_league(&pool, "ListLeague", "", "elo", "public", "test_user")
            .await
            .unwrap();

    let a = PlayerRepository::create_player(&pool, "LstAlpha", "human")
        .await
        .unwrap();
    let b = PlayerRepository::create_player(&pool, "LstBeta", "human")
        .await
        .unwrap();

    PlayerRepository::add_to_league(&pool, &league.id, &a.id)
        .await
        .unwrap();
    PlayerRepository::add_to_league(&pool, &league.id, &b.id)
        .await
        .unwrap();

    let players = PlayerRepository::list_players(&pool, &league.id, &PlayerFilter::default())
        .await
        .unwrap();

    assert_eq!(players.len(), 2);
    assert!(players.iter().any(|p| p.id == a.id));
    assert!(players.iter().any(|p| p.id == b.id));

    // All players should have league_id set in context
    for p in &players {
        assert_eq!(
            p.league_id.as_deref(),
            Some(league.id.as_str()),
            "Players listed in league context should have league_id set"
        );
    }
}
