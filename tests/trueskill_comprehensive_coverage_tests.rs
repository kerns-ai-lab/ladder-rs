use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    trueskill::{
        FactorGraph, GaussianDistribution, PriorFactor, TrueSkill, TrueSkillImplementation,
        TrueSkillRating, TrueSkillTeam,
    },
};

#[test]
fn test_trueskill_system_default_creation() {
    let ts = TrueSkill::new();
    let rating = ts.create_rating();

    // Check default parameters
    assert_eq!(rating.mean(), 25.0);
    assert!((rating.variance() - (25.0 / 3.0).powi(2)).abs() < 1e-10);

    // Test that we can create a custom rating
    let custom_rating = ts.create_rating_with_values(30.0, 100.0);
    assert_eq!(custom_rating.mean(), 30.0);
    assert_eq!(custom_rating.variance(), 100.0);
}

#[test]
fn test_trueskill_system_simplified_creation() {
    let ts = TrueSkill::new_simplified();
    assert_eq!(ts.implementation(), TrueSkillImplementation::Simplified);

    let rating = ts.create_rating();
    assert_eq!(rating.mean(), 25.0);
}

#[test]
fn test_trueskill_system_factor_graph_creation() {
    let ts = TrueSkill::new_factor_graph();
    assert_eq!(ts.implementation(), TrueSkillImplementation::FactorGraph);

    let rating = ts.create_rating();
    assert_eq!(rating.mean(), 25.0);
}

#[test]
fn test_trueskill_custom_parameters() {
    let result = TrueSkill::with_parameters(
        1500.0,  // mu_0
        200.0,   // sigma_0_squared
        100.0,   // beta_squared
        10.0,    // gamma_squared
        0.05,    // draw_probability
        TrueSkillImplementation::Simplified,
    );

    assert!(result.is_ok());
    let ts = result.unwrap();

    let rating = ts.create_rating();
    assert_eq!(rating.mean(), 1500.0);
    assert_eq!(rating.variance(), 200.0);
}

#[test]
fn test_trueskill_invalid_parameters() {
    // Invalid mu_0
    let result = TrueSkill::with_parameters(
        -1.0,
        100.0,
        50.0,
        5.0,
        0.1,
        TrueSkillImplementation::Simplified,
    );
    assert!(result.is_err());

    // Invalid sigma_0_squared
    let result = TrueSkill::with_parameters(
        25.0,
        -100.0,
        50.0,
        5.0,
        0.1,
        TrueSkillImplementation::Simplified,
    );
    assert!(result.is_err());

    // Invalid beta_squared
    let result = TrueSkill::with_parameters(
        25.0,
        100.0,
        -50.0,
        5.0,
        0.1,
        TrueSkillImplementation::Simplified,
    );
    assert!(result.is_err());

    // Invalid gamma_squared
    let result = TrueSkill::with_parameters(
        25.0,
        100.0,
        50.0,
        -5.0,
        0.1,
        TrueSkillImplementation::Simplified,
    );
    assert!(result.is_err());

    // Invalid draw_probability (0.0)
    let result = TrueSkill::with_parameters(
        25.0,
        100.0,
        50.0,
        5.0,
        0.0,
        TrueSkillImplementation::Simplified,
    );
    assert!(result.is_err());

    // Invalid draw_probability (1.0)
    let result = TrueSkill::with_parameters(
        25.0,
        100.0,
        50.0,
        5.0,
        1.0,
        TrueSkillImplementation::Simplified,
    );
    assert!(result.is_err());

    // Invalid draw_probability (> 1.0)
    let result = TrueSkill::with_parameters(
        25.0,
        100.0,
        50.0,
        5.0,
        1.5,
        TrueSkillImplementation::Simplified,
    );
    assert!(result.is_err());
}

#[test]
fn test_trueskill_implementation_switching() {
    let mut ts = TrueSkill::new_simplified();
    assert_eq!(ts.implementation(), TrueSkillImplementation::Simplified);

    ts.set_implementation(TrueSkillImplementation::FactorGraph);
    assert_eq!(ts.implementation(), TrueSkillImplementation::FactorGraph);

    ts.set_implementation(TrueSkillImplementation::Simplified);
    assert_eq!(ts.implementation(), TrueSkillImplementation::Simplified);
}

#[test]
fn test_trueskill_convergence_parameters() {
    let mut ts = TrueSkill::new_factor_graph();
    let (threshold, max_iter) = ts.convergence_parameters();

    // Default values
    assert_eq!(threshold, 0.001);
    assert_eq!(max_iter, 50);

    // Set custom values
    ts.set_convergence_parameters(0.0001, 100);
    let (new_threshold, new_max_iter) = ts.convergence_parameters();
    assert_eq!(new_threshold, 0.0001);
    assert_eq!(new_max_iter, 100);
}

#[test]
fn test_trueskill_rating_creation_and_properties() {
    let valid_rating = TrueSkillRating::new(25.0, 64.0);
    assert!(valid_rating.is_ok());

    let rating = valid_rating.unwrap();
    assert_eq!(rating.mean(), 25.0);
    assert_eq!(rating.std_dev(), 8.0); // sqrt(64)
    assert_eq!(rating.mean(), 25.0);
    assert_eq!(rating.variance(), 64.0);
    assert_eq!(rating.std_dev(), 8.0);
    assert_eq!(rating.conservative_rating(), 25.0 - 3.0 * 8.0); // μ - 3σ

    // precision() and precision_adjusted_mean() are not available on TrueSkillRating

    // Test invalid rating creation
    let invalid_rating = TrueSkillRating::new(25.0, -1.0);
    assert!(
        invalid_rating.is_err(),
        "Negative variance should be rejected"
    );

    let zero_variance = TrueSkillRating::new(25.0, 0.0);
    assert!(zero_variance.is_err(), "Zero variance should be rejected");
}

#[test]
fn test_trueskill_team_operations() {
    let rating1 = TrueSkillRating::new(25.0, 64.0).unwrap();
    let rating2 = TrueSkillRating::new(30.0, 81.0).unwrap();

    // Test team creation
    let team = TrueSkillTeam::from_player_ratings(vec![rating1.clone(), rating2.clone()]);
    assert_eq!(team.player_ratings().len(), 2);
    assert_eq!(team.player_ratings().len(), 2);

    // Test TeamRating trait implementation
    let team_from_ratings = TrueSkillTeam::from_player_ratings(vec![rating1, rating2]);
    assert_eq!(team_from_ratings.player_ratings().len(), 2);
    assert_eq!(team_from_ratings.player_ratings()[0].mean(), 25.0);
    assert_eq!(team_from_ratings.player_ratings()[1].mean(), 30.0);

    // Test team statistics
    assert_eq!(team_from_ratings.team_mean(), 25.0 + 30.0);
    assert_eq!(team_from_ratings.team_variance(), 64.0 + 81.0);
}

#[test]
fn test_gaussian_distribution_creation() {
    // Valid creation
    let dist = GaussianDistribution::new(25.0, 64.0);
    assert!(dist.is_ok());

    let d = dist.unwrap();
    assert_eq!(d.mean(), 25.0);
    assert_eq!(d.variance(), 64.0);
    assert_eq!(d.precision(), 1.0 / 64.0);
    assert_eq!(d.precision_mean(), 25.0 / 64.0);

    // Invalid creation
    let invalid_dist = GaussianDistribution::new(25.0, 0.0);
    assert!(invalid_dist.is_err());

    let negative_var = GaussianDistribution::new(25.0, -1.0);
    assert!(negative_var.is_err());
}

#[test]
fn test_gaussian_distribution_from_precision_mean() {
    let dist = GaussianDistribution::from_precision_mean(0.5, 0.25);
    assert_eq!(dist.precision_mean(), 0.5);
    assert_eq!(dist.precision(), 0.25);
    assert_eq!(dist.mean(), 0.5 / 0.25); // 2.0
    assert_eq!(dist.variance(), 1.0 / 0.25); // 4.0
}

#[test]
fn test_gaussian_distribution_operations() {
    let dist1 = GaussianDistribution::new(25.0, 64.0).unwrap();
    let dist2 = GaussianDistribution::new(30.0, 100.0).unwrap();

    // Test absolute difference
    let diff = dist1.absolute_difference(&dist2);
    assert!(diff > 0.0);

    // Test multiplication
    let product = dist1.multiply(&dist2);
    assert!(product.precision() > 0.0);
    assert!(product.precision() > dist1.precision());
    assert!(product.precision() > dist2.precision());

    // Test division
    let quotient = dist1.divide(&dist2);
    assert!(quotient.precision() < dist1.precision());
}

#[test]
fn test_gaussian_distribution_edge_cases() {
    // Test with infinite variance
    let dist = GaussianDistribution::from_mean_and_variance(25.0, f64::INFINITY);
    assert_eq!(dist.mean(), 25.0);
    assert!(dist.variance().is_infinite());
    assert_eq!(dist.precision(), 0.0);

    // Test with very small variance
    let small_var_dist = GaussianDistribution::new(25.0, 1e-10).unwrap();
    assert_eq!(small_var_dist.mean(), 25.0);
    assert!(small_var_dist.precision() > 1e9);
}

#[test]
fn test_factor_graph_basic_operations() {
    let mut fg = FactorGraph::new();

    // Add variables
    let var1_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));
    let var2_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));

    assert_eq!(var1_id, 0);
    assert_eq!(var2_id, 1);

    // Get variables
    let var1 = fg.get_variable(var1_id);
    assert!(var1.is_ok());
    assert_eq!(var1.unwrap().value().precision(), 0.0);

    // Invalid variable ID
    let invalid_var = fg.get_variable(999);
    assert!(invalid_var.is_err());
}

#[test]
fn test_prior_factor() {
    let var_id = 0;
    let prior = PriorFactor::new(var_id, 25.0, 64.0);
    assert!(prior.is_ok());

    let mut factor = prior.unwrap();
    assert_eq!(factor.connected_variables(), vec![var_id]);

    // Test message operations
    let msg = factor.message_to(var_id);
    assert!(msg.is_ok());

    // Invalid variable ID should error
    let invalid_msg = factor.message_to(999);
    assert!(invalid_msg.is_err());
}

#[test]
fn test_factor_graph_with_prior() {
    let mut fg = FactorGraph::new();
    let var_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));

    let prior = PriorFactor::new(var_id, 25.0, 64.0).unwrap();
    fg.add_factor(Box::new(prior));

    // Run schedule loop
    let result = fg.run_schedule_loop(1e-6, 10);
    assert!(result.is_ok());

    // Check that variable has been updated
    let var = fg.get_variable(var_id).unwrap();
    assert!((var.value().mean() - 25.0).abs() < 1e-6);
    assert!((var.value().variance() - 64.0).abs() < 1e-6);
}

#[test]
fn test_trueskill_rating_to_gaussian() {
    let rating = TrueSkillRating::new(25.0, 64.0).unwrap();
    let gaussian = rating.to_gaussian();
    assert!(gaussian.is_ok());

    let g = gaussian.unwrap();
    assert_eq!(g.mean(), 25.0);
    assert_eq!(g.variance(), 64.0);
}

#[test]
fn test_trueskill_rating_from_mean_and_std_dev() {
    let rating = TrueSkillRating::from_mean_and_std_dev(25.0, 8.0);
    assert!(rating.is_ok());

    let r = rating.unwrap();
    assert_eq!(r.mean(), 25.0);
    assert_eq!(r.std_dev(), 8.0);
    assert_eq!(r.variance(), 64.0);

    // Invalid std dev
    let invalid = TrueSkillRating::from_mean_and_std_dev(25.0, -1.0);
    assert!(invalid.is_err());

    let zero_std = TrueSkillRating::from_mean_and_std_dev(25.0, 0.0);
    assert!(zero_std.is_err());
}

#[test]
fn test_trueskill_simple_two_player_rating() {
    let ts = TrueSkill::new_simplified();

    let player1 = ts.create_rating();
    let player2 = ts.create_rating();

    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);

    // Test win outcome
    let outcome = GameOutcome::win(0, 2);
    let result = ts.rate(&[team1.clone(), team2.clone()], &outcome);
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.len(), 2);

    let winner = &updated[0].player_ratings()[0];
    let loser = &updated[1].player_ratings()[0];

    assert!(winner.mean() > 25.0);
    assert!(loser.mean() < 25.0);
    assert!(winner.variance() < (25.0 / 3.0).powi(2));
    assert!(loser.variance() < (25.0 / 3.0).powi(2));

    // Test draw outcome
    let draw_outcome = GameOutcome::draw(2);
    let draw_result = ts.rate(&[team1, team2], &draw_outcome);
    assert!(draw_result.is_ok());

    let draw_updated = draw_result.unwrap();
    let p1_draw = &draw_updated[0].player_ratings()[0];
    let p2_draw = &draw_updated[1].player_ratings()[0];

    // In a draw between equal players, means should stay close
    assert!((p1_draw.mean() - 25.0).abs() < 2.0);
    assert!((p2_draw.mean() - 25.0).abs() < 2.0);
}

#[test]
fn test_trueskill_rating_system_trait() {
    let ts = TrueSkill::new();

    // Test create_rating
    let rating = ts.create_rating();
    assert_eq!(rating.mean(), 25.0);

    // Test create_rating_with_values
    let custom = ts.create_rating_with_values(30.0, 100.0);
    assert_eq!(custom.mean(), 30.0);
    assert_eq!(custom.variance(), 100.0);

    // Test rate with invalid inputs
    let empty_teams: Vec<TrueSkillTeam> = vec![];
    let outcome = GameOutcome::new(vec![]);
    let result = ts.rate(&empty_teams, &outcome);
    assert!(result.is_err());

    // Test with mismatched teams and ranks
    let team = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let wrong_outcome = GameOutcome::new(vec![1, 2]); // 2 ranks for 1 team
    let result = ts.rate(&[team], &wrong_outcome);
    assert!(result.is_err());
}

#[test]
fn test_trueskill_implementation_enum() {
    let simplified = TrueSkillImplementation::Simplified;
    let factor_graph = TrueSkillImplementation::FactorGraph;

    // Test Debug trait
    assert_eq!(format!("{:?}", simplified), "Simplified");
    assert_eq!(format!("{:?}", factor_graph), "FactorGraph");

    // Test Clone trait
    let cloned = simplified.clone();
    assert_eq!(cloned, TrueSkillImplementation::Simplified);

    // Test PartialEq
    assert_eq!(simplified, TrueSkillImplementation::Simplified);
    assert_ne!(simplified, factor_graph);
}

#[test]
fn test_trueskill_match_quality_not_implemented() {
    let ts = TrueSkill::new();

    let team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);

    let result = ts.calculate_match_quality(&[team1, team2]);
    assert!(result.is_err());
}

#[test]
fn test_trueskill_factor_graph_rating() {
    let ts = TrueSkill::new_factor_graph();

    let player1 = ts.create_rating();
    let player2 = ts.create_rating();

    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);

    let outcome = GameOutcome::win(0, 2);
    let result = ts.rate(&[team1, team2], &outcome);
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.len(), 2);

    // Should produce valid ratings
    let winner = &updated[0].player_ratings()[0];
    let loser = &updated[1].player_ratings()[0];

    assert!(winner.variance() > 0.0);
    assert!(loser.variance() > 0.0);
}

#[test]
fn test_trueskill_multi_team_error() {
    let ts = TrueSkill::new_simplified();

    let team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let team3 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);

    let outcome = GameOutcome::new(vec![1, 2, 3]);
    let result = ts.rate(&[team1, team2, team3], &outcome);

    // Simplified implementation only supports 2 teams
    assert!(result.is_err());
}

#[test]
fn test_trueskill_rating_cloning() {
    let rating1 = TrueSkillRating::new(25.0, 64.0).unwrap();
    let rating2 = rating1.clone();

    assert_eq!(rating1.mean(), rating2.mean());
    assert_eq!(rating1.variance(), rating2.variance());
}

#[test]
fn test_trueskill_team_cloning() {
    let rating = TrueSkillRating::new(25.0, 64.0).unwrap();
    let team1 = TrueSkillTeam::from_player_ratings(vec![rating]);
    let team2 = team1.clone();

    assert_eq!(team1.player_ratings().len(), team2.player_ratings().len());
    assert_eq!(
        team1.player_ratings()[0].mean(),
        team2.player_ratings()[0].mean()
    );
}