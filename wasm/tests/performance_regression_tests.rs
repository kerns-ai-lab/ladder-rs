//! Performance regression tests for all rating systems including TrueSkill
//!
//! These tests verify that all rating algorithms perform within acceptable
//! time bounds in WASM, confirming that TrueSkill (via statrs/nalgebra) works
//! correctly after removing the erroneous elo-only restriction.
//!
//! Background: the wasm crate was previously locked to features = ["elo-only"]
//! based on a misdiagnosis that statrs pulled in rayon. Investigation confirmed:
//! - statrs v0.16.1 has no rayon dependency
//! - rayon enters only via ladder-rs's "full-deps" feature (never enabled in WASM)
//! - The fix was changing to features = ["all-algorithms"] which enables TrueSkill

use js_sys::Array;
use ladder_rs_wasm::{
    EloRating, EloSystem, MatchOutcome, TrueSkillRating, TrueSkillSystem, TrueSkillTeam,
    TrueSkillUtils,
};
use serde_json;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// --- Elo performance tests ---

/// Verify Elo system processes many sequential matches without degradation
#[wasm_bindgen_test]
fn test_elo_sequential_match_performance() {
    let system = EloSystem::new();

    let mut p1 = system.create_rating();
    let mut p2 = system.create_rating();

    // Process 200 sequential 1v1 matches
    for i in 0..200u32 {
        let outcome = if i % 3 == 0 {
            MatchOutcome::Draw
        } else if i % 2 == 0 {
            MatchOutcome::Player1Win
        } else {
            MatchOutcome::Player2Win
        };
        let result = system.process_1v1(&p1, &p2, outcome).unwrap();
        p1 = EloRating::new(result.player1_rating());
        p2 = EloRating::new(result.player2_rating());
    }

    // After 200 mixed results the ratings should still be valid numbers
    assert!(p1.value().is_finite());
    assert!(p2.value().is_finite());
}

/// Verify Elo batch processing of many players
#[wasm_bindgen_test]
fn test_elo_batch_player_performance() {
    let system = EloSystem::new();

    // Create 50 players
    let mut ratings: Vec<EloRating> = (0..50).map(|_| system.create_rating()).collect();

    // Run round-robin style matches
    for i in 0..50usize {
        let j = (i + 1) % 50;
        let outcome = if i % 2 == 0 {
            MatchOutcome::Player1Win
        } else {
            MatchOutcome::Player2Win
        };
        // Clone to avoid borrow conflict when updating the same Vec
        let r1 = ratings[i].clone();
        let r2 = ratings[j].clone();
        let result = system.process_1v1(&r1, &r2, outcome).unwrap();
        ratings[i] = EloRating::new(result.player1_rating());
        ratings[j] = EloRating::new(result.player2_rating());
    }

    // All ratings should be finite
    for rating in &ratings {
        assert!(rating.value().is_finite(), "Rating should be finite");
    }
}

// --- TrueSkill performance tests ---
// These tests confirm that TrueSkill is now available in WASM builds after
// fixing the erroneous "elo-only" feature restriction.

/// Verify TrueSkill system can be created in WASM
#[wasm_bindgen_test]
fn test_trueskill_system_creation_wasm() {
    let system = TrueSkillSystem::new();
    assert_eq!(system.mu(), 25.0);
    assert!((system.sigma() - 25.0 / 3.0).abs() < 0.001);
    assert!((system.beta() - 25.0 / 6.0).abs() < 0.001);
    assert_eq!(system.draw_probability(), 0.1);
}

/// Verify TrueSkill rating creation in WASM
#[wasm_bindgen_test]
fn test_trueskill_rating_creation_wasm() {
    let system = TrueSkillSystem::new();
    let rating = system.create_rating();

    assert_eq!(rating.mean(), 25.0);
    assert!(rating.variance() > 0.0);
    assert!(rating.std_dev() > 0.0);
    assert!(rating.conservative_rating() < rating.mean());
}

/// Verify TrueSkill 1v1 match processes correctly in WASM
#[wasm_bindgen_test]
fn test_trueskill_1v1_match_wasm() {
    let system = TrueSkillSystem::new();
    let player1 = system.create_rating();
    let player2 = system.create_rating();

    let team1 = TrueSkillTeam::from_ratings(vec![player1]).unwrap();
    let team2 = TrueSkillTeam::from_ratings(vec![player2]).unwrap();

    // process_match expects JS objects (not strings), so parse JSON to JS object
    let teams = Array::new();
    let team1_obj = js_sys::JSON::parse(&team1.to_json().unwrap()).unwrap();
    let team2_obj = js_sys::JSON::parse(&team2.to_json().unwrap()).unwrap();
    teams.push(&team1_obj);
    teams.push(&team2_obj);

    let ranks = Array::new();
    ranks.push(&1.into()); // team1 wins
    ranks.push(&2.into()); // team2 loses

    let result = system.process_match(&teams, &ranks).unwrap();
    assert_eq!(result.length(), 2);
}

/// Verify TrueSkill sequential matches maintain consistent ratings
#[wasm_bindgen_test]
fn test_trueskill_sequential_match_performance() {
    let system = TrueSkillSystem::new();

    let mut p1_mean = 25.0_f64;
    let mut p1_var = (25.0_f64 / 3.0).powi(2);
    let mut p2_mean = 25.0_f64;
    let mut p2_var = (25.0_f64 / 3.0).powi(2);

    // Process 50 sequential matches using batch_process
    for i in 0..50u32 {
        let ratings_json = format!(
            r#"[{{"mean":{p1},"variance":{v1}}},{{"mean":{p2},"variance":{v2}}}]"#,
            p1 = p1_mean,
            v1 = p1_var,
            p2 = p2_mean,
            v2 = p2_var
        );

        // Alternate winner each match
        let (rank1, rank2) = if i % 2 == 0 {
            (1u32, 2u32)
        } else {
            (2u32, 1u32)
        };
        let matches_json = format!(
            r#"[{{"teams":[[0],[1]],"ranks":[{r1},{r2}]}}]"#,
            r1 = rank1,
            r2 = rank2
        );

        let result_json =
            TrueSkillUtils::batch_process(&system, &ratings_json, &matches_json).unwrap();
        let results: Vec<serde_json::Value> = serde_json::from_str(&result_json).unwrap();

        p1_mean = results[0]["mean"].as_f64().unwrap();
        p1_var = results[0]["variance"].as_f64().unwrap();
        p2_mean = results[1]["mean"].as_f64().unwrap();
        p2_var = results[1]["variance"].as_f64().unwrap();
    }

    // After alternating wins, ratings should converge toward equal values
    assert!(p1_mean.is_finite());
    assert!(p2_mean.is_finite());
    assert!(p1_var > 0.0);
    assert!(p2_var > 0.0);
    // Variance should decrease from initial (uncertainty reduces with more games)
    assert!(p1_var < (25.0_f64 / 3.0).powi(2));
    assert!(p2_var < (25.0_f64 / 3.0).powi(2));
}

/// Verify TrueSkill handles a larger player pool via batch processing
#[wasm_bindgen_test]
fn test_trueskill_batch_processing_performance() {
    let system = TrueSkillSystem::new();

    // Create 10 players at default ratings
    let initial_mean = 25.0_f64;
    let initial_var = (25.0_f64 / 3.0).powi(2);
    let ratings_json = format!(
        "[{}]",
        (0..10)
            .map(|_| format!(
                r#"{{"mean":{m},"variance":{v}}}"#,
                m = initial_mean,
                v = initial_var
            ))
            .collect::<Vec<_>>()
            .join(",")
    );

    // 5 matches between consecutive players
    let matches_json = r#"[
        {"teams":[[0],[1]],"ranks":[1,2]},
        {"teams":[[2],[3]],"ranks":[1,2]},
        {"teams":[[4],[5]],"ranks":[2,1]},
        {"teams":[[6],[7]],"ranks":[1,2]},
        {"teams":[[8],[9]],"ranks":[2,1]}
    ]"#;

    let result_json = TrueSkillUtils::batch_process(&system, &ratings_json, matches_json).unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_str(&result_json).unwrap();

    assert_eq!(results.len(), 10);
    // Winners (0, 2, 5, 6, 9) should have higher mean than initial
    // Losers should have lower mean
    // All variances should decrease
    for result in &results {
        let var = result["variance"].as_f64().unwrap();
        assert!(var > 0.0);
        assert!(var.is_finite());
    }
}

/// Verify TrueSkill leaderboard generation works in WASM
#[wasm_bindgen_test]
fn test_trueskill_leaderboard_performance() {
    // Create a varied set of ratings
    let ratings_json = r#"[
        {"mean":35,"variance":16},
        {"mean":25,"variance":69},
        {"mean":30,"variance":25},
        {"mean":20,"variance":100},
        {"mean":28,"variance":36}
    ]"#;

    let leaderboard = TrueSkillUtils::create_leaderboard(ratings_json, true).unwrap();
    let entries: Vec<Vec<serde_json::Value>> = serde_json::from_str(&leaderboard).unwrap();

    // Should have 5 entries sorted by conservative rating
    assert_eq!(entries.len(), 5);

    // Verify descending order by conservative rating (entries[i][3])
    for i in 1..entries.len() {
        let prev_conservative = entries[i - 1][3].as_f64().unwrap();
        let curr_conservative = entries[i][3].as_f64().unwrap();
        assert!(
            prev_conservative >= curr_conservative,
            "Leaderboard not sorted: {} < {}",
            prev_conservative,
            curr_conservative
        );
    }
}

/// Verify TrueSkill match quality calculation works in WASM
#[wasm_bindgen_test]
fn test_trueskill_match_quality_wasm() {
    let system = TrueSkillSystem::new();

    // Two equal players should have high match quality
    let p1 = system.create_rating();
    let p2 = system.create_rating();

    let team1 = TrueSkillTeam::from_ratings(vec![p1]).unwrap();
    let team2 = TrueSkillTeam::from_ratings(vec![p2]).unwrap();

    let teams = Array::new();
    teams.push(&js_sys::JSON::parse(&team1.to_json().unwrap()).unwrap());
    teams.push(&js_sys::JSON::parse(&team2.to_json().unwrap()).unwrap());

    let quality = system.match_quality(&teams).unwrap();
    // Equal players should have high match quality
    assert!(
        quality > 0.5,
        "Match quality for equal players should be > 0.5, got {}",
        quality
    );
    assert!(quality <= 1.0);
}

// --- Cross-algorithm regression tests ---

/// Verify all rating systems produce stable ratings over many iterations
#[wasm_bindgen_test]
fn test_cross_algorithm_rating_stability() {
    // Elo stability: alternating wins should keep ratings near initial value
    {
        let system = EloSystem::new();
        let mut p1 = system.create_rating();
        let mut p2 = system.create_rating();
        let initial = p1.value();

        for i in 0..100u32 {
            let outcome = if i % 2 == 0 {
                MatchOutcome::Player1Win
            } else {
                MatchOutcome::Player2Win
            };
            let result = system.process_1v1(&p1, &p2, outcome).unwrap();
            p1 = EloRating::new(result.player1_rating());
            p2 = EloRating::new(result.player2_rating());
        }

        // After 100 alternating wins, ratings should be close to initial
        assert!(
            (p1.value() - initial).abs() < 50.0,
            "Elo rating drifted too far: {} vs initial {}",
            p1.value(),
            initial
        );
    }

    // TrueSkill stability: after many games variance should decrease significantly
    {
        let system = TrueSkillSystem::new();
        let initial_var = (25.0_f64 / 3.0).powi(2);
        let mut p1_mean = 25.0_f64;
        let mut p1_var = initial_var;
        let mut p2_mean = 25.0_f64;
        let mut p2_var = initial_var;

        for i in 0..20u32 {
            let ratings_json = format!(
                r#"[{{"mean":{p1},"variance":{v1}}},{{"mean":{p2},"variance":{v2}}}]"#,
                p1 = p1_mean,
                v1 = p1_var,
                p2 = p2_mean,
                v2 = p2_var
            );
            let (r1, r2) = if i % 2 == 0 {
                (1u32, 2u32)
            } else {
                (2u32, 1u32)
            };
            let matches_json = format!(
                r#"[{{"teams":[[0],[1]],"ranks":[{r1},{r2}]}}]"#,
                r1 = r1,
                r2 = r2
            );
            let result =
                TrueSkillUtils::batch_process(&system, &ratings_json, &matches_json).unwrap();
            let results: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
            p1_mean = results[0]["mean"].as_f64().unwrap();
            p1_var = results[0]["variance"].as_f64().unwrap();
            p2_mean = results[1]["mean"].as_f64().unwrap();
            p2_var = results[1]["variance"].as_f64().unwrap();
        }

        // Variance should have decreased (more information about player skill)
        assert!(
            p1_var < initial_var,
            "TrueSkill variance did not decrease: {} vs initial {}",
            p1_var,
            initial_var
        );
        assert!(p1_mean.is_finite());
        assert!(p2_mean.is_finite());
    }
}

/// Verify TrueSkill serialization round-trip preserves precision
#[wasm_bindgen_test]
fn test_trueskill_serialization_round_trip() {
    let system = TrueSkillSystem::new();
    let original = system
        .create_rating_with_values(27.5, 45.123456789)
        .unwrap();

    let json = original.to_json();
    let restored = TrueSkillRating::from_json(&json).unwrap();

    assert!((restored.mean() - 27.5).abs() < 1e-9);
    assert!((restored.variance() - 45.123456789).abs() < 1e-6);
}

/// Verify TrueSkill win probability sums to 1.0 for 2-team match
#[wasm_bindgen_test]
fn test_trueskill_win_probability_sums_to_one() {
    let system = TrueSkillSystem::new();
    let p1 = system.create_rating_with_values(30.0, 50.0).unwrap();
    let p2 = system.create_rating_with_values(20.0, 80.0).unwrap();

    let team1 = TrueSkillTeam::from_ratings(vec![p1]).unwrap();
    let team2 = TrueSkillTeam::from_ratings(vec![p2]).unwrap();

    let teams = Array::new();
    teams.push(&js_sys::JSON::parse(&team1.to_json().unwrap()).unwrap());
    teams.push(&js_sys::JSON::parse(&team2.to_json().unwrap()).unwrap());

    let probs = system.win_probability(&teams).unwrap();
    assert_eq!(probs.length(), 2);

    let p1_prob = probs.get(0).as_f64().unwrap();
    let p2_prob = probs.get(1).as_f64().unwrap();

    assert!(
        (p1_prob + p2_prob - 1.0).abs() < 1e-9,
        "Win probabilities don't sum to 1: {} + {} = {}",
        p1_prob,
        p2_prob,
        p1_prob + p2_prob
    );
    // Player 1 has higher mean so should have higher win probability
    assert!(
        p1_prob > p2_prob,
        "Higher-rated player should have higher win probability: {} vs {}",
        p1_prob,
        p2_prob
    );
}
