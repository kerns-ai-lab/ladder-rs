use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    trueskill::{
        TrueSkill, TrueSkillImplementation, TrueSkillRating, TrueSkillTeam,
        GaussianDistribution, FactorGraph, GaussianPriorFactor,
    },
    error::Error,
};

#[test]
fn test_trueskill_custom_parameters() {
    // Test with custom parameters
    let custom_ts = TrueSkill::with_parameters(
        30.0,          // Higher initial mean
        100.0,         // Lower initial variance  
        25.0,          // Higher beta (more random)
        1.0,           // Higher gamma (more dynamic)
        0.05,          // Lower draw probability
        TrueSkillImplementation::Simplified,
    ).unwrap();
    
    let default_ts = TrueSkill::new();
    
    let player1 = custom_ts.create_rating();
    let player2 = custom_ts.create_rating();
    
    assert_eq!(player1.mean(), 30.0, "Custom mu_0 should be used");
    assert_eq!(player1.variance(), 100.0, "Custom sigma_0_squared should be used");
    
    // Test that custom and default systems produce different results
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    
    let outcome = GameOutcome::win(0, 2);
    
    let custom_result = custom_ts.rate(&[team1.clone(), team2.clone()], &outcome).unwrap();
    
    let default_player1 = default_ts.create_rating();
    let default_player2 = default_ts.create_rating();
    let default_team1 = TrueSkillTeam::from_player_ratings(vec![default_player1]);
    let default_team2 = TrueSkillTeam::from_player_ratings(vec![default_player2]);
    
    let default_result = default_ts.rate(&[default_team1, default_team2], &outcome).unwrap();
    
    // Results should differ due to different parameters
    let custom_winner = &custom_result[0].player_ratings()[0];
    let default_winner = &default_result[0].player_ratings()[0];
    
    assert_ne!(custom_winner.mean(), default_winner.mean(), 
               "Custom parameters should produce different results");
}

#[test]
fn test_trueskill_parameter_validation() {
    // Test invalid mu_0
    let result = TrueSkill::with_parameters(
        -1.0, 100.0, 25.0, 1.0, 0.1, TrueSkillImplementation::Simplified
    );
    assert!(result.is_err(), "Negative mu_0 should be rejected");
    
    // Test invalid sigma_0_squared
    let result = TrueSkill::with_parameters(
        25.0, -1.0, 25.0, 1.0, 0.1, TrueSkillImplementation::Simplified
    );
    assert!(result.is_err(), "Negative sigma_0_squared should be rejected");
    
    // Test invalid beta_squared
    let result = TrueSkill::with_parameters(
        25.0, 100.0, -1.0, 1.0, 0.1, TrueSkillImplementation::Simplified
    );
    assert!(result.is_err(), "Negative beta_squared should be rejected");
    
    // Test invalid gamma_squared
    let result = TrueSkill::with_parameters(
        25.0, 100.0, 25.0, -1.0, 0.1, TrueSkillImplementation::Simplified
    );
    assert!(result.is_err(), "Negative gamma_squared should be rejected");
    
    // Test invalid draw_probability (too low)
    let result = TrueSkill::with_parameters(
        25.0, 100.0, 25.0, 1.0, 0.0, TrueSkillImplementation::Simplified
    );
    assert!(result.is_err(), "Zero draw_probability should be rejected");
    
    // Test invalid draw_probability (too high)
    let result = TrueSkill::with_parameters(
        25.0, 100.0, 25.0, 1.0, 1.0, TrueSkillImplementation::Simplified
    );
    assert!(result.is_err(), "Draw_probability of 1.0 should be rejected");
}

#[test]
fn test_trueskill_draw_probability_effects() {
    // Test with very low draw probability
    let low_draw_ts = TrueSkill::with_parameters(
        25.0, (25.0/3.0).powi(2), (25.0/6.0).powi(2), (25.0/300.0).powi(2), 
        0.01, TrueSkillImplementation::Simplified
    ).unwrap();
    
    // Test with higher draw probability
    let high_draw_ts = TrueSkill::with_parameters(
        25.0, (25.0/3.0).powi(2), (25.0/6.0).powi(2), (25.0/300.0).powi(2), 
        0.3, TrueSkillImplementation::Simplified
    ).unwrap();
    
    let player1 = TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap();
    let player2 = TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1.clone()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2.clone()]);
    
    let draw_outcome = GameOutcome::draw(2);
    
    let low_draw_result = low_draw_ts.rate(&[team1.clone(), team2.clone()], &draw_outcome).unwrap();
    let high_draw_result = high_draw_ts.rate(&[team1, team2], &draw_outcome).unwrap();
    
    // Different draw probabilities should produce different rating updates
    let low_draw_rating = &low_draw_result[0].player_ratings()[0];
    let high_draw_rating = &high_draw_result[0].player_ratings()[0];
    
    // Both should remain close to initial for a draw, but with different variances
    assert!(low_draw_rating.variance() != high_draw_rating.variance(),
            "Different draw probabilities should affect variance updates");
}

#[test]
fn test_trueskill_rating_creation_and_properties() {
    // Test rating creation with validation
    let valid_rating = TrueSkillRating::new(25.0, 64.0);
    assert!(valid_rating.is_ok());
    
    let rating = valid_rating.unwrap();
    assert_eq!(rating.mu(), 25.0);
    assert_eq!(rating.sigma(), 8.0); // sqrt(64)
    assert_eq!(rating.mean(), 25.0);
    assert_eq!(rating.variance(), 64.0);
    assert_eq!(rating.standard_deviation(), 8.0);
    assert_eq!(rating.conservative_rating(), 25.0 - 3.0 * 8.0); // μ - 3σ
    
    // Test precision calculations
    assert_eq!(rating.precision(), 1.0 / 64.0);
    assert_eq!(rating.precision_adjusted_mean(), (1.0 / 64.0) * 25.0);
    
    // Test invalid rating creation
    let invalid_rating = TrueSkillRating::new(25.0, -1.0);
    assert!(invalid_rating.is_err(), "Negative variance should be rejected");
    
    let zero_variance = TrueSkillRating::new(25.0, 0.0);
    assert!(zero_variance.is_err(), "Zero variance should be rejected");
}

#[test]
fn test_trueskill_team_operations() {
    let rating1 = TrueSkillRating::new(25.0, 64.0).unwrap();
    let rating2 = TrueSkillRating::new(30.0, 81.0).unwrap();
    
    // Test team creation
    let team = TrueSkillTeam::new(vec![rating1.clone(), rating2.clone()]);
    assert_eq!(team.players().len(), 2);
    assert_eq!(team.player_ratings().len(), 2);
    
    // Test TeamRating trait implementation
    let team_from_ratings = TrueSkillTeam::from_player_ratings(vec![rating1, rating2]);
    assert_eq!(team_from_ratings.player_ratings().len(), 2);
    assert_eq!(team_from_ratings.player_ratings()[0].mean(), 25.0);
    assert_eq!(team_from_ratings.player_ratings()[1].mean(), 30.0);
}

#[test]
fn test_trueskill_error_conditions() {
    let ts = TrueSkill::new();
    
    // Test insufficient teams
    let team = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let outcome = GameOutcome::new(vec![1]);
    
    let result = ts.rate(&[team], &outcome);
    assert!(result.is_err(), "Single team should cause error");
    
    // Test mismatched ranks and teams
    let team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let bad_outcome = GameOutcome::new(vec![1, 2, 3]); // 3 ranks for 2 teams
    
    let result = ts.rate(&[team1, team2], &bad_outcome);
    assert!(result.is_err(), "Mismatched ranks should cause error");
}

#[test]
fn test_trueskill_implementation_differences() {
    let simplified_ts = TrueSkill::new_simplified();
    let factor_graph_ts = TrueSkill::new_factor_graph();
    
    // Verify implementation types
    assert_eq!(simplified_ts.implementation, TrueSkillImplementation::Simplified);
    assert_eq!(factor_graph_ts.implementation, TrueSkillImplementation::FactorGraph);
    
    let player1 = TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap();
    let player2 = TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1.clone()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2.clone()]);
    
    let outcome = GameOutcome::win(0, 2);
    
    // Both implementations should work
    let simplified_result = simplified_ts.rate(&[team1.clone(), team2.clone()], &outcome);
    assert!(simplified_result.is_ok(), "Simplified implementation should work");
    
    let factor_graph_result = factor_graph_ts.rate(&[team1, team2], &outcome);
    assert!(factor_graph_result.is_ok(), "Factor graph implementation should work");
}

#[test]
fn test_trueskill_rating_convergence() {
    let ts = TrueSkill::new();
    
    let mut player1 = TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap();
    let mut player2 = TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap();
    
    // Simulate player 1 consistently beating player 2
    for _ in 0..50 {
        let team1 = TrueSkillTeam::from_player_ratings(vec![player1.clone()]);
        let team2 = TrueSkillTeam::from_player_ratings(vec![player2.clone()]);
        
        let outcome = GameOutcome::win(0, 2);
        let result = ts.rate(&[team1, team2], &outcome).unwrap();
        
        player1 = result[0].player_ratings()[0].clone();
        player2 = result[1].player_ratings()[0].clone();
    }
    
    // After many games, ratings should converge with significant difference
    assert!(player1.mean() > 30.0, "Consistent winner should have higher rating");
    assert!(player2.mean() < 20.0, "Consistent loser should have lower rating");
    assert!(player1.variance() < (25.0/3.0).powi(2), "Variance should decrease with more games");
    assert!(player2.variance() < (25.0/3.0).powi(2), "Variance should decrease with more games");
}

#[test]
fn test_trueskill_upset_scenarios() {
    let ts = TrueSkill::new();
    
    // Create players with significantly different ratings
    let strong_player = TrueSkillRating::new(35.0, 25.0).unwrap(); // High skill, low uncertainty
    let weak_player = TrueSkillRating::new(15.0, 64.0).unwrap();   // Low skill, high uncertainty
    
    let strong_team = TrueSkillTeam::from_player_ratings(vec![strong_player.clone()]);
    let weak_team = TrueSkillTeam::from_player_ratings(vec![weak_player.clone()]);
    
    // Upset: weak player beats strong player
    let upset_outcome = GameOutcome::win(1, 2);
    let result = ts.rate(&[strong_team, weak_team], &upset_outcome).unwrap();
    
    let updated_strong = &result[0].player_ratings()[0];
    let updated_weak = &result[1].player_ratings()[0];
    
    // Strong player should lose significant rating
    assert!(updated_strong.mean() < strong_player.mean(), 
            "Strong player should lose rating in upset");
    
    // Weak player should gain significant rating  
    assert!(updated_weak.mean() > weak_player.mean(),
            "Weak player should gain rating in upset");
    
    // Weak player with higher uncertainty should have larger rating change
    let strong_change = (strong_player.mean() - updated_strong.mean()).abs();
    let weak_change = (updated_weak.mean() - weak_player.mean()).abs();
    
    assert!(weak_change > strong_change,
            "Player with higher uncertainty should have larger rating change");
}

#[test]
fn test_trueskill_draw_scenarios() {
    let ts = TrueSkill::new();
    
    // Test draw between equally matched players
    let player1 = TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap();
    let player2 = TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1.clone()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2.clone()]);
    
    let draw_outcome = GameOutcome::draw(2);
    let result = ts.rate(&[team1, team2], &draw_outcome).unwrap();
    
    let updated1 = &result[0].player_ratings()[0];
    let updated2 = &result[1].player_ratings()[0];
    
    // Equal players in a draw should have minimal rating change
    assert!((updated1.mean() - player1.mean()).abs() < 1.0,
            "Equal players should have minimal rating change in draw");
    assert!((updated2.mean() - player2.mean()).abs() < 1.0,
            "Equal players should have minimal rating change in draw");
    
    // Variance should still decrease (gained information)
    assert!(updated1.variance() < player1.variance(),
            "Variance should decrease after game");
    assert!(updated2.variance() < player2.variance(),
            "Variance should decrease after game");
}

#[test]
fn test_trueskill_gaussian_distribution() {
    // Test GaussianDistribution functionality
    let gaussian1 = GaussianDistribution::new(5.0, 2.0).unwrap();
    assert_eq!(gaussian1.mean(), 5.0);
    assert_eq!(gaussian1.variance(), 2.0);
    
    let gaussian2 = GaussianDistribution::new(3.0, 1.0).unwrap();
    
    // Test multiplication (product in precision form)
    let product = gaussian1.multiply(&gaussian2);
    assert!(product.mean() != gaussian1.mean(), "Multiplication should change distribution");
    
    // Test division
    let quotient = gaussian1.divide(&gaussian2);
    assert!(quotient.mean() != gaussian1.mean(), "Division should change distribution");
    
    // Test absolute difference
    let diff = gaussian1.absolute_difference(&gaussian2);
    assert!(diff > 0.0, "Different distributions should have positive difference");
    
    // Test from_mean_and_variance
    let gaussian3 = GaussianDistribution::from_mean_and_variance(10.0, 4.0);
    assert_eq!(gaussian3.mean(), 10.0);
    assert_eq!(gaussian3.variance(), 4.0);
    
    // Test infinite variance case
    let infinite_var = GaussianDistribution::from_mean_and_variance(0.0, f64::INFINITY);
    assert_eq!(infinite_var.precision(), 0.0);
    assert!(infinite_var.variance().is_infinite());
}

#[test]
fn test_trueskill_factor_graph_components() {
    // Test FactorGraph creation and basic operations
    let mut fg = FactorGraph::new();
    
    // Add variables
    let var1_id = fg.add_variable(GaussianDistribution::new(5.0, 1.0).unwrap());
    let var2_id = fg.add_variable(GaussianDistribution::new(3.0, 2.0).unwrap());
    
    // Add factors
    fg.add_factor(Box::new(GaussianPriorFactor::new(var1_id, 5.0, 1.0).unwrap()));
    fg.add_factor(Box::new(GaussianPriorFactor::new(var2_id, 3.0, 2.0).unwrap()));
    
    // Test variable retrieval
    let var1 = fg.get_variable(var1_id).unwrap();
    assert_eq!(var1.id(), var1_id);
    
    // Test schedule loop
    let result = fg.run_schedule_loop(1e-6, 10);
    assert!(result.is_ok(), "Schedule loop should converge");
}

#[test]
fn test_trueskill_system_defaults() {
    let ts = TrueSkill::new();
    let default_ts = TrueSkill::default();
    
    // Test that new() and default() create equivalent systems
    let rating1 = ts.create_rating();
    let rating2 = default_ts.create_rating();
    
    assert_eq!(rating1.mean(), rating2.mean());
    assert_eq!(rating1.variance(), rating2.variance());
    assert_eq!(rating1.mean(), 25.0);
    assert_eq!(rating1.variance(), (25.0/3.0).powi(2));
}

#[test]
fn test_trueskill_create_rating_with_values() {
    let ts = TrueSkill::new();
    
    // Test create_rating_with_values
    let custom_rating = ts.create_rating_with_values(30.0, 100.0);
    assert_eq!(custom_rating.mean(), 30.0);
    assert_eq!(custom_rating.variance(), 100.0);
}

#[test]
fn test_trueskill_match_quality_not_implemented() {
    let ts = TrueSkill::new();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    
    // Match quality calculation should return error (not yet implemented)
    let result = ts.calculate_match_quality(&[team1, team2]);
    assert!(result.is_err(), "Match quality calculation should not be implemented yet");
}