use ladder_rs::{
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
    core::{RatingSystem, TeamRating, GameOutcome},
};

#[test]
fn test_trueskill_basic_1v1() {
    let ts = TrueSkill::new();
    
    // Create two players with default ratings
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();
    
    // Verify default values
    assert_eq!(player1.mean(), 25.0);
    assert_eq!(player2.mean(), 25.0);
    assert!((player1.variance() - (25.0/3.0).powi(2)).abs() < 1e-10);
    
    // Create teams (single players)
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    let teams = vec![team1, team2];
    
    // Player 1 wins
    let outcome = GameOutcome::win(0, 2);
    
    // Update ratings
    let updated_teams = ts.rate(&teams, &outcome).expect("Rating update should succeed");
    
    // Check that winner's mean increased and loser's mean decreased
    let updated_player1 = &updated_teams[0].player_ratings()[0];
    let updated_player2 = &updated_teams[1].player_ratings()[0];
    
    println!("Player 1: {} -> {}", 25.0, updated_player1.mean());
    println!("Player 2: {} -> {}", 25.0, updated_player2.mean());
    
    assert!(updated_player1.mean() > 25.0, "Winner should have higher rating");
    assert!(updated_player2.mean() < 25.0, "Loser should have lower rating");
    
    // Verify variance decreased (less uncertainty)
    assert!(updated_player1.variance() < (25.0/3.0).powi(2), "Winner's variance should decrease");
    assert!(updated_player2.variance() < (25.0/3.0).powi(2), "Loser's variance should decrease");
}

#[test]
fn test_trueskill_match_quality() {
    let ts = TrueSkill::new();
    
    // Create two equally skilled players
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    let teams = vec![team1, team2];
    
    let quality = ts.calculate_match_quality(&teams).expect("Quality calculation should succeed");
    
    println!("Match quality: {}", quality);
    
    // Quality should be between 0 and 1
    assert!(quality >= 0.0 && quality <= 1.0);
    
    // Equal players should have relatively high match quality
    assert!(quality > 0.4, "Equal players should have good match quality");
}

#[test]
fn test_trueskill_different_skills() {
    let ts = TrueSkill::new();
    
    // Create players with different skill levels
    let strong_player = ts.create_rating_with_values(30.0, 1.0); // High skill, low uncertainty
    let weak_player = ts.create_rating_with_values(20.0, 1.0);   // Low skill, low uncertainty
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![strong_player]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![weak_player]);
    let teams = vec![team1, team2];
    
    let quality = ts.calculate_match_quality(&teams).expect("Quality calculation should succeed");
    
    println!("Match quality (different skills): {}", quality);
    
    // Different skill levels should result in lower match quality
    assert!(quality < 0.4, "Different skill levels should have lower match quality");
}

#[test]
fn test_trueskill_draw() {
    let ts = TrueSkill::new();
    
    // Create two players with default ratings
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    let teams = vec![team1, team2];
    
    // Draw game (both teams have rank 1)
    let outcome = GameOutcome::draw(2);
    
    // Update ratings
    let updated_teams = ts.rate(&teams, &outcome).expect("Rating update should succeed");
    
    let updated_player1 = &updated_teams[0].player_ratings()[0];
    let updated_player2 = &updated_teams[1].player_ratings()[0];
    
    println!("Draw - Player 1: {} -> {}", 25.0, updated_player1.mean());
    println!("Draw - Player 2: {} -> {}", 25.0, updated_player2.mean());
    
    // In a draw, means should remain close to original values
    assert!((updated_player1.mean() - 25.0).abs() < 2.0, "Player 1 mean should not change much in draw");
    assert!((updated_player2.mean() - 25.0).abs() < 2.0, "Player 2 mean should not change much in draw");
    
    // But variance should still decrease due to gaining information
    assert!(updated_player1.variance() < (25.0/3.0).powi(2), "Player 1 variance should decrease");
    assert!(updated_player2.variance() < (25.0/3.0).powi(2), "Player 2 variance should decrease");
}

#[test]
fn test_trueskill_team_game() {
    let ts = TrueSkill::new();
    
    // Create a 2v2 team game
    let player1 = ts.create_rating();
    let player2 = ts.create_rating();
    let player3 = ts.create_rating();
    let player4 = ts.create_rating();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1, player2]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player3, player4]);
    let teams = vec![team1, team2];
    
    // Team 1 wins
    let outcome = GameOutcome::win(0, 2);
    
    // Update ratings
    let updated_teams = ts.rate(&teams, &outcome).expect("Rating update should succeed");
    
    let winning_team = &updated_teams[0];
    let losing_team = &updated_teams[1];
    
    // Check that all winning players' means increased
    for player in winning_team.player_ratings() {
        assert!(player.mean() > 25.0, "Winning player should have higher rating");
    }
    
    // Check that all losing players' means decreased
    for player in losing_team.player_ratings() {
        assert!(player.mean() < 25.0, "Losing player should have lower rating");
    }
    
    println!("Team game completed successfully");
}