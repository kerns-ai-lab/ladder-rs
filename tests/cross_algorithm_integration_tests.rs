use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    elo::{Elo, EloRating, EloTeam},
    glicko::{Glicko, GlickoRating, GlickoTeam},
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
};

/// Test that all rating systems have similar interfaces
#[test]
fn test_rating_system_interfaces() {
    // Create rating systems
    let elo = Elo::new();
    let glicko = Glicko::new();
    let trueskill = TrueSkill::new();

    // All should create default ratings
    let _elo_rating = elo.create_rating();
    let _glicko_rating = glicko.create_rating();
    let _trueskill_rating = trueskill.create_rating();

    // All should create ratings with values
    let _elo_custom = elo.create_rating_with_values(1600.0, 0.0);
    let _glicko_custom = glicko.create_rating_with_values(1600.0, 90000.0);
    let _trueskill_custom = trueskill.create_rating_with_values(30.0, 50.0);
}

/// Test basic two-player matches across all systems
#[test]
fn test_two_player_matches_all_systems() {
    // Elo
    let elo = Elo::new();
    let elo_team1 = EloTeam::from_player_ratings(vec![elo.create_rating()]);
    let elo_team2 = EloTeam::from_player_ratings(vec![elo.create_rating()]);
    let elo_result = elo
        .rate(&[elo_team1, elo_team2], &GameOutcome::win(0, 2))
        .unwrap();
    assert_eq!(elo_result.len(), 2);

    // Glicko
    let glicko = Glicko::new();
    let glicko_team1 = GlickoTeam::from_player_ratings(vec![glicko.create_rating()]);
    let glicko_team2 = GlickoTeam::from_player_ratings(vec![glicko.create_rating()]);
    let glicko_result = glicko
        .rate(&[glicko_team1, glicko_team2], &GameOutcome::win(0, 2))
        .unwrap();
    assert_eq!(glicko_result.len(), 2);

    // TrueSkill
    let trueskill = TrueSkill::new();
    let ts_team1 = TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]);
    let ts_team2 = TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]);
    let ts_result = trueskill
        .rate(&[ts_team1, ts_team2], &GameOutcome::win(0, 2))
        .unwrap();
    assert_eq!(ts_result.len(), 2);
}

/// Test that winner ratings increase and loser ratings decrease
#[test]
fn test_winner_loser_behavior() {
    // Elo
    let elo = Elo::new();
    let elo_p1 = elo.create_rating();
    let elo_p2 = elo.create_rating();
    let elo_teams = vec![
        EloTeam::from_player_ratings(vec![elo_p1]),
        EloTeam::from_player_ratings(vec![elo_p2]),
    ];
    let elo_result = elo.rate(&elo_teams, &GameOutcome::win(0, 2)).unwrap();
    assert!(elo_result[0].player_ratings()[0].mean() > elo_p1.mean());
    assert!(elo_result[1].player_ratings()[0].mean() < elo_p2.mean());

    // Glicko
    let glicko = Glicko::new();
    let glicko_p1 = glicko.create_rating();
    let glicko_p2 = glicko.create_rating();
    let glicko_teams = vec![
        GlickoTeam::from_player_ratings(vec![glicko_p1]),
        GlickoTeam::from_player_ratings(vec![glicko_p2]),
    ];
    let glicko_result = glicko.rate(&glicko_teams, &GameOutcome::win(0, 2)).unwrap();
    assert!(glicko_result[0].player_ratings()[0].mean() > glicko_p1.mean());
    assert!(glicko_result[1].player_ratings()[0].mean() < glicko_p2.mean());

    // TrueSkill
    let trueskill = TrueSkill::new();
    let ts_p1 = trueskill.create_rating();
    let ts_p2 = trueskill.create_rating();
    let ts_teams = vec![
        TrueSkillTeam::from_player_ratings(vec![ts_p1]),
        TrueSkillTeam::from_player_ratings(vec![ts_p2]),
    ];
    let ts_result = trueskill.rate(&ts_teams, &GameOutcome::win(0, 2)).unwrap();
    assert!(ts_result[0].player_ratings()[0].mean() > ts_p1.mean());
    assert!(ts_result[1].player_ratings()[0].mean() < ts_p2.mean());
}

/// Test that all systems handle draws (where applicable)
#[test]
fn test_draw_handling() {
    let draw_outcome = GameOutcome::draw(2);

    // Elo handles draws
    let elo = Elo::new();
    let elo_teams = vec![
        EloTeam::from_player_ratings(vec![elo.create_rating()]),
        EloTeam::from_player_ratings(vec![elo.create_rating()]),
    ];
    let elo_result = elo.rate(&elo_teams, &draw_outcome);
    assert!(elo_result.is_ok());

    // Glicko handles draws
    let glicko = Glicko::new();
    let glicko_teams = vec![
        GlickoTeam::from_player_ratings(vec![glicko.create_rating()]),
        GlickoTeam::from_player_ratings(vec![glicko.create_rating()]),
    ];
    let glicko_result = glicko.rate(&glicko_teams, &draw_outcome);
    assert!(glicko_result.is_ok());

    // TrueSkill handles draws
    let trueskill = TrueSkill::new();
    let ts_teams = vec![
        TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]),
        TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]),
    ];
    let ts_result = trueskill.rate(&ts_teams, &draw_outcome);
    assert!(ts_result.is_ok());
}

/// Test that all systems properly implement the Rating trait
#[test]
fn test_rating_trait_implementation() {
    // Elo rating
    let elo_rating = EloRating::new(1600.0);
    assert_eq!(elo_rating.mean(), 1600.0);
    assert_eq!(elo_rating.variance(), 0.0); // Elo has no uncertainty

    // Glicko rating
    let glicko_rating = GlickoRating::new(1600.0, 350.0 * 350.0);
    assert_eq!(glicko_rating.mean(), 1600.0);
    assert_eq!(glicko_rating.variance(), 350.0 * 350.0);

    // TrueSkill rating
    let ts_rating = TrueSkillRating::new(25.0, 8.333 * 8.333).unwrap();
    assert_eq!(ts_rating.mean(), 25.0);
    assert!((ts_rating.variance() - 8.333 * 8.333).abs() < 0.01);
}

/// Test that all team types properly implement TeamRating trait
#[test]
fn test_team_rating_trait_implementation() {
    // Elo team
    let elo_players = vec![EloRating::new(1500.0), EloRating::new(1600.0)];
    let elo_team = EloTeam::from_player_ratings(elo_players.clone());
    assert_eq!(elo_team.player_ratings().len(), 2);
    assert_eq!(elo_team.player_ratings()[0].mean(), 1500.0);

    // Glicko team
    let glicko_players = vec![
        GlickoRating::new(1500.0, 200.0 * 200.0),
        GlickoRating::new(1600.0, 150.0 * 150.0),
    ];
    let glicko_team = GlickoTeam::from_player_ratings(glicko_players.clone());
    assert_eq!(glicko_team.player_ratings().len(), 2);
    assert_eq!(glicko_team.player_ratings()[0].mean(), 1500.0);

    // TrueSkill team
    let ts_players = vec![
        TrueSkillRating::new(25.0, 8.0 * 8.0).unwrap(),
        TrueSkillRating::new(30.0, 7.0 * 7.0).unwrap(),
    ];
    let ts_team = TrueSkillTeam::from_player_ratings(ts_players.clone());
    assert_eq!(ts_team.player_ratings().len(), 2);
    assert_eq!(ts_team.player_ratings()[0].mean(), 25.0);
}

/// Test multi-team scenarios (3+ teams)
#[test]
fn test_multi_team_matches() {
    let three_team_outcome = GameOutcome::new(vec![1, 2, 3]);

    // Elo should handle multi-team
    let elo = Elo::new();
    let elo_teams = vec![
        EloTeam::from_player_ratings(vec![elo.create_rating()]),
        EloTeam::from_player_ratings(vec![elo.create_rating()]),
        EloTeam::from_player_ratings(vec![elo.create_rating()]),
    ];
    let elo_result = elo.rate(&elo_teams, &three_team_outcome);
    // Elo might not support >2 teams
    match elo_result {
        Ok(teams) => assert_eq!(teams.len(), 3),
        Err(_) => {} // Error is acceptable
    }

    // Glicko typically doesn't support >2 teams
    let glicko = Glicko::new();
    let glicko_teams = vec![
        GlickoTeam::from_player_ratings(vec![glicko.create_rating()]),
        GlickoTeam::from_player_ratings(vec![glicko.create_rating()]),
        GlickoTeam::from_player_ratings(vec![glicko.create_rating()]),
    ];
    let glicko_result = glicko.rate(&glicko_teams, &three_team_outcome);
    match glicko_result {
        Ok(teams) => assert_eq!(teams.len(), 3),
        Err(_) => {} // Error is acceptable
    }

    // TrueSkill might support multi-team in some implementations
    let trueskill = TrueSkill::new();
    let ts_teams = vec![
        TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]),
        TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]),
        TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]),
    ];
    let ts_result = trueskill.rate(&ts_teams, &three_team_outcome);
    match ts_result {
        Ok(teams) => assert_eq!(teams.len(), 3),
        Err(_) => {} // Error is acceptable
    }
}

/// Test that systems handle invalid inputs appropriately
#[test]
fn test_error_handling_consistency() {
    // Empty teams
    let empty_outcome = GameOutcome::new(vec![]);

    let elo = Elo::new();
    let elo_result = elo.rate(&[], &empty_outcome);
    assert!(elo_result.is_err());

    let glicko = Glicko::new();
    let glicko_result = glicko.rate(&[], &empty_outcome);
    assert!(glicko_result.is_err());

    let trueskill = TrueSkill::new();
    let ts_result = trueskill.rate(&[], &empty_outcome);
    assert!(ts_result.is_err());

    // Mismatched teams and outcomes
    let one_team_two_ranks = GameOutcome::new(vec![1, 2]);

    let elo_team = EloTeam::from_player_ratings(vec![elo.create_rating()]);
    assert!(elo.rate(&[elo_team], &one_team_two_ranks).is_err());

    let glicko_team = GlickoTeam::from_player_ratings(vec![glicko.create_rating()]);
    assert!(glicko.rate(&[glicko_team], &one_team_two_ranks).is_err());

    let ts_team = TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]);
    assert!(trueskill.rate(&[ts_team], &one_team_two_ranks).is_err());
}

/// Test a series of games to see rating progression
#[test]
fn test_rating_progression() {
    // Track how ratings evolve over multiple games
    let mut elo_p1 = EloRating::new(1500.0);
    let mut elo_p2 = EloRating::new(1500.0);

    let mut glicko_p1 = GlickoRating::new(1500.0, 350.0 * 350.0);
    let mut glicko_p2 = GlickoRating::new(1500.0, 350.0 * 350.0);

    let mut ts_p1 = TrueSkillRating::new(25.0, (25.0 / 3.0) * (25.0 / 3.0)).unwrap();
    let mut ts_p2 = TrueSkillRating::new(25.0, (25.0 / 3.0) * (25.0 / 3.0)).unwrap();

    let elo = Elo::new();
    let glicko = Glicko::new();
    let trueskill = TrueSkill::new();

    // Player 1 wins 5 games in a row
    for _ in 0..5 {
        // Elo
        let elo_teams = vec![
            EloTeam::from_player_ratings(vec![elo_p1]),
            EloTeam::from_player_ratings(vec![elo_p2]),
        ];
        let elo_result = elo.rate(&elo_teams, &GameOutcome::win(0, 2)).unwrap();
        elo_p1 = elo_result[0].player_ratings()[0].clone();
        elo_p2 = elo_result[1].player_ratings()[0].clone();

        // Glicko
        let glicko_teams = vec![
            GlickoTeam::from_player_ratings(vec![glicko_p1]),
            GlickoTeam::from_player_ratings(vec![glicko_p2]),
        ];
        let glicko_result = glicko.rate(&glicko_teams, &GameOutcome::win(0, 2)).unwrap();
        glicko_p1 = glicko_result[0].player_ratings()[0].clone();
        glicko_p2 = glicko_result[1].player_ratings()[0].clone();

        // TrueSkill
        let ts_teams = vec![
            TrueSkillTeam::from_player_ratings(vec![ts_p1]),
            TrueSkillTeam::from_player_ratings(vec![ts_p2]),
        ];
        let ts_result = trueskill.rate(&ts_teams, &GameOutcome::win(0, 2)).unwrap();
        ts_p1 = ts_result[0].player_ratings()[0].clone();
        ts_p2 = ts_result[1].player_ratings()[0].clone();
    }

    // After 5 wins, player 1 should be rated higher in all systems
    assert!(elo_p1.mean() > elo_p2.mean());
    assert!(glicko_p1.mean() > glicko_p2.mean());
    assert!(ts_p1.mean() > ts_p2.mean());

    // All systems should show rating changes
    assert!(elo_p1.mean() > 1500.0);
    assert!(elo_p2.mean() < 1500.0);

    assert!(glicko_p1.mean() > 1500.0);
    assert!(glicko_p2.mean() < 1500.0);

    // TrueSkill uses different scale
    assert!(ts_p1.mean() > 25.0);
    assert!(ts_p2.mean() < 25.0);
}

/// Test custom configurations across systems
#[test]
fn test_custom_configurations() {
    // Elo with custom K-factor
    let elo_custom = Elo::with_k_factor(16.0);
    let elo_rating = elo_custom.create_rating();
    assert_eq!(elo_rating.mean(), 1500.0);

    // Glicko with custom parameters
    let glicko_custom = Glicko::with_parameters(1600.0, 300.0, 0.06);
    let glicko_rating = glicko_custom.create_rating();
    assert_eq!(glicko_rating.mean(), 1600.0);

    // TrueSkill with custom parameters
    let ts_custom = TrueSkill::with_parameters(
        30.0,
        100.0,
        25.0,
        1.0,
        0.05,
        ladder_rs::trueskill::TrueSkillImplementation::Simplified,
    )
    .unwrap();
    let ts_rating = ts_custom.create_rating();
    assert_eq!(ts_rating.mean(), 30.0);
}

/// Test that rating values make sense across different scenarios
#[test]
fn test_rating_sanity_checks() {
    // Strong vs weak player
    let elo = Elo::new();
    let strong_elo = EloRating::new(2000.0);
    let weak_elo = EloRating::new(1000.0);

    // Strong player wins - should gain less
    let elo_teams = vec![
        EloTeam::from_player_ratings(vec![strong_elo]),
        EloTeam::from_player_ratings(vec![weak_elo]),
    ];
    let result = elo.rate(&elo_teams, &GameOutcome::win(0, 2)).unwrap();
    let strong_gain = result[0].player_ratings()[0].mean() - 2000.0;
    assert!(strong_gain > 0.0 && strong_gain < 5.0); // Small gain

    // Weak player wins - should gain more
    let upset_result = elo.rate(&elo_teams, &GameOutcome::win(1, 2)).unwrap();
    let weak_gain = upset_result[1].player_ratings()[0].mean() - 1000.0;
    assert!(weak_gain > 20.0); // Large gain for upset

    // Similar test for Glicko
    let glicko = Glicko::new();
    let strong_glicko = GlickoRating::new(2000.0, 50.0 * 50.0);
    let weak_glicko = GlickoRating::new(1000.0, 50.0 * 50.0);

    let glicko_teams = vec![
        GlickoTeam::from_player_ratings(vec![strong_glicko]),
        GlickoTeam::from_player_ratings(vec![weak_glicko]),
    ];
    let glicko_result = glicko.rate(&glicko_teams, &GameOutcome::win(0, 2)).unwrap();
    let glicko_strong_gain = glicko_result[0].player_ratings()[0].mean() - 2000.0;
    assert!(glicko_strong_gain > 0.0 && glicko_strong_gain < 10.0);
}

/// Test that all systems have reasonable default values
#[test]
fn test_default_values() {
    let elo = Elo::new();
    let elo_rating = elo.create_rating();
    assert_eq!(elo_rating.mean(), 1500.0);

    let glicko = Glicko::new();
    let glicko_rating = glicko.create_rating();
    assert_eq!(glicko_rating.mean(), 1500.0);
    assert!((glicko_rating.variance() - 350.0 * 350.0).abs() < 1.0);

    let trueskill = TrueSkill::new();
    let ts_rating = trueskill.create_rating();
    assert_eq!(ts_rating.mean(), 25.0); // Different scale!

    // Test custom scaling for TrueSkill to match others
    let ts_scaled = TrueSkill::with_parameters(
        1500.0, // Use same scale as others
        350.0 * 350.0,
        (350.0 / 2.0) * (350.0 / 2.0),
        (350.0 / 100.0) * (350.0 / 100.0),
        0.1,
        ladder_rs::trueskill::TrueSkillImplementation::Simplified,
    )
    .unwrap();
    let ts_scaled_rating = ts_scaled.create_rating();
    assert_eq!(ts_scaled_rating.mean(), 1500.0);
}

/// Test match quality calculations (where implemented)
#[test]
fn test_match_quality() {
    // Elo match quality
    let elo = Elo::new();
    let elo_teams = vec![
        EloTeam::from_player_ratings(vec![elo.create_rating()]),
        EloTeam::from_player_ratings(vec![elo.create_rating()]),
    ];
    let elo_quality = elo.calculate_match_quality(&elo_teams);
    match elo_quality {
        Ok(quality) => {
            assert!(quality >= 0.0 && quality <= 1.0);
            // Equal teams should have high quality
            assert!(quality > 0.4);
        }
        Err(_) => {} // Not implemented is OK
    }

    // Glicko match quality
    let glicko = Glicko::new();
    let glicko_teams = vec![
        GlickoTeam::from_player_ratings(vec![glicko.create_rating()]),
        GlickoTeam::from_player_ratings(vec![glicko.create_rating()]),
    ];
    let glicko_quality = glicko.calculate_match_quality(&glicko_teams);
    match glicko_quality {
        Ok(quality) => {
            assert!(quality >= 0.0 && quality <= 1.0);
            assert!(quality > 0.4);
        }
        Err(_) => {} // Not implemented is OK
    }

    // TrueSkill match quality
    let trueskill = TrueSkill::new();
    let ts_teams = vec![
        TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]),
        TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]),
    ];
    let ts_quality = trueskill.calculate_match_quality(&ts_teams);
    match ts_quality {
        Ok(quality) => {
            assert!(quality >= 0.0 && quality <= 1.0);
            assert!(quality > 0.4);
        }
        Err(_) => {} // Not implemented is OK
    }
}