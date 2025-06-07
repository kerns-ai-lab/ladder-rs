use ladder_rs::{
    core::{GameOutcome, RatingSystem, TeamRating},
    trueskill::{TrueSkill, TrueSkillImplementation, TrueSkillRating, TrueSkillTeam},
};

#[test]
fn test_factor_graph_convergence() {
    let ts = TrueSkill::new_factor_graph();
    
    // Create two players with default ratings
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    
    // Player 1 wins
    let outcome = GameOutcome::new(vec![1, 2]);
    
    // Rate using factor graph
    let result = ts.rate(&[team1, team2], &outcome);
    assert!(result.is_ok());
    
    let updated_teams = result.unwrap();
    assert_eq!(updated_teams.len(), 2);
    
    // Winner should have higher rating than loser
    let winner_rating = &updated_teams[0].player_ratings()[0];
    let loser_rating = &updated_teams[1].player_ratings()[0];
    
    assert!(winner_rating.mean() > loser_rating.mean());
}

#[test]
fn test_factor_graph_vs_simplified() {
    // Test that factor graph gives similar results to simplified implementation
    let ts_simplified = TrueSkill::new_simplified();
    let ts_factor_graph = TrueSkill::new_factor_graph();
    
    let initial_rating1 = TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap();
    let initial_rating2 = TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![initial_rating1.clone()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![initial_rating2.clone()]);
    
    let outcome = GameOutcome::new(vec![1, 2]);
    
    // Rate with both implementations
    let simplified_result = ts_simplified.rate(&[team1.clone(), team2.clone()], &outcome).unwrap();
    let factor_graph_result = ts_factor_graph.rate(&[team1, team2], &outcome).unwrap();
    
    // Results should be reasonably close (within 10% for this simple case)
    let simplified_winner = &simplified_result[0].player_ratings()[0];
    let factor_graph_winner = &factor_graph_result[0].player_ratings()[0];
    
    let mean_diff = (simplified_winner.mean() - factor_graph_winner.mean()).abs();
    let variance_diff = (simplified_winner.variance() - factor_graph_winner.variance()).abs();
    
    // Allow some difference due to different computational approaches
    assert!(mean_diff < 2.0, "Mean difference too large: {}", mean_diff);
    assert!(variance_diff < 5.0, "Variance difference too large: {}", variance_diff);
}

#[test]
fn test_convergence_threshold() {
    // Test that the convergence implementation respects threshold settings
    let ts_loose = TrueSkill::with_parameters(
        25.0,
        (25.0/3.0).powi(2),
        (25.0/6.0).powi(2),
        (25.0/300.0).powi(2),
        0.1,
        TrueSkillImplementation::FactorGraph,
    ).unwrap();
    
    let player1 = ts_loose.create_rating();
    let player2 = ts_loose.create_rating();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    
    let outcome = GameOutcome::new(vec![1, 2]);
    
    // Should complete without error (convergence reached)
    let result = ts_loose.rate(&[team1, team2], &outcome);
    assert!(result.is_ok());
}

#[test]
fn test_gaussian_distribution_absolute_difference() {
    use ladder_rs::trueskill::GaussianDistribution;
    
    let dist1 = GaussianDistribution::new(25.0, 64.0).unwrap(); // σ=8
    let dist2 = GaussianDistribution::new(30.0, 81.0).unwrap(); // σ=9
    
    // Test absolute difference calculation following CONVERGENCE.md guidance
    let diff = dist1.absolute_difference(&dist2);
    
    // Should be max of precision_mean_diff and sqrt(precision_diff)
    let precision1 = 1.0 / 64.0;
    let precision2 = 1.0 / 81.0;
    let precision_mean1 = precision1 * 25.0;
    let precision_mean2 = precision2 * 30.0;
    
    let precision_mean_diff = (precision_mean1 - precision_mean2).abs();
    let precision_diff = (precision1 - precision2).abs().sqrt();
    let expected = precision_mean_diff.max(precision_diff);
    
    assert!((diff - expected).abs() < 1e-10);
}