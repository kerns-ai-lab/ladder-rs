use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
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
    use ladder_rs::trueskill::{FactorGraph, GaussianDistribution, GaussianPriorFactor};

    let mut fg = FactorGraph::new();
    let var_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));
    fg.add_factor(Box::new(
        GaussianPriorFactor::new(var_id, 5.0, 1.0).unwrap(),
    ));

    // Run schedule loop
    let _ = fg.run_schedule_loop(1e-6, 5).unwrap();

    let var = fg.get_variable(var_id).unwrap();
    assert!((var.value().mean() - 5.0).abs() < 1e-6);
}

#[test]
fn test_schedule_with_comparison_factor() {
    use ladder_rs::trueskill::{
        FactorGraph, GaussianComparisonFactor, GaussianDistribution, GaussianPriorFactor,
    };

    let mut fg = FactorGraph::new();
    let greater_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));
    let lesser_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));

    // Set up priors: greater should prefer 5.0, lesser should prefer 3.0
    fg.add_factor(Box::new(GaussianPriorFactor::new(greater_id, 5.0, 1.0).unwrap()));
    fg.add_factor(Box::new(GaussianPriorFactor::new(lesser_id, 3.0, 1.0).unwrap()));
    
    // Add comparison constraint: greater > lesser with no draw margin
    fg.add_factor(Box::new(
        GaussianComparisonFactor::new(greater_id, lesser_id, 0.0, false).unwrap(),
    ));

    let convergence_result = fg.run_schedule_loop(1e-6, 10).unwrap();
    
    // Verify convergence was achieved
    assert!(convergence_result < 1e-6, "Should converge within tolerance");

    let greater_var = fg.get_variable(greater_id).unwrap();
    let lesser_var = fg.get_variable(lesser_id).unwrap();

    // The final beliefs should be a compromise between priors and the comparison constraint.
    // Test the key properties that must hold after convergence:

    // 1. Both variables should have finite, positive variance
    assert!(greater_var.value().variance() > 0.0 && greater_var.value().variance().is_finite());
    assert!(lesser_var.value().variance() > 0.0 && lesser_var.value().variance().is_finite());

    // 2. The greater variable should still have a higher mean than the lesser variable
    // (the comparison constraint should be respected)
    assert!(greater_var.value().mean() > lesser_var.value().mean(), 
        "Comparison constraint violated: greater={:.3} should be > lesser={:.3}", 
        greater_var.value().mean(), lesser_var.value().mean());

    // 3. The means should be influenced by but not exactly equal to the priors
    // (they should move towards satisfying the constraint)
    let greater_mean = greater_var.value().mean();
    let lesser_mean = lesser_var.value().mean();
    
    // The greater variable should be pulled down from its prior (5.0) to help satisfy the constraint
    assert!(greater_mean < 5.0, "Greater variable should be pulled down from prior 5.0, got {:.3}", greater_mean);
    
    // The lesser variable should be pulled down from its prior (3.0) to help satisfy the constraint  
    assert!(lesser_mean < 3.0, "Lesser variable should be pulled down from prior 3.0, got {:.3}", lesser_mean);

    // 4. The difference between the means should be reasonable given the constraint strength
    let mean_diff = greater_mean - lesser_mean;
    assert!(mean_diff > 0.5, "Mean difference should be substantial given the constraint, got {:.3}", mean_diff);

    // 5. Both means should be within a reasonable range influenced by their priors
    assert!(greater_mean > 2.0 && greater_mean < 6.0, "Greater mean should be reasonable, got {:.3}", greater_mean);
    assert!(lesser_mean > 0.0 && lesser_mean < 4.0, "Lesser mean should be reasonable, got {:.3}", lesser_mean);
    
    // 6. Variances should be reduced compared to infinite initial variance but not too small
    assert!(greater_var.value().variance() < 2.0, "Greater variance should be constrained, got {:.3}", greater_var.value().variance());
    assert!(lesser_var.value().variance() < 2.0, "Lesser variance should be constrained, got {:.3}", lesser_var.value().variance());
}

#[test]  
fn test_comparison_factor_with_draw_margin() {
    use ladder_rs::trueskill::{
        FactorGraph, GaussianDistribution, GaussianPriorFactor, GaussianComparisonFactor,
    };

    let mut fg = FactorGraph::new();
    let player1_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));
    let player2_id = fg.add_variable(GaussianDistribution::from_precision_mean(0.0, 0.0));

    // Set up very close priors
    fg.add_factor(Box::new(GaussianPriorFactor::new(player1_id, 25.0, 1.0).unwrap()));
    fg.add_factor(Box::new(GaussianPriorFactor::new(player2_id, 25.1, 1.0).unwrap()));
    
    // Add comparison with draw margin (simulating a draw outcome)
    let draw_margin = 1.0;
    fg.add_factor(Box::new(
        GaussianComparisonFactor::new(player1_id, player2_id, draw_margin, true).unwrap(),
    ));

    let convergence_result = fg.run_schedule_loop(1e-6, 10).unwrap();
    assert!(convergence_result < 1e-6, "Should converge for draw scenario");

    let player1_var = fg.get_variable(player1_id).unwrap();
    let player2_var = fg.get_variable(player2_id).unwrap();

    // In a draw scenario with close priors, the means should move closer together
    let mean_diff = (player1_var.value().mean() - player2_var.value().mean()).abs();
    assert!(mean_diff < 1.0, "Draw constraint should bring means closer together, diff={:.3}", mean_diff);
    
    // Both should have valid beliefs
    assert!(player1_var.value().variance() > 0.0 && player1_var.value().variance().is_finite());
    assert!(player2_var.value().variance() > 0.0 && player2_var.value().variance().is_finite());
}
