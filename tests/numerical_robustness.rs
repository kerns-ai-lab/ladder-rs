use ladder_rs::{
    trueskill::{TrueSkill, TrueSkillTeam},
    core::{Rating, RatingSystem, TeamRating, GameOutcome},
};

#[test]
fn test_extreme_skill_differences() {
    let ts = TrueSkill::new();
    
    // Create players with extreme skill differences
    let very_strong = ts.create_rating_with_values(50.0, 5.0);
    let very_weak = ts.create_rating_with_values(1.0, 5.0);
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![very_strong]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![very_weak]);
    
    // Strong player wins (expected)
    let outcome = GameOutcome::win(0, 2);
    let result = ts.rate(&[team1.clone(), team2.clone()], &outcome);
    assert!(result.is_ok(), "Should handle extreme skill differences");
    
    // Weak player wins (major upset)
    let outcome = GameOutcome::win(1, 2);
    let result = ts.rate(&[team1, team2], &outcome);
    assert!(result.is_ok(), "Should handle major upsets");
}

#[test]
fn test_very_certain_players() {
    let ts = TrueSkill::new();
    
    // Create players with very low variance (very certain about their skill)
    let certain_strong = ts.create_rating_with_values(30.0, 0.1);
    let certain_weak = ts.create_rating_with_values(20.0, 0.1);
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![certain_strong]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![certain_weak]);
    
    let outcome = GameOutcome::win(0, 2);
    let result = ts.rate(&[team1, team2], &outcome);
    assert!(result.is_ok(), "Should handle very certain players");
    
    let updated_teams = result.unwrap();
    let updated_strong = &updated_teams[0].player_ratings()[0];
    let updated_weak = &updated_teams[1].player_ratings()[0];
    
    // Changes should be small for very certain players
    assert!((updated_strong.mean() - 30.0).abs() < 1.0, "Certain strong player should change little");
    assert!((updated_weak.mean() - 20.0).abs() < 1.0, "Certain weak player should change little");
}

#[test]
fn test_very_uncertain_players() {
    let ts = TrueSkill::new();
    
    // Create players with very high variance (very uncertain about their skill)
    let uncertain1 = ts.create_rating_with_values(25.0, 1000.0);
    let uncertain2 = ts.create_rating_with_values(25.0, 1000.0);
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![uncertain1]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![uncertain2]);
    
    let outcome = GameOutcome::win(0, 2);
    let result = ts.rate(&[team1, team2], &outcome);
    assert!(result.is_ok(), "Should handle very uncertain players");
    
    let updated_teams = result.unwrap();
    let updated_winner = &updated_teams[0].player_ratings()[0];
    let updated_loser = &updated_teams[1].player_ratings()[0];
    
    // Changes should be larger for very uncertain players
    assert!((updated_winner.mean() - 25.0).abs() > 0.1, "Uncertain winner should change noticeably");
    assert!((updated_loser.mean() - 25.0).abs() > 0.1, "Uncertain loser should change noticeably");
    
    // Variance should decrease (become more certain)
    assert!(updated_winner.variance() < 1000.0, "Winner variance should decrease");
    assert!(updated_loser.variance() < 1000.0, "Loser variance should decrease");
}

#[test]
fn test_multiple_games_sequence() {
    let ts = TrueSkill::new();
    
    let mut team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let mut team2 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    
    // Simulate a sequence of games where player 1 consistently wins
    for i in 0..10 {
        let outcome = GameOutcome::win(0, 2);
        let result = ts.rate(&[team1.clone(), team2.clone()], &outcome);
        assert!(result.is_ok(), "Game {} should succeed", i + 1);
        
        let updated_teams = result.unwrap();
        team1 = updated_teams[0].clone();
        team2 = updated_teams[1].clone();
        
        // Verify ratings remain valid
        assert!(team1.player_ratings()[0].variance() > 0.0, "Team 1 variance should remain positive");
        assert!(team2.player_ratings()[0].variance() > 0.0, "Team 2 variance should remain positive");
    }
    
    // After 10 wins, player 1 should have much higher rating than player 2
    let final_rating1 = &team1.player_ratings()[0];
    let final_rating2 = &team2.player_ratings()[0];
    
    assert!(final_rating1.mean() > final_rating2.mean() + 5.0, 
            "After 10 wins, winner should have significantly higher rating");
    
    println!("After 10 games: Player 1: {:.2} ± {:.2}, Player 2: {:.2} ± {:.2}", 
             final_rating1.mean(), final_rating1.standard_deviation(),
             final_rating2.mean(), final_rating2.standard_deviation());
}

#[test]
fn test_alternating_wins() {
    let ts = TrueSkill::new();
    
    let mut team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let mut team2 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    
    // Simulate alternating wins
    for i in 0..10 {
        let outcome = if i % 2 == 0 {
            GameOutcome::win(0, 2)  // Player 1 wins
        } else {
            GameOutcome::win(1, 2)  // Player 2 wins
        };
        
        let result = ts.rate(&[team1.clone(), team2.clone()], &outcome);
        assert!(result.is_ok(), "Alternating game {} should succeed", i + 1);
        
        let updated_teams = result.unwrap();
        team1 = updated_teams[0].clone();
        team2 = updated_teams[1].clone();
    }
    
    // After alternating wins, players should have similar ratings but lower variance
    let final_rating1 = &team1.player_ratings()[0];
    let final_rating2 = &team2.player_ratings()[0];
    
    assert!((final_rating1.mean() - final_rating2.mean()).abs() < 2.0, 
            "After alternating wins, players should have similar ratings");
    
    // Both should have lower variance than initial
    let initial_variance = (25.0_f64 / 3.0).powi(2);
    assert!(final_rating1.variance() < initial_variance, "Player 1 variance should decrease");
    assert!(final_rating2.variance() < initial_variance, "Player 2 variance should decrease");
    
    println!("After alternating games: Player 1: {:.2} ± {:.2}, Player 2: {:.2} ± {:.2}", 
             final_rating1.mean(), final_rating1.standard_deviation(),
             final_rating2.mean(), final_rating2.standard_deviation());
}

#[test]
fn test_draw_sequence() {
    let ts = TrueSkill::new();
    
    let mut team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let mut team2 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    
    // Simulate a sequence of draws
    for i in 0..5 {
        let outcome = GameOutcome::draw(2);
        let result = ts.rate(&[team1.clone(), team2.clone()], &outcome);
        assert!(result.is_ok(), "Draw game {} should succeed", i + 1);
        
        let updated_teams = result.unwrap();
        team1 = updated_teams[0].clone();
        team2 = updated_teams[1].clone();
    }
    
    // After draws, players should have similar ratings and lower variance
    let final_rating1 = &team1.player_ratings()[0];
    let final_rating2 = &team2.player_ratings()[0];
    
    assert!((final_rating1.mean() - 25.0).abs() < 1.0, "Player 1 should stay near initial rating");
    assert!((final_rating2.mean() - 25.0).abs() < 1.0, "Player 2 should stay near initial rating");
    
    // Both should have lower variance than initial
    let initial_variance = (25.0_f64 / 3.0).powi(2);
    assert!(final_rating1.variance() < initial_variance, "Player 1 variance should decrease");
    assert!(final_rating2.variance() < initial_variance, "Player 2 variance should decrease");
    
    println!("After draw sequence: Player 1: {:.2} ± {:.2}, Player 2: {:.2} ± {:.2}", 
             final_rating1.mean(), final_rating1.standard_deviation(),
             final_rating2.mean(), final_rating2.standard_deviation());
}