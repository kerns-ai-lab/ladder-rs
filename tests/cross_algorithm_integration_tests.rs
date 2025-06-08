use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    elo::{EloRating, EloSystem, EloTeamRating},
    glicko::{Glicko, Glicko2, Glicko2Rating, Glicko2TeamRating, GlickoRating, GlickoTeamRating},
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
};

/// Test that compares behavior across different rating systems
#[test]
fn test_rating_system_comparisons() {
    // Create rating systems
    let elo = EloSystem::new();
    let glicko = Glicko::new();
    let glicko2 = Glicko2::new();
    let trueskill = TrueSkill::new_simplified();

    // Test default rating creation
    let elo_rating = elo.create_rating();
    let glicko_rating = glicko.create_rating();
    let glicko2_rating = glicko2.create_rating();
    let trueskill_rating = trueskill.create_rating();

    // Elo starts at 1500, others at different values
    assert_eq!(elo_rating.mean(), 1500.0);
    assert_eq!(glicko_rating.mean(), 1500.0);
    assert_eq!(glicko2_rating.mean(), 1500.0);
    assert_eq!(trueskill_rating.mean(), 25.0);

    // Different variance handling
    assert_eq!(elo_rating.variance(), 0.0); // Point estimate
    assert!(glicko_rating.variance() > 0.0);
    assert!(glicko2_rating.variance() > 0.0);
    assert!(trueskill_rating.variance() > 0.0);
}

/// Test creating ratings with custom values across systems
#[test]
fn test_custom_rating_creation() {
    let elo = EloSystem::new();
    let glicko = Glicko::new();
    let glicko2 = Glicko2::new();
    let trueskill = TrueSkill::new_simplified();

    let mean = 1600.0;
    let variance = 10000.0; // σ = 100

    let elo_rating = elo.create_rating_with_values(mean, variance);
    let glicko_rating = glicko.create_rating_with_values(mean, variance);
    let glicko2_rating = glicko2.create_rating_with_values(mean, variance);
    let trueskill_rating = trueskill.create_rating_with_values(mean, variance);

    // All should have the same mean
    assert_eq!(elo_rating.mean(), mean);
    assert_eq!(glicko_rating.mean(), mean);
    assert_eq!(glicko2_rating.mean(), mean);
    assert_eq!(trueskill_rating.mean(), mean);

    // Variance handling differs
    assert_eq!(elo_rating.variance(), 0.0); // Elo ignores variance
    assert_eq!(glicko_rating.variance(), variance);
    assert_eq!(glicko2_rating.variance(), variance);
    assert_eq!(trueskill_rating.variance(), variance);
}

/// Test conservative rating calculations across systems
#[test]
fn test_conservative_ratings() {
    let mean = 1600.0;
    let variance = 10000.0; // σ = 100

    let elo_rating = EloRating::new(mean);
    let glicko_rating = GlickoRating::new(mean, 100.0);
    let glicko2_rating = Glicko2Rating::new(mean, 100.0, 0.06);
    let trueskill_rating = TrueSkillRating::new(mean, variance).unwrap();

    // Conservative ratings use different formulas
    assert_eq!(elo_rating.conservative_rating(), mean); // No uncertainty
    assert_eq!(glicko_rating.conservative_rating(), mean - 2.0 * 100.0); // μ - 2σ
    assert_eq!(glicko2_rating.conservative_rating(), mean - 2.0 * 100.0); // μ - 2σ
    assert_eq!(trueskill_rating.conservative_rating(), mean - 3.0 * 100.0); // μ - 3σ
}

/// Test a simple match across all rating systems
#[test]
fn test_simple_match_across_systems() {
    // Set up rating systems with similar scales for comparison
    let elo = EloSystem::new();
    let glicko = Glicko::new();
    let glicko2 = Glicko2::new();
    let trueskill = TrueSkill::with_parameters(
        1500.0,                            // μ₀ (same as Elo/Glicko)
        350.0 * 350.0,                     // σ₀² (same as Glicko default)
        (350.0 / 2.0) * (350.0 / 2.0),     // β² (half of initial uncertainty)
        (350.0 / 100.0) * (350.0 / 100.0), // γ² (skill drift)
        0.1,                               // draw probability
        ladder_rs::trueskill::TrueSkillImplementation::Simplified,
    )
    .unwrap();

    // Create equal players in each system
    let elo_p1 = elo.create_rating();
    let elo_p2 = elo.create_rating();

    let glicko_p1 = glicko.create_rating();
    let glicko_p2 = glicko.create_rating();

    let glicko2_p1 = glicko2.create_rating();
    let glicko2_p2 = glicko2.create_rating();

    let ts_p1 = trueskill.create_rating();
    let ts_p2 = trueskill.create_rating();

    // Create teams
    let elo_team1 = EloTeamRating::new(elo_p1);
    let elo_team2 = EloTeamRating::new(elo_p2);

    let glicko_team1 = GlickoTeamRating::from_player_ratings(vec![glicko_p1]);
    let glicko_team2 = GlickoTeamRating::from_player_ratings(vec![glicko_p2]);

    let glicko2_team1 = Glicko2TeamRating::from_player_ratings(vec![glicko2_p1]);
    let glicko2_team2 = Glicko2TeamRating::from_player_ratings(vec![glicko2_p2]);

    let ts_team1 = TrueSkillTeam::from_player_ratings(vec![ts_p1]);
    let ts_team2 = TrueSkillTeam::from_player_ratings(vec![ts_p2]);

    // Player 1 wins in all systems
    let outcome = GameOutcome::win(0, 2);

    let elo_result = elo.rate(&[elo_team1, elo_team2], &outcome).unwrap();
    let glicko_result = glicko
        .rate(&[glicko_team1, glicko_team2], &outcome)
        .unwrap();
    let glicko2_result = glicko2
        .rate(&[glicko2_team1, glicko2_team2], &outcome)
        .unwrap();
    let ts_result = trueskill.rate(&[ts_team1, ts_team2], &outcome).unwrap();

    // Winner should increase in all systems
    assert!(elo_result[0].player_ratings()[0].mean() > 1500.0);
    assert!(glicko_result[0].player_ratings()[0].mean() > 1500.0);
    assert!(glicko2_result[0].player_ratings()[0].mean() > 1500.0);
    assert!(ts_result[0].player_ratings()[0].mean() > 1500.0);

    // Loser should decrease in all systems
    assert!(elo_result[1].player_ratings()[0].mean() < 1500.0);
    assert!(glicko_result[1].player_ratings()[0].mean() < 1500.0);
    assert!(glicko2_result[1].player_ratings()[0].mean() < 1500.0);
    assert!(ts_result[1].player_ratings()[0].mean() < 1500.0);

    println!(
        "Elo winner: {:.3}, loser: {:.3}",
        elo_result[0].player_ratings()[0].mean(),
        elo_result[1].player_ratings()[0].mean()
    );
    println!(
        "Glicko winner: {:.3}, loser: {:.3}",
        glicko_result[0].player_ratings()[0].mean(),
        glicko_result[1].player_ratings()[0].mean()
    );
    println!(
        "Glicko2 winner: {:.3}, loser: {:.3}",
        glicko2_result[0].player_ratings()[0].mean(),
        glicko2_result[1].player_ratings()[0].mean()
    );
    println!(
        "TrueSkill winner: {:.3}, loser: {:.3}",
        ts_result[0].player_ratings()[0].mean(),
        ts_result[1].player_ratings()[0].mean()
    );
}

/// Test match quality across supported systems
#[test]
fn test_match_quality_across_systems() {
    let elo = EloSystem::new();
    let glicko = Glicko::new();
    let glicko2 = Glicko2::new();

    // Create equal players
    let elo_team1 = EloTeamRating::new(EloRating::new(1500.0));
    let elo_team2 = EloTeamRating::new(EloRating::new(1500.0));

    let glicko_team1 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);
    let glicko_team2 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);

    let glicko2_team1 =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);
    let glicko2_team2 =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);

    // Calculate match quality (TrueSkill doesn't implement this yet)
    let elo_quality = elo
        .calculate_match_quality(&[elo_team1, elo_team2])
        .unwrap();
    let glicko_quality = glicko
        .calculate_match_quality(&[glicko_team1, glicko_team2])
        .unwrap();
    let glicko2_quality = glicko2
        .calculate_match_quality(&[glicko2_team1, glicko2_team2])
        .unwrap();

    // Equal players should have high match quality
    assert!(elo_quality > 0.9);
    assert!(glicko_quality > 0.9);
    assert!(glicko2_quality > 0.9);

    println!("Match qualities for equal players:");
    println!("Elo: {:.6}", elo_quality);
    println!("Glicko: {:.6}", glicko_quality);
    println!("Glicko2: {:.6}", glicko2_quality);
}

/// Test error handling consistency across systems
#[test]
fn test_error_handling_consistency() {
    let elo = EloSystem::new();
    let glicko = Glicko::new();
    let glicko2 = Glicko2::new();
    let trueskill = TrueSkill::new_simplified();

    // Test invalid team counts (all should error on wrong number of teams)
    let elo_team = EloTeamRating::new(EloRating::new(1500.0));
    let glicko_team = GlickoTeamRating::from_player_ratings(vec![GlickoRating::default()]);
    let glicko2_team = Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::default()]);
    let ts_team = TrueSkillTeam::from_player_ratings(vec![trueskill.create_rating()]);

    let single_team_outcome = GameOutcome::new(vec![1]);

    // All should reject single team games (except TrueSkill which might support it)
    assert!(elo.rate(&[elo_team], &single_team_outcome).is_err());
    assert!(glicko.rate(&[glicko_team], &single_team_outcome).is_err());
    assert!(glicko2.rate(&[glicko2_team], &single_team_outcome).is_err());
    assert!(trueskill.rate(&[ts_team], &single_team_outcome).is_err());
}

/// Test series of games across systems to compare convergence
#[test]
fn test_convergence_across_systems() {
    // Create systems with similar initial uncertainty
    let elo = EloSystem::new();
    let glicko = Glicko::new();
    let glicko2 = Glicko2::new();
    let trueskill = TrueSkill::with_parameters(
        1500.0,                            // μ₀
        350.0 * 350.0,                     // σ₀²
        (350.0 / 2.0) * (350.0 / 2.0),     // β²
        (350.0 / 100.0) * (350.0 / 100.0), // γ²
        0.1,
        ladder_rs::trueskill::TrueSkillImplementation::Simplified,
    )
    .unwrap();

    // Start with equal players
    let mut elo_p1 = elo.create_rating();
    let mut elo_p2 = elo.create_rating();

    let mut glicko_p1 = glicko.create_rating();
    let mut glicko_p2 = glicko.create_rating();

    let mut glicko2_p1 = glicko2.create_rating();
    let mut glicko2_p2 = glicko2.create_rating();

    let mut ts_p1 = trueskill.create_rating();
    let mut ts_p2 = trueskill.create_rating();

    // Player 1 wins 7 out of 10 games, but in a mixed pattern to avoid recency bias
    let game_outcomes = [0, 0, 1, 0, 0, 1, 0, 0, 1, 0]; // P1 wins 7, P2 wins 3
    for &winner in game_outcomes.iter() {
        let outcome = GameOutcome::win(winner, 2);

        // Elo
        let elo_team1 = EloTeamRating::new(elo_p1);
        let elo_team2 = EloTeamRating::new(elo_p2);
        let elo_result = elo.rate(&[elo_team1, elo_team2], &outcome).unwrap();
        elo_p1 = elo_result[0].player_ratings()[0].clone();
        elo_p2 = elo_result[1].player_ratings()[0].clone();

        // Glicko
        let glicko_team1 = GlickoTeamRating::from_player_ratings(vec![glicko_p1]);
        let glicko_team2 = GlickoTeamRating::from_player_ratings(vec![glicko_p2]);
        let glicko_result = glicko
            .rate(&[glicko_team1, glicko_team2], &outcome)
            .unwrap();
        glicko_p1 = glicko_result[0].player_ratings()[0].clone();
        glicko_p2 = glicko_result[1].player_ratings()[0].clone();

        // Glicko2
        let glicko2_team1 = Glicko2TeamRating::from_player_ratings(vec![glicko2_p1]);
        let glicko2_team2 = Glicko2TeamRating::from_player_ratings(vec![glicko2_p2]);
        let glicko2_result = glicko2
            .rate(&[glicko2_team1, glicko2_team2], &outcome)
            .unwrap();
        glicko2_p1 = glicko2_result[0].player_ratings()[0].clone();
        glicko2_p2 = glicko2_result[1].player_ratings()[0].clone();

        // TrueSkill
        let ts_team1 = TrueSkillTeam::from_player_ratings(vec![ts_p1]);
        let ts_team2 = TrueSkillTeam::from_player_ratings(vec![ts_p2]);
        let ts_result = trueskill.rate(&[ts_team1, ts_team2], &outcome).unwrap();
        ts_p1 = ts_result[0].player_ratings()[0].clone();
        ts_p2 = ts_result[1].player_ratings()[0].clone();
    }

    // After 10 games (7-3 record), player 1 should be rated higher in all systems
    
    assert!(elo_p1.mean() > elo_p2.mean(), "Elo: P1={} should be > P2={}", elo_p1.mean(), elo_p2.mean());
    assert!(glicko_p1.mean() > glicko_p2.mean(), "Glicko: P1={} should be > P2={}", glicko_p1.mean(), glicko_p2.mean());
    assert!(glicko2_p1.mean() > glicko2_p2.mean(), "Glicko2: P1={} should be > P2={}", glicko2_p1.mean(), glicko2_p2.mean());
    assert!(ts_p1.mean() > ts_p2.mean(), "TrueSkill: P1={} should be > P2={}", ts_p1.mean(), ts_p2.mean());

    println!("Final ratings after 7-3 record:");
    println!("Elo: P1={:.3}, P2={:.3}", elo_p1.mean(), elo_p2.mean());
    println!(
        "Glicko: P1={:.3}, P2={:.3}",
        glicko_p1.mean(),
        glicko_p2.mean()
    );
    println!(
        "Glicko2: P1={:.3}, P2={:.3}",
        glicko2_p1.mean(),
        glicko2_p2.mean()
    );
    println!("TrueSkill: P1={:.3}, P2={:.3}", ts_p1.mean(), ts_p2.mean());

    // Uncertainty should decrease in systems that track it
    assert_eq!(elo_p1.variance(), 0.0); // Elo doesn't track uncertainty
    assert!(glicko_p1.variance() < 350.0 * 350.0);
    assert!(glicko2_p1.variance() < 350.0 * 350.0);
    assert!(ts_p1.variance() < 350.0 * 350.0);
}

/// Test draw handling across systems
#[test]
fn test_draw_handling() {
    let elo = EloSystem::new();
    let glicko = Glicko::new();
    let glicko2 = Glicko2::new();
    let trueskill = TrueSkill::new_simplified();

    // Equal players
    let elo_team1 = EloTeamRating::new(EloRating::new(1500.0));
    let elo_team2 = EloTeamRating::new(EloRating::new(1500.0));

    let glicko_team1 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);
    let glicko_team2 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);

    let glicko2_team1 =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);
    let glicko2_team2 =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);

    // Convert TrueSkill to same scale
    let ts_team1 = TrueSkillTeam::from_player_ratings(vec![
        trueskill.create_rating_with_values(1500.0, 200.0 * 200.0)
    ]);
    let ts_team2 = TrueSkillTeam::from_player_ratings(vec![
        trueskill.create_rating_with_values(1500.0, 200.0 * 200.0)
    ]);

    let draw_outcome = GameOutcome::draw(2);

    // All systems should handle draws
    let elo_result = elo.rate(&[elo_team1, elo_team2], &draw_outcome).unwrap();
    let glicko_result = glicko
        .rate(&[glicko_team1, glicko_team2], &draw_outcome)
        .unwrap();
    let glicko2_result = glicko2
        .rate(&[glicko2_team1, glicko2_team2], &draw_outcome)
        .unwrap();
    let ts_result = trueskill
        .rate(&[ts_team1, ts_team2], &draw_outcome)
        .unwrap();

    // In draws between equal players, means should stay close to original
    assert!((elo_result[0].player_ratings()[0].mean() - 1500.0).abs() < 1.0);
    assert!((glicko_result[0].player_ratings()[0].mean() - 1500.0).abs() < 5.0);
    assert!((glicko2_result[0].player_ratings()[0].mean() - 1500.0).abs() < 5.0);
    assert!((ts_result[0].player_ratings()[0].mean() - 1500.0).abs() < 5.0);
}

/// Test scaling differences between systems
#[test]
fn test_scaling_differences() {
    // Different systems use different scales by default
    let elo = EloSystem::new();
    let glicko = Glicko::new();
    let glicko2 = Glicko2::new();
    let trueskill = TrueSkill::new(); // Default TrueSkill uses μ=25

    let elo_rating = elo.create_rating();
    let glicko_rating = glicko.create_rating();
    let glicko2_rating = glicko2.create_rating();
    let ts_rating = trueskill.create_rating();

    // Verify different default scales
    assert_eq!(elo_rating.mean(), 1500.0);
    assert_eq!(glicko_rating.mean(), 1500.0);
    assert_eq!(glicko2_rating.mean(), 1500.0);
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

/// Test rating trait consistency
#[test]
fn test_rating_trait_consistency() {
    let elo_rating = EloRating::new(1600.0);
    let glicko_rating = GlickoRating::new(1600.0, 100.0);
    let glicko2_rating = Glicko2Rating::new(1600.0, 100.0, 0.06);
    let ts_rating = TrueSkillRating::new(1600.0, 10000.0).unwrap();

    // All should implement Rating trait consistently
    assert_eq!(elo_rating.mean(), 1600.0);
    assert_eq!(glicko_rating.mean(), 1600.0);
    assert_eq!(glicko2_rating.mean(), 1600.0);
    assert_eq!(ts_rating.mean(), 1600.0);

    // Standard deviation calculations
    assert_eq!(elo_rating.standard_deviation(), 0.0);
    assert_eq!(glicko_rating.standard_deviation(), 100.0);
    assert_eq!(glicko2_rating.standard_deviation(), 100.0);
    assert_eq!(ts_rating.standard_deviation(), 100.0);

    // Conservative ratings use different multipliers
    assert_eq!(elo_rating.conservative_rating(), 1600.0); // No uncertainty
    assert_eq!(glicko_rating.conservative_rating(), 1400.0); // μ - 2σ
    assert_eq!(glicko2_rating.conservative_rating(), 1400.0); // μ - 2σ
    assert_eq!(ts_rating.conservative_rating(), 1300.0); // μ - 3σ
}
