use crate::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    error::{Error, Result},
};
use statrs::distribution::{Normal, ContinuousCDF, Continuous};
use std::collections::HashMap;

/// Implementation of Microsoft's TrueSkill rating system.
/// 
/// TrueSkill is a Bayesian skill rating system developed by Microsoft Research
/// primarily for Xbox Live. It extends the Elo and Glicko rating systems by
/// supporting teams and providing match quality prediction.
#[derive(Debug, Clone)]
pub struct TrueSkill {
    /// Initial mean (μ₀) for new players (default: 25.0)
    mu_0: f64,
    
    /// Initial variance (σ₀²) for new players (default: (25/3)²)
    sigma_0_squared: f64,
    
    /// Performance variance (β²) (default: (σ₀/2)²)
    beta_squared: f64,
    
    /// Dynamics variance (γ²) - rate at which player skills can evolve over time (default: (σ₀/100)²)
    gamma_squared: f64,
    
    /// Draw probability - used to calculate draw margin
    draw_probability: f64,
    
    /// Draw margin (ε) - calculated from draw_probability
    draw_margin: f64,
    
    /// Convergence threshold for message passing iterations
    convergence_threshold: f64,
    
    /// Maximum number of iterations for message passing
    max_iterations: usize,
}

impl Default for TrueSkill {
    fn default() -> Self {
        let mu_0: f64 = 25.0;
        let sigma_0: f64 = mu_0 / 3.0;
        
        Self {
            mu_0,
            sigma_0_squared: sigma_0 * sigma_0,
            beta_squared: (sigma_0 / 2.0) * (sigma_0 / 2.0),
            gamma_squared: (sigma_0 / 100.0) * (sigma_0 / 100.0),
            draw_probability: 0.10, // 10% default draw probability
            draw_margin: 0.0, // Will be calculated in new()
            convergence_threshold: 0.0001,
            max_iterations: 20,
        }.calculate_draw_margin()
    }
}

impl TrueSkill {
    /// Creates a new TrueSkill instance with default parameters.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Creates a new TrueSkill instance with custom parameters.
    pub fn with_parameters(
        mu_0: f64,
        sigma_0_squared: f64,
        beta_squared: f64,
        gamma_squared: f64,
        draw_probability: f64,
    ) -> Result<Self> {
        if mu_0 <= 0.0 {
            return Err(Error::InvalidConfiguration(
                "Initial mean (mu_0) must be positive".to_string(),
            ));
        }
        
        if sigma_0_squared <= 0.0 {
            return Err(Error::InvalidConfiguration(
                "Initial variance (sigma_0_squared) must be positive".to_string(),
            ));
        }
        
        if beta_squared <= 0.0 {
            return Err(Error::InvalidConfiguration(
                "Performance variance (beta_squared) must be positive".to_string(),
            ));
        }
        
        if gamma_squared < 0.0 {
            return Err(Error::InvalidConfiguration(
                "Dynamics variance (gamma_squared) must be non-negative".to_string(),
            ));
        }
        
        if draw_probability <= 0.0 || draw_probability >= 1.0 {
            return Err(Error::InvalidConfiguration(
                "Draw probability must be between 0 and 1 (exclusive)".to_string(),
            ));
        }
        
        Ok(Self {
            mu_0,
            sigma_0_squared,
            beta_squared,
            gamma_squared,
            draw_probability,
            draw_margin: 0.0, // Will be calculated below
            convergence_threshold: 0.0001,
            max_iterations: 20,
        }.calculate_draw_margin())
    }
    
    /// Calculates the draw margin (ε) based on the draw probability
    fn calculate_draw_margin(mut self) -> Self {
        // For simplicity, we'll use the two-player case as reference
        // σ_diff = √(2β² + σ₁² + σ₂²) ≈ √(2β² + 2σ₀²)
        let sigma_diff = (2.0 * self.beta_squared + 2.0 * self.sigma_0_squared).sqrt();
        
        // Using the formula from the TrueSkill paper:
        // draw_probability = Φ(ε/σ_diff) - Φ(-ε/σ_diff)
        // This requires numerical solving, but we can use approximation:
        
        // For standard normal distribution:
        // Φ(x) - Φ(-x) = 2Φ(x) - 1 for x > 0
        // So draw_probability = 2Φ(ε/σ_diff) - 1
        // Therefore: Φ(ε/σ_diff) = (draw_probability + 1) / 2
        // And: ε/σ_diff = Φ⁻¹((draw_probability + 1) / 2)
        
        let normal = Normal::new(0.0, 1.0).unwrap();
        let inv_cdf_input = (self.draw_probability + 1.0) / 2.0;
        let epsilon_div_sigma = normal.inverse_cdf(inv_cdf_input);
        
        self.draw_margin = epsilon_div_sigma * sigma_diff;
        self
    }
}

/// TrueSkill player rating represented by a Gaussian distribution.
#[derive(Debug, Clone, PartialEq)]
pub struct TrueSkillRating {
    /// Mean (μ) of the skill distribution
    mean: f64,
    
    /// Variance (σ²) of the skill distribution
    variance: f64,
}

impl TrueSkillRating {
    /// Creates a new TrueSkill rating with the given mean and variance.
    pub fn new(mean: f64, variance: f64) -> Result<Self> {
        if variance <= 0.0 {
            return Err(Error::InvalidInput(
                "Variance must be positive".to_string(),
            ));
        }
        
        Ok(Self { mean, variance })
    }
    
    /// Returns the precision (π = 1/σ²) of the rating.
    pub fn precision(&self) -> f64 {
        1.0 / self.variance
    }
    
    /// Returns the precision-adjusted mean (τ = πμ) of the rating.
    pub fn precision_adjusted_mean(&self) -> f64 {
        self.precision() * self.mean
    }
}

impl Rating for TrueSkillRating {
    fn mean(&self) -> f64 {
        self.mean
    }
    
    fn variance(&self) -> f64 {
        self.variance
    }
    
    fn standard_deviation(&self) -> f64 {
        self.variance.sqrt()
    }
    
    /// Returns a conservative skill estimate (μ - 3σ), as recommended by the TrueSkill paper.
    fn conservative_rating(&self) -> f64 {
        self.mean - 3.0 * self.standard_deviation()
    }
}

/// Represents a team in TrueSkill, consisting of multiple players.
#[derive(Debug, Clone)]
pub struct TrueSkillTeam {
    /// Ratings of individual players in the team
    ratings: Vec<TrueSkillRating>,
}

impl TeamRating for TrueSkillTeam {
    type PlayerRating = TrueSkillRating;
    
    fn player_ratings(&self) -> &[Self::PlayerRating] {
        &self.ratings
    }
    
    fn from_player_ratings(ratings: Vec<Self::PlayerRating>) -> Self {
        Self { ratings }
    }
}

/// Factor graph primitives for the TrueSkill implementation
mod factor_graph {
    use super::*;
    
    /// Represents a message in the factor graph (a Gaussian)
    #[derive(Debug, Clone, PartialEq)]
    pub struct GaussianMessage {
        /// Precision (π = 1/σ²)
        pub precision: f64,
        
        /// Precision-adjusted mean (τ = πμ)
        pub precision_adjusted_mean: f64,
    }
    
    impl GaussianMessage {
        /// Creates a new Gaussian message with the given precision and precision-adjusted mean.
        pub fn new(precision: f64, precision_adjusted_mean: f64) -> Self {
            Self { precision, precision_adjusted_mean }
        }
        
        /// Creates a uniform (non-informative) message
        pub fn uniform() -> Self {
            Self { precision: 0.0, precision_adjusted_mean: 0.0 }
        }
        
        /// Creates a new Gaussian message from mean and variance.
        pub fn from_mean_and_variance(mean: f64, variance: f64) -> Result<Self> {
            if variance <= 0.0 {
                return Err(Error::InvalidInput(
                    "Variance must be positive".to_string(),
                ));
            }
            
            let precision = 1.0 / variance;
            let precision_adjusted_mean = precision * mean;
            
            Ok(Self { precision, precision_adjusted_mean })
        }
        
        /// Returns the mean (μ) of the message.
        pub fn mean(&self) -> f64 {
            if self.precision.abs() < f64::EPSILON {
                0.0
            } else {
                self.precision_adjusted_mean / self.precision
            }
        }
        
        /// Returns the variance (σ²) of the message.
        pub fn variance(&self) -> f64 {
            if self.precision.abs() < f64::EPSILON {
                f64::INFINITY
            } else {
                1.0 / self.precision
            }
        }
        
        /// Multiplies two Gaussian messages (product of Gaussians)
        pub fn multiply(&self, other: &Self) -> Self {
            Self {
                precision: self.precision + other.precision,
                precision_adjusted_mean: self.precision_adjusted_mean + other.precision_adjusted_mean,
            }
        }
        
        /// Divides two Gaussian messages (division of Gaussians)
        /// Returns Ok(uniform) if the division would result in non-positive precision
        pub fn divide(&self, other: &Self) -> Result<Self> {
            let new_precision = self.precision - other.precision;
            let new_precision_adjusted_mean = self.precision_adjusted_mean - other.precision_adjusted_mean;
            
            if new_precision <= 1e-10 {
                // Return a uniform message instead of failing
                Ok(Self::uniform())
            } else {
                Ok(Self {
                    precision: new_precision,
                    precision_adjusted_mean: new_precision_adjusted_mean,
                })
            }
        }
    }
    
    /// Simplified TrueSkill implementation using a different approach
    /// This implementation doesn't use full message passing but a simpler approximation
    pub struct SimplifiedTrueSkillUpdater {
        beta_squared: f64,
        gamma_squared: f64,
        draw_margin: f64,
    }
    
    impl SimplifiedTrueSkillUpdater {
        pub fn new(beta_squared: f64, gamma_squared: f64, draw_margin: f64) -> Self {
            Self {
                beta_squared,
                gamma_squared,
                draw_margin,
            }
        }
        
        pub fn update_ratings(
            &self,
            team1_rating: &TrueSkillRating,
            team2_rating: &TrueSkillRating,
            outcome: TwoPlayerOutcome,
        ) -> Result<(TrueSkillRating, TrueSkillRating)> {
            // Add dynamics variance
            let s1_mean = team1_rating.mean();
            let s1_var = team1_rating.variance() + self.gamma_squared;
            let s2_mean = team2_rating.mean();
            let s2_var = team2_rating.variance() + self.gamma_squared;
            
            // Performance variance for each player
            let p1_var = s1_var + self.beta_squared;
            let p2_var = s2_var + self.beta_squared;
            
            // Team performance difference variance
            let diff_var = p1_var + p2_var;
            let diff_std = diff_var.sqrt();
            let diff_mean = s1_mean - s2_mean;
            
            // Calculate V and W functions based on outcome
            let normal = Normal::new(0.0, 1.0).unwrap();
            let t = diff_mean / diff_std;
            let epsilon = self.draw_margin / diff_std;
            
            let (v, w) = match outcome {
                TwoPlayerOutcome::Player1Wins => {
                    let cdf_val = normal.cdf(t - epsilon);
                    if cdf_val < 1e-10 {
                        (0.0, 0.0)
                    } else {
                        let v = normal.pdf(t - epsilon) / cdf_val;
                        let w = v * (v + t - epsilon);
                        (v, w)
                    }
                }
                TwoPlayerOutcome::Player2Wins => {
                    let cdf_val = normal.cdf(-t - epsilon);
                    if cdf_val < 1e-10 {
                        (0.0, 0.0)
                    } else {
                        let v = normal.pdf(-t - epsilon) / cdf_val;
                        let w = v * (v - t - epsilon);
                        (-v, w) // Negative v for player 2 winning
                    }
                }
                TwoPlayerOutcome::Draw => {
                    let phi_upper = normal.cdf(epsilon - t);
                    let phi_lower = normal.cdf(-epsilon - t);
                    let pdf_upper = normal.pdf(epsilon - t);
                    let pdf_lower = normal.pdf(-epsilon - t);
                    
                    let denom = phi_upper - phi_lower;
                    if denom.abs() < 1e-10 {
                        (0.0, 0.0)
                    } else {
                        let v = (pdf_lower - pdf_upper) / denom;
                        let w = v * v + 
                            ((epsilon - t) * pdf_upper + (epsilon + t) * pdf_lower) / denom;
                        (v, w)
                    }
                }
            };
            
            // Update means using the TrueSkill update formulas
            let c_squared = diff_var;
            let player1_update = (s1_var / c_squared) * v * diff_std;
            let player2_update = -(s2_var / c_squared) * v * diff_std;
            
            let new_s1_mean = s1_mean + player1_update;
            let new_s2_mean = s2_mean + player2_update;
            
            // Update variances (reduce uncertainty)
            let variance_update_factor = 1.0 - (w / c_squared) * s1_var * s2_var / c_squared;
            let variance_update_factor = variance_update_factor.max(0.1).min(1.0); // Clamp between 0.1 and 1.0
            
            let new_s1_var = s1_var * variance_update_factor;
            let new_s2_var = s2_var * variance_update_factor;
            
            let updated_rating1 = TrueSkillRating::new(new_s1_mean, new_s1_var)?;
            let updated_rating2 = TrueSkillRating::new(new_s2_mean, new_s2_var)?;
            
            Ok((updated_rating1, updated_rating2))
        }
    }
    
    #[derive(Debug, Clone)]
    pub enum TwoPlayerOutcome {
        Player1Wins,
        Player2Wins,
        Draw,
    }
}

use factor_graph::*;

impl RatingSystem for TrueSkill {
    type PlayerRating = TrueSkillRating;
    type TeamRating = TrueSkillTeam;
    type Outcome = GameOutcome;
    
    fn create_rating(&self) -> Self::PlayerRating {
        // Default rating with initial parameters
        TrueSkillRating {
            mean: self.mu_0,
            variance: self.sigma_0_squared,
        }
    }
    
    fn create_rating_with_values(&self, mean: f64, variance: f64) -> Self::PlayerRating {
        TrueSkillRating { mean, variance }
    }
    
    fn rate(
        &self,
        rating_groups: &[Self::TeamRating],
        outcome: &Self::Outcome,
    ) -> Result<Vec<Self::TeamRating>> {
        if rating_groups.len() < 2 {
            return Err(Error::InvalidInput(
                "At least two teams required for rating update".to_string(),
            ));
        }
        
        let ranks = outcome.ranks();
        
        if ranks.len() != rating_groups.len() {
            return Err(Error::InvalidInput(
                "Number of ranks must match number of teams".to_string(),
            ));
        }
        
        // For now, only handle two-player games
        if rating_groups.len() != 2 {
            return Err(Error::InvalidInput(
                "Only two-player games are currently supported".to_string(),
            ));
        }
        
        // Only handle single-player teams for now
        if rating_groups[0].player_ratings().len() != 1 || rating_groups[1].player_ratings().len() != 1 {
            return Err(Error::InvalidInput(
                "Only single-player teams are currently supported".to_string(),
            ));
        }
        
        let player1_rating = &rating_groups[0].player_ratings()[0];
        let player2_rating = &rating_groups[1].player_ratings()[0];
        
        // Determine outcome
        let two_player_outcome = if ranks[0] < ranks[1] {
            TwoPlayerOutcome::Player1Wins
        } else if ranks[0] > ranks[1] {
            TwoPlayerOutcome::Player2Wins
        } else {
            TwoPlayerOutcome::Draw
        };
        
        // Use simplified updater
        let updater = SimplifiedTrueSkillUpdater::new(
            self.beta_squared,
            self.gamma_squared,
            self.draw_margin,
        );
        
        let (updated_rating1, updated_rating2) = updater.update_ratings(
            player1_rating,
            player2_rating,
            two_player_outcome,
        )?;
        
        let updated_team1 = TrueSkillTeam::from_player_ratings(vec![updated_rating1]);
        let updated_team2 = TrueSkillTeam::from_player_ratings(vec![updated_rating2]);
        
        Ok(vec![updated_team1, updated_team2])
    }
    
    fn calculate_match_quality(&self, _rating_groups: &[Self::TeamRating]) -> Result<f64> {
        // This will be implemented in Phase 4
        Err(Error::Other("TrueSkill match quality calculation not yet implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trueskill_creation() {
        let ts = TrueSkill::new();
        let rating = ts.create_rating();
        
        assert_eq!(rating.mean(), 25.0);
        assert!((rating.variance() - (25.0_f64/3.0).powi(2)).abs() < 1e-10);
    }

    #[test]
    fn test_trueskill_basic_update() {
        let ts = TrueSkill::new();
        
        // Create two players with default ratings
        let player1 = ts.create_rating();
        let player2 = ts.create_rating();
        
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
        assert!(winner_rating.mean() > 25.0, "Winner's rating should increase from 25.0 to {}", winner_rating.mean());
        
        // Loser should have lower mean than initial
        assert!(loser_rating.mean() < 25.0, "Loser's rating should decrease from 25.0 to {}", loser_rating.mean());
        
        // Both should have lower variance (more certain)
        assert!(winner_rating.variance() < (25.0_f64/3.0).powi(2), "Winner's variance should decrease");
        assert!(loser_rating.variance() < (25.0_f64/3.0).powi(2), "Loser's variance should decrease");
        
        // Check that ratings are reasonable
        assert!(winner_rating.mean() >= 20.0 && winner_rating.mean() <= 30.0);
        assert!(loser_rating.mean() >= 20.0 && loser_rating.mean() <= 30.0);
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
        
        // Verify that we have two teams back
        assert_eq!(updated_teams.len(), 2);
        
        let player1_rating = &updated_teams[0].player_ratings()[0];
        let player2_rating = &updated_teams[1].player_ratings()[0];
        
        println!("Player 1 after draw: μ={:.3}, σ²={:.3}", player1_rating.mean(), player1_rating.variance());
        println!("Player 2 after draw: μ={:.3}, σ²={:.3}", player2_rating.mean(), player2_rating.variance());
        
        // In a draw between equal players, means should stay approximately the same
        assert!((player1_rating.mean() - 25.0).abs() < 1.0, "Player 1 mean should be close to 25");
        assert!((player2_rating.mean() - 25.0).abs() < 1.0, "Player 2 mean should be close to 25");
        
        // Variances should decrease (more certainty)
        assert!(player1_rating.variance() < (25.0_f64/3.0).powi(2), "Player 1 variance should decrease");
        assert!(player2_rating.variance() < (25.0_f64/3.0).powi(2), "Player 2 variance should decrease");
        
        // Check that ratings are reasonable
        assert!(player1_rating.variance() > 0.0);
        assert!(player2_rating.variance() > 0.0);
    }

    #[test]
    fn test_trueskill_different_skill_levels() {
        let ts = TrueSkill::new();
        
        // Create players with different skill levels
        let strong_player = ts.create_rating_with_values(30.0, 10.0); // Strong player
        let weak_player = ts.create_rating_with_values(20.0, 10.0);   // Weak player
        
        // Create teams
        let team1 = TrueSkillTeam::from_player_ratings(vec![strong_player]);
        let team2 = TrueSkillTeam::from_player_ratings(vec![weak_player]);
        
        // Strong player wins (expected outcome)
        let outcome = GameOutcome::win(0, 2);
        let updated_teams = ts.rate(&[team1, team2], &outcome).expect("Rating update should succeed");
        
        let strong_updated = &updated_teams[0].player_ratings()[0];
        let weak_updated = &updated_teams[1].player_ratings()[0];
        
        println!("Strong player: {:.3} -> {:.3}", 30.0, strong_updated.mean());
        println!("Weak player: {:.3} -> {:.3}", 20.0, weak_updated.mean());
        
        // Changes should be smaller when the expected outcome happens
        assert!((strong_updated.mean() - 30.0).abs() < 2.0, "Strong player rating should change less when winning as expected");
        assert!((weak_updated.mean() - 20.0).abs() < 2.0, "Weak player rating should change less when losing as expected");
        
        // But strong player should still increase and weak should decrease
        assert!(strong_updated.mean() > 30.0, "Strong player should still gain rating");
        assert!(weak_updated.mean() < 20.0, "Weak player should still lose rating");
    }

    #[test]
    fn test_trueskill_upset() {
        let ts = TrueSkill::new();
        
        // Create players with different skill levels
        let strong_player = ts.create_rating_with_values(30.0, 10.0); // Strong player
        let weak_player = ts.create_rating_with_values(20.0, 10.0);   // Weak player
        
        // Create teams
        let team1 = TrueSkillTeam::from_player_ratings(vec![strong_player]);
        let team2 = TrueSkillTeam::from_player_ratings(vec![weak_player]);
        
        // Weak player wins (upset!)
        let outcome = GameOutcome::win(1, 2);  // Team 2 (weak player) wins
        let updated_teams = ts.rate(&[team1, team2], &outcome).expect("Rating update should succeed");
        
        let strong_updated = &updated_teams[0].player_ratings()[0];
        let weak_updated = &updated_teams[1].player_ratings()[0];
        
        println!("Strong player after upset: {:.3} -> {:.3}", 30.0, strong_updated.mean());
        println!("Weak player after upset: {:.3} -> {:.3}", 20.0, weak_updated.mean());
        
        // Changes should be larger when an unexpected outcome happens
        assert!((strong_updated.mean() - 30.0).abs() > 1.0, "Strong player should lose significant rating when upset");
        assert!((weak_updated.mean() - 20.0).abs() > 1.0, "Weak player should gain significant rating when causing upset");
        
        // Strong player should decrease and weak should increase
        assert!(strong_updated.mean() < 30.0, "Strong player should lose rating when upset");
        assert!(weak_updated.mean() > 20.0, "Weak player should gain rating when causing upset");
    }
}