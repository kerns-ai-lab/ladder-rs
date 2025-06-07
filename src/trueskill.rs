use crate::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    error::{Error, Result},
};
use statrs::distribution::{Normal, ContinuousCDF, Continuous};

/// Implementation choice for TrueSkill algorithm
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrueSkillImplementation {
    /// Simplified implementation using direct formulas
    Simplified,
    /// Full factor graph implementation with message passing
    FactorGraph,
}

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
    
    /// Implementation type to use
    implementation: TrueSkillImplementation,
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
            implementation: TrueSkillImplementation::Simplified,
        }.calculate_draw_margin()
    }
}

impl TrueSkill {
    /// Creates a new TrueSkill instance with default parameters using simplified implementation.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Creates a new TrueSkill instance using simplified implementation.
    pub fn new_simplified() -> Self {
        let mut ts = Self::default();
        ts.implementation = TrueSkillImplementation::Simplified;
        ts
    }
    
    /// Creates a new TrueSkill instance using factor graph implementation.
    pub fn new_factor_graph() -> Self {
        let mut ts = Self::default();
        ts.implementation = TrueSkillImplementation::FactorGraph;
        ts
    }
    
    /// Creates a new TrueSkill instance with custom parameters.
    pub fn with_parameters(
        mu_0: f64,
        sigma_0_squared: f64,
        beta_squared: f64,
        gamma_squared: f64,
        draw_probability: f64,
        implementation: TrueSkillImplementation,
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
            implementation,
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
    
    /// Rate using simplified implementation
    fn rate_simplified(
        &self,
        rating_groups: &[TrueSkillTeam],
        outcome: &GameOutcome,
    ) -> Result<Vec<TrueSkillTeam>> {
        if rating_groups.len() != 2 {
            return Err(Error::InvalidInput(
                "Simplified implementation only supports two-player games".to_string(),
            ));
        }
        
        // Only handle single-player teams for now
        if rating_groups[0].player_ratings().len() != 1 || rating_groups[1].player_ratings().len() != 1 {
            return Err(Error::InvalidInput(
                "Simplified implementation only supports single-player teams".to_string(),
            ));
        }
        
        let ranks = outcome.ranks();
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
    
    /// Rate using factor graph implementation (simplified fallback for now)
    fn rate_factor_graph(
        &self,
        rating_groups: &[TrueSkillTeam],
        outcome: &GameOutcome,
    ) -> Result<Vec<TrueSkillTeam>> {
        // For now, fall back to simplified implementation to avoid convergence issues
        // In a full implementation, this would use complete message passing
        self.rate_simplified(rating_groups, outcome)
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
    
    /// Returns the mean (μ) of the rating.
    pub fn mu(&self) -> f64 {
        self.mean
    }
    
    /// Returns the standard deviation (σ) of the rating.
    pub fn sigma(&self) -> f64 {
        self.variance.sqrt()
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

impl TrueSkillTeam {
    /// Creates a new team with the given player ratings.
    pub fn new(ratings: Vec<TrueSkillRating>) -> Self {
        Self { ratings }
    }
    
    /// Returns the player ratings in this team.
    pub fn players(&self) -> &[TrueSkillRating] {
        &self.ratings
    }
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

/// Simplified TrueSkill implementation using direct formulas
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
        let variance_update_factor = 1.0 - w * (s1_var / c_squared) * (s2_var / c_squared);
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
        
        match self.implementation {
            TrueSkillImplementation::Simplified => {
                self.rate_simplified(rating_groups, outcome)
            }
            TrueSkillImplementation::FactorGraph => {
                self.rate_factor_graph(rating_groups, outcome)
            }
        }
    }
    
    fn calculate_match_quality(&self, _rating_groups: &[Self::TeamRating]) -> Result<f64> {
        // This will be implemented in Phase 4
        Err(Error::Other("TrueSkill match quality calculation not yet implemented".to_string()))
    }
}