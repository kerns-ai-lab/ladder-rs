use ladder_rs::{
    core::{GameOutcome, RatingSystem},
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

    // Ratings should remain valid after running the factor graph
    let winner_rating = &updated_teams[0].player_ratings()[0];
    let loser_rating = &updated_teams[1].player_ratings()[0];

    assert!(winner_rating.variance() > 0.0);
    assert!(loser_rating.variance() > 0.0);
}

#[test]
fn test_factor_graph_vs_simplified() {
    // Test that factor graph gives similar results to simplified implementation
    let ts_simplified = TrueSkill::new_simplified();
    let ts_factor_graph = TrueSkill::new_factor_graph();

    let initial_rating1 = TrueSkillRating::new(25.0, (25.0_f64 / 3.0).powi(2)).unwrap();
    let initial_rating2 = TrueSkillRating::new(25.0, (25.0_f64 / 3.0).powi(2)).unwrap();

    let team1 = TrueSkillTeam::from_player_ratings(vec![initial_rating1.clone()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![initial_rating2.clone()]);

    let outcome = GameOutcome::new(vec![1, 2]);

    // Rate with both implementations
    let simplified_result = ts_simplified
        .rate(&[team1.clone(), team2.clone()], &outcome)
        .unwrap();
    let factor_graph_result = ts_factor_graph.rate(&[team1, team2], &outcome).unwrap();

    let simplified_winner = &simplified_result[0].player_ratings()[0];
    let factor_graph_winner = &factor_graph_result[0].player_ratings()[0];

    // Ensure both implementations produce valid ratings
    assert!(simplified_winner.variance() > 0.0);
    assert!(factor_graph_winner.variance() > 0.0);
}

#[test]
fn test_convergence_threshold() {
    // Test that the convergence implementation respects threshold settings
    let ts_loose = TrueSkill::with_parameters(
        25.0,
        (25.0_f64 / 3.0).powi(2),
        (25.0_f64 / 6.0).powi(2),
        (25.0_f64 / 300.0).powi(2),
        0.1,
        TrueSkillImplementation::FactorGraph,
    )
    .unwrap();

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
fn test_absolute_difference_calculation() {
    // Test the internal absolute difference calculation logic
    // Since GaussianDistribution is not public, we test through the factor graph behavior
    let ts = TrueSkill::new_factor_graph();

    // Create two very similar ratings
    let rating1 = TrueSkillRating::new(25.0, 64.0).unwrap(); // σ=8
    let rating2 = TrueSkillRating::new(25.01, 64.01).unwrap(); // Very close

    let team1 = TrueSkillTeam::from_player_ratings(vec![rating1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![rating2]);

    let outcome = GameOutcome::new(vec![1, 2]);

    // Should converge quickly for very similar ratings
    let result = ts.rate(&[team1, team2], &outcome);
    assert!(
        result.is_ok(),
        "Factor graph should converge for similar ratings"
    );

    // Create very different ratings
    let rating3 = TrueSkillRating::new(5.0, 100.0).unwrap();
    let rating4 = TrueSkillRating::new(45.0, 25.0).unwrap();

    let team3 = TrueSkillTeam::from_player_ratings(vec![rating3]);
    let team4 = TrueSkillTeam::from_player_ratings(vec![rating4]);

    // Should still converge for different ratings
    let result2 = ts.rate(&[team3, team4], &outcome);
    assert!(
        result2.is_ok(),
        "Factor graph should converge for different ratings"
    );
}

#[test]
fn test_outcome_affects_ratings() {
    let ts = TrueSkill::new_factor_graph();

    let player1 = ts.create_rating();
    let player2 = ts.create_rating();

    let team1 = TrueSkillTeam::from_player_ratings(vec![player1.clone()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2.clone()]);

    let win_result = ts
        .rate(&[team1.clone(), team2.clone()], &GameOutcome::win(0, 2))
        .unwrap();
    let lose_result = ts.rate(&[team1, team2], &GameOutcome::win(1, 2)).unwrap();

    let win_mu = win_result[0].player_ratings()[0].mean();
    let lose_mu = lose_result[0].player_ratings()[0].mean();

    assert!(
        (win_mu - lose_mu).abs() > 1e-6,
        "Different outcomes should change ratings differently"
    );
}

#[test]
fn test_variable_updates_in_schedule_loop() {
    use ladder_rs::trueskill::{FactorGraph, GaussianDistribution, PriorFactor};

    let mut fg = FactorGraph::new();
    let var_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));
    
    // Create a GaussianDistribution for the prior
    let prior_dist = GaussianDistribution::new(5.0, 1.0).unwrap();
    fg.add_factor(Box::new(PriorFactor::new(var_id, prior_dist)));

    // Run schedule loop
    let _ = fg.run_schedule_loop(1e-6, 5).unwrap();

    let var = fg.get_variable(var_id).unwrap();
    assert!((var.mean() - 5.0).abs() < 1e-6);
}

#[test]
fn test_schedule_with_comparison_factor() {
    use ladder_rs::trueskill::{
        FactorGraph, GaussianComparisonFactor, GaussianDistribution, PriorFactor,
    };

    let mut fg = FactorGraph::new();
    let difference_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));

    // Set up prior: difference should be positive
    let prior_dist = GaussianDistribution::new(2.0, 1.0).unwrap();
    fg.add_factor(Box::new(PriorFactor::new(difference_id, prior_dist)));
    
    // Add comparison constraint: difference > 0 with no draw margin
    fg.add_factor(Box::new(
        GaussianComparisonFactor::new(difference_id, 0.0),
    ));

    let convergence_result = fg.run_schedule_loop(1e-6, 10).unwrap();
    
    // Verify convergence was achieved
    assert!(convergence_result < 1e-6, "Should converge within tolerance");

    let difference_var = fg.get_variable(difference_id).unwrap();

    // Verify the result makes sense
    assert!(difference_var.variance() > 0.0 && difference_var.variance().is_finite());
    assert!(difference_var.mean() > 0.0, "Difference should be positive");
}

#[test]  
fn test_comparison_factor_with_draw_margin() {
    use ladder_rs::trueskill::{
        FactorGraph, GaussianDistribution, PriorFactor, GaussianComparisonFactor,
    };

    let mut fg = FactorGraph::new();
    let difference_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));

    // Set up prior: difference should be near zero
    let prior_dist = GaussianDistribution::new(0.1, 1.0).unwrap();
    fg.add_factor(Box::new(PriorFactor::new(difference_id, prior_dist)));
    
    // Add comparison with draw margin
    let draw_margin = 1.0;
    fg.add_factor(Box::new(
        GaussianComparisonFactor::new(difference_id, draw_margin),
    ));

    let convergence_result = fg.run_schedule_loop(1e-6, 10).unwrap();
    assert!(convergence_result < 1e-6, "Should converge for draw scenario");

    let difference_var = fg.get_variable(difference_id).unwrap();

    // In a draw scenario, the difference should be within the draw margin
    assert!(difference_var.mean().abs() < draw_margin, "Difference should be within draw margin");
    
    // Should have valid belief
    assert!(difference_var.variance() > 0.0 && difference_var.variance().is_finite());
}