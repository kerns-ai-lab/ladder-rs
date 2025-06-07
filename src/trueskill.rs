use crate::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    error::{Error, Result},
};
use statrs::distribution::{Normal, ContinuousCDF};

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
    }
    
    /// Variable node types in the TrueSkill factor graph
    #[derive(Debug, Clone, PartialEq)]
    pub enum VariableNode {
        /// Player skill (s_i)
        Skill(GaussianMessage),
        
        /// Player performance (p_i)
        Performance(GaussianMessage),
        
        /// Team performance (t_j)
        TeamPerformance(GaussianMessage),
        
        /// Performance difference (d_k)
        PerformanceDiff(GaussianMessage),
    }
    
    /// Factor node types in the TrueSkill factor graph
    #[derive(Debug)]
    pub enum FactorNode {
        /// Prior factor connecting to skill variable
        Prior,
        
        /// Likelihood factor connecting skill to performance
        Likelihood {
            /// Performance variance (β²)
            beta_squared: f64,
        },
        
        /// Sum factor for team performance
        Sum {
            /// Coefficients for each player's contribution
            coefficients: Vec<f64>,
        },
        
        /// Difference factor for performance difference
        Difference,
        
        /// Comparison factor for win/loss/draw outcomes
        Comparison {
            /// Epsilon for draw margin
            epsilon: f64,
            /// Whether this is a draw
            is_draw: bool,
        },
    }
}

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
        _rating_groups: &[Self::TeamRating],
        _outcome: &Self::Outcome,
    ) -> Result<Vec<Self::TeamRating>> {
        // This will be implemented in Phase 3
        Err(Error::Other("TrueSkill rating update not yet implemented".to_string()))
    }
    
    fn calculate_match_quality(&self, _rating_groups: &[Self::TeamRating]) -> Result<f64> {
        // This will be implemented in Phase 4
        Err(Error::Other("TrueSkill match quality calculation not yet implemented".to_string()))
    }
}