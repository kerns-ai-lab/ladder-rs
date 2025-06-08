use ladder_rs::{
    core::{GameOutcome, RatingSystem},
    error::Error,
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
};

#[test]
fn test_default_trueskill_system() {
    let system = TrueSkill::new();
    
    // Test default rating creation
    let rating = system.create_rating();
    assert_eq!(rating.mean(), 25.0);
    assert!((rating.variance() - 69.44444444).abs() < 0.00001);
}

#[test]
fn test_trueskill_rating_creation() {
    // Test valid rating creation
    let rating = TrueSkillRating::new(30.0, 100.0).unwrap();
    assert_eq!(rating.mean(), 30.0);
    assert_eq!(rating.variance(), 100.0);
    
    // Test invalid rating creation (negative variance)
    let result = TrueSkillRating::new(30.0, -1.0);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Test invalid rating creation (zero variance)
    let result = TrueSkillRating::new(30.0, 0.0);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

#[test]
fn test_trueskill_team_creation() {
    let player1 = TrueSkillRating::new(25.0, 64.0).unwrap();
    let player2 = TrueSkillRating::new(27.0, 81.0).unwrap();
    
    // Test single player team
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1.clone()]);
    assert_eq!(team1.player_ratings().len(), 1);
    assert_eq!(team1.player_ratings()[0].mean(), 25.0);
    
    // Test multi-player team
    let team2 = TrueSkillTeam::from_player_ratings(vec![player1.clone(), player2.clone()]);
    assert_eq!(team2.player_ratings().len(), 2);
    assert_eq!(team2.player_ratings()[0].mean(), 25.0);
    assert_eq!(team2.player_ratings()[1].mean(), 27.0);
}

#[test]
fn test_simple_two_player_match() {
    let system = TrueSkill::new();
    
    // Create two identical players
    let player1 = system.create_rating();
    let player2 = system.create_rating();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    
    // Player 1 wins
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[team1, team2], &outcome).unwrap();
    
    let winner = &result[0].player_ratings()[0];
    let loser = &result[1].player_ratings()[0];
    
    // Winner's rating should increase
    assert!(winner.mean() > 25.0);
    // Loser's rating should decrease
    assert!(loser.mean() < 25.0);
    
    // Both should have reduced variance (more certainty)
    assert!(winner.variance() < 69.44444444);
    assert!(loser.variance() < 69.44444444);
}

#[test]
fn test_draw_outcome() {
    let system = TrueSkill::new();
    
    let player1 = system.create_rating();
    let player2 = system.create_rating();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
    
    let outcome = GameOutcome::draw(2);
    let result = system.rate(&[team1, team2], &outcome).unwrap();
    
    let p1_after = &result[0].player_ratings()[0];
    let p2_after = &result[1].player_ratings()[0];
    
    // In a draw between equal players, ratings should stay close to original
    assert!((p1_after.mean() - 25.0).abs() < 1.0);
    assert!((p2_after.mean() - 25.0).abs() < 1.0);
    
    // Variance should still decrease
    assert!(p1_after.variance() < 69.44444444);
    assert!(p2_after.variance() < 69.44444444);
}

#[test]
fn test_trueskill_rating_methods() {
    let mean = 25.0;
    let variance = 64.0;
    let rating = TrueSkillRating::new(mean, variance).unwrap();
    
    // Test basic properties
    assert_eq!(rating.mean(), mean);
    assert_eq!(rating.variance(), variance);
    assert_eq!(rating.std_dev(), 8.0);

    // precision() and precision_adjusted_mean() are not available on TrueSkillRating

    // Test conservative rating
    assert_eq!(rating.conservative_rating(), mean - 3.0 * 8.0);
}

#[test]
fn test_invalid_number_of_teams() {
    let system = TrueSkill::new();
    
    // Test with no teams
    let outcome = GameOutcome::new(vec![]);
    let result = system.rate(&[], &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Test with one team
    let team = TrueSkillTeam::from_player_ratings(vec![system.create_rating()]);
    let outcome = GameOutcome::new(vec![1]);
    let result = system.rate(&[team], &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

#[test]
fn test_mismatched_teams_and_ranks() {
    let system = TrueSkill::new();
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![system.create_rating()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![system.create_rating()]);
    
    // 2 teams but 3 ranks
    let outcome = GameOutcome::new(vec![1, 2, 3]);
    let result = system.rate(&[team1, team2], &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

#[test]
fn test_rating_progression() {
    use ladder_rs::core::{GameOutcome, TeamRating};
    
    let system = TrueSkill::new();
    let mut player1 = system.create_rating();
    let mut player2 = system.create_rating();
    
    // Simulate 10 games where player 1 always wins
    for _ in 0..10 {
        let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
        let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
        
        let outcome = GameOutcome::win(0, 2);
        let result = system.rate(&[team1, team2], &outcome).unwrap();
        
        player1 = result[0].player_ratings()[0].clone();
        player2 = result[1].player_ratings()[0].clone();
    }
    
    // After 10 wins, player 1 should be significantly higher rated
    // TrueSkill is conservative, so expect ~2 point difference
    assert!(player1.mean() > player2.mean() + 2.0);
    
    // Both players should have lower variance (more games = more certainty)
    assert!(player1.variance() < 40.0);
    assert!(player2.variance() < 40.0);
}

#[test]
fn test_upset_victory() {
    let system = TrueSkill::new();
    
    // Create a strong and weak player
    let strong = TrueSkillRating::new(35.0, 36.0).unwrap();
    let weak = TrueSkillRating::new(15.0, 36.0).unwrap();
    
    let team_strong = TrueSkillTeam::from_player_ratings(vec![strong]);
    let team_weak = TrueSkillTeam::from_player_ratings(vec![weak]);
    
    // Weak player wins (upset)
    let outcome = GameOutcome::win(1, 2);
    let result = system.rate(&[team_strong, team_weak], &outcome).unwrap();
    
    let strong_after = &result[0].player_ratings()[0];
    let weak_after = &result[1].player_ratings()[0];
    
    // Strong player should lose significant rating
    assert!(strong_after.mean() < 35.0);
    
    // Weak player should gain significant rating
    assert!(weak_after.mean() > 15.0);
    
    // The change should be noticeable for an upset
    // TrueSkill is conservative, so changes are smaller than expected
    assert!((weak_after.mean() - 15.0) > 0.1);
    assert!((35.0 - strong_after.mean()) > 0.1);
}