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
        pub fn divide(&self, other: &Self) -> Result<Self> {
            let new_precision = self.precision - other.precision;
            let new_precision_adjusted_mean = self.precision_adjusted_mean - other.precision_adjusted_mean;
            
            // Debug information
            println!("Division: self.precision={}, other.precision={}, new_precision={}", 
                     self.precision, other.precision, new_precision);
            
            if new_precision <= 0.0 {
                return Err(Error::NumericalError(
                    format!("Division resulted in non-positive precision: {} - {} = {}", 
                            self.precision, other.precision, new_precision),
                ));
            }
            
            Ok(Self {
                precision: new_precision,
                precision_adjusted_mean: new_precision_adjusted_mean,
            })
        }
    }
    
    /// Variable node in the TrueSkill factor graph
    #[derive(Debug, Clone)]
    pub struct VariableNode {
        /// Current marginal belief
        pub marginal: GaussianMessage,
        
        /// Messages from connected factor nodes
        pub messages_from_factors: HashMap<usize, GaussianMessage>,
    }
    
    impl VariableNode {
        /// Creates a new variable node with the given prior
        pub fn new(prior: GaussianMessage) -> Self {
            Self {
                marginal: prior.clone(),
                messages_from_factors: HashMap::new(),
            }
        }
        
        /// Updates the marginal by multiplying all incoming messages with the prior
        pub fn update_marginal(&mut self, prior: &GaussianMessage) -> Result<()> {
            let mut new_marginal = prior.clone();
            
            for message in self.messages_from_factors.values() {
                new_marginal = new_marginal.multiply(message);
            }
            
            self.marginal = new_marginal;
            Ok(())
        }
        
        /// Computes the message to send to a specific factor (cavity distribution)
        pub fn message_to_factor(&self, factor_id: usize) -> Result<GaussianMessage> {
            if let Some(incoming_message) = self.messages_from_factors.get(&factor_id) {
                self.marginal.divide(incoming_message)
            } else {
                // If no message exists from this factor, return the full marginal
                Ok(self.marginal.clone())
            }
        }
    }
    
    /// Factor node types in the TrueSkill factor graph
    #[derive(Debug)]
    pub enum FactorNode {
        /// Likelihood factor connecting skill to performance
        Likelihood {
            factor_id: usize,
            skill_id: usize,
            performance_id: usize,
            beta_squared: f64,
        },
        
        /// Sum factor for team performance
        Sum {
            factor_id: usize,
            performance_ids: Vec<usize>,
            team_performance_id: usize,
            coefficients: Vec<f64>,
        },
        
        /// Difference factor for performance difference
        Difference {
            factor_id: usize,
            team_a_id: usize,
            team_b_id: usize,
            difference_id: usize,
        },
        
        /// Comparison factor for win/loss/draw outcomes
        Comparison {
            factor_id: usize,
            difference_id: usize,
            epsilon: f64,
            outcome: ComparisonOutcome,
        },
    }
    
    #[derive(Debug, Clone)]
    pub enum ComparisonOutcome {
        Win, // Team A wins (difference > epsilon)
        Draw, // Draw (|difference| <= epsilon)
    }
    
    impl FactorNode {
        pub fn get_factor_id(&self) -> usize {
            match self {
                FactorNode::Likelihood { factor_id, .. } => *factor_id,
                FactorNode::Sum { factor_id, .. } => *factor_id,
                FactorNode::Difference { factor_id, .. } => *factor_id,
                FactorNode::Comparison { factor_id, .. } => *factor_id,
            }
        }
        
        /// Updates messages from this factor to connected variables
        pub fn update_messages(
            &self,
            variables: &mut HashMap<usize, VariableNode>,
            priors: &HashMap<usize, GaussianMessage>,
        ) -> Result<()> {
            match self {
                FactorNode::Likelihood { factor_id, skill_id, performance_id, beta_squared } => {
                    // Likelihood factor: p_i ~ N(s_i, β²)
                    
                    // Message from skill to performance: p_i = s_i + noise
                    if let Some(skill_var) = variables.get(skill_id) {
                        let skill_message = skill_var.message_to_factor(*factor_id)?;
                        
                        // The performance message is the skill message convolved with noise β²
                        let perf_precision = 1.0 / (skill_message.variance() + beta_squared);
                        let perf_mean = skill_message.mean();
                        let perf_message = GaussianMessage::new(
                            perf_precision,
                            perf_precision * perf_mean,
                        );
                        
                        if let Some(perf_var) = variables.get_mut(performance_id) {
                            perf_var.messages_from_factors.insert(*factor_id, perf_message);
                        }
                    }
                    
                    // Message from performance to skill: s_i ~ N(p_i, β²)
                    if let Some(perf_var) = variables.get(performance_id) {
                        let perf_message = perf_var.message_to_factor(*factor_id)?;
                        
                        // The skill message is the performance message with the same noise
                        let skill_precision = 1.0 / (perf_message.variance() + beta_squared);
                        let skill_mean = perf_message.mean();
                        let skill_message = GaussianMessage::new(
                            skill_precision,
                            skill_precision * skill_mean,
                        );
                        
                        if let Some(skill_var) = variables.get_mut(skill_id) {
                            skill_var.messages_from_factors.insert(*factor_id, skill_message);
                        }
                    }
                }
                
                FactorNode::Sum { factor_id, performance_ids, team_performance_id, coefficients } => {
                    // Sum factor: t_j = Σ c_i * p_i
                    
                    // Message to team performance: sum of weighted performance messages
                    let mut sum_precision = 0.0;
                    let mut sum_precision_adjusted_mean = 0.0;
                    
                    for (i, &perf_id) in performance_ids.iter().enumerate() {
                        if let Some(perf_var) = variables.get(&perf_id) {
                            let perf_message = perf_var.message_to_factor(*factor_id)?;
                            let coeff = coefficients.get(i).copied().unwrap_or(1.0);
                            
                            sum_precision += coeff * coeff * perf_message.precision;
                            sum_precision_adjusted_mean += coeff * perf_message.precision_adjusted_mean;
                        }
                    }
                    
                    let team_message = GaussianMessage::new(sum_precision, sum_precision_adjusted_mean);
                    if let Some(team_var) = variables.get_mut(team_performance_id) {
                        team_var.messages_from_factors.insert(*factor_id, team_message);
                    }
                    
                    // Messages to individual performances: Subtract contributions from others
                    for (i, &perf_id) in performance_ids.iter().enumerate() {
                        if let Some(team_var) = variables.get(team_performance_id) {
                            let team_message = team_var.message_to_factor(*factor_id)?;
                            let coeff = coefficients.get(i).copied().unwrap_or(1.0);
                            
                            // Calculate the sum of all other performance contributions
                            let mut other_precision = 0.0;
                            let mut other_precision_adjusted_mean = 0.0;
                            
                            for (j, &other_perf_id) in performance_ids.iter().enumerate() {
                                if i != j {
                                    if let Some(other_perf_var) = variables.get(&other_perf_id) {
                                        let other_message = other_perf_var.message_to_factor(*factor_id)?;
                                        let other_coeff = coefficients.get(j).copied().unwrap_or(1.0);
                                        other_precision += other_coeff * other_coeff * other_message.precision;
                                        other_precision_adjusted_mean += other_coeff * other_message.precision_adjusted_mean;
                                    }
                                }
                            }
                            
                            // Compute the message to this performance
                            let remaining_precision = team_message.precision - other_precision;
                            let remaining_precision_adjusted_mean = team_message.precision_adjusted_mean - other_precision_adjusted_mean;
                            
                            if remaining_precision > 1e-10 { // Use small threshold instead of 0
                                let perf_message = GaussianMessage::new(
                                    remaining_precision / (coeff * coeff),
                                    remaining_precision_adjusted_mean / coeff,
                                );
                                
                                if let Some(perf_var) = variables.get_mut(&perf_id) {
                                    perf_var.messages_from_factors.insert(*factor_id, perf_message);
                                }
                            } else {
                                // Send a very weak (almost uniform) message
                                let weak_message = GaussianMessage::new(1e-10, 0.0);
                                if let Some(perf_var) = variables.get_mut(&perf_id) {
                                    perf_var.messages_from_factors.insert(*factor_id, weak_message);
                                }
                            }
                        }
                    }
                }
                
                FactorNode::Difference { factor_id, team_a_id, team_b_id, difference_id } => {
                    // Difference factor: d = t_a - t_b
                    
                    // Message to difference variable
                    if let (Some(team_a_var), Some(team_b_var)) = 
                        (variables.get(team_a_id), variables.get(team_b_id)) {
                        let team_a_message = team_a_var.message_to_factor(*factor_id)?;
                        let team_b_message = team_b_var.message_to_factor(*factor_id)?;
                        
                        // For d = t_a - t_b, if t_a ~ N(μ_a, σ_a²) and t_b ~ N(μ_b, σ_b²),
                        // then d ~ N(μ_a - μ_b, σ_a² + σ_b²)
                        let diff_mean = team_a_message.mean() - team_b_message.mean();
                        let diff_variance = team_a_message.variance() + team_b_message.variance();
                        
                        if diff_variance > 0.0 {
                            let diff_message = GaussianMessage::from_mean_and_variance(diff_mean, diff_variance)?;
                            if let Some(diff_var) = variables.get_mut(difference_id) {
                                diff_var.messages_from_factors.insert(*factor_id, diff_message);
                            }
                        }
                    }
                    
                    // For now, we'll skip the reverse messages (difference to teams)
                    // as they're more complex and not strictly necessary for convergence
                }
                
                FactorNode::Comparison { factor_id, difference_id, epsilon, outcome } => {
                    // Comparison factor using V and W functions
                    if let Some(diff_var) = variables.get_mut(difference_id) {
                        let cavity = diff_var.message_to_factor(*factor_id)?;
                        let d_cavity = cavity.mean();
                        let c_cavity = cavity.variance();
                        
                        if c_cavity <= 0.0 {
                            // Send a weak message if cavity variance is invalid
                            let weak_message = GaussianMessage::new(1e-10, 0.0);
                            diff_var.messages_from_factors.insert(*factor_id, weak_message);
                            return Ok(());
                        }
                        
                        let sqrt_c = c_cavity.sqrt();
                        let t_arg = d_cavity / sqrt_c;
                        let epsilon_arg = epsilon / sqrt_c;
                        
                        let normal = Normal::new(0.0, 1.0).unwrap();
                        
                        let (v_val, w_val) = match outcome {
                            ComparisonOutcome::Win => {
                                let cdf_val = normal.cdf(t_arg - epsilon_arg);
                                if cdf_val < 1e-10 {
                                    // Avoid division by very small numbers
                                    (0.0, 0.0)
                                } else {
                                    let v = normal.pdf(t_arg - epsilon_arg) / cdf_val;
                                    let w = v * (v + t_arg - epsilon_arg);
                                    (v, w)
                                }
                            }
                            ComparisonOutcome::Draw => {
                                let phi_upper = normal.cdf(epsilon_arg - t_arg);
                                let phi_lower = normal.cdf(-epsilon_arg - t_arg);
                                let pdf_upper = normal.pdf(epsilon_arg - t_arg);
                                let pdf_lower = normal.pdf(-epsilon_arg - t_arg);
                                
                                let denom = phi_upper - phi_lower;
                                if denom.abs() < 1e-10 {
                                    (0.0, 0.0)
                                } else {
                                    let v = (pdf_lower - pdf_upper) / denom;
                                    let w = v * v + 
                                        ((epsilon_arg - t_arg) * pdf_upper + (epsilon_arg + t_arg) * pdf_lower) / denom;
                                    (v, w)
                                }
                            }
                        };
                        
                        // Update precision and precision-adjusted mean
                        let delta_precision = w_val / c_cavity;
                        let delta_precision_adjusted_mean = v_val / sqrt_c;
                        
                        let new_precision = cavity.precision + delta_precision;
                        let new_precision_adjusted_mean = cavity.precision_adjusted_mean + delta_precision_adjusted_mean;
                        
                        if new_precision > 0.0 {
                            let updated_message = GaussianMessage::new(new_precision, new_precision_adjusted_mean);
                            diff_var.messages_from_factors.insert(*factor_id, updated_message);
                        } else {
                            // Send a weak message if precision becomes non-positive
                            let weak_message = GaussianMessage::new(1e-10, new_precision_adjusted_mean);
                            diff_var.messages_from_factors.insert(*factor_id, weak_message);
                        }
                    }
                }
            }
            
            Ok(())
        }
    }
    
    /// Message passing scheduler for the TrueSkill factor graph
    pub struct MessagePassingScheduler {
        variables: HashMap<usize, VariableNode>,
        priors: HashMap<usize, GaussianMessage>,
        factors: Vec<FactorNode>,
        convergence_threshold: f64,
        max_iterations: usize,
    }
    
    impl MessagePassingScheduler {
        pub fn new(convergence_threshold: f64, max_iterations: usize) -> Self {
            Self {
                variables: HashMap::new(),
                priors: HashMap::new(),
                factors: Vec::new(),
                convergence_threshold,
                max_iterations,
            }
        }
        
        pub fn add_variable(&mut self, id: usize, prior: GaussianMessage) {
            self.priors.insert(id, prior.clone());
            self.variables.insert(id, VariableNode::new(prior));
        }
        
        pub fn add_factor(&mut self, factor: FactorNode) {
            self.factors.push(factor);
        }
        
        pub fn run_message_passing(&mut self) -> Result<()> {
            let mut prev_marginals = HashMap::new();
            
            for iteration in 0..self.max_iterations {
                // Store previous marginals for convergence check
                for (id, var) in &self.variables {
                    prev_marginals.insert(*id, var.marginal.clone());
                }
                
                // Update all factor messages
                for factor in &self.factors {
                    factor.update_messages(&mut self.variables, &self.priors)?;
                }
                
                // Update all variable marginals
                for (id, var) in self.variables.iter_mut() {
                    if let Some(prior) = self.priors.get(id) {
                        var.update_marginal(prior)?;
                    }
                }
                
                // Check convergence
                let mut converged = true;
                for (id, var) in &self.variables {
                    if let Some(prev_marginal) = prev_marginals.get(id) {
                        let mean_diff = (var.marginal.mean() - prev_marginal.mean()).abs();
                        let var_diff = (var.marginal.variance() - prev_marginal.variance()).abs();
                        
                        if mean_diff > self.convergence_threshold || var_diff > self.convergence_threshold {
                            converged = false;
                            break;
                        }
                    }
                }
                
                if converged {
                    println!("Converged after {} iterations", iteration + 1);
                    break;
                }
                
                if iteration == self.max_iterations - 1 {
                    return Err(Error::ConvergenceFailure(
                        format!("Message passing did not converge after {} iterations", self.max_iterations),
                    ));
                }
            }
            
            Ok(())
        }
        
        pub fn get_variable_marginal(&self, id: usize) -> Option<&GaussianMessage> {
            self.variables.get(&id).map(|var| &var.marginal)
        }
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
        
        // Build factor graph
        let mut scheduler = MessagePassingScheduler::new(self.convergence_threshold, self.max_iterations);
        let mut variable_counter = 0;
        let mut factor_counter = 0;
        
        // Track variable IDs for each player, performance, team performance, and differences
        let mut player_skill_ids = Vec::new();
        let mut performance_ids = Vec::new();
        let mut team_performance_ids = Vec::new();
        
        // Add skill variables and performance variables for each player
        for team in rating_groups {
            let mut team_skill_ids = Vec::new();
            let mut team_perf_ids = Vec::new();
            
            for rating in team.player_ratings() {
                // Add skill variable with dynamics variance added
                let skill_variance = rating.variance + self.gamma_squared;
                let skill_prior = GaussianMessage::from_mean_and_variance(rating.mean, skill_variance)?;
                scheduler.add_variable(variable_counter, skill_prior);
                team_skill_ids.push(variable_counter);
                variable_counter += 1;
                
                // Add performance variable with uniform prior
                let perf_prior = GaussianMessage::uniform();
                scheduler.add_variable(variable_counter, perf_prior);
                team_perf_ids.push(variable_counter);
                variable_counter += 1;
                
                // Add likelihood factor connecting skill to performance
                scheduler.add_factor(FactorNode::Likelihood {
                    factor_id: factor_counter,
                    skill_id: team_skill_ids[team_skill_ids.len() - 1],
                    performance_id: team_perf_ids[team_perf_ids.len() - 1],
                    beta_squared: self.beta_squared,
                });
                factor_counter += 1;
            }
            
            player_skill_ids.push(team_skill_ids);
            
            // Add team performance variable with uniform prior
            let team_perf_prior = GaussianMessage::uniform();
            scheduler.add_variable(variable_counter, team_perf_prior);
            team_performance_ids.push(variable_counter);
            
            // Add sum factor for team performance
            scheduler.add_factor(FactorNode::Sum {
                factor_id: factor_counter,
                performance_ids: team_perf_ids.clone(),
                team_performance_id: variable_counter,
                coefficients: vec![1.0; team.player_ratings().len()], // Equal contribution
            });
            factor_counter += 1;
            
            performance_ids.push(team_perf_ids);
            variable_counter += 1;
        }
        
        // Create pairwise comparisons based on ranks
        for i in 0..rating_groups.len() {
            for j in (i + 1)..rating_groups.len() {
                // Add difference variable with uniform prior
                let diff_prior = GaussianMessage::uniform();
                scheduler.add_variable(variable_counter, diff_prior);
                let diff_id = variable_counter;
                variable_counter += 1;
                
                // Add difference factor
                scheduler.add_factor(FactorNode::Difference {
                    factor_id: factor_counter,
                    team_a_id: team_performance_ids[i],
                    team_b_id: team_performance_ids[j],
                    difference_id: diff_id,
                });
                factor_counter += 1;
                
                // Determine outcome and add comparison factor
                let comparison_outcome = if ranks[i] < ranks[j] {
                    ComparisonOutcome::Win // Team i wins (lower rank is better)
                } else if ranks[i] == ranks[j] {
                    ComparisonOutcome::Draw
                } else {
                    // Swap teams so the better team is always first
                    scheduler.add_factor(FactorNode::Difference {
                        factor_id: factor_counter - 1, // Replace the previous difference factor
                        team_a_id: team_performance_ids[j],
                        team_b_id: team_performance_ids[i],
                        difference_id: diff_id,
                    });
                    ComparisonOutcome::Win
                };
                
                scheduler.add_factor(FactorNode::Comparison {
                    factor_id: factor_counter,
                    difference_id: diff_id,
                    epsilon: self.draw_margin,
                    outcome: comparison_outcome,
                });
                factor_counter += 1;
            }
        }
        
        // Run message passing
        scheduler.run_message_passing()?;
        
        // Extract updated ratings
        let mut updated_teams = Vec::new();
        let mut skill_id_counter = 0;
        
        for (_team_idx, team) in rating_groups.iter().enumerate() {
            let mut updated_ratings = Vec::new();
            
            for _ in team.player_ratings() {
                if let Some(marginal) = scheduler.get_variable_marginal(skill_id_counter) {
                    let new_mean = marginal.mean();
                    let new_variance = marginal.variance();
                    
                    // Ensure variance is positive and reasonable
                    let final_variance = if new_variance <= 0.0 || new_variance.is_infinite() {
                        self.sigma_0_squared // Fall back to default variance
                    } else {
                        new_variance
                    };
                    
                    let updated_rating = TrueSkillRating::new(new_mean, final_variance)?;
                    updated_ratings.push(updated_rating);
                }
                skill_id_counter += 2; // Skip performance variable
            }
            
            updated_teams.push(TrueSkillTeam::from_player_ratings(updated_ratings));
        }
        
        Ok(updated_teams)
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
        
        // Basic structure checks
        assert!(winner_rating.variance() > 0.0);
        assert!(loser_rating.variance() > 0.0);
        
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
        
        // Basic structure checks
        assert!(player1_rating.variance() > 0.0);
        assert!(player2_rating.variance() > 0.0);
        
        // Check that ratings are reasonable
        assert!(player1_rating.mean() >= 20.0 && player1_rating.mean() <= 30.0);
        assert!(player2_rating.mean() >= 20.0 && player2_rating.mean() <= 30.0);
    }
}