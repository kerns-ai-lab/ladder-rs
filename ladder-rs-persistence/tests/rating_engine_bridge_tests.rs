//! Rating Engine Bridge unit tests for ladder-rs-persistence
//!
//! These tests verify the full behavior of RatingEngineBridge including
//! compute algorithm dispatch, rating changes, conservative_rating,
//! convergence_quality, and to_snapshots conversion.
//!
//! Tests for `compute()` and `to_snapshots()` compile against stubs
//! (will fail at runtime with Unknown errors — expected for TDD RED).
//! `conservative_rating()` IS implemented and should pass.
//!
//! Task: ladder-rs-907.5.1

use ladder_rs_persistence::{
    BridgeResult, MatchInput, PersistenceError, RatingEngineBridge, RatingInput, RatingOutput,
};

// ============================================================================
// TEST HELPERS
// ============================================================================

/// Create a simple two-player match with placements [1, 2] and no draws.
fn make_match_input(rating1: f64, rating2: f64) -> MatchInput {
    MatchInput {
        ratings: vec![
            RatingInput {
                rating: rating1,
                uncertainty: None,
                volatility: None,
            },
            RatingInput {
                rating: rating2,
                uncertainty: None,
                volatility: None,
            },
        ],
        placements: vec![1, 2],
        draws: vec![false, false],
    }
}

/// Create a two-player match with Glicko-2 uncertainties (RD values).
fn make_glicko2_input(rating1: f64, rd1: f64, rating2: f64, rd2: f64) -> MatchInput {
    MatchInput {
        ratings: vec![
            RatingInput {
                rating: rating1,
                uncertainty: Some(rd1),
                volatility: Some(0.06),
            },
            RatingInput {
                rating: rating2,
                uncertainty: Some(rd2),
                volatility: Some(0.06),
            },
        ],
        placements: vec![1, 2],
        draws: vec![false, false],
    }
}

/// Create a two-player match with TrueSkill uncertainties (sigma values).
fn make_trueskill_input(rating1: f64, sigma1: f64, rating2: f64, sigma2: f64) -> MatchInput {
    MatchInput {
        ratings: vec![
            RatingInput {
                rating: rating1,
                uncertainty: Some(sigma1),
                volatility: None,
            },
            RatingInput {
                rating: rating2,
                uncertainty: Some(sigma2),
                volatility: None,
            },
        ],
        placements: vec![1, 2],
        draws: vec![false, false],
    }
}

/// Create a match with a draw between both participants.
fn make_draw_input(rating1: f64, rating2: f64) -> MatchInput {
    MatchInput {
        ratings: vec![
            RatingInput {
                rating: rating1,
                uncertainty: None,
                volatility: None,
            },
            RatingInput {
                rating: rating2,
                uncertainty: None,
                volatility: None,
            },
        ],
        placements: vec![1, 1], // same placement = draw
        draws: vec![true, true],
    }
}

/// Create a single-participant match (edge case).
fn make_single_participant_input(rating: f64) -> MatchInput {
    MatchInput {
        ratings: vec![RatingInput {
            rating,
            uncertainty: None,
            volatility: None,
        }],
        placements: vec![1],
        draws: vec![false],
    }
}

/// Create a multi-participant match (3+ players).
fn make_multi_participant_input(ratings: &[f64]) -> MatchInput {
    let n = ratings.len();
    MatchInput {
        ratings: ratings
            .iter()
            .map(|&r| RatingInput {
                rating: r,
                uncertainty: None,
                volatility: None,
            })
            .collect(),
        placements: (1..=n as u32).collect(),
        draws: vec![false; n],
    }
}

/// Standard player IDs for two-player matches.
fn two_player_ids() -> Vec<String> {
    vec!["player-a".to_string(), "player-b".to_string()]
}

/// Standard player IDs for three-player matches.
fn three_player_ids() -> Vec<String> {
    vec![
        "player-a".to_string(),
        "player-b".to_string(),
        "player-c".to_string(),
    ]
}

// ============================================================================
// SECTION 1: compute() — algorithm dispatch
// ============================================================================

mod compute_algorithm_dispatch {
    use super::*;

    // --- Elo ---

    #[test]
    fn elo_returns_ok_bridge_result() {
        let input = make_match_input(1500.0, 1500.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        // Stub: currently returns Err(Unknown(...)). Will be Ok once implemented.
        assert!(result.is_ok(), "Elo compute should return Ok");
    }

    #[test]
    fn elo_output_count_matches_participant_count() {
        let input = make_match_input(1500.0, 1500.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        assert_eq!(
            bridge_result.outputs.len(),
            2,
            "Output count should match input count"
        );
    }

    // --- Glicko-2 ---

    #[test]
    fn glicko2_returns_ok_bridge_result() {
        let input = make_glicko2_input(1500.0, 350.0, 1500.0, 350.0);
        let result = RatingEngineBridge::compute(
            "glicko2",
            &input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        assert!(result.is_ok(), "Glicko-2 compute should return Ok");
    }

    #[test]
    fn glicko2_output_count_matches_participant_count() {
        let input = make_glicko2_input(1500.0, 350.0, 1500.0, 350.0);
        let result = RatingEngineBridge::compute(
            "glicko2",
            &input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        assert_eq!(
            bridge_result.outputs.len(),
            2,
            "Output count should match input count"
        );
    }

    #[test]
    fn glicko2_outputs_have_uncertainty_fields() {
        let input = make_glicko2_input(1500.0, 350.0, 1500.0, 350.0);
        let result = RatingEngineBridge::compute(
            "glicko2",
            &input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        for output in &bridge_result.outputs {
            assert!(
                output.uncertainty.is_some(),
                "Glicko-2 outputs should have uncertainty (RD)"
            );
        }
    }

    // --- TrueSkill ---

    #[test]
    fn trueskill_returns_ok_bridge_result() {
        let input = make_trueskill_input(25.0, 8.333, 25.0, 8.333);
        let result = RatingEngineBridge::compute(
            "trueskill",
            &input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        assert!(result.is_ok(), "TrueSkill compute should return Ok");
    }

    #[test]
    fn trueskill_output_count_matches_participant_count() {
        let input = make_trueskill_input(25.0, 8.333, 25.0, 8.333);
        let result = RatingEngineBridge::compute(
            "trueskill",
            &input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        assert_eq!(
            bridge_result.outputs.len(),
            2,
            "Output count should match input count"
        );
    }

    #[test]
    fn trueskill_outputs_have_uncertainty_fields() {
        let input = make_trueskill_input(25.0, 8.333, 25.0, 8.333);
        let result = RatingEngineBridge::compute(
            "trueskill",
            &input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        for output in &bridge_result.outputs {
            assert!(
                output.uncertainty.is_some(),
                "TrueSkill outputs should have uncertainty (sigma)"
            );
        }
    }

    // --- Unknown algorithm ---

    #[test]
    fn unknown_algorithm_returns_error() {
        let input = make_match_input(1500.0, 1500.0);
        let result = RatingEngineBridge::compute(
            "invalid_algo",
            &input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        assert!(result.is_err(), "Unknown algorithm should return error");
    }

    #[test]
    fn empty_algorithm_returns_error() {
        let input = make_match_input(1500.0, 1500.0);
        let result =
            RatingEngineBridge::compute("", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_err(), "Empty algorithm should return error");
    }
}

// ============================================================================
// SECTION 2: compute() — rating changes
// ============================================================================

mod compute_rating_changes {
    use super::*;

    #[test]
    fn winner_gains_rating() {
        let input = make_match_input(1500.0, 1500.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        // Player A (placement 1) is the winner — should gain rating
        let winner = &bridge_result.outputs[0];
        let loser = &bridge_result.outputs[1];
        assert!(
            winner.rating > 1500.0,
            "Winner (placement 1) should gain rating: {} > 1500.0",
            winner.rating
        );
        assert!(
            loser.rating < 1500.0,
            "Loser (placement 2) should lose rating: {} < 1500.0",
            loser.rating
        );
    }

    #[test]
    fn draw_produces_smaller_rating_changes_than_win() {
        let win_input = make_match_input(1500.0, 1500.0);
        let draw_input = make_draw_input(1500.0, 1500.0);

        let win_result = RatingEngineBridge::compute(
            "elo",
            &win_input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        let draw_result = RatingEngineBridge::compute(
            "elo",
            &draw_input,
            &two_player_ids(),
            "season-1",
            "match-2",
        );

        assert!(win_result.is_ok());
        assert!(draw_result.is_ok());

        let win = win_result.unwrap();
        let draw = draw_result.unwrap();

        let win_change = (win.outputs[0].rating - 1500.0).abs();
        let draw_change = (draw.outputs[0].rating - 1500.0).abs();

        assert!(
            draw_change < win_change,
            "Draw rating change ({}) should be smaller than win change ({})",
            draw_change,
            win_change
        );
    }

    #[test]
    fn draw_both_players_move_toward_each_other() {
        let input = make_draw_input(1600.0, 1400.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        // Higher-rated player should move down, lower-rated should move up
        let higher = &bridge_result.outputs[0]; // 1600 rated
        let lower = &bridge_result.outputs[1]; // 1400 rated
        assert!(
            higher.rating < 1600.0,
            "Higher-rated player should lose rating on a draw: {} < 1600.0",
            higher.rating
        );
        assert!(
            lower.rating > 1400.0,
            "Lower-rated player should gain rating on a draw: {} > 1400.0",
            lower.rating
        );
    }

    #[test]
    fn upset_produces_larger_rating_changes() {
        // Normal match: higher beats lower
        let normal_input = make_match_input(1600.0, 1400.0);
        // Upset match: lower beats higher (swap placements)
        let upset_input = MatchInput {
            ratings: vec![
                RatingInput {
                    rating: 1600.0,
                    uncertainty: None,
                    volatility: None,
                },
                RatingInput {
                    rating: 1400.0,
                    uncertainty: None,
                    volatility: None,
                },
            ],
            placements: vec![2, 1], // lower-rated wins!
            draws: vec![false, false],
        };

        let normal_result = RatingEngineBridge::compute(
            "elo",
            &normal_input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        let upset_result = RatingEngineBridge::compute(
            "elo",
            &upset_input,
            &two_player_ids(),
            "season-1",
            "match-2",
        );

        assert!(normal_result.is_ok());
        assert!(upset_result.is_ok());

        let normal = normal_result.unwrap();
        let upset = upset_result.unwrap();

        // In the normal case, the 1600 player (winner) gains a small amount
        // In the upset case, the 1600 player (loser) loses a large amount
        let normal_high_change = (normal.outputs[0].rating - 1600.0).abs();
        let upset_high_change = (upset.outputs[0].rating - 1600.0).abs();

        assert!(
            upset_high_change > normal_high_change,
            "Upset should produce larger rating change ({}) than expected outcome ({})",
            upset_high_change,
            normal_high_change
        );
    }

    #[test]
    fn multiple_participants_all_get_outputs() {
        let input = make_multi_participant_input(&[1500.0, 1550.0, 1450.0]);
        let result =
            RatingEngineBridge::compute("elo", &input, &three_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        assert_eq!(
            bridge_result.outputs.len(),
            3,
            "All 3 participants should get outputs"
        );
    }

    #[test]
    fn multiple_participants_winner_gains_most() {
        let input = make_multi_participant_input(&[1500.0, 1500.0, 1500.0]);
        let result =
            RatingEngineBridge::compute("elo", &input, &three_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();

        // Player A wins (placement 1), should have highest rating
        let winner_rating = bridge_result.outputs[0].rating;
        let second_rating = bridge_result.outputs[1].rating;
        let third_rating = bridge_result.outputs[2].rating;

        assert!(
            winner_rating > second_rating,
            "Winner (placement 1: {}) should have higher rating than second place ({})",
            winner_rating,
            second_rating
        );
        assert!(
            winner_rating > third_rating,
            "Winner should have higher rating than last place"
        );
    }
}

// ============================================================================
// SECTION 3: conservative_rating() — per-algorithm formula
// ============================================================================

mod conservative_rating_tests {
    use super::*;

    // --- Elo ---

    #[test]
    fn elo_conservative_equals_rating() {
        let result = RatingEngineBridge::conservative_rating("elo", 1500.0, None);
        assert_eq!(result, 1500.0);
    }

    #[test]
    fn elo_with_uncertainty_ignores_uncertainty() {
        let result = RatingEngineBridge::conservative_rating("elo", 1500.0, Some(100.0));
        assert_eq!(result, 1500.0);
    }

    #[test]
    fn elo_with_zero_rating() {
        let result = RatingEngineBridge::conservative_rating("elo", 0.0, None);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn elo_with_negative_rating() {
        let result = RatingEngineBridge::conservative_rating("elo", -100.0, None);
        assert_eq!(result, -100.0);
    }

    // --- Glicko / Glicko-2 ---

    #[test]
    fn glicko2_conservative_subtracts_2rd() {
        let result = RatingEngineBridge::conservative_rating("glicko2", 1500.0, Some(50.0));
        assert_eq!(result, 1400.0, "1500 - 2*50 = 1400");
    }

    #[test]
    fn glicko2_with_large_rd() {
        let result = RatingEngineBridge::conservative_rating("glicko2", 1500.0, Some(350.0));
        assert_eq!(result, 800.0, "1500 - 2*350 = 800");
    }

    #[test]
    fn glicko2_with_zero_rd() {
        let result = RatingEngineBridge::conservative_rating("glicko2", 1500.0, Some(0.0));
        assert_eq!(result, 1500.0);
    }

    #[test]
    fn glicko2_none_uncertainty_returns_rating() {
        let result = RatingEngineBridge::conservative_rating("glicko2", 1500.0, None);
        assert_eq!(result, 1500.0);
    }

    #[test]
    fn glicko_alias_same_as_glicko2() {
        let glicko2_result = RatingEngineBridge::conservative_rating("glicko2", 1500.0, Some(50.0));
        let glicko_result = RatingEngineBridge::conservative_rating("glicko", 1500.0, Some(50.0));
        assert_eq!(glicko_result, glicko2_result);
    }

    #[test]
    fn glicko2_conservative_can_be_negative() {
        let result = RatingEngineBridge::conservative_rating("glicko2", 100.0, Some(200.0));
        assert_eq!(result, -300.0, "100 - 2*200 = -300");
    }

    #[test]
    fn glicko2_with_negative_rating() {
        let result = RatingEngineBridge::conservative_rating("glicko2", -50.0, Some(10.0));
        assert_eq!(result, -70.0, "-50 - 2*10 = -70");
    }

    // --- TrueSkill ---

    #[test]
    fn trueskill_conservative_subtracts_3sigma() {
        let result = RatingEngineBridge::conservative_rating("trueskill", 25.0, Some(5.0));
        assert_eq!(result, 10.0, "25 - 3*5 = 10");
    }

    #[test]
    fn trueskill_with_high_sigma() {
        let result = RatingEngineBridge::conservative_rating("trueskill", 25.0, Some(8.333));
        // 25 - 3*8.333 = 25 - 24.999 ≈ 0.001
        let expected = 25.0 - 3.0 * 8.333;
        assert!(
            (result - expected).abs() < 0.001,
            "Expected ~{}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn trueskill_with_zero_sigma() {
        let result = RatingEngineBridge::conservative_rating("trueskill", 30.0, Some(0.0));
        assert_eq!(result, 30.0);
    }

    #[test]
    fn trueskill_none_uncertainty_returns_rating() {
        let result = RatingEngineBridge::conservative_rating("trueskill", 25.0, None);
        assert_eq!(result, 25.0);
    }

    #[test]
    fn trueskill_conservative_can_be_negative() {
        let result = RatingEngineBridge::conservative_rating("trueskill", 5.0, Some(10.0));
        assert_eq!(result, -25.0, "5 - 3*10 = -25");
    }

    // --- Unknown algorithm ---

    #[test]
    fn unknown_algorithm_defaults_to_rating() {
        let result = RatingEngineBridge::conservative_rating("unknown", 1500.0, Some(50.0));
        assert_eq!(result, 1500.0);
    }

    #[test]
    fn empty_algorithm_defaults_to_rating() {
        let result = RatingEngineBridge::conservative_rating("", 1500.0, Some(50.0));
        assert_eq!(result, 1500.0);
    }

    // --- Floating-point safety ---

    #[test]
    fn very_large_rating_value() {
        let result = RatingEngineBridge::conservative_rating("elo", 1_000_000.0, None);
        assert_eq!(result, 1_000_000.0);
    }

    #[test]
    fn very_large_uncertainty_glicko2() {
        let result = RatingEngineBridge::conservative_rating("glicko2", 5000.0, Some(500.0));
        assert_eq!(result, 4000.0);
    }

    #[test]
    fn very_small_uncertainty_glicko2() {
        let result = RatingEngineBridge::conservative_rating("glicko2", 1500.0, Some(0.0001));
        let expected = 1500.0 - 2.0 * 0.0001;
        assert!(
            (result - expected).abs() < 1e-10,
            "Small uncertainty should compute cleanly"
        );
    }
}

// ============================================================================
// SECTION 4: convergence_quality
// ============================================================================

mod convergence_quality_tests {
    use super::*;

    #[test]
    fn bridge_result_has_convergence_quality_field() {
        let input = make_match_input(1500.0, 1500.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        // The field should exist and be accessible
        let _quality: &str = &bridge_result.convergence_quality;
    }

    #[test]
    fn convergence_quality_is_non_empty() {
        let input = make_match_input(1500.0, 1500.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        assert!(
            !bridge_result.convergence_quality.is_empty(),
            "convergence_quality should not be empty"
        );
    }

    #[test]
    fn convergence_quality_is_converged_or_degraded() {
        let input = make_match_input(1500.0, 1500.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        let quality = &bridge_result.convergence_quality;
        assert!(
            quality == "converged" || quality == "degraded",
            "convergence_quality should be 'converged' or 'degraded', got '{}'",
            quality
        );
    }

    #[test]
    fn glicko2_return_has_convergence_quality() {
        let input = make_glicko2_input(1500.0, 350.0, 1500.0, 350.0);
        let result = RatingEngineBridge::compute(
            "glicko2",
            &input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        let quality = &bridge_result.convergence_quality;
        assert!(
            quality == "converged" || quality == "degraded",
            "Glicko-2 convergence_quality should be 'converged' or 'degraded', got '{}'",
            quality
        );
    }

    #[test]
    fn trueskill_return_has_convergence_quality() {
        let input = make_trueskill_input(25.0, 8.333, 25.0, 8.333);
        let result = RatingEngineBridge::compute(
            "trueskill",
            &input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        let quality = &bridge_result.convergence_quality;
        assert!(
            quality == "converged" || quality == "degraded",
            "TrueSkill convergence_quality should be 'converged' or 'degraded', got '{}'",
            quality
        );
    }
}

// ============================================================================
// SECTION 5: to_snapshots()
// ============================================================================

mod to_snapshots_tests {
    use super::*;

    /// Minimal helper: construct a BridgeResult manually for to_snapshots tests.
    fn make_simple_bridge_result() -> BridgeResult {
        BridgeResult {
            outputs: vec![
                RatingOutput {
                    rating: 1520.0,
                    uncertainty: Some(30.0),
                    volatility: Some(0.06),
                    conservative_rating: 1460.0,
                },
                RatingOutput {
                    rating: 1480.0,
                    uncertainty: Some(30.0),
                    volatility: Some(0.06),
                    conservative_rating: 1420.0,
                },
            ],
            convergence_quality: "converged".to_string(),
        }
    }

    #[test]
    fn to_snapshots_returns_correct_count() {
        let bridge = make_simple_bridge_result();
        let result = RatingEngineBridge::to_snapshots(&bridge, &two_player_ids(), "season-1", 5);
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(
            snapshots.len(),
            2,
            "Should return one snapshot per participant"
        );
    }

    #[test]
    fn to_snapshots_player_id_matches_input_order() {
        let bridge = make_simple_bridge_result();
        let result = RatingEngineBridge::to_snapshots(&bridge, &two_player_ids(), "season-1", 5);
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(snapshots[0].player_id, "player-a");
        assert_eq!(snapshots[1].player_id, "player-b");
    }

    #[test]
    fn to_snapshots_season_id_is_set() {
        let bridge = make_simple_bridge_result();
        let result =
            RatingEngineBridge::to_snapshots(&bridge, &two_player_ids(), "my-season-id", 5);
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        for snapshot in &snapshots {
            assert_eq!(snapshot.season_id, "my-season-id");
        }
    }

    #[test]
    fn to_snapshots_rating_value_matches_output() {
        let bridge = make_simple_bridge_result();
        let result = RatingEngineBridge::to_snapshots(&bridge, &two_player_ids(), "season-1", 5);
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(snapshots[0].rating_value, 1520.0);
        assert_eq!(snapshots[1].rating_value, 1480.0);
    }

    #[test]
    fn to_snapshots_uncertainty_matches_output() {
        let bridge = make_simple_bridge_result();
        let result = RatingEngineBridge::to_snapshots(&bridge, &two_player_ids(), "season-1", 5);
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(snapshots[0].uncertainty, Some(30.0));
        assert_eq!(snapshots[1].uncertainty, Some(30.0));
    }

    #[test]
    fn to_snapshots_rating_period_matches_input() {
        let bridge = make_simple_bridge_result();
        let result = RatingEngineBridge::to_snapshots(&bridge, &two_player_ids(), "season-1", 42);
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        for snapshot in &snapshots {
            assert_eq!(snapshot.rating_period, 42);
        }
    }

    #[test]
    fn to_snapshots_empty_player_ids_returns_error() {
        let bridge = make_simple_bridge_result();
        let result = RatingEngineBridge::to_snapshots(&bridge, &[], "season-1", 5);
        // Should error because there are outputs but no player IDs
        assert!(result.is_err());
    }

    #[test]
    fn to_snapshots_mismatched_lengths_returns_error() {
        let bridge = make_simple_bridge_result(); // 2 outputs
        let result = RatingEngineBridge::to_snapshots(
            &bridge,
            &["player-a".to_string()], // only 1 player ID
            "season-1",
            5,
        );
        assert!(
            result.is_err(),
            "Mismatched lengths (2 outputs, 1 player_id) should error"
        );
    }

    #[test]
    fn to_snapshots_too_many_player_ids_returns_error() {
        let bridge = make_simple_bridge_result(); // 2 outputs
        let result = RatingEngineBridge::to_snapshots(
            &bridge,
            &three_player_ids(), // 3 player IDs
            "season-1",
            5,
        );
        assert!(
            result.is_err(),
            "Mismatched lengths (2 outputs, 3 player_ids) should error"
        );
    }

    #[test]
    fn to_snapshots_single_player() {
        let bridge = BridgeResult {
            outputs: vec![RatingOutput {
                rating: 1500.0,
                uncertainty: None,
                volatility: None,
                conservative_rating: 1500.0,
            }],
            convergence_quality: "converged".to_string(),
        };
        let result =
            RatingEngineBridge::to_snapshots(&bridge, &["solo-player".to_string()], "season-1", 1);
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].player_id, "solo-player");
        assert_eq!(snapshots[0].rating_value, 1500.0);
    }

    #[test]
    fn to_snapshots_glicko2_outputs_include_uncertainty() {
        let bridge = BridgeResult {
            outputs: vec![RatingOutput {
                rating: 1525.0,
                uncertainty: Some(45.0),
                volatility: Some(0.0599),
                conservative_rating: 1435.0,
            }],
            convergence_quality: "converged".to_string(),
        };
        let result = RatingEngineBridge::to_snapshots(
            &bridge,
            &["glicko-player".to_string()],
            "season-1",
            3,
        );
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(snapshots[0].uncertainty, Some(45.0));
        assert_eq!(snapshots[0].volatility, Some(0.0599));
    }

    #[test]
    fn to_snapshots_trueskill_outputs_include_uncertainty() {
        let bridge = BridgeResult {
            outputs: vec![RatingOutput {
                rating: 27.5,
                uncertainty: Some(5.0),
                volatility: None,
                conservative_rating: 12.5,
            }],
            convergence_quality: "converged".to_string(),
        };
        let result = RatingEngineBridge::to_snapshots(
            &bridge,
            &["trueskill-player".to_string()],
            "season-1",
            3,
        );
        assert!(result.is_ok());
        let snapshots = result.unwrap();
        assert_eq!(snapshots[0].uncertainty, Some(5.0));
        assert_eq!(snapshots[0].volatility, None);
    }
}

// ============================================================================
// SECTION 6: Edge cases
// ============================================================================

mod edge_cases {
    use super::*;

    // --- Single participant ---

    #[test]
    fn single_participant_returns_single_output() {
        let input = make_single_participant_input(1500.0);
        let result = RatingEngineBridge::compute(
            "elo",
            &input,
            &["solo".to_string()],
            "season-1",
            "match-1",
        );
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        assert_eq!(
            bridge_result.outputs.len(),
            1,
            "Single participant should produce single output"
        );
    }

    #[test]
    fn single_participant_rating_barely_changes() {
        let input = make_single_participant_input(1500.0);
        let result = RatingEngineBridge::compute(
            "elo",
            &input,
            &["solo".to_string()],
            "season-1",
            "match-1",
        );
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        let change = (bridge_result.outputs[0].rating - 1500.0).abs();
        assert!(
            change < 10.0,
            "Single participant rating should barely change, got change of {}",
            change
        );
    }

    // --- Identical ratings ---

    #[test]
    fn identical_ratings_produce_near_zero_net_change() {
        let input = make_match_input(1500.0, 1500.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        // Sum of rating changes should be near zero (zero-sum or near-zero-sum)
        let changes: Vec<f64> = bridge_result
            .outputs
            .iter()
            .zip(input.ratings.iter())
            .map(|(out, inp)| out.rating - inp.rating)
            .collect();
        let net_change: f64 = changes.iter().sum();
        assert!(
            net_change.abs() < 1.0,
            "Net rating change should be near zero for Elo, got {}",
            net_change
        );
    }

    // --- Very large rating differences ---

    #[test]
    fn very_large_rating_difference_not_nan() {
        let input = make_match_input(3000.0, 500.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        for output in &bridge_result.outputs {
            assert!(
                !output.rating.is_nan(),
                "Rating should not be NaN for large difference"
            );
            assert!(
                output.rating.is_finite(),
                "Rating should be finite for large difference"
            );
        }
    }

    #[test]
    fn very_large_rating_difference_not_infinite() {
        let input = make_match_input(1_000_000.0, 0.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        // May or may not error; if it succeeds outputs must be finite
        if let Ok(bridge_result) = result {
            for output in &bridge_result.outputs {
                assert!(
                    output.rating.is_finite(),
                    "Rating should be finite for extreme values"
                );
            }
        }
    }

    // --- Zero rating inputs ---

    #[test]
    fn zero_ratings_produce_valid_output() {
        let input = make_match_input(0.0, 0.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        for output in &bridge_result.outputs {
            assert!(
                output.rating.is_finite(),
                "Zero rating inputs should produce finite outputs"
            );
        }
    }

    #[test]
    fn zero_rating_input_winner_and_loser_change() {
        let input = make_match_input(0.0, 0.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        let winner_change = bridge_result.outputs[0].rating - 0.0;
        let loser_change = bridge_result.outputs[1].rating - 0.0;
        assert!(winner_change > 0.0, "Winner should gain from zero");
        assert!(loser_change < 0.0, "Loser should lose from zero");
        // For Elo, changes should be symmetric (zero-sum)
        let net = winner_change + loser_change;
        assert!(net.abs() < 0.01, "Zero-sum: winner+loser change ≈ 0");
    }

    // --- Negative rating inputs ---

    #[test]
    fn negative_ratings_produce_valid_output() {
        let input = make_match_input(-200.0, -300.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        for output in &bridge_result.outputs {
            assert!(
                output.rating.is_finite(),
                "Negative rating inputs should produce finite outputs"
            );
        }
    }

    #[test]
    fn negative_ratings_winner_still_gains() {
        let input = make_match_input(-200.0, -300.0);
        let result =
            RatingEngineBridge::compute("elo", &input, &two_player_ids(), "season-1", "match-1");
        assert!(result.is_ok());
        let bridge_result = result.unwrap();
        // Player A (-200 rating) is the winner (placement 1)
        assert!(
            bridge_result.outputs[0].rating > -200.0,
            "Winner with negative rating should gain rating"
        );
        assert!(
            bridge_result.outputs[1].rating < -300.0,
            "Loser with negative rating should lose rating"
        );
    }

    // --- Struct serialization round-trip ---

    #[test]
    fn match_input_serializes_and_deserializes() {
        let input = make_match_input(1500.0, 1400.0);
        let json = serde_json::to_string(&input).expect("Serialize MatchInput");
        let parsed: MatchInput = serde_json::from_str(&json).expect("Deserialize MatchInput");
        assert_eq!(parsed.ratings.len(), 2);
        assert_eq!(parsed.placements, vec![1, 2]);
        assert_eq!(parsed.draws, vec![false, false]);
        assert!((parsed.ratings[0].rating - 1500.0).abs() < f64::EPSILON);
        assert!((parsed.ratings[1].rating - 1400.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bridge_result_serializes_and_deserializes() {
        let bridge = BridgeResult {
            outputs: vec![
                RatingOutput {
                    rating: 1520.0,
                    uncertainty: Some(30.0),
                    volatility: Some(0.06),
                    conservative_rating: 1460.0,
                },
                RatingOutput {
                    rating: 1480.0,
                    uncertainty: Some(30.0),
                    volatility: Some(0.06),
                    conservative_rating: 1420.0,
                },
            ],
            convergence_quality: "converged".to_string(),
        };
        let json = serde_json::to_string(&bridge).expect("Serialize BridgeResult");
        let parsed: BridgeResult = serde_json::from_str(&json).expect("Deserialize BridgeResult");
        assert_eq!(parsed.outputs.len(), 2);
        assert_eq!(parsed.convergence_quality, "converged");
        assert!((parsed.outputs[0].rating - 1520.0).abs() < f64::EPSILON);
        assert_eq!(parsed.outputs[0].uncertainty, Some(30.0));
    }
}

// ============================================================================
// SECTION 7: Error types
// ============================================================================

mod error_types {
    use super::*;

    #[test]
    fn compute_error_is_persistence_error() {
        let input = make_match_input(1500.0, 1500.0);
        let result = RatingEngineBridge::compute(
            "unknown_algo",
            &input,
            &two_player_ids(),
            "season-1",
            "match-1",
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Verify it's the correct error type via pattern matching
        match err {
            PersistenceError::Unknown(_) | PersistenceError::InvalidInput(_) => {
                // Expected error variants
            }
            _ => panic!("Expected Unknown or InvalidInput, got {:?}", err),
        }
    }

    #[test]
    fn to_snapshots_mismatch_is_invalid_input() {
        let bridge = BridgeResult {
            outputs: vec![
                RatingOutput {
                    rating: 1520.0,
                    uncertainty: None,
                    volatility: None,
                    conservative_rating: 1520.0,
                },
                RatingOutput {
                    rating: 1480.0,
                    uncertainty: None,
                    volatility: None,
                    conservative_rating: 1480.0,
                },
            ],
            convergence_quality: "converged".to_string(),
        };
        let result =
            RatingEngineBridge::to_snapshots(&bridge, &["only-one".to_string()], "season-1", 1);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            PersistenceError::InvalidInput(_) | PersistenceError::Unknown(_) => {
                // Expected
            }
            _ => panic!("Expected InvalidInput, got {:?}", err),
        }
    }
}

// ============================================================================
// SECTION 8: conservative_rating on BridgeResult.outputs
// ============================================================================

mod conservative_rating_in_outputs {
    use super::*;

    /// Verify that the `conservative_rating` field in RatingOutput is populated
    /// appropriately (i.e., the bridge pre-computes it from rating and uncertainty
    /// using the algorithm-specific formula).
    ///
    /// Since compute() is stubbed, this uses a manually-constructed BridgeResult
    /// to verify the field exists and can hold the expected formula results.

    #[test]
    fn elo_output_conservative_equals_rating() {
        let output = RatingOutput {
            rating: 1520.0,
            uncertainty: None,
            volatility: None,
            conservative_rating: 1520.0, // Elo: no penalty
        };
        assert_eq!(output.conservative_rating, output.rating);
    }

    #[test]
    fn glicko2_output_conservative_subtracts_2rd() {
        let rating = 1500.0;
        let rd = 50.0;
        let conservative = RatingEngineBridge::conservative_rating("glicko2", rating, Some(rd));

        let output = RatingOutput {
            rating,
            uncertainty: Some(rd),
            volatility: Some(0.06),
            conservative_rating: conservative,
        };
        assert_eq!(output.conservative_rating, rating - 2.0 * rd);
    }

    #[test]
    fn trueskill_output_conservative_subtracts_3sigma() {
        let rating = 25.0;
        let sigma = 8.333;
        let conservative =
            RatingEngineBridge::conservative_rating("trueskill", rating, Some(sigma));

        let output = RatingOutput {
            rating,
            uncertainty: Some(sigma),
            volatility: None,
            conservative_rating: conservative,
        };
        assert_eq!(output.conservative_rating, rating - 3.0 * sigma);
    }

    #[test]
    fn zero_uncertainty_output_conservative_equals_rating() {
        for algo in &["elo", "glicko2", "trueskill"] {
            let conservative = RatingEngineBridge::conservative_rating(algo, 1500.0, Some(0.0));
            assert_eq!(
                conservative, 1500.0,
                "{}: zero uncertainty should yield conservative == rating",
                algo
            );
        }
    }

    #[test]
    fn none_uncertainty_output_conservative_equals_rating() {
        for algo in &["elo", "glicko2", "trueskill"] {
            let conservative = RatingEngineBridge::conservative_rating(algo, 1500.0, None);
            assert_eq!(
                conservative, 1500.0,
                "{}: None uncertainty should yield conservative == rating",
                algo
            );
        }
    }
}
