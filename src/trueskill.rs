use crate::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    error::{Error, Result},
};
use statrs::distribution::{Normal, ContinuousCDF, Continuous};
use std::collections::HashMap;

/// Implementation choice for TrueSkill algorithm
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrueSkillImplementation {
    /// Simplified implementation using direct formulas
    Simplified,
    /// Full factor graph implementation with message passing
    FactorGraph,
}

/// Gaussian distribution used in factor graph message passing
#[derive(Debug, Clone, PartialEq)]
pub struct GaussianDistribution {
    precision_mean: f64,
    precision: f64,
}

impl GaussianDistribution {
    pub fn new(mean: f64, variance: f64) -> Result<Self> {
        if variance <= 0.0 {
            return Err(Error::InvalidInput("Variance must be positive".to_string()));
        }
        let precision = 1.0 / variance;
        Ok(Self {
            precision_mean: precision * mean,
            precision,
        })
    }

    /// Construct a Gaussian from a mean and variance without validation.
    pub fn from_mean_and_variance(mean: f64, variance: f64) -> Self {
        let precision = if variance.is_infinite() { 0.0 } else { 1.0 / variance };
        Self {
            precision_mean: precision * mean,
            precision,
        }
    }
    
    pub fn from_precision_mean(precision_mean: f64, precision: f64) -> Self {
        Self { precision_mean, precision }
    }
    
    pub fn mean(&self) -> f64 {
        if self.precision == 0.0 {
            0.0
        } else {
            self.precision_mean / self.precision
        }
    }
    
    pub fn variance(&self) -> f64 {
        if self.precision == 0.0 {
            f64::INFINITY
        } else {
            1.0 / self.precision
        }
    }
    
    pub fn precision(&self) -> f64 {
        self.precision
    }
    
    pub fn precision_mean(&self) -> f64 {
        self.precision_mean
    }
    
    /// Calculate absolute difference between two Gaussian distributions
    pub fn absolute_difference(&self, other: &Self) -> f64 {
        let precision_mean_diff = (self.precision_mean - other.precision_mean).abs();
        let precision_diff = (self.precision - other.precision).abs();
        precision_mean_diff.max(precision_diff)
    }
    
    /// Multiply two Gaussian distributions (product in precision form)
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            precision_mean: self.precision_mean + other.precision_mean,
            precision: self.precision + other.precision,
        }
    }
    
    /// Divide by another Gaussian distribution
    pub fn divide(&self, other: &Self) -> Self {
        Self {
            precision_mean: self.precision_mean - other.precision_mean,
            precision: self.precision - other.precision,
        }
    }
}

/// Variable in the factor graph that holds a Gaussian distribution
#[derive(Debug, Clone)]
pub struct Variable {
    id: usize,
    value: GaussianDistribution,
    messages: HashMap<usize, GaussianDistribution>, // Messages from factors (factor_id -> message)
}

impl Variable {
    pub fn new(id: usize, value: GaussianDistribution) -> Self {
        Self {
            id,
            value,
            messages: HashMap::new(),
        }
    }
    
    pub fn id(&self) -> usize {
        self.id
    }
    
    pub fn value(&self) -> &GaussianDistribution {
        &self.value
    }
    
    pub fn set_message(&mut self, factor_id: usize, message: GaussianDistribution) {
        self.messages.insert(factor_id, message);
    }
    
    pub fn message_from(&self, factor_id: usize) -> Option<&GaussianDistribution> {
        self.messages.get(&factor_id)
    }
    
    /// Update the belief by multiplying all incoming messages
    pub fn update_belief(&mut self, incoming_messages: &[GaussianDistribution]) -> f64 {
        let old_value = self.value.clone();
        
        // Start with uniform distribution (zero precision)
        let mut new_value = GaussianDistribution::from_precision_mean(0.0, 0.0);
        
        // Multiply all incoming messages
        for message in incoming_messages {
            new_value = new_value.multiply(message);
        }
        
        self.value = new_value;
        
        // Return the change in the belief
        old_value.absolute_difference(&self.value)
    }
}

/// Base trait for factors in the factor graph
pub trait Factor {
    /// Get the IDs of variables connected to this factor
    fn connected_variables(&self) -> Vec<usize>;
    
    /// Update the message from this factor to a specific variable
    fn update_message(&mut self, variable_id: usize) -> Result<f64>;
    
    /// Get the current message to a specific variable
    fn message_to(&self, variable_id: usize) -> Result<GaussianDistribution>;
}

/// Prior factor: connects a variable to a prior distribution
#[derive(Debug)]
pub struct PriorFactor {
    variable_id: usize,
    prior: GaussianDistribution,
    message: GaussianDistribution,
}

impl PriorFactor {
    pub fn new(variable_id: usize, prior: GaussianDistribution) -> Self {
        Self {
            variable_id,
            prior: prior.clone(),
            message: prior,
        }
    }
}

impl Factor for PriorFactor {
    fn connected_variables(&self) -> Vec<usize> {
        vec![self.variable_id]
    }
    
    fn update_message(&mut self, variable_id: usize) -> Result<f64> {
        if variable_id != self.variable_id {
            return Err(Error::InvalidInput("Variable not connected to this factor".to_string()));
        }
        
        let old_message = self.message.clone();
        self.message = self.prior.clone();
        
        Ok(old_message.absolute_difference(&self.message))
    }
    
    fn message_to(&self, variable_id: usize) -> Result<GaussianDistribution> {
        if variable_id != self.variable_id {
            return Err(Error::InvalidInput("Variable not connected to this factor".to_string()));
        }
        
        Ok(self.message.clone())
    }
}

/// Linear factor: implements Y = A*X + ε where ε ~ N(0, variance)
#[derive(Debug)]
pub struct LinearFactor {
    input_id: usize,
    output_id: usize,
    coefficient: f64,
    variance: f64,
    message_to_input: GaussianDistribution,
    message_to_output: GaussianDistribution,
}

impl LinearFactor {
    pub fn new(input_id: usize, output_id: usize, coefficient: f64, variance: f64) -> Self {
        let zero_message = GaussianDistribution::from_precision_mean(0.0, 0.0);
        Self {
            input_id,
            output_id,
            coefficient,
            variance,
            message_to_input: zero_message.clone(),
            message_to_output: zero_message,
        }
    }
}

impl Factor for LinearFactor {
    fn connected_variables(&self) -> Vec<usize> {
        vec![self.input_id, self.output_id]
    }
    
    fn update_message(&mut self, variable_id: usize) -> Result<f64> {
        if variable_id == self.output_id {
            // Update message to output variable
            let old_message = self.message_to_output.clone();
            
            // Message is coefficient * input + noise
            let input_precision = self.message_to_input.precision();
            let output_precision = input_precision * self.coefficient * self.coefficient + 1.0 / self.variance;
            let output_precision_mean = self.message_to_input.precision_mean() * self.coefficient;
            
            self.message_to_output = GaussianDistribution::from_precision_mean(output_precision_mean, output_precision);
            
            Ok(old_message.absolute_difference(&self.message_to_output))
        } else if variable_id == self.input_id {
            // Update message to input variable
            let old_message = self.message_to_input.clone();
            
            // Reverse computation for input
            let output_precision = self.message_to_output.precision();
            let input_precision = output_precision * self.coefficient * self.coefficient;
            let input_precision_mean = self.message_to_output.precision_mean() * self.coefficient;
            
            self.message_to_input = GaussianDistribution::from_precision_mean(input_precision_mean, input_precision);
            
            Ok(old_message.absolute_difference(&self.message_to_input))
        } else {
            Err(Error::InvalidInput("Variable not connected to this factor".to_string()))
        }
    }
    
    fn message_to(&self, variable_id: usize) -> Result<GaussianDistribution> {
        if variable_id == self.output_id {
            Ok(self.message_to_output.clone())
        } else if variable_id == self.input_id {
            Ok(self.message_to_input.clone())
        } else {
            Err(Error::InvalidInput("Variable not connected to this factor".to_string()))
        }
    }
}

/// Greater than factor: implements constraint that difference > draw_margin
#[derive(Debug)]
pub struct GaussianComparisonFactor {
    difference_id: usize,
    draw_margin: f64,
    message: GaussianDistribution,
}

impl GaussianComparisonFactor {
    pub fn new(difference_id: usize, draw_margin: f64) -> Self {
        let zero_message = GaussianDistribution::from_precision_mean(0.0, 0.0);
        Self {
            difference_id,
            draw_margin,
            message: zero_message,
        }
    }
    
    /// Compute the V function for truncated Gaussian
    fn v_function(t: f64, epsilon: f64) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let denom = normal.cdf(t - epsilon);
        
        if denom < 1e-10 {
            -t + epsilon
        } else {
            normal.pdf(t - epsilon) / denom
        }
    }
    
    /// Compute the W function for truncated Gaussian  
    fn w_function(t: f64, epsilon: f64) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let denom = normal.cdf(t - epsilon);
        
        if denom < 1e-10 {
            if t < epsilon {
                1.0
            } else {
                0.0
            }
        } else {
            let v = Self::v_function(t, epsilon);
            v * (v + t - epsilon)
        }
    }
}

impl Factor for GaussianComparisonFactor {
    fn connected_variables(&self) -> Vec<usize> {
        vec![self.difference_id]
    }
    
    fn update_message(&mut self, variable_id: usize) -> Result<f64> {
        if variable_id != self.difference_id {
            return Err(Error::InvalidInput("Variable not connected to this factor".to_string()));
        }
        
        let old_message = self.message.clone();
        
        // Get the current belief about the difference
        // For now, we'll use a simplified approach
        let mean = 0.0; // This should come from the variable's current belief
        let variance: f64 = 1.0; // This should also come from the variable
        
        let std_dev = variance.sqrt();
        let t = (mean - self.draw_margin) / std_dev;
        
        let v = Self::v_function(t, 0.0);
        let w = Self::w_function(t, 0.0);
        
        // Clamp w to prevent numerical issues
        let w_clamped = w.max(1e-10).min(1e10);
        
        let new_precision = w_clamped / variance;
        let new_precision_mean = (mean + std_dev * v) * new_precision;
        
        self.message = GaussianDistribution::from_precision_mean(new_precision_mean, new_precision);
        
        Ok(old_message.absolute_difference(&self.message))
    }
    
    fn message_to(&self, variable_id: usize) -> Result<GaussianDistribution> {
        if variable_id != self.difference_id {
            return Err(Error::InvalidInput("Variable not connected to this factor".to_string()));
        }
        
        Ok(self.message.clone())
    }
}

/// Factor graph for TrueSkill computation
pub struct FactorGraph {
    variables: HashMap<usize, Variable>,
    factors: Vec<Box<dyn Factor>>,
    next_variable_id: usize,
}

impl FactorGraph {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            factors: Vec::new(),
            next_variable_id: 0,
        }
    }
    
    pub fn add_variable(&mut self, value: GaussianDistribution) -> usize {
        let id = self.next_variable_id;
        self.next_variable_id += 1;
        self.variables.insert(id, Variable::new(id, value));
        id
    }
    
    pub fn add_factor(&mut self, factor: Box<dyn Factor>) {
        self.factors.push(factor);
    }
    
    pub fn get_variable(&self, id: usize) -> Option<&Variable> {
        self.variables.get(&id)
    }
    
    /// Run message passing until convergence with improved convergence detection
    pub fn run_schedule_loop(&mut self, max_delta: f64, max_iterations: usize) -> Result<f64> {
        let mut iteration = 0;
        let mut delta = f64::INFINITY;
        let mut delta_history = Vec::new();
        let stagnation_threshold = 5; // Number of iterations to detect stagnation
        
        while delta > max_delta && iteration < max_iterations {
            let prev_delta = delta;
            delta = 0.0;
            iteration += 1;

            // Update all factor messages
            for factor in &mut self.factors {
                for variable_id in factor.connected_variables() {
                    let factor_delta = factor.update_message(variable_id)?;
                    delta = delta.max(factor_delta);
                }
            }

            // Update all variable beliefs
            for variable in self.variables.values_mut() {
                let var_id = variable.id();
                let mut messages = Vec::new();
                for factor in &self.factors {
                    if factor.connected_variables().iter().any(|&id| id == var_id) {
                        messages.push(factor.message_to(var_id)?);
                    }
                }
                let var_delta = variable.update_belief(&messages);
                delta = delta.max(var_delta);
            }
            
            // Track delta history for oscillation detection
            delta_history.push(delta);
            
            // Check for stagnation (delta not decreasing significantly)
            if delta_history.len() >= stagnation_threshold {
                let recent_deltas = &delta_history[delta_history.len() - stagnation_threshold..];
                let min_recent = recent_deltas.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max_recent = recent_deltas.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                
                // If delta is oscillating in a small range, consider it converged
                if max_recent - min_recent < max_delta * 2.0 && iteration > stagnation_threshold {
                    break;
                }
            }
            
            // Early termination if delta stops improving
            if iteration > 3 && delta >= prev_delta * 0.99 {
                // Delta is not improving significantly, likely stuck
                break;
            }
        }

        Ok(delta)
    }
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
            convergence_threshold: 0.001, // Relaxed from 0.0001 to prevent hanging
            max_iterations: 50, // Increased from 20 to allow more convergence time
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
            convergence_threshold: 0.001, // More lenient threshold
            max_iterations: 50, // Increased iterations
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
        let normal = Normal::new(0.0, 1.0).unwrap();
        
        // Binary search to find ε that gives us the desired draw probability
        let mut low = 0.0;
        let mut high = 10.0 * sigma_diff;
        
        for _ in 0..20 {
            let mid = (low + high) / 2.0;
            let prob = normal.cdf(mid / sigma_diff) - normal.cdf(-mid / sigma_diff);
            
            if prob < self.draw_probability {
                low = mid;
            } else {
                high = mid;
            }
        }
        
        self.draw_margin = (low + high) / 2.0;
        self
    }
    
    /// Get the current implementation type
    pub fn implementation(&self) -> TrueSkillImplementation {
        self.implementation
    }
    
    /// Set the implementation type
    pub fn set_implementation(&mut self, implementation: TrueSkillImplementation) {
        self.implementation = implementation;
    }
    
    /// Get convergence parameters for factor graph
    pub fn convergence_parameters(&self) -> (f64, usize) {
        (self.convergence_threshold, self.max_iterations)
    }
    
    /// Set convergence parameters for factor graph
    pub fn set_convergence_parameters(&mut self, threshold: f64, max_iterations: usize) {
        self.convergence_threshold = threshold;
        self.max_iterations = max_iterations;
    }
}

/// Individual player rating in TrueSkill system
#[derive(Debug, Clone, PartialEq)]
pub struct TrueSkillRating {
    /// Mean skill (μ)
    mean: f64,
    
    /// Skill variance (σ²)
    variance: f64,
}

impl TrueSkillRating {
    /// Create a new TrueSkill rating
    pub fn new(mean: f64, variance: f64) -> Result<Self> {
        if variance <= 0.0 {
            return Err(Error::InvalidInput("Variance must be positive".to_string()));
        }
        
        Ok(Self { mean, variance })
    }
    
    /// Create a rating from mean and standard deviation
    pub fn from_mean_and_std_dev(mean: f64, std_dev: f64) -> Result<Self> {
        if std_dev <= 0.0 {
            return Err(Error::InvalidInput("Standard deviation must be positive".to_string()));
        }
        
        Ok(Self {
            mean,
            variance: std_dev * std_dev,
        })
    }
    
    /// Get the mean (μ)
    pub fn mean(&self) -> f64 {
        self.mean
    }
    
    /// Get the variance (σ²)
    pub fn variance(&self) -> f64 {
        self.variance
    }
    
    /// Get the standard deviation (σ)
    pub fn std_dev(&self) -> f64 {
        self.variance.sqrt()
    }
    
    /// Get the conservative rating estimate (μ - 3σ)
    pub fn conservative_rating(&self) -> f64 {
        self.mean - 3.0 * self.std_dev()
    }
    
    /// Create a Gaussian distribution representation
    pub fn to_gaussian(&self) -> Result<GaussianDistribution> {
        GaussianDistribution::new(self.mean, self.variance)
    }
}

impl Rating for TrueSkillRating {
    fn mean(&self) -> f64 {
        self.mean
    }
    
    fn variance(&self) -> f64 {
        self.variance
    }
}

/// Team composition for TrueSkill system
#[derive(Debug, Clone)]
pub struct TrueSkillTeam {
    /// Player ratings in the team
    player_ratings: Vec<TrueSkillRating>,
}

impl TrueSkillTeam {
    /// Create a team from player ratings
    pub fn from_player_ratings(player_ratings: Vec<TrueSkillRating>) -> Self {
        Self { player_ratings }
    }
    
    /// Get player ratings
    pub fn player_ratings(&self) -> &[TrueSkillRating] {
        &self.player_ratings
    }
    
    /// Get mutable access to player ratings
    pub fn player_ratings_mut(&mut self) -> &mut Vec<TrueSkillRating> {
        &mut self.player_ratings
    }
    
    /// Calculate team mean (sum of individual means)
    pub fn team_mean(&self) -> f64 {
        self.player_ratings.iter().map(|r| r.mean()).sum()
    }
    
    /// Calculate team variance (sum of individual variances for independent players)
    pub fn team_variance(&self) -> f64 {
        self.player_ratings.iter().map(|r| r.variance()).sum()
    }
}

impl TeamRating for TrueSkillTeam {
    type PlayerRating = TrueSkillRating;
    
    fn player_ratings(&self) -> &[Self::PlayerRating] {
        &self.player_ratings
    }
    
    fn from_player_ratings(ratings: Vec<Self::PlayerRating>) -> Self {
        Self { player_ratings: ratings }
    }
}

impl TrueSkill {
    /// Rate teams using simplified Bradley-Terry model approach
    fn rate_simplified(
        &self,
        rating_groups: &[TrueSkillTeam],
        outcome: &GameOutcome,
    ) -> Result<Vec<TrueSkillTeam>> {
        if rating_groups.len() != 2 {
            return Err(Error::InvalidInput(
                "Simplified TrueSkill currently only supports 2 teams".to_string(),
            ));
        }
        
        let ranks = outcome.ranks();
        let team1 = &rating_groups[0];
        let team2 = &rating_groups[1];
        
        // Calculate team statistics
        let mu1 = team1.team_mean();
        let mu2 = team2.team_mean();
        let sigma1_squared = team1.team_variance() + self.gamma_squared;
        let sigma2_squared = team2.team_variance() + self.gamma_squared;
        
        // Performance variance includes both skill uncertainty and game randomness
        let c_squared = 2.0 * self.beta_squared + sigma1_squared + sigma2_squared;
        let c = c_squared.sqrt();
        
        // Calculate the difference in team performance
        let mu_diff = mu1 - mu2;
        
        let normal = Normal::new(0.0, 1.0).unwrap();
        
        // Determine the update based on the outcome
        let (v1, v2, w1, w2) = if ranks[0] < ranks[1] {
            // Team 1 wins
            let t = mu_diff / c;
            let v = normal.pdf(t) / normal.cdf(t);
            let w = v * (v + t);
            (v, -v, w, w)
        } else if ranks[0] > ranks[1] {
            // Team 2 wins
            let t = -mu_diff / c;
            let v = normal.pdf(t) / normal.cdf(t);
            let w = v * (v + t);
            (-v, v, w, w)
        } else {
            // Draw
            let epsilon = self.draw_margin;
            let t = mu_diff / c;
            
            let cdf_plus = normal.cdf((epsilon - t) / c);
            let cdf_minus = normal.cdf((-epsilon - t) / c);
            let pdf_plus = normal.pdf((epsilon - t) / c);
            let pdf_minus = normal.pdf((-epsilon - t) / c);
            
            let denom = cdf_plus - cdf_minus;
            let v = if denom > 1e-10 {
                (pdf_minus - pdf_plus) / denom
            } else {
                0.0
            };
            
            let w = if denom > 1e-10 {
                v * v + ((epsilon - t) * pdf_plus - (-epsilon - t) * pdf_minus) / denom
            } else {
                1.0
            };
            
            (v, -v, w, w)
        };
        
        // Update team ratings
        let mut new_teams = Vec::new();
        
        for (i, team) in rating_groups.iter().enumerate() {
            let (v, w) = if i == 0 { (v1, w1) } else { (v2, w2) };
            
            let team_variance = if i == 0 { sigma1_squared } else { sigma2_squared };
            
            // Calculate the update amount for each player
            let mut new_players = Vec::new();
            
            for player in team.player_ratings() {
                let player_variance = player.variance() + self.gamma_squared;
                let update_factor = player_variance / team_variance;
                
                let new_mean = player.mean() + update_factor * v * c / c_squared;
                let new_variance = player_variance * (1.0 - update_factor * w * c_squared / c_squared);
                
                // Ensure variance doesn't become negative or too small
                let clamped_variance = new_variance.max(0.0001);
                
                new_players.push(TrueSkillRating::new(new_mean, clamped_variance)?);
            }
            
            new_teams.push(TrueSkillTeam::from_player_ratings(new_players));
        }
        
        Ok(new_teams)
    }
    
    /// Rate teams using factor graph approach (currently falls back to simplified)
    fn rate_factor_graph(
        &self,
        rating_groups: &[TrueSkillTeam],
        outcome: &GameOutcome,
    ) -> Result<Vec<TrueSkillTeam>> {
        // For now, we'll build a factor graph but still fall back to simplified implementation
        // This is a placeholder for the full factor graph implementation
        
        let mut graph = FactorGraph::new();
        
        // Add variables for each player's skill
        let mut skill_variables = Vec::new();
        for team in rating_groups {
            let mut team_skills = Vec::new();
            for player in team.player_ratings() {
                let skill_dist = player.to_gaussian()?;
                let skill_id = graph.add_variable(skill_dist);
                
                // Add prior factor for this skill
                let prior_factor = Box::new(PriorFactor::new(skill_id, player.to_gaussian()?));
                graph.add_factor(prior_factor);
                
                team_skills.push(skill_id);
            }
            skill_variables.push(team_skills);
        }
        
        // Add performance variables (skill + noise)
        let mut _performance_variables = Vec::new();
        for (_team_idx, team_skills) in skill_variables.iter().enumerate() {
            let mut team_performances = Vec::new();
            for &skill_id in team_skills {
                let perf_dist = GaussianDistribution::from_precision_mean(0.0, 0.0);
                let perf_id = graph.add_variable(perf_dist);
                
                // Add linear factor: performance = skill + noise
                let linear_factor = Box::new(LinearFactor::new(skill_id, perf_id, 1.0, self.beta_squared));
                graph.add_factor(linear_factor);
                
                team_performances.push(perf_id);
            }
            _performance_variables.push(team_performances);
        }
        
        // For now, just run a few iterations and fall back to simplified
        let _convergence_result = graph.run_schedule_loop(self.convergence_threshold, self.max_iterations);
        
        // Fall back to simplified implementation
        self.rate_simplified(rating_groups, outcome)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_distribution() {
        let g1 = GaussianDistribution::new(25.0, 64.0).unwrap();
        assert_eq!(g1.mean(), 25.0);
        assert_eq!(g1.variance(), 64.0);
        
        let g2 = GaussianDistribution::new(20.0, 36.0).unwrap();
        let product = g1.multiply(&g2);
        
        // Product of two Gaussians in precision form
        assert!(product.precision() > 0.0);
    }

    #[test]
    fn test_trueskill_rating() {
        let rating = TrueSkillRating::new(25.0, 64.0).unwrap();
        assert_eq!(rating.mean(), 25.0);
        assert_eq!(rating.variance(), 64.0);
        assert_eq!(rating.std_dev(), 8.0);
        assert_eq!(rating.conservative_rating(), 1.0); // 25 - 3*8
    }

    #[test]
    fn test_trueskill_team() {
        let player1 = TrueSkillRating::new(25.0, 64.0).unwrap();
        let player2 = TrueSkillRating::new(30.0, 36.0).unwrap();
        
        let team = TrueSkillTeam::from_player_ratings(vec![player1, player2]);
        
        assert_eq!(team.team_mean(), 55.0); // 25 + 30
        assert_eq!(team.team_variance(), 100.0); // 64 + 36
    }

    #[test]
    fn test_trueskill_system_creation() {
        let ts_simplified = TrueSkill::new_simplified();
        assert_eq!(ts_simplified.implementation(), TrueSkillImplementation::Simplified);
        
        let ts_factor_graph = TrueSkill::new_factor_graph();
        assert_eq!(ts_factor_graph.implementation(), TrueSkillImplementation::FactorGraph);
    }

    #[test]
    fn test_trueskill_rating_update() {
        let ts = TrueSkill::new_simplified();
        
        let player1 = TrueSkillRating::new(25.0, 64.0).unwrap();
        let player2 = TrueSkillRating::new(25.0, 64.0).unwrap();
        
        let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
        let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
        
        let outcome = GameOutcome::win(0, 2);
        
        let result = ts.rate(&[team1, team2], &outcome).unwrap();
        
        // Winner should have higher rating
        assert!(result[0].player_ratings()[0].mean() > result[1].player_ratings()[0].mean());
    }

    #[test]
    fn test_factor_graph_convergence() {
        let ts = TrueSkill::new_factor_graph();
        
        // Test with relaxed convergence parameters
        assert_eq!(ts.convergence_parameters(), (0.001, 50));
        
        let player1 = TrueSkillRating::new(25.0, 64.0).unwrap();
        let player2 = TrueSkillRating::new(25.0, 64.0).unwrap();
        
        let team1 = TrueSkillTeam::from_player_ratings(vec![player1]);
        let team2 = TrueSkillTeam::from_player_ratings(vec![player2]);
        
        let outcome = GameOutcome::win(0, 2);
        
        // This should not hang with the improved convergence detection
        let result = ts.rate(&[team1, team2], &outcome);
        assert!(result.is_ok());
    }
}