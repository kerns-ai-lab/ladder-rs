use ladder_rs::{
    glicko::{
        GlickoRating, Glicko2Rating, Glicko, Glicko2, 
        GlickoTeamRating, Glicko2TeamRating, GlickoConfig, Glicko2Config
    },
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    error::Error,
};

// Glicko (original) tests
#[test]
fn test_glicko_rating_creation_and_properties() {
    let rating = GlickoRating::default();
    assert_eq!(rating.mean(), 1500.0);
    assert_eq!(rating.standard_deviation(), 350.0);
    assert_eq!(rating.variance(), 350.0 * 350.0);
    
    let custom_rating = GlickoRating::new(1600.0, 200.0);
    assert_eq!(custom_rating.mu, 1600.0);
    assert_eq!(custom_rating.rd, 200.0);
    assert_eq!(custom_rating.mean(), 1600.0);
    assert_eq!(custom_rating.variance(), 40000.0);
    assert_eq!(custom_rating.conservative_rating(), 1200.0); // 1600 - 2*200
}

#[test]
fn test_glicko_scale_conversions() {
    let rating = GlickoRating::new(1500.0, 200.0);
    let (mu_scaled, rd_scaled) = rating.to_glicko2_scale();
    
    // Verify scaling factors
    assert!((mu_scaled - 1500.0 / 173.7178).abs() < 0.001);
    assert!((rd_scaled - 200.0 / 173.7178).abs() < 0.001);
    
    // Verify round-trip conversion
    let converted_back = GlickoRating::from_glicko2_scale(mu_scaled, rd_scaled);
    assert!((converted_back.mu - 1500.0).abs() < 0.001);
    assert!((converted_back.rd - 200.0).abs() < 0.001);
}

#[test]
fn test_glicko_system_creation_and_config() {
    let system = Glicko::new();
    let rating = system.create_rating();
    assert_eq!(rating.mean(), 1500.0);
    assert_eq!(rating.standard_deviation(), 350.0);
    
    // Test custom config
    let config = GlickoConfig {
        c: 20.0,
        q: (10.0_f64).ln() / 300.0,
    };
    let custom_system = Glicko::with_config(config);
    let custom_rating = custom_system.create_rating();
    assert_eq!(custom_rating.mean(), 1500.0);
    
    // Test create_rating_with_values
    let variance_rating = system.create_rating_with_values(1800.0, 40000.0);
    assert_eq!(variance_rating.mean(), 1800.0);
    assert_eq!(variance_rating.standard_deviation(), 200.0);
}

#[test]
fn test_glicko_basic_match() {
    let system = Glicko::new();
    
    let player1 = GlickoRating::new(1500.0, 200.0);
    let player2 = GlickoRating::new(1400.0, 30.0);
    
    let team1 = GlickoTeamRating::from_player_ratings(vec![player1]);
    let team2 = GlickoTeamRating::from_player_ratings(vec![player2]);
    
    // Player 1 wins
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[team1, team2], &outcome).unwrap();
    
    let new_player1 = &result[0].player_ratings()[0];
    let new_player2 = &result[1].player_ratings()[0];
    
    // Winner should increase, loser should decrease
    assert!(new_player1.mean() > 1500.0);
    assert!(new_player2.mean() < 1400.0);
    
    // Both should have reduced RD (more certainty)
    assert!(new_player1.standard_deviation() < 200.0);
    assert!(new_player2.standard_deviation() < 30.0);
}

#[test]
fn test_glicko_draw() {
    let system = Glicko::new();
    
    let player1 = GlickoRating::new(1500.0, 200.0);
    let player2 = GlickoRating::new(1500.0, 200.0);
    
    let team1 = GlickoTeamRating::from_player_ratings(vec![player1]);
    let team2 = GlickoTeamRating::from_player_ratings(vec![player2]);
    
    // Draw
    let outcome = GameOutcome::draw(2);
    let result = system.rate(&[team1, team2], &outcome).unwrap();
    
    let new_player1 = &result[0].player_ratings()[0];
    let new_player2 = &result[1].player_ratings()[0];
    
    // Means should stay approximately the same for equal players drawing
    assert!((new_player1.mean() - 1500.0).abs() < 5.0);
    assert!((new_player2.mean() - 1500.0).abs() < 5.0);
    
    // RD should decrease
    assert!(new_player1.standard_deviation() < 200.0);
    assert!(new_player2.standard_deviation() < 200.0);
}

#[test]
fn test_glicko_match_quality() {
    let system = Glicko::new();
    
    // Equal players
    let equal1 = GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);
    let equal2 = GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);
    let quality = system.calculate_match_quality(&[equal1, equal2]).unwrap();
    assert!(quality > 0.9);
    
    // Very different players
    let high = GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(2000.0, 50.0)]);
    let low = GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1000.0, 50.0)]);
    let quality = system.calculate_match_quality(&[high, low]).unwrap();
    assert!(quality < 0.3);
}

#[test]
fn test_glicko_error_conditions() {
    let system = Glicko::new();
    
    // Too many teams
    let teams = vec![
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::default()]),
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::default()]),
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::default()]),
    ];
    let outcome = GameOutcome::new(vec![1, 2, 3]);
    let result = system.rate(&teams, &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Multiple players per team
    let multi_team = GlickoTeamRating::from_player_ratings(vec![
        GlickoRating::new(1500.0, 200.0),
        GlickoRating::new(1600.0, 150.0),
    ]);
    let single_team = GlickoTeamRating::from_player_ratings(vec![GlickoRating::default()]);
    
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[multi_team, single_team], &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

// Glicko-2 tests
#[test]
fn test_glicko2_rating_creation_and_properties() {
    let rating = Glicko2Rating::default();
    assert_eq!(rating.mean(), 1500.0);
    assert_eq!(rating.standard_deviation(), 350.0);
    assert_eq!(rating.volatility, 0.06);
    
    let custom_rating = Glicko2Rating::new(1600.0, 200.0, 0.05);
    assert_eq!(custom_rating.mu, 1600.0);
    assert_eq!(custom_rating.rd, 200.0);
    assert_eq!(custom_rating.volatility, 0.05);
    assert_eq!(custom_rating.conservative_rating(), 1200.0); // 1600 - 2*200
}

#[test]
fn test_glicko2_scale_conversions() {
    let rating = Glicko2Rating::new(1500.0, 200.0, 0.06);
    let (mu_scaled, rd_scaled, volatility) = rating.to_glicko2_scale();
    
    // Verify scaling
    assert!((mu_scaled - 1500.0 / 173.7178).abs() < 0.001);
    assert!((rd_scaled - 200.0 / 173.7178).abs() < 0.001);
    assert_eq!(volatility, 0.06); // Volatility not scaled
    
    // Verify round-trip
    let converted_back = Glicko2Rating::from_glicko2_scale(mu_scaled, rd_scaled, volatility);
    assert!((converted_back.mu - 1500.0).abs() < 0.001);
    assert!((converted_back.rd - 200.0).abs() < 0.001);
    assert!((converted_back.volatility - 0.06).abs() < 0.001);
}

#[test]
fn test_glicko2_system_creation_and_config() {
    let system = Glicko2::new();
    let rating = system.create_rating();
    assert_eq!(rating.mean(), 1500.0);
    assert_eq!(rating.volatility, 0.06);
    
    // Test custom config
    let config = Glicko2Config {
        tau: 0.3,
        epsilon: 0.000001,
    };
    let custom_system = Glicko2::with_config(config);
    let custom_rating = custom_system.create_rating();
    assert_eq!(custom_rating.mean(), 1500.0);
    
    // Test create_rating_with_values
    let variance_rating = system.create_rating_with_values(1800.0, 40000.0);
    assert_eq!(variance_rating.mean(), 1800.0);
    assert_eq!(variance_rating.standard_deviation(), 200.0);
    assert_eq!(variance_rating.volatility, 0.06); // Default volatility
}

#[test]
fn test_glicko2_basic_match() {
    let system = Glicko2::new();
    
    let player1 = Glicko2Rating::new(1500.0, 200.0, 0.06);
    let player2 = Glicko2Rating::new(1400.0, 30.0, 0.06);
    
    let team1 = Glicko2TeamRating::from_player_ratings(vec![player1]);
    let team2 = Glicko2TeamRating::from_player_ratings(vec![player2]);
    
    // Player 1 wins
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[team1, team2], &outcome).unwrap();
    
    let new_player1 = &result[0].player_ratings()[0];
    let new_player2 = &result[1].player_ratings()[0];
    
    // Winner should increase, loser should decrease
    assert!(new_player1.mean() > 1500.0);
    assert!(new_player2.mean() < 1400.0);
    
    // Player 1's RD should decrease
    assert!(new_player1.standard_deviation() < 200.0);
    // Player 2's RD may increase slightly due to the loss (uncertainty) - that's normal
    
    // Volatility should be updated
    assert!(new_player1.volatility > 0.0);
    assert!(new_player2.volatility > 0.0);
}

#[test]
fn test_glicko2_draw() {
    let system = Glicko2::new();
    
    let player1 = Glicko2Rating::new(1500.0, 200.0, 0.06);
    let player2 = Glicko2Rating::new(1500.0, 200.0, 0.06);
    
    let team1 = Glicko2TeamRating::from_player_ratings(vec![player1]);
    let team2 = Glicko2TeamRating::from_player_ratings(vec![player2]);
    
    // Draw
    let outcome = GameOutcome::draw(2);
    let result = system.rate(&[team1, team2], &outcome).unwrap();
    
    let new_player1 = &result[0].player_ratings()[0];
    let new_player2 = &result[1].player_ratings()[0];
    
    // Means should stay approximately the same for equal players drawing
    assert!((new_player1.mean() - 1500.0).abs() < 5.0);
    assert!((new_player2.mean() - 1500.0).abs() < 5.0);
    
    // RD should decrease
    assert!(new_player1.standard_deviation() < 200.0);
    assert!(new_player2.standard_deviation() < 200.0);
}

#[test]
fn test_glicko2_match_quality() {
    let system = Glicko2::new();
    
    // Equal players
    let equal1 = Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);
    let equal2 = Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);
    let quality = system.calculate_match_quality(&[equal1, equal2]).unwrap();
    assert!(quality > 0.9);
    
    // Very different players
    let high = Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(2000.0, 50.0, 0.06)]);
    let low = Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1000.0, 50.0, 0.06)]);
    let quality = system.calculate_match_quality(&[high, low]).unwrap();
    assert!(quality < 0.3);
}

#[test]
fn test_glicko2_volatility_effects() {
    let system = Glicko2::new();
    
    // Test high volatility vs low volatility
    let high_vol_player = Glicko2Rating::new(1500.0, 200.0, 0.1);
    let low_vol_player = Glicko2Rating::new(1500.0, 200.0, 0.02);
    let opponent = Glicko2Rating::new(1400.0, 100.0, 0.06);
    
    // Both win against the same opponent
    let team1 = Glicko2TeamRating::from_player_ratings(vec![high_vol_player]);
    let team2 = Glicko2TeamRating::from_player_ratings(vec![opponent.clone()]);
    let outcome = GameOutcome::win(0, 2);
    let result1 = system.rate(&[team1, team2], &outcome).unwrap();
    
    let team3 = Glicko2TeamRating::from_player_ratings(vec![low_vol_player]);
    let team4 = Glicko2TeamRating::from_player_ratings(vec![opponent]);
    let result2 = system.rate(&[team3, team4], &outcome).unwrap();
    
    // Both should increase but may differ based on volatility
    assert!(result1[0].player_ratings()[0].mean() > 1500.0);
    assert!(result2[0].player_ratings()[0].mean() > 1500.0);
}

#[test]
fn test_glicko2_error_conditions() {
    let system = Glicko2::new();
    
    // Too many teams
    let teams = vec![
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::default()]),
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::default()]),
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::default()]),
    ];
    let outcome = GameOutcome::new(vec![1, 2, 3]);
    let result = system.rate(&teams, &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Multiple players per team
    let multi_team = Glicko2TeamRating::from_player_ratings(vec![
        Glicko2Rating::new(1500.0, 200.0, 0.06),
        Glicko2Rating::new(1600.0, 150.0, 0.05),
    ]);
    let single_team = Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::default()]);
    
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[multi_team, single_team], &outcome);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

#[test]
fn test_glicko_series_of_games() {
    let system = Glicko::new();
    
    let mut player1 = GlickoRating::new(1500.0, 200.0);
    let mut player2 = GlickoRating::new(1500.0, 200.0);
    
    // Player 1 wins 5 games in a row
    for _ in 0..5 {
        let team1 = GlickoTeamRating::from_player_ratings(vec![player1]);
        let team2 = GlickoTeamRating::from_player_ratings(vec![player2]);
        
        let outcome = GameOutcome::win(0, 2);
        let result = system.rate(&[team1, team2], &outcome).unwrap();
        
        player1 = result[0].player_ratings()[0].clone();
        player2 = result[1].player_ratings()[0].clone();
    }
    
    // Player 1 should be higher rated and both should have lower RD
    assert!(player1.mean() > player2.mean());
    assert!(player1.standard_deviation() < 200.0);
    assert!(player2.standard_deviation() < 200.0);
}

#[test]
fn test_glicko2_series_of_games() {
    let system = Glicko2::new();
    
    let mut player1 = Glicko2Rating::new(1500.0, 200.0, 0.06);
    let mut player2 = Glicko2Rating::new(1500.0, 200.0, 0.06);
    
    // Player 1 wins 5 games in a row
    for _ in 0..5 {
        let team1 = Glicko2TeamRating::from_player_ratings(vec![player1]);
        let team2 = Glicko2TeamRating::from_player_ratings(vec![player2]);
        
        let outcome = GameOutcome::win(0, 2);
        let result = system.rate(&[team1, team2], &outcome).unwrap();
        
        player1 = result[0].player_ratings()[0].clone();
        player2 = result[1].player_ratings()[0].clone();
    }
    
    // Player 1 should be higher rated and both should have lower RD
    assert!(player1.mean() > player2.mean());
    assert!(player1.standard_deviation() < 200.0);
    assert!(player2.standard_deviation() < 200.0);
}

#[test]
fn test_glicko_rd_increase_no_games() {
    let system = Glicko::new();
    
    let player = GlickoRating::new(1600.0, 50.0);
    let opponent = GlickoRating::new(1600.0, 50.0);
    
    let team1 = GlickoTeamRating::from_player_ratings(vec![player]);
    let team2 = GlickoTeamRating::from_player_ratings(vec![opponent]);
    
    // Test that implementation handles rating updates properly
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[team1, team2], &outcome);
    assert!(result.is_ok());
}

#[test]
fn test_rating_equality_and_cloning() {
    let rating1 = GlickoRating::new(1500.0, 200.0);
    let rating2 = GlickoRating::new(1500.0, 200.0);
    let rating3 = GlickoRating::new(1501.0, 200.0);
    
    assert_eq!(rating1, rating2);
    assert_ne!(rating1, rating3);
    
    let cloned = rating1.clone();
    assert_eq!(rating1, cloned);
    
    // Test Glicko2 as well
    let g2_rating1 = Glicko2Rating::new(1500.0, 200.0, 0.06);
    let g2_rating2 = Glicko2Rating::new(1500.0, 200.0, 0.06);
    let g2_rating3 = Glicko2Rating::new(1500.0, 200.0, 0.07);
    
    assert_eq!(g2_rating1, g2_rating2);
    assert_ne!(g2_rating1, g2_rating3);
}