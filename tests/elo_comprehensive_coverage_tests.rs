use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    elo::{EloSystem, EloRating, EloTeamRating},
};

#[test]
fn test_elo_extreme_rating_differences() {
    let system = EloSystem::new();
    
    // Test extreme rating difference (high vs low)
    let high_player = EloRating::new(2800.0);
    let low_player = EloRating::new(800.0);
    
    let team1 = EloTeamRating::new(high_player);
    let team2 = EloTeamRating::new(low_player);
    
    // High-rated player wins (expected)
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[team1.clone(), team2.clone()], &outcome).unwrap();
    
    // Rating change should be minimal for expected result
    let rating_change = result[0].player_ratings()[0].rating() - 2800.0;
    assert!(rating_change.abs() < 2.0, "Expected result should cause minimal rating change");
    
    // Low-rated player wins (upset)
    let upset_outcome = GameOutcome::win(1, 2);
    let upset_result = system.rate(&[team1, team2], &upset_outcome).unwrap();
    
    // Low-rated player should gain significant rating, high-rated should lose significant rating
    let low_gain = upset_result[1].player_ratings()[0].rating() - 800.0;
    let high_loss = 2800.0 - upset_result[0].player_ratings()[0].rating();
    
    assert!(low_gain > 10.0, "Upset victory should cause significant rating gain");
    assert!(high_loss > 10.0, "Upset loss should cause significant rating loss");
}

#[test]
fn test_elo_custom_parameters() {
    // Test with high K-factor (volatile ratings)
    let high_k_system = EloSystem::with_parameters(50.0, 0.2, 300.0, 1200.0);
    
    let player1 = EloRating::new(1200.0);
    let player2 = EloRating::new(1200.0);
    
    let team1 = EloTeamRating::new(player1);
    let team2 = EloTeamRating::new(player2);
    
    let outcome = GameOutcome::win(0, 2);
    let result = high_k_system.rate(&[team1, team2], &outcome).unwrap();
    
    let rating_change = result[0].player_ratings()[0].rating() - 1200.0;
    assert!(rating_change > 15.0, "High K-factor should cause larger rating changes");
    
    // Test with low K-factor (stable ratings)
    let low_k_system = EloSystem::with_parameters(5.0, 0.05, 100.0, 1500.0);
    
    let stable_player1 = EloRating::new(1500.0);
    let stable_player2 = EloRating::new(1500.0);
    
    let stable_team1 = EloTeamRating::new(stable_player1);
    let stable_team2 = EloTeamRating::new(stable_player2);
    
    let stable_result = low_k_system.rate(&[stable_team1, stable_team2], &outcome).unwrap();
    
    let stable_change = stable_result[0].player_ratings()[0].rating() - 1500.0;
    assert!(stable_change < 5.0, "Low K-factor should cause smaller rating changes");
}

#[test]
fn test_elo_rating_sequence_convergence() {
    let system = EloSystem::new();
    
    let mut player1_rating = EloRating::new(1500.0);
    let mut player2_rating = EloRating::new(1500.0);
    
    // Simulate player 1 consistently beating player 2
    for _ in 0..50 {
        let team1 = EloTeamRating::new(player1_rating.clone());
        let team2 = EloTeamRating::new(player2_rating.clone());
        
        let outcome = GameOutcome::win(0, 2);
        let result = system.rate(&[team1, team2], &outcome).unwrap();
        
        player1_rating = result[0].player_ratings()[0].clone();
        player2_rating = result[1].player_ratings()[0].clone();
    }
    
    // After many wins, ratings should stabilize with significant difference
    assert!(player1_rating.rating() > 1600.0, "Consistent winner should have higher rating");
    assert!(player2_rating.rating() < 1400.0, "Consistent loser should have lower rating");
    assert!(player1_rating.rating() - player2_rating.rating() > 200.0, "Rating difference should be significant");
}

#[test]
fn test_elo_boundary_conditions() {
    let system = EloSystem::new();
    
    // Test very close to boundary ratings
    let boundary_high = EloRating::new(3000.0);
    let boundary_low = EloRating::new(0.1);
    
    let team_high = EloTeamRating::new(boundary_high);
    let team_low = EloTeamRating::new(boundary_low);
    
    // Both win and loss scenarios should work
    let outcome_high_wins = GameOutcome::win(0, 2);
    let result_high_wins = system.rate(&[team_high.clone(), team_low.clone()], &outcome_high_wins);
    assert!(result_high_wins.is_ok(), "Boundary ratings should be handled gracefully");
    
    let outcome_low_wins = GameOutcome::win(1, 2);
    let result_low_wins = system.rate(&[team_high, team_low], &outcome_low_wins);
    assert!(result_low_wins.is_ok(), "Boundary ratings should be handled gracefully");
}

#[test]
fn test_elo_error_conditions() {
    let system = EloSystem::new();
    
    // Test invalid team count
    let team1 = EloTeamRating::new(EloRating::new(1500.0));
    let outcome = GameOutcome::new(vec![1]);
    
    let result = system.rate(&[team1], &outcome);
    assert!(result.is_err(), "Single team should cause error");
    
    // Test mismatched outcome
    let team1 = EloTeamRating::new(EloRating::new(1500.0));
    let team2 = EloTeamRating::new(EloRating::new(1500.0));
    let bad_outcome = GameOutcome::new(vec![1, 2, 3]); // 3 ranks for 2 teams
    
    let result = system.rate(&[team1, team2], &bad_outcome);
    assert!(result.is_err(), "Mismatched outcome should cause error");
}

#[test]
fn test_elo_win_probability_edge_cases() {
    let system = EloSystem::new();
    
    // Test with identical ratings (should be ~0.5)
    let quality_equal = system.calculate_match_quality(&[
        EloTeamRating::new(EloRating::new(1500.0)),
        EloTeamRating::new(EloRating::new(1500.0)),
    ]).unwrap();
    
    assert!((quality_equal - 1.0).abs() < 0.01, "Equal players should have maximum quality");
    
    // Test with very different ratings (should be low quality)
    let quality_different = system.calculate_match_quality(&[
        EloTeamRating::new(EloRating::new(2500.0)),
        EloTeamRating::new(EloRating::new(1000.0)),
    ]).unwrap();
    
    assert!(quality_different < 0.3, "Very different players should have low quality");
}

#[test]
fn test_elo_draw_scenarios() {
    let system = EloSystem::new();
    
    // Test multiple draw scenarios
    let mut player1_rating = EloRating::new(1600.0);
    let mut player2_rating = EloRating::new(1400.0);
    
    // Series of draws should gradually equalize ratings
    for _ in 0..20 {
        let team1 = EloTeamRating::new(player1_rating.clone());
        let team2 = EloTeamRating::new(player2_rating.clone());
        
        let draw_outcome = GameOutcome::draw(2);
        let result = system.rate(&[team1, team2], &draw_outcome).unwrap();
        
        player1_rating = result[0].player_ratings()[0].clone();
        player2_rating = result[1].player_ratings()[0].clone();
    }
    
    // After many draws, ratings should converge
    let rating_diff = (player1_rating.rating() - player2_rating.rating()).abs();
    assert!(rating_diff < 50.0, "Many draws should converge ratings: diff={}", rating_diff);
}

#[test]
fn test_elo_rating_properties() {
    let rating = EloRating::new(1650.0);
    
    // Test Rating trait implementation
    assert_eq!(rating.mean(), 1650.0);
    assert_eq!(rating.variance(), 0.0); // Elo has no variance
    assert_eq!(rating.standard_deviation(), 0.0);
    assert_eq!(rating.conservative_rating(), 1650.0); // Same as rating for Elo
}

#[test]
fn test_elo_team_rating_properties() {
    let player_rating = EloRating::new(1500.0);
    let team = EloTeamRating::new(player_rating.clone());
    
    // Test TeamRating trait implementation
    assert_eq!(team.player_ratings().len(), 1);
    assert_eq!(team.player_ratings()[0].rating(), 1500.0);
    
    // Test from_player_ratings
    let team2 = EloTeamRating::from_player_ratings(vec![player_rating]);
    assert_eq!(team2.player_ratings().len(), 1);
    assert_eq!(team2.player_ratings()[0].rating(), 1500.0);
}

#[test]
fn test_elo_system_defaults() {
    let system = EloSystem::new();
    let default_system = EloSystem::default();
    
    // Both should create same default rating
    let rating1 = system.create_rating();
    let rating2 = default_system.create_rating();
    
    assert_eq!(rating1.rating(), rating2.rating());
    assert_eq!(rating1.rating(), 1500.0); // Default starting rating
}

#[test]
fn test_elo_create_rating_with_values() {
    let system = EloSystem::new();
    
    // Test create_rating_with_values (variance should be ignored for Elo)
    let custom_rating = system.create_rating_with_values(1800.0, 100.0);
    assert_eq!(custom_rating.rating(), 1800.0);
    assert_eq!(custom_rating.variance(), 0.0); // Elo ignores variance
}