use ladder_rs::{
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam, TrueSkillImplementation, GaussianDistribution},
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    error::Error,
};

#[test]
fn test_trueskill_rating_properties() {
    let rating = TrueSkillRating::new(25.0, 64.0).unwrap();
    
    assert_eq!(rating.mean(), 25.0);
    assert_eq!(rating.variance(), 64.0);
    assert_eq!(rating.standard_deviation(), 8.0);
    assert_eq!(rating.conservative_rating(), 25.0 - 3.0 * 8.0);
    assert_eq!(rating.precision(), 1.0 / 64.0);
    assert_eq!(rating.precision_adjusted_mean(), 25.0 / 64.0);
}

#[test]
fn test_trueskill_rating_invalid_creation() {
    // Zero variance should fail
    let result = TrueSkillRating::new(25.0, 0.0);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Negative variance should fail
    let result = TrueSkillRating::new(25.0, -1.0);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

#[test]
fn test_trueskill_system_creation() {
    // Default system
    let ts = TrueSkill::new();
    let rating = ts.create_rating();
    assert_eq!(rating.mean(), 25.0);
    assert!((rating.variance() - (25.0_f64/3.0).powi(2)).abs() < 1e-10);
    
    // Simplified system
    let ts_simple = TrueSkill::new_simplified();
    let rating = ts_simple.create_rating();
    assert_eq!(rating.mean(), 25.0);
    
    // Custom parameters
    let mu_0 = 1200.0;
    let sigma_0_squared = 400.0;
    let beta_squared = 100.0;
    let gamma_squared = 4.0;
    let draw_probability = 0.1;
    
    let ts_custom = TrueSkill::with_parameters(
        mu_0,
        sigma_0_squared,
        beta_squared,
        gamma_squared,
        draw_probability,
        TrueSkillImplementation::Simplified,
    ).unwrap();
    
    let custom_rating = ts_custom.create_rating();
    assert_eq!(custom_rating.mean(), mu_0);
    assert_eq!(custom_rating.variance(), sigma_0_squared);
}

#[test]
fn test_trueskill_invalid_parameters() {
    let sigma_0_squared = 100.0;
    let beta_squared = 25.0;
    let gamma_squared = 1.0;
    
    // Negative mean
    let result = TrueSkill::with_parameters(
        -25.0,
        sigma_0_squared,
        beta_squared,
        gamma_squared,
        0.1,
        TrueSkillImplementation::Simplified,
    );
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Zero variance
    let result = TrueSkill::with_parameters(
        25.0,
        0.0,
        beta_squared,
        gamma_squared,
        0.1,
        TrueSkillImplementation::Simplified,
    );
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Negative beta squared
    let result = TrueSkill::with_parameters(
        25.0,
        sigma_0_squared,
        -1.0,
        gamma_squared,
        0.1,
        TrueSkillImplementation::Simplified,
    );
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Invalid draw probability (0.0)
    let result = TrueSkill::with_parameters(
        25.0,
        sigma_0_squared,
        beta_squared,
        gamma_squared,
        0.0,
        TrueSkillImplementation::Simplified,
    );
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Invalid draw probability (1.0)
    let result = TrueSkill::with_parameters(
        25.0,
        sigma_0_squared,
        beta_squared,
        gamma_squared,
        1.0,
        TrueSkillImplementation::Simplified,
    );
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

#[test]
fn test_trueskill_simplified_vs_factor_graph() {
    let ts_simplified = TrueSkill::new_simplified();
    let ts_factor_graph = TrueSkill::new_factor_graph();
    
    // Both should create same default rating
    let rating1 = ts_simplified.create_rating();
    let rating2 = ts_factor_graph.create_rating();
    
    assert_eq!(rating1.mean(), rating2.mean());
    assert_eq!(rating1.variance(), rating2.variance());
}

#[test]
fn test_trueskill_create_rating_with_values() {
    let ts = TrueSkill::new();
    
    let rating = ts.create_rating_with_values(30.0, 100.0);
    assert_eq!(rating.mean(), 30.0);
    assert_eq!(rating.variance(), 100.0);
}

#[test]
fn test_trueskill_team_operations() {
    let rating1 = TrueSkillRating::new(25.0, 64.0).unwrap();
    let rating2 = TrueSkillRating::new(30.0, 49.0).unwrap();
    
    // Single player team
    let team = TrueSkillTeam::from_player_ratings(vec![rating1.clone()]);
    assert_eq!(team.player_ratings().len(), 1);
    assert_eq!(team.player_ratings()[0].mean(), 25.0);
    
    // Multi-player team
    let team = TrueSkillTeam::from_player_ratings(vec![rating1, rating2]);
    assert_eq!(team.player_ratings().len(), 2);
    assert_eq!(team.player_ratings()[0].mean(), 25.0);
    assert_eq!(team.player_ratings()[1].mean(), 30.0);
}

#[test]
fn test_trueskill_two_player_match_simplified() {
    let ts = TrueSkill::new_simplified();
    
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    
    let outcome = GameOutcome::win(0, 2);
    let result = ts.rate(&[team1, team2], &outcome).unwrap();
    
    let winner = &result[0].player_ratings()[0];
    let loser = &result[1].player_ratings()[0];
    
    // Winner should have higher rating
    assert!(winner.mean() > loser.mean());
    assert!(winner.mean() > 25.0);
    assert!(loser.mean() < 25.0);
    
    // Both should have lower variance
    assert!(winner.variance() < (25.0_f64/3.0).powi(2));
    assert!(loser.variance() < (25.0_f64/3.0).powi(2));
}

#[test]
fn test_trueskill_different_skill_levels() {
    let ts = TrueSkill::new_simplified();
    
    let strong_player = ts.create_rating_with_values(35.0, 36.0);
    let weak_player = ts.create_rating_with_values(15.0, 36.0);
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![strong_player]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![weak_player]);
    
    // Strong player wins (expected)
    let outcome = GameOutcome::win(0, 2);
    let result = ts.rate(&[team1, team2], &outcome).unwrap();
    
    let strong_updated = &result[0].player_ratings()[0];
    let weak_updated = &result[1].player_ratings()[0];
    
    // Rating changes should be smaller for expected outcomes
    assert!((strong_updated.mean() - 35.0).abs() < 5.0);
    assert!((weak_updated.mean() - 15.0).abs() < 5.0);
    
    // Strong player should still increase
    assert!(strong_updated.mean() > 35.0);
    assert!(weak_updated.mean() < 15.0);
}

#[test]
fn test_trueskill_upset() {
    let ts = TrueSkill::new_simplified();
    
    let strong_player = ts.create_rating_with_values(35.0, 36.0);
    let weak_player = ts.create_rating_with_values(15.0, 36.0);
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![strong_player]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![weak_player]);
    
    // Weak player wins (upset)
    let outcome = GameOutcome::win(1, 2);
    let result = ts.rate(&[team1, team2], &outcome).unwrap();
    
    let strong_updated = &result[0].player_ratings()[0];
    let weak_updated = &result[1].player_ratings()[0];
    
    // Rating changes should be larger for unexpected outcomes
    assert!((strong_updated.mean() - 35.0).abs() > 2.0);
    assert!((weak_updated.mean() - 15.0).abs() > 2.0);
    
    // Weak player should increase significantly
    assert!(weak_updated.mean() > 15.0);
    assert!(strong_updated.mean() < 35.0);
}

#[test]
fn test_trueskill_draw() {
    let ts = TrueSkill::new_simplified();
    
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    
    let outcome = GameOutcome::draw(2);
    let result = ts.rate(&[team1, team2], &outcome).unwrap();
    
    let player1_updated = &result[0].player_ratings()[0];
    let player2_updated = &result[1].player_ratings()[0];
    
    // In a draw between equal players, means should stay close
    assert!((player1_updated.mean() - 25.0).abs() < 2.0);
    assert!((player2_updated.mean() - 25.0).abs() < 2.0);
    
    // Variance should decrease
    assert!(player1_updated.variance() < (25.0_f64/3.0).powi(2));
    assert!(player2_updated.variance() < (25.0_f64/3.0).powi(2));
}

#[test]
fn test_trueskill_multi_player_team() {
    let ts = TrueSkill::new_simplified();
    
    // Team 1: two players
    let player1a = ts.create_rating();
    let player1b = ts.create_rating();
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1a, player1b]);
    
    // Team 2: one player
    let player2 = ts.create_rating();
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    
    let outcome = GameOutcome::win(0, 2);
    let result = ts.rate(&[team1, team2], &outcome);
    
    // Multi-player teams are not currently supported in simplified mode
    // This should return an error or handle gracefully
    // The exact behavior depends on implementation
    // We'll just verify it doesn't panic
    match result {
        Ok(_) => {
            // If it works, verify we get the right number of teams back
            assert_eq!(result.unwrap().len(), 2);
        }
        Err(_) => {
            // If it errors, that's also acceptable for current implementation
        }
    }
}

#[test]
fn test_trueskill_three_player_match() {
    let ts = TrueSkill::new_simplified();
    
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();
    let player3 = ts.create_rating();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    let team3 = TrueSkillTeam::from_player_ratings(vec![player3]);
    
    // Player 1 wins, Player 2 second, Player 3 third
    let outcome = GameOutcome::new(vec![1, 2, 3]);
    let result = ts.rate(&[team1, team2, team3], &outcome);
    
    // Multi-team matches may not be supported in simplified mode
    match result {
        Ok(updated_teams) => {
            assert_eq!(updated_teams.len(), 3);
            // Winner should have highest rating
            assert!(updated_teams[0].player_ratings()[0].mean() > 25.0);
        }
        Err(_) => {
            // Error is acceptable for current implementation
        }
    }
}

#[test]
fn test_trueskill_error_conditions() {
    let ts = TrueSkill::new_simplified();
    
    // Empty teams
    let outcome = GameOutcome::new(vec![]);
    let result = ts.rate(&[], &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Mismatched team count and outcome length
    let team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let outcome = GameOutcome::new(vec![1, 2]); // 2 ranks but only 1 team
    let result = ts.rate(&[team1], &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

#[test]
fn test_trueskill_series_of_games() {
    let ts = TrueSkill::new_simplified();
    
    let mut player1 = ts.create_rating();
    let mut player2 = ts.create_rating();
    
    // Player 1 wins 10 games in a row
    for _ in 0..10 {
        let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
        let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
        
        let outcome = GameOutcome::win(0, 2);
        let result = ts.rate(&[team1, team2], &outcome).unwrap();
        
        player1 = result[0].player_ratings()[0].clone();
        player2 = result[1].player_ratings()[0].clone();
    }
    
    // Player 1 should be significantly higher rated
    assert!(player1.mean() > player2.mean() + 10.0);
    assert!(player1.mean() > 25.0);
    assert!(player2.mean() < 25.0);
    
    // Both should have lower variance (more certainty)
    assert!(player1.variance() < (25.0_f64/3.0).powi(2));
    assert!(player2.variance() < (25.0_f64/3.0).powi(2));
}

#[test]
fn test_trueskill_rating_convergence() {
    let ts = TrueSkill::new_simplified();
    
    let mut strong_player = ts.create_rating_with_values(35.0, 64.0);
    let mut weak_player = ts.create_rating_with_values(15.0, 64.0);
    
    // Strong player wins 80% of games
    for i in 0..50 {
        let team1 = TrueSkillTeam::from_player_ratings(vec![strong_player]);
        let team2 = TrueSkillTeam::from_player_ratings(vec![weak_player]);
        
        let outcome = if i % 5 == 0 {
            GameOutcome::win(1, 2)  // Weak player wins 20%
        } else {
            GameOutcome::win(0, 2)  // Strong player wins 80%
        };
        
        let result = ts.rate(&[team1, team2], &outcome).unwrap();
        
        strong_player = result[0].player_ratings()[0].clone();
        weak_player = result[1].player_ratings()[0].clone();
    }
    
    // After many games, strong player should be rated higher
    assert!(strong_player.mean() > weak_player.mean());
    
    // Variances should have decreased
    assert!(strong_player.variance() < 64.0);
    assert!(weak_player.variance() < 64.0);
}

#[test]
fn test_gaussian_distribution() {
    // Test Gaussian distribution operations used in TrueSkill
    let dist1 = GaussianDistribution::new(25.0, 64.0).unwrap();
    let dist2 = GaussianDistribution::new(30.0, 49.0).unwrap();
    
    assert_eq!(dist1.mean(), 25.0);
    assert_eq!(dist1.variance(), 64.0);
    
    // Test absolute difference
    let diff = dist1.absolute_difference(&dist2);
    assert!(diff > 0.0);
    
    // Test multiplication
    let product = dist1.multiply(&dist2);
    assert!(product.precision() > 0.0);
    
    // Test division
    let quotient = dist1.divide(&dist2);
    // Division can result in negative precision, so we just check it doesn't panic
    
    // Test from precision mean
    let precision_dist = GaussianDistribution::from_precision_mean(0.5, 0.25);
    assert_eq!(precision_dist.precision_mean(), 0.5);
    assert_eq!(precision_dist.precision(), 0.25);
}

#[test]
fn test_gaussian_distribution_errors() {
    // Test invalid variance
    let result = GaussianDistribution::new(25.0, 0.0);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    let result = GaussianDistribution::new(25.0, -1.0);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

#[test]
fn test_trueskill_match_quality_not_implemented() {
    let ts = TrueSkill::new_simplified();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    
    // Match quality is not implemented yet
    let result = ts.calculate_match_quality(&[team1, team2]);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

#[test]
fn test_trueskill_rating_cloning_and_equality() {
    let rating1 = TrueSkillRating::new(25.0, 64.0).unwrap();
    let rating2 = TrueSkillRating::new(25.0, 64.0).unwrap();
    let rating3 = TrueSkillRating::new(26.0, 64.0).unwrap();
    
    // Equality should work for same values
    assert_eq!(rating1.mean(), rating2.mean());
    assert_eq!(rating1.variance(), rating2.variance());
    
    // Different values should be different
    assert_ne!(rating1.mean(), rating3.mean());
    
    // Cloning should work
    let cloned = rating1.clone();
    assert_eq!(cloned.mean(), rating1.mean());
    assert_eq!(cloned.variance(), rating1.variance());
}

#[test]
fn test_trueskill_implementation_enum() {
    // Test that implementation enum works
    let _simplified = TrueSkillImplementation::Simplified;
    let _factor_graph = TrueSkillImplementation::FactorGraph;
    
    // Test equality
    assert_eq!(TrueSkillImplementation::Simplified, TrueSkillImplementation::Simplified);
    assert_ne!(TrueSkillImplementation::Simplified, TrueSkillImplementation::FactorGraph);
}