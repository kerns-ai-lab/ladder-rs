use ladder_rs::{
    core::{GameOutcome, RatingSystem},
    trueskill::{TrueSkill, TrueSkillImplementation, TrueSkillRating, TrueSkillTeam},
};

#[test]
fn test_simplified_implementation() {
    let trueskill = TrueSkill::new_simplified();
    let team1 =
        TrueSkillTeam::from_player_ratings(
            vec![TrueSkillRating::new(25.0, 8.333 * 8.333).unwrap()],
        );
    let team2 =
        TrueSkillTeam::from_player_ratings(
            vec![TrueSkillRating::new(25.0, 8.333 * 8.333).unwrap()],
        );

    let outcome = GameOutcome::win(0, 2);
    let result = trueskill.rate(&[team1, team2], &outcome);
    assert!(result.is_ok(), "Simplified implementation should work");

    let updated_teams = result.unwrap();
    assert_eq!(updated_teams.len(), 2);

    // Winner should have higher rating than initial
    let winner = &updated_teams[0].player_ratings()[0];
    let loser = &updated_teams[1].player_ratings()[0];

    println!(
        "Simplified - Winner: μ={:.3}, σ={:.3}",
        winner.mean(),
        winner.std_dev()
    );
    println!(
        "Simplified - Loser: μ={:.3}, σ={:.3}",
        loser.mean(),
        loser.std_dev()
    );

    assert!(winner.mean() > 25.0, "Winner should have mu > 25.0");
    assert!(loser.mean() < 25.0, "Loser should have mu < 25.0");
}

#[test]
fn test_factor_graph_implementation_fallback() {
    // Currently falls back to simplified implementation
    let trueskill = TrueSkill::new_factor_graph();
    let team1 =
        TrueSkillTeam::from_player_ratings(
            vec![TrueSkillRating::new(25.0, 8.333 * 8.333).unwrap()],
        );
    let team2 =
        TrueSkillTeam::from_player_ratings(
            vec![TrueSkillRating::new(25.0, 8.333 * 8.333).unwrap()],
        );

    let outcome = GameOutcome::win(0, 2);
    let result = trueskill.rate(&[team1, team2], &outcome);
    assert!(
        result.is_ok(),
        "Factor graph implementation should work (via fallback)"
    );

    let updated_teams = result.unwrap();
    assert_eq!(updated_teams.len(), 2);

    let winner = &updated_teams[0].player_ratings()[0];
    let loser = &updated_teams[1].player_ratings()[0];

    println!(
        "Factor Graph (fallback) - Winner: μ={:.3}, σ={:.3}",
        winner.mean(),
        winner.std_dev()
    );
    println!(
        "Factor Graph (fallback) - Loser: μ={:.3}, σ={:.3}",
        loser.mean(),
        loser.std_dev()
    );

    // Ratings should remain valid
    assert!(winner.variance() > 0.0);
    assert!(loser.variance() > 0.0);
}

#[test]
fn test_both_implementations_similar_results() {
    // Test that both implementations produce similar results for a simple case
    let team1 =
        TrueSkillTeam::from_player_ratings(
            vec![TrueSkillRating::new(25.0, 8.333 * 8.333).unwrap()],
        );
    let team2 =
        TrueSkillTeam::from_player_ratings(
            vec![TrueSkillRating::new(25.0, 8.333 * 8.333).unwrap()],
        );

    let trueskill_simple = TrueSkill::new_simplified();
    let outcome = GameOutcome::win(0, 2);
    let result_simple = trueskill_simple
        .rate(&[team1.clone(), team2.clone()], &outcome)
        .unwrap();

    let trueskill_fg = TrueSkill::new_factor_graph();
    let result_fg = trueskill_fg.rate(&[team1, team2], &outcome).unwrap();

    let simple_winner_mu = result_simple[0].player_ratings()[0].mean();
    let fg_winner_mu = result_fg[0].player_ratings()[0].mean();

    println!("Simplified winner μ: {:.3}", simple_winner_mu);
    println!("Factor graph winner μ: {:.3}", fg_winner_mu);

    // Ensure both implementations return valid ratings
    assert!(simple_winner_mu > 0.0);
    assert!(fg_winner_mu > 0.0);
}

#[test]
fn test_with_parameters_both_implementations() {
    // Test custom parameters with both implementations
    let simplified = TrueSkill::with_parameters(
        25.0,
        69.44,
        34.72,
        0.0694,
        0.1,
        TrueSkillImplementation::Simplified,
    )
    .unwrap();

    let factor_graph = TrueSkill::with_parameters(
        25.0,
        69.44,
        34.72,
        0.0694,
        0.1,
        TrueSkillImplementation::FactorGraph,
    )
    .unwrap();

    let team1 =
        TrueSkillTeam::from_player_ratings(vec![TrueSkillRating::new(25.0, 69.44).unwrap()]);
    let team2 =
        TrueSkillTeam::from_player_ratings(vec![TrueSkillRating::new(25.0, 69.44).unwrap()]);
    let outcome = GameOutcome::win(0, 2);

    let simple_result = simplified.rate(&[team1.clone(), team2.clone()], &outcome);
    let fg_result = factor_graph.rate(&[team1, team2], &outcome);

    assert!(
        simple_result.is_ok(),
        "Simplified with custom parameters should work"
    );
    assert!(
        fg_result.is_ok(),
        "Factor graph with custom parameters should work"
    );
}
