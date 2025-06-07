use ladder_rs::{
    trueskill::{TrueSkill, TrueSkillRating},
    core::{Rating, RatingSystem},
};

#[test]
fn test_trueskill_default_parameters() {
    let ts = TrueSkill::new();
    let rating = ts.create_rating();
    
    // Check default values
    assert_eq!(rating.mean(), 25.0);
    
    let ratio: f64 = 25.0 / 3.0;
    let expected_variance: f64 = ratio * ratio;
    assert!((rating.variance() - expected_variance).abs() < f64::EPSILON);
}

#[test]
fn test_trueskill_custom_parameters() {
    let mu_0: f64 = 1500.0;
    let sigma_0: f64 = 300.0;
    let sigma_0_squared: f64 = sigma_0 * sigma_0;
    let beta_squared: f64 = (sigma_0 / 2.0) * (sigma_0 / 2.0);
    let gamma_squared: f64 = (sigma_0 / 100.0) * (sigma_0 / 100.0);
    let draw_probability: f64 = 0.05;
    
    let ts = TrueSkill::with_parameters(
        mu_0,
        sigma_0_squared,
        beta_squared,
        gamma_squared,
        draw_probability,
    ).unwrap();
    
    let rating = ts.create_rating();
    
    // Check custom values
    assert_eq!(rating.mean(), mu_0);
    assert!((rating.variance() - sigma_0_squared).abs() < f64::EPSILON);
}

#[test]
fn test_trueskill_invalid_parameters() {
    // Calculate values once
    let sigma_0: f64 = 25.0 / 3.0;
    let sigma_0_squared: f64 = sigma_0 * sigma_0;
    let beta: f64 = 25.0 / 6.0;
    let beta_squared: f64 = beta * beta;
    let gamma: f64 = 25.0 / 300.0;
    let gamma_squared: f64 = gamma * gamma;
    
    // Test negative mean
    let result = TrueSkill::with_parameters(
        -25.0,
        sigma_0_squared,
        beta_squared,
        gamma_squared,
        0.1,
    );
    assert!(result.is_err());
    
    // Test zero variance
    let result = TrueSkill::with_parameters(
        25.0,
        0.0,
        beta_squared,
        gamma_squared,
        0.1,
    );
    assert!(result.is_err());
    
    // Test negative beta squared
    let result = TrueSkill::with_parameters(
        25.0,
        sigma_0_squared,
        -1.0,
        gamma_squared,
        0.1,
    );
    assert!(result.is_err());
    
    // Test invalid draw probability
    let result = TrueSkill::with_parameters(
        25.0,
        sigma_0_squared,
        beta_squared,
        gamma_squared,
        0.0,
    );
    assert!(result.is_err());
    
    let result = TrueSkill::with_parameters(
        25.0,
        sigma_0_squared,
        beta_squared,
        gamma_squared,
        1.0,
    );
    assert!(result.is_err());
}

#[test]
fn test_trueskill_rating_operations() {
    let mean = 25.0;
    let variance = 64.0;
    let rating = TrueSkillRating::new(mean, variance).unwrap();
    
    // Test basic properties
    assert_eq!(rating.mean(), mean);
    assert_eq!(rating.variance(), variance);
    assert_eq!(rating.standard_deviation(), 8.0);
    
    // Test precision
    assert!((rating.precision() - 1.0/variance).abs() < f64::EPSILON);
    
    // Test precision-adjusted mean
    assert!((rating.precision_adjusted_mean() - mean/variance).abs() < f64::EPSILON);
    
    // Test conservative rating
    assert_eq!(rating.conservative_rating(), mean - 3.0 * 8.0);
}

#[test]
fn test_invalid_rating_creation() {
    // Test zero variance
    let result = TrueSkillRating::new(25.0, 0.0);
    assert!(result.is_err());
    
    // Test negative variance
    let result = TrueSkillRating::new(25.0, -1.0);
    assert!(result.is_err());
}

#[test]
fn test_rating_system_implementation() {
    let ts = TrueSkill::new();
    
    // Test create_rating
    let rating = ts.create_rating();
    assert_eq!(rating.mean(), 25.0);
    
    // Test create_rating_with_values
    let custom_rating = ts.create_rating_with_values(30.0, 100.0);
    assert_eq!(custom_rating.mean(), 30.0);
    assert_eq!(custom_rating.variance(), 100.0);
    
    // Check that rate and calculate_match_quality return proper errors (since they're not implemented yet)
    use ladder_rs::core::{TeamRating, GameOutcome};
    use ladder_rs::trueskill::TrueSkillTeam;
    
    let team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let teams = vec![team1, team2];
    let outcome = GameOutcome::win(0, 2).unwrap();
    
    let rate_result = ts.rate(&teams, &outcome);
    assert!(rate_result.is_err());
    
    let quality_result = ts.calculate_match_quality(&teams);
    assert!(quality_result.is_err());
}