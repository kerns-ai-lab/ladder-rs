use ladder_rs::{
    core::{GameOutcome, RatingSystem, TeamRating},
    trueskill::{TrueSkill, TrueSkillTeam},
};

#[test]
fn test_trueskill_basic_functionality() {
    let ts = TrueSkill::new();
    
    // Create two players with default ratings
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();
    
    // Verify default values
    assert_eq!(player1.mean(), 25.0);
    assert_eq!(player2.mean(), 25.0);
    assert!((player1.variance() - (25.0/3.0).powi(2)).abs() < 1e-10);
    
    // Create teams
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    
    // Create a game outcome where team 1 wins
    let outcome = GameOutcome::win(0, 2);
    
    // Update ratings
    let updated_teams = ts.rate(&[team1, team2], &outcome).expect("Rating update should succeed");
    
    // Verify that we have two teams back
    assert_eq!(updated_teams.len(), 2);
    
    // Verify that the winner's rating increased and loser's decreased
    let winner_rating = &updated_teams[0].player_ratings()[0];
    let loser_rating = &updated_teams[1].player_ratings()[0];
    
    println!("Winner: μ={:.3}, σ²={:.3}", winner_rating.mean(), winner_rating.variance());
    println!("Loser: μ={:.3}, σ²={:.3}", loser_rating.mean(), loser_rating.variance());
    
    // Winner should have higher mean than initial
    assert!(winner_rating.mean() > 25.0, "Winner's rating should increase");
    
    // Loser should have lower mean than initial
    assert!(loser_rating.mean() < 25.0, "Loser's rating should decrease");
    
    // Both should have lower variance (more certain)
    assert!(winner_rating.variance() < (25.0/3.0).powi(2), "Winner's variance should decrease");
    assert!(loser_rating.variance() < (25.0/3.0).powi(2), "Loser's variance should decrease");
}

#[test]
fn test_trueskill_draw() {
    let ts = TrueSkill::new();
    
    // Create two players with default ratings
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();
    
    // Create teams
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    
    // Create a draw outcome
    let outcome = GameOutcome::draw(2);
    
    // Update ratings
    let updated_teams = ts.rate(&[team1, team2], &outcome).expect("Rating update should succeed");
    
    // In a draw between equal players, means should stay approximately the same
    let player1_rating = &updated_teams[0].player_ratings()[0];
    let player2_rating = &updated_teams[1].player_ratings()[0];
    
    println!("Player 1 after draw: μ={:.3}, σ²={:.3}", player1_rating.mean(), player1_rating.variance());
    println!("Player 2 after draw: μ={:.3}, σ²={:.3}", player2_rating.mean(), player2_rating.variance());
    
    // Means should be close to original (small changes due to information gain)
    assert!((player1_rating.mean() - 25.0).abs() < 1.0, "Player 1 mean should be close to 25");
    assert!((player2_rating.mean() - 25.0).abs() < 1.0, "Player 2 mean should be close to 25");
    
    // Variances should decrease (more certainty)
    assert!(player1_rating.variance() < (25.0/3.0).powi(2), "Player 1 variance should decrease");
    assert!(player2_rating.variance() < (25.0/3.0).powi(2), "Player 2 variance should decrease");
}