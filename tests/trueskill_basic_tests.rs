use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    trueskill::{TrueSkill, TrueSkillTeam},
};

#[test]
fn test_trueskill_creation() {
    let ts = TrueSkill::new();
    let rating = ts.create_rating();

    assert_eq!(rating.mean(), 25.0);
    assert!((rating.variance() - (25.0_f64 / 3.0).powi(2)).abs() < 1e-10);
}

#[test]
fn test_trueskill_basic_update_simplified() {
    let ts = TrueSkill::new_simplified();

    // Create two players with default ratings
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();

    // Create teams
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);

    // Create a game outcome where team 1 wins
    let outcome = GameOutcome::win(0, 2);

    // Update ratings
    let updated_teams = ts
        .rate(&[team1, team2], &outcome)
        .expect("Rating update should succeed");

    // Verify that we have two teams back
    assert_eq!(updated_teams.len(), 2);

    // Verify that the winner's rating increased and loser's decreased
    let winner_rating = &updated_teams[0].player_ratings()[0];
    let loser_rating = &updated_teams[1].player_ratings()[0];

    println!(
        "Simplified - Winner: μ={:.3}, σ²={:.3}",
        winner_rating.mean(),
        winner_rating.variance()
    );
    println!(
        "Simplified - Loser: μ={:.3}, σ²={:.3}",
        loser_rating.mean(),
        loser_rating.variance()
    );

    // Winner should have higher mean than initial
    assert!(
        winner_rating.mean() > 25.0,
        "Winner's rating should increase from 25.0 to {}",
        winner_rating.mean()
    );

    // Loser should have lower mean than initial
    assert!(
        loser_rating.mean() < 25.0,
        "Loser's rating should decrease from 25.0 to {}",
        loser_rating.mean()
    );

    // Both should have lower variance (more certain)
    assert!(
        winner_rating.variance() < (25.0_f64 / 3.0).powi(2),
        "Winner's variance should decrease"
    );
    assert!(
        loser_rating.variance() < (25.0_f64 / 3.0).powi(2),
        "Loser's variance should decrease"
    );

    // Check that ratings are reasonable
    assert!(winner_rating.mean() >= 20.0 && winner_rating.mean() <= 30.0);
    assert!(loser_rating.mean() >= 20.0 && loser_rating.mean() <= 30.0);
}

#[test]
fn test_trueskill_draw_simplified() {
    let ts = TrueSkill::new_simplified();

    // Create two players with default ratings
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();

    // Create teams
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);

    // Create a draw outcome
    let outcome = GameOutcome::draw(2);

    // Update ratings
    let updated_teams = ts
        .rate(&[team1, team2], &outcome)
        .expect("Rating update should succeed");

    // Verify that we have two teams back
    assert_eq!(updated_teams.len(), 2);

    let player1_rating = &updated_teams[0].player_ratings()[0];
    let player2_rating = &updated_teams[1].player_ratings()[0];

    println!(
        "Simplified Draw - Player 1: μ={:.3}, σ²={:.3}",
        player1_rating.mean(),
        player1_rating.variance()
    );
    println!(
        "Simplified Draw - Player 2: μ={:.3}, σ²={:.3}",
        player2_rating.mean(),
        player2_rating.variance()
    );

    // In a draw between equal players, means should stay approximately the same
    assert!(
        (player1_rating.mean() - 25.0).abs() < 1.0,
        "Player 1 mean should be close to 25"
    );
    assert!(
        (player2_rating.mean() - 25.0).abs() < 1.0,
        "Player 2 mean should be close to 25"
    );

    // Variances should decrease (more certainty)
    assert!(
        player1_rating.variance() < (25.0_f64 / 3.0).powi(2),
        "Player 1 variance should decrease"
    );
    assert!(
        player2_rating.variance() < (25.0_f64 / 3.0).powi(2),
        "Player 2 variance should decrease"
    );

    // Check that ratings are reasonable
    assert!(player1_rating.variance() > 0.0);
    assert!(player2_rating.variance() > 0.0);
}

#[test]
fn test_trueskill_different_skill_levels() {
    let ts = TrueSkill::new_simplified();

    // Create players with different skill levels
    let strong_player = ts.create_rating_with_values(30.0, 10.0); // Strong player
    let weak_player = ts.create_rating_with_values(20.0, 10.0); // Weak player

    // Create teams
    let team1 = TrueSkillTeam::from_player_ratings(vec![strong_player]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![weak_player]);

    // Strong player wins (expected outcome)
    let outcome = GameOutcome::win(0, 2);
    let updated_teams = ts
        .rate(&[team1, team2], &outcome)
        .expect("Rating update should succeed");

    let strong_updated = &updated_teams[0].player_ratings()[0];
    let weak_updated = &updated_teams[1].player_ratings()[0];

    println!("Strong player: {:.3} -> {:.3}", 30.0, strong_updated.mean());
    println!("Weak player: {:.3} -> {:.3}", 20.0, weak_updated.mean());

    // Changes should be smaller when the expected outcome happens
    assert!(
        (strong_updated.mean() - 30.0).abs() < 2.0,
        "Strong player rating should change less when winning as expected"
    );
    assert!(
        (weak_updated.mean() - 20.0).abs() < 2.0,
        "Weak player rating should change less when losing as expected"
    );

    // But strong player should still increase and weak should decrease
    assert!(
        strong_updated.mean() > 30.0,
        "Strong player should still gain rating"
    );
    assert!(
        weak_updated.mean() < 20.0,
        "Weak player should still lose rating"
    );
}

#[test]
fn test_trueskill_upset() {
    let ts = TrueSkill::new_simplified();

    // Create players with different skill levels
    let strong_player = ts.create_rating_with_values(30.0, 10.0); // Strong player
    let weak_player = ts.create_rating_with_values(20.0, 10.0); // Weak player

    // Create teams
    let team1 = TrueSkillTeam::from_player_ratings(vec![strong_player]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![weak_player]);

    // Weak player wins (upset!)
    let outcome = GameOutcome::win(1, 2); // Team 2 (weak player) wins
    let updated_teams = ts
        .rate(&[team1, team2], &outcome)
        .expect("Rating update should succeed");

    let strong_updated = &updated_teams[0].player_ratings()[0];
    let weak_updated = &updated_teams[1].player_ratings()[0];

    println!(
        "Strong player after upset: {:.3} -> {:.3}",
        30.0,
        strong_updated.mean()
    );
    println!(
        "Weak player after upset: {:.3} -> {:.3}",
        20.0,
        weak_updated.mean()
    );

    // Changes should be larger when an unexpected outcome happens
    assert!(
        (strong_updated.mean() - 30.0).abs() > 1.0,
        "Strong player should lose significant rating when upset"
    );
    assert!(
        (weak_updated.mean() - 20.0).abs() > 1.0,
        "Weak player should gain significant rating when causing upset"
    );

    // Strong player should decrease and weak should increase
    assert!(
        strong_updated.mean() < 30.0,
        "Strong player should lose rating when upset"
    );
    assert!(
        weak_updated.mean() > 20.0,
        "Weak player should gain rating when causing upset"
    );
}
