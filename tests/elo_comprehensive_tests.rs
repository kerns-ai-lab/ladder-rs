use ladder_rs::{
    elo::{EloRating, EloSystem, EloTeamRating},
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    error::Error,
};

#[test]
fn test_elo_custom_parameters() {
    let system = EloSystem::with_parameters(32.0, 0.05, 150.0, 1200.0);
    let rating = system.create_rating();
    
    assert_eq!(rating.rating(), 1200.0);
    assert_eq!(rating.mean(), 1200.0);
    assert_eq!(rating.variance(), 0.0);
    assert_eq!(rating.conservative_rating(), 1200.0);
}

#[test]
fn test_elo_extreme_rating_differences() {
    let system = EloSystem::new();
    
    // Very high rated player vs very low rated player
    let high_player = EloRating::new(2500.0);
    let low_player = EloRating::new(800.0);
    
    let team1 = EloTeamRating::new(high_player);
    let team2 = EloTeamRating::new(low_player);
    
    // High player wins (expected)
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[team1.clone(), team2.clone()], &outcome).unwrap();
    
    // Rating changes should be small for expected outcome
    let new_high = result[0].player_ratings()[0].rating();
    let new_low = result[1].player_ratings()[0].rating();
    
    assert!(new_high > 2500.0);
    assert!(new_low < 800.0);
    assert!((new_high - 2500.0) < 5.0); // Small change for favorite
    assert!((800.0 - new_low) < 50.0); // Larger change for underdog
}

#[test]
fn test_elo_upset_scenario() {
    let system = EloSystem::new();
    
    let high_player = EloRating::new(2000.0);
    let low_player = EloRating::new(1200.0);
    
    let team1 = EloTeamRating::new(high_player);
    let team2 = EloTeamRating::new(low_player);
    
    // Low player wins (upset)
    let outcome = GameOutcome::win(1, 2);
    let result = system.rate(&[team1, team2], &outcome).unwrap();
    
    let new_high = result[0].player_ratings()[0].rating();
    let new_low = result[1].player_ratings()[0].rating();
    
    assert!(new_high < 2000.0);
    assert!(new_low > 1200.0);
    
    // Changes should be larger for upset
    assert!((2000.0 - new_high) > 10.0);
    assert!((new_low - 1200.0) > 10.0);
}

#[test]
fn test_elo_series_of_games() {
    let system = EloSystem::new();
    
    let mut player1 = EloRating::new(1500.0);
    let mut player2 = EloRating::new(1500.0);
    
    // Player 1 wins 5 games in a row
    for _ in 0..5 {
        let team1 = EloTeamRating::new(player1.clone());
        let team2 = EloTeamRating::new(player2.clone());
        
        let outcome = GameOutcome::win(0, 2);
        let result = system.rate(&[team1, team2], &outcome).unwrap();
        
        player1 = result[0].player_ratings()[0].clone();
        player2 = result[1].player_ratings()[0].clone();
    }
    
    // Player 1 should be significantly higher rated
    assert!(player1.rating() > 1550.0);
    assert!(player2.rating() < 1450.0);
    assert!(player1.rating() > player2.rating() + 100.0);
}

#[test]
fn test_elo_alternating_wins() {
    let system = EloSystem::new();
    
    let mut player1 = EloRating::new(1500.0);
    let mut player2 = EloRating::new(1500.0);
    
    // Alternating wins over 10 games
    for i in 0..10 {
        let team1 = EloTeamRating::new(player1.clone());
        let team2 = EloTeamRating::new(player2.clone());
        
        let outcome = if i % 2 == 0 {
            GameOutcome::win(0, 2)  // Player 1 wins
        } else {
            GameOutcome::win(1, 2)  // Player 2 wins
        };
        
        let result = system.rate(&[team1, team2], &outcome).unwrap();
        
        player1 = result[0].player_ratings()[0].clone();
        player2 = result[1].player_ratings()[0].clone();
    }
    
    // Ratings should remain close to original with alternating results
    assert!((player1.rating() - 1500.0).abs() < 50.0);
    assert!((player2.rating() - 1500.0).abs() < 50.0);
}

#[test]
fn test_elo_all_draws() {
    let system = EloSystem::new();
    
    let mut player1 = EloRating::new(1500.0);
    let mut player2 = EloRating::new(1500.0);
    
    // 10 draws in a row
    for _ in 0..10 {
        let team1 = EloTeamRating::new(player1.clone());
        let team2 = EloTeamRating::new(player2.clone());
        
        let outcome = GameOutcome::draw(2);
        let result = system.rate(&[team1, team2], &outcome).unwrap();
        
        player1 = result[0].player_ratings()[0].clone();
        player2 = result[1].player_ratings()[0].clone();
    }
    
    // With all draws between equal players, ratings should barely change
    assert!((player1.rating() - 1500.0).abs() < 1.0);
    assert!((player2.rating() - 1500.0).abs() < 1.0);
}

#[test]
fn test_elo_match_quality_edge_cases() {
    let system = EloSystem::new();
    
    // Equal players should have perfect match quality
    let equal1 = EloTeamRating::new(EloRating::new(1500.0));
    let equal2 = EloTeamRating::new(EloRating::new(1500.0));
    let quality = system.calculate_match_quality(&[equal1, equal2]).unwrap();
    assert!((quality - 1.0).abs() < 0.001);
    
    // Very different players should have poor match quality
    let high = EloTeamRating::new(EloRating::new(2500.0));
    let low = EloTeamRating::new(EloRating::new(500.0));
    let quality = system.calculate_match_quality(&[high, low]).unwrap();
    assert!(quality < 0.1);
    
    // Moderately different players
    let player1 = EloTeamRating::new(EloRating::new(1600.0));
    let player2 = EloTeamRating::new(EloRating::new(1400.0));
    let quality = system.calculate_match_quality(&[player1, player2]).unwrap();
    assert!(quality > 0.5 && quality < 1.0);
}

#[test]
fn test_elo_rating_boundaries() {
    let system = EloSystem::new();
    
    // Test very high ratings
    let high_player = EloRating::new(3000.0);
    let normal_player = EloRating::new(1500.0);
    
    let team1 = EloTeamRating::new(high_player);
    let team2 = EloTeamRating::new(normal_player);
    
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[team1, team2], &outcome);
    assert!(result.is_ok());
    
    // Test very low ratings
    let low_player = EloRating::new(100.0);
    let normal_player = EloRating::new(1500.0);
    
    let team1 = EloTeamRating::new(low_player);
    let team2 = EloTeamRating::new(normal_player);
    
    let outcome = GameOutcome::win(1, 2);
    let result = system.rate(&[team1, team2], &outcome);
    assert!(result.is_ok());
}

#[test]
fn test_elo_k_factor_effects() {
    // High K-factor should cause larger rating changes
    let high_k_system = EloSystem::with_parameters(50.0, 0.1, 200.0, 1500.0);
    
    let player1 = EloRating::new(1500.0);
    let player2 = EloRating::new(1500.0);
    
    let team1 = EloTeamRating::new(player1);
    let team2 = EloTeamRating::new(player2);
    
    let outcome = GameOutcome::win(0, 2);
    let result = high_k_system.rate(&[team1, team2], &outcome).unwrap();
    
    let rating_change = (result[0].player_ratings()[0].rating() - 1500.0).abs();
    
    // Low K-factor should cause smaller rating changes
    let low_k_system = EloSystem::with_parameters(5.0, 0.1, 200.0, 1500.0);
    
    let player1 = EloRating::new(1500.0);
    let player2 = EloRating::new(1500.0);
    
    let team1 = EloTeamRating::new(player1);
    let team2 = EloTeamRating::new(player2);
    
    let outcome = GameOutcome::win(0, 2);
    let result2 = low_k_system.rate(&[team1, team2], &outcome).unwrap();
    
    let rating_change2 = (result2[0].player_ratings()[0].rating() - 1500.0).abs();
    
    assert!(rating_change > rating_change2 * 2.0);
}

#[test]
fn test_elo_error_conditions() {
    let system = EloSystem::new();
    
    // Test invalid team count
    let team1 = EloTeamRating::new(EloRating::new(1500.0));
    let outcome = GameOutcome::new(vec![1]);
    let result = system.rate(&[team1], &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Test too many teams
    let teams = vec![
        EloTeamRating::new(EloRating::new(1500.0)),
        EloTeamRating::new(EloRating::new(1500.0)),
        EloTeamRating::new(EloRating::new(1500.0)),
    ];
    let outcome = GameOutcome::new(vec![1, 2, 3]);
    let result = system.rate(&teams, &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Test multiple players per team (not supported by Elo)
    let multi_player_team = EloTeamRating::from_player_ratings(vec![
        EloRating::new(1500.0),
        EloRating::new(1600.0),
    ]);
    let single_player_team = EloTeamRating::new(EloRating::new(1500.0));
    
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[multi_player_team, single_player_team], &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

#[test]
fn test_elo_win_probability_through_match_quality() {
    let system = EloSystem::new();
    
    // Test extreme rating differences through match quality
    let high = EloTeamRating::new(EloRating::new(2500.0));
    let low = EloTeamRating::new(EloRating::new(500.0));
    let quality = system.calculate_match_quality(&[high, low]).unwrap();
    assert!(quality < 0.01); // Very poor match quality indicates very uneven
    
    // Test identical ratings
    let equal1 = EloTeamRating::new(EloRating::new(1500.0));
    let equal2 = EloTeamRating::new(EloRating::new(1500.0));
    let quality = system.calculate_match_quality(&[equal1, equal2]).unwrap();
    assert!((quality - 1.0).abs() < 0.001); // Perfect match quality
}

#[test]
fn test_elo_create_rating_with_values() {
    let system = EloSystem::new();
    
    // Variance parameter should be ignored for Elo
    let rating = system.create_rating_with_values(1800.0, 100.0);
    assert_eq!(rating.rating(), 1800.0);
    assert_eq!(rating.mean(), 1800.0);
    assert_eq!(rating.variance(), 0.0);
}

#[test]
fn test_elo_rating_trait_methods() {
    let rating = EloRating::new(1750.5);
    
    assert_eq!(rating.mean(), 1750.5);
    assert_eq!(rating.variance(), 0.0);
    assert_eq!(rating.standard_deviation(), 0.0);
    assert_eq!(rating.conservative_rating(), 1750.5);
}

#[test]
fn test_elo_team_rating_operations() {
    let rating1 = EloRating::new(1500.0);
    let rating2 = EloRating::new(1600.0);
    
    // Test single player team
    let team1 = EloTeamRating::new(rating1.clone());
    assert_eq!(team1.player_ratings().len(), 1);
    assert_eq!(team1.player_ratings()[0].rating(), 1500.0);
    
    // Test multi-player team creation (for API compatibility)
    let team2 = EloTeamRating::from_player_ratings(vec![rating1, rating2]);
    assert_eq!(team2.player_ratings().len(), 2);
    assert_eq!(team2.player_ratings()[0].rating(), 1500.0);
    assert_eq!(team2.player_ratings()[1].rating(), 1600.0);
}

#[test]
fn test_elo_rating_equality() {
    let rating1 = EloRating::new(1500.0);
    let rating2 = EloRating::new(1500.0);
    let rating3 = EloRating::new(1501.0);
    
    assert_eq!(rating1, rating2);
    assert_ne!(rating1, rating3);
}

#[test]
fn test_elo_system_consistency() {
    let system = EloSystem::new();
    
    // Rating changes should be consistent across multiple calls
    let player1 = EloRating::new(1500.0);
    let player2 = EloRating::new(1600.0);
    
    let team1 = EloTeamRating::new(player1.clone());
    let team2 = EloTeamRating::new(player2.clone());
    
    let outcome = GameOutcome::win(0, 2);
    
    let result1 = system.rate(&[team1.clone(), team2.clone()], &outcome).unwrap();
    let result2 = system.rate(&[team1, team2], &outcome).unwrap();
    
    // Results should be identical
    assert_eq!(result1[0].player_ratings()[0].rating(), result2[0].player_ratings()[0].rating());
    assert_eq!(result1[1].player_ratings()[0].rating(), result2[1].player_ratings()[0].rating());
}

#[test]
fn test_elo_rating_convergence() {
    let system = EloSystem::new();
    
    // Test that repeated games between players converge to stable ratings
    let mut strong_player = EloRating::new(1700.0);
    let mut weak_player = EloRating::new(1300.0);
    
    // Strong player wins 80% of games
    for i in 0..100 {
        let team1 = EloTeamRating::new(strong_player.clone());
        let team2 = EloTeamRating::new(weak_player.clone());
        
        let outcome = if i % 5 == 0 {
            GameOutcome::win(1, 2)  // Weak player wins 20%
        } else {
            GameOutcome::win(0, 2)  // Strong player wins 80%
        };
        
        let result = system.rate(&[team1, team2], &outcome).unwrap();
        
        strong_player = result[0].player_ratings()[0].clone();
        weak_player = result[1].player_ratings()[0].clone();
    }
    
    // After many games, ratings should reflect the 80/20 win rate
    assert!(strong_player.rating() > 1700.0);
    assert!(weak_player.rating() < 1300.0);
    
    // The gap should be significant but not extreme
    let gap = strong_player.rating() - weak_player.rating();
    assert!(gap > 200.0);
    assert!(gap < 800.0);
}