use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    glicko::{
        Glicko, Glicko2, Glicko2Config, Glicko2Rating, Glicko2TeamRating, GlickoConfig,
        GlickoRating, GlickoTeamRating,
    },
};

#[test]
fn test_glicko_rating_period_updates() {
    let glicko = Glicko::new();

    // Test player with no games (RD should increase)
    let inactive_player = GlickoRating::new(1500.0, 50.0);
    let team = GlickoTeamRating::from_player_ratings(vec![inactive_player.clone()]);

    // Since we can't directly test no-games scenario, test that RD decreases after games
    let opponent = GlickoRating::new(1400.0, 80.0);
    let opponent_team = GlickoTeamRating::from_player_ratings(vec![opponent]);

    let outcome = GameOutcome::win(0, 2);
    let result = glicko.rate(&[team, opponent_team], &outcome).unwrap();

    let updated_rating = &result[0].player_ratings()[0];
    assert!(
        updated_rating.rd < inactive_player.rd,
        "RD should decrease after games"
    );
    assert!(
        updated_rating.mu > inactive_player.mu,
        "Rating should increase after win"
    );
}

#[test]
fn test_glicko_custom_configuration() {
    // Test with custom q value (conversion factor)
    let custom_config = GlickoConfig {
        c: 15.8, // Same as default
        q: (10.0_f64).ln() / 300.0, // Different q value to affect g() function
    };

    let custom_glicko = Glicko::with_config(custom_config);
    let default_glicko = Glicko::new();

    let player = GlickoRating::new(1500.0, 100.0);
    let opponent = GlickoRating::new(1600.0, 120.0);

    let team1 = GlickoTeamRating::from_player_ratings(vec![player.clone()]);
    let team2 = GlickoTeamRating::from_player_ratings(vec![opponent.clone()]);

    let outcome = GameOutcome::win(0, 2);

    let custom_result = custom_glicko
        .rate(&[team1.clone(), team2.clone()], &outcome)
        .unwrap();
    let default_result = default_glicko.rate(&[team1, team2], &outcome).unwrap();

    // Custom config should produce different results
    let custom_rating = &custom_result[0].player_ratings()[0];
    let default_rating = &default_result[0].player_ratings()[0];

    // The difference might be small, so just check they're different enough to notice
    let mu_diff = (custom_rating.mu - default_rating.mu).abs();
    let rd_diff = (custom_rating.rd - default_rating.rd).abs();

    assert!(
        mu_diff > 0.001 || rd_diff > 0.001,
        "Custom config should affect rating updates"
    );
}

#[test]
fn test_glicko2_volatility_updates() {
    let glicko2 = Glicko2::new();

    // Test player with consistent performance (volatility should decrease)
    let stable_player = Glicko2Rating::new(1500.0, 200.0, 0.06);
    let consistent_opponent = Glicko2Rating::new(1400.0, 100.0, 0.06);

    let mut current_rating = stable_player.clone();

    // Multiple games with expected results
    for _ in 0..5 {
        let team1 = Glicko2TeamRating::from_player_ratings(vec![current_rating.clone()]);
        let team2 = Glicko2TeamRating::from_player_ratings(vec![consistent_opponent.clone()]);

        let outcome = GameOutcome::win(0, 2); // Consistent wins
        let result = glicko2.rate(&[team1, team2], &outcome).unwrap();

        current_rating = result[0].player_ratings()[0].clone();
    }

    // Volatility should decrease with consistent performance
    assert!(
        current_rating.volatility < stable_player.volatility,
        "Consistent performance should reduce volatility"
    );
    assert!(
        current_rating.rd < stable_player.rd,
        "Multiple games should reduce rating deviation"
    );
}

#[test]
fn test_glicko2_scale_conversions() {
    let rating = Glicko2Rating::new(1500.0, 200.0, 0.06);

    // Test conversion to Glicko-2 scale
    let (mu_scaled, rd_scaled, vol_scaled) = rating.to_glicko2_scale();

    // Verify scale conversion (divide by 173.7178)
    assert!((mu_scaled - 1500.0 / 173.7178).abs() < 1e-10);
    assert!((rd_scaled - 200.0 / 173.7178).abs() < 1e-10);
    assert_eq!(vol_scaled, 0.06); // Volatility unchanged

    // Test round-trip conversion
    let converted_back = Glicko2Rating::from_glicko2_scale(mu_scaled, rd_scaled, vol_scaled);

    assert!((converted_back.mu - rating.mu).abs() < 1e-10);
    assert!((converted_back.rd - rating.rd).abs() < 1e-10);
    assert!((converted_back.volatility - rating.volatility).abs() < 1e-10);
}

#[test]
fn test_glicko2_custom_configuration() {
    // Test with custom tau and epsilon values
    let custom_config = Glicko2Config {
        tau: 0.8,        // Higher than default 0.5
        epsilon: 0.0001, // Tighter convergence
    };

    let custom_glicko2 = Glicko2::with_config(custom_config);
    let default_glicko2 = Glicko2::new();

    let player = Glicko2Rating::new(1600.0, 150.0, 0.08);
    let opponent = Glicko2Rating::new(1500.0, 200.0, 0.05);

    let team1 = Glicko2TeamRating::from_player_ratings(vec![player.clone()]);
    let team2 = Glicko2TeamRating::from_player_ratings(vec![opponent.clone()]);

    let outcome = GameOutcome::win(1, 2); // Upset

    let custom_result = custom_glicko2
        .rate(&[team1.clone(), team2.clone()], &outcome)
        .unwrap();
    let default_result = default_glicko2.rate(&[team1, team2], &outcome).unwrap();

    // Different configurations should produce different volatilities
    let custom_volatility = custom_result[0].player_ratings()[0].volatility;
    let default_volatility = default_result[0].player_ratings()[0].volatility;

    assert_ne!(
        custom_volatility, default_volatility,
        "Custom tau should affect volatility updates"
    );
}

#[test]
fn test_glicko_rating_properties() {
    let rating = GlickoRating::new(1650.0, 120.0);

    // Test Rating trait implementation
    assert_eq!(rating.mean(), 1650.0);
    assert_eq!(rating.variance(), 120.0 * 120.0);
    assert_eq!(rating.standard_deviation(), 120.0);
    assert_eq!(rating.conservative_rating(), 1650.0 - 2.0 * 120.0); // μ - 2σ

    // Test Glicko-specific methods
    assert_eq!(rating.mu, 1650.0);
    assert_eq!(rating.rd, 120.0);
}

#[test]
fn test_glicko2_rating_properties() {
    let rating = Glicko2Rating::new(1750.0, 80.0, 0.045);

    // Test Rating trait implementation
    assert_eq!(rating.mean(), 1750.0);
    assert_eq!(rating.variance(), 80.0 * 80.0);
    assert_eq!(rating.standard_deviation(), 80.0);
    assert_eq!(rating.conservative_rating(), 1750.0 - 2.0 * 80.0); // μ - 2σ

    // Test Glicko2-specific properties
    assert_eq!(rating.mu, 1750.0);
    assert_eq!(rating.rd, 80.0);
    assert_eq!(rating.volatility, 0.045);
}

#[test]
fn test_glicko_extreme_rd_values() {
    let glicko = Glicko::new();

    // Test with very high RD (new/inactive player)
    let new_player = GlickoRating::new(1500.0, 350.0);
    let experienced_player = GlickoRating::new(1600.0, 30.0);

    let team1 = GlickoTeamRating::from_player_ratings(vec![new_player.clone()]);
    let team2 = GlickoTeamRating::from_player_ratings(vec![experienced_player.clone()]);

    let outcome = GameOutcome::win(0, 2); // New player wins
    let result = glicko.rate(&[team1, team2], &outcome).unwrap();

    let updated_new = &result[0].player_ratings()[0];
    let updated_experienced = &result[1].player_ratings()[0];

    // New player should have large rating change due to high RD
    let new_change = (updated_new.mu - new_player.mu).abs();
    let exp_change = (updated_experienced.mu - experienced_player.mu).abs();

    assert!(
        new_change > exp_change,
        "Player with higher RD should have larger rating change"
    );

    // RD should decrease for new player
    assert!(
        updated_new.rd < new_player.rd,
        "RD should decrease after game"
    );
}

#[test]
fn test_glicko_error_conditions() {
    let glicko = Glicko::new();

    // Test invalid team counts
    let team = GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 100.0)]);

    // Single team
    let single_outcome = GameOutcome::new(vec![1]);
    let result = glicko.rate(&[team.clone()], &single_outcome);
    assert!(result.is_err(), "Single team should cause error");

    // Three teams
    let three_outcome = GameOutcome::new(vec![1, 2, 3]);
    let result = glicko.rate(&[team.clone(), team.clone(), team], &three_outcome);
    assert!(result.is_err(), "Three teams should cause error for Glicko");
}

#[test]
fn test_glicko2_error_conditions() {
    let glicko2 = Glicko2::new();

    // Test invalid team counts
    let team =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 100.0, 0.06)]);

    // Multiple teams (Glicko2 only supports 1v1)
    let multi_outcome = GameOutcome::new(vec![1, 2, 3]);
    let result = glicko2.rate(&[team.clone(), team.clone(), team], &multi_outcome);
    assert!(
        result.is_err(),
        "Multiple teams should cause error for Glicko2"
    );
}

#[test]
fn test_glicko_match_quality_edge_cases() {
    let glicko = Glicko::new();

    // Test with identical ratings and RDs
    let identical_rating = GlickoRating::new(1500.0, 100.0);
    let team1 = GlickoTeamRating::from_player_ratings(vec![identical_rating.clone()]);
    let team2 = GlickoTeamRating::from_player_ratings(vec![identical_rating]);

    let quality = glicko.calculate_match_quality(&[team1, team2]).unwrap();
    assert!(
        quality > 0.9,
        "Identical players should have high match quality"
    );

    // Test with very different ratings
    let high_rating = GlickoRating::new(2000.0, 50.0);
    let low_rating = GlickoRating::new(1000.0, 50.0);

    let team_high = GlickoTeamRating::from_player_ratings(vec![high_rating]);
    let team_low = GlickoTeamRating::from_player_ratings(vec![low_rating]);

    let quality_diff = glicko
        .calculate_match_quality(&[team_high, team_low])
        .unwrap();
    assert!(
        quality_diff < 0.3,
        "Very different players should have low match quality"
    );
}

#[test]
fn test_glicko_draw_handling() {
    let glicko = Glicko::new();

    let player1 = GlickoRating::new(1600.0, 80.0);
    let player2 = GlickoRating::new(1400.0, 120.0);

    let team1 = GlickoTeamRating::from_player_ratings(vec![player1.clone()]);
    let team2 = GlickoTeamRating::from_player_ratings(vec![player2.clone()]);

    // Test draw outcome
    let draw_outcome = GameOutcome::draw(2);
    let result = glicko.rate(&[team1, team2], &draw_outcome).unwrap();

    let updated1 = &result[0].player_ratings()[0];
    let updated2 = &result[1].player_ratings()[0];

    // In a draw, higher rated player should lose rating, lower should gain
    assert!(
        updated1.mu < player1.mu,
        "Higher rated player should lose rating in draw"
    );
    assert!(
        updated2.mu > player2.mu,
        "Lower rated player should gain rating in draw"
    );
}

#[test]
fn test_glicko_convergence_over_time() {
    let glicko = Glicko::new();

    let mut player1 = GlickoRating::new(1500.0, 200.0);
    let mut player2 = GlickoRating::new(1500.0, 200.0);

    // Simulate player 1 consistently beating player 2
    for _ in 0..20 {
        let team1 = GlickoTeamRating::from_player_ratings(vec![player1.clone()]);
        let team2 = GlickoTeamRating::from_player_ratings(vec![player2.clone()]);

        let outcome = GameOutcome::win(0, 2);
        let result = glicko.rate(&[team1, team2], &outcome).unwrap();

        player1 = result[0].player_ratings()[0].clone();
        player2 = result[1].player_ratings()[0].clone();
    }

    // After many games, ratings should converge with significant difference
    assert!(
        player1.mu > 1600.0,
        "Consistent winner should have higher rating"
    );
    assert!(
        player2.mu < 1400.0,
        "Consistent loser should have lower rating"
    );

    // RD should decrease with more games (but may not go below a certain threshold)
    assert!(
        player1.rd <= 200.0,
        "RD should not increase with more games"
    );
    assert!(
        player2.rd <= 200.0,
        "RD should not increase with more games"
    );
}

#[test]
fn test_glicko_defaults_and_creation() {
    let glicko = Glicko::new();
    let default_glicko = Glicko::default();

    // Test default rating creation
    let rating1 = glicko.create_rating();
    let rating2 = default_glicko.create_rating();

    assert_eq!(rating1.mu, rating2.mu);
    assert_eq!(rating1.rd, rating2.rd);
    assert_eq!(rating1.mu, 1500.0);
    assert_eq!(rating1.rd, 350.0);

    // Test create_rating_with_values
    let custom_rating = glicko.create_rating_with_values(1800.0, 10000.0);
    assert_eq!(custom_rating.mu, 1800.0);
    assert_eq!(custom_rating.rd, 100.0); // sqrt(10000)
}

#[test]
fn test_glicko2_defaults_and_creation() {
    let glicko2 = Glicko2::new();
    let default_glicko2 = Glicko2::default();

    // Test default rating creation
    let rating1 = glicko2.create_rating();
    let rating2 = default_glicko2.create_rating();

    assert_eq!(rating1.mu, rating2.mu);
    assert_eq!(rating1.rd, rating2.rd);
    assert_eq!(rating1.volatility, rating2.volatility);
    assert_eq!(rating1.mu, 1500.0);
    assert_eq!(rating1.rd, 350.0);
    assert_eq!(rating1.volatility, 0.06);

    // Test create_rating_with_values
    let custom_rating = glicko2.create_rating_with_values(1900.0, 6400.0);
    assert_eq!(custom_rating.mu, 1900.0);
    assert_eq!(custom_rating.rd, 80.0); // sqrt(6400)
    assert_eq!(custom_rating.volatility, 0.06); // Default volatility
}
