use crate::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating, Outcome},
    error::{Error, Result},
};
use statrs::distribution::{Normal, ContinuousCDF, Continuous};

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

/// Factor graph implementation for TrueSkill message passing
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
        
        /// Multiplies this message with another (for combining Gaussians)
        pub fn multiply(&self, other: &GaussianMessage) -> GaussianMessage {
            GaussianMessage {
                precision: self.precision + other.precision,
                precision_adjusted_mean: self.precision_adjusted_mean + other.precision_adjusted_mean,
            }
        }
        
        /// Divides this message by another (for cavity distributions)
        pub fn divide(&self, other: &GaussianMessage) -> Result<GaussianMessage> {
            let new_precision = self.precision - other.precision;
            let new_precision_adjusted_mean = self.precision_adjusted_mean - other.precision_adjusted_mean;
            
            if new_precision <= 0.0 {
                return Err(Error::NumericalError(
                    "Division resulted in non-positive precision".to_string(),
                ));
            }
            
            Ok(GaussianMessage {
                precision: new_precision,
                precision_adjusted_mean: new_precision_adjusted_mean,
            })
        }
    }
    
    /// V function for comparison factors (win case)
    pub fn v_win(t: f64, epsilon: f64) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let argument = t - epsilon;
        let pdf = normal.pdf(argument);
        let cdf = normal.cdf(argument);
        
        if cdf.abs() < f64::EPSILON {
            -argument // Approximation when CDF approaches 0
        } else {
            pdf / cdf
        }
    }
    
    /// W function for comparison factors (win case)
    pub fn w_win(t: f64, epsilon: f64) -> f64 {
        let v = v_win(t, epsilon);
        v * (v + t - epsilon)
    }
    
    /// V function for comparison factors (draw case)
    pub fn v_draw(t: f64, epsilon: f64) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let pdf_upper = normal.pdf(epsilon - t);
        let pdf_lower = normal.pdf(-epsilon - t);
        let cdf_upper = normal.cdf(epsilon - t);
        let cdf_lower = normal.cdf(-epsilon - t);
        
        let denominator = cdf_upper - cdf_lower;
        
        if denominator.abs() < f64::EPSILON {
            0.0 // When draw probability approaches 0
        } else {
            (pdf_lower - pdf_upper) / denominator
        }
    }
    
    /// W function for comparison factors (draw case)
    pub fn w_draw(t: f64, epsilon: f64) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let v = v_draw(t, epsilon);
        let pdf_upper = normal.pdf(epsilon - t);
        let pdf_lower = normal.pdf(-epsilon - t);
        let cdf_upper = normal.cdf(epsilon - t);
        let cdf_lower = normal.cdf(-epsilon - t);
        
        let denominator = cdf_upper - cdf_lower;
        
        if denominator.abs() < f64::EPSILON {
            0.0
        } else {
            let term = ((epsilon - t) * pdf_upper + (epsilon + t) * pdf_lower) / denominator;
            v * v + term
        }
    }
    
    /// Factor graph for a single game
    pub struct FactorGraph {
        /// Number of players
        num_players: usize,
        /// Number of teams
        num_teams: usize,
        /// Player assignments to teams
        team_assignments: Vec<usize>,
        /// Game outcome (ranks)
        ranks: Vec<usize>,
        /// TrueSkill parameters
        beta_squared: f64,
        epsilon: f64,
        
        // Variables (marginals)
        /// Skill variables (s_i)
        skills: Vec<GaussianMessage>,
        /// Performance variables (p_i)
        performances: Vec<GaussianMessage>,
        /// Team performance variables (t_j)
        team_performances: Vec<GaussianMessage>,
        /// Performance difference variables (d_k)
        performance_diffs: Vec<GaussianMessage>,
        
        // Messages
        /// Messages from skills to performances
        skill_to_perf_messages: Vec<GaussianMessage>,
        /// Messages from performances to skills
        perf_to_skill_messages: Vec<GaussianMessage>,
        /// Messages from performances to team performances
        perf_to_team_messages: Vec<Vec<GaussianMessage>>,
        /// Messages from team performances to performances
        team_to_perf_messages: Vec<Vec<GaussianMessage>>,
        /// Messages from team performances to differences
        team_to_diff_messages: Vec<Vec<GaussianMessage>>,
        /// Messages from differences to team performances
        diff_to_team_messages: Vec<Vec<GaussianMessage>>,
        /// Messages from differences to comparisons
        diff_to_comp_messages: Vec<GaussianMessage>,
        /// Messages from comparisons to differences
        comp_to_diff_messages: Vec<GaussianMessage>,
    }
    
    impl FactorGraph {
        /// Creates a new factor graph for the given game setup
        pub fn new(
            player_skills: &[TrueSkillRating],
            team_assignments: Vec<usize>,
            ranks: Vec<usize>,
            beta_squared: f64,
            epsilon: f64,
        ) -> Result<Self> {
            let num_players = player_skills.len();
            let num_teams = team_assignments.iter().max().map(|&x| x + 1).unwrap_or(0);
            
            if team_assignments.len() != num_players {
                return Err(Error::InvalidInput(
                    "Team assignments length must match number of players".to_string(),
                ));
            }
            
            if ranks.len() != num_teams {
                return Err(Error::InvalidInput(
                    "Ranks length must match number of teams".to_string(),
                ));
            }
            
            // Initialize skill variables from current player ratings
            let skills: Vec<GaussianMessage> = player_skills
                .iter()
                .map(|rating| {
                    GaussianMessage::from_mean_and_variance(rating.mean(), rating.variance())
                })
                .collect::<Result<Vec<_>>>()?;
            
            // Initialize other variables with uninformative priors
            let performances = vec![GaussianMessage::new(0.0, 0.0); num_players];
            let team_performances = vec![GaussianMessage::new(0.0, 0.0); num_teams];
            
            // Calculate number of comparisons (adjacent teams in ranking)
            let num_comparisons = if num_teams > 1 { num_teams - 1 } else { 0 };
            let performance_diffs = vec![GaussianMessage::new(0.0, 0.0); num_comparisons];
            
            // Initialize messages
            let skill_to_perf_messages = vec![GaussianMessage::new(0.0, 0.0); num_players];
            let perf_to_skill_messages = vec![GaussianMessage::new(0.0, 0.0); num_players];
            
            let mut perf_to_team_messages = vec![Vec::new(); num_teams];
            let mut team_to_perf_messages = vec![Vec::new(); num_teams];
            
            // Initialize team message arrays based on team compositions
            for team_idx in 0..num_teams {
                let team_size = team_assignments.iter().filter(|&&t| t == team_idx).count();
                perf_to_team_messages[team_idx] = vec![GaussianMessage::new(0.0, 0.0); team_size];
                team_to_perf_messages[team_idx] = vec![GaussianMessage::new(0.0, 0.0); team_size];
            }
            
            let team_to_diff_messages = vec![vec![GaussianMessage::new(0.0, 0.0); 2]; num_comparisons];
            let diff_to_team_messages = vec![vec![GaussianMessage::new(0.0, 0.0); 2]; num_comparisons];
            let diff_to_comp_messages = vec![GaussianMessage::new(0.0, 0.0); num_comparisons];
            let comp_to_diff_messages = vec![GaussianMessage::new(0.0, 0.0); num_comparisons];
            
            Ok(FactorGraph {
                num_players,
                num_teams,
                team_assignments,
                ranks,
                beta_squared,
                epsilon,
                skills,
                performances,
                team_performances,
                performance_diffs,
                skill_to_perf_messages,
                perf_to_skill_messages,
                perf_to_team_messages,
                team_to_perf_messages,
                team_to_diff_messages,
                diff_to_team_messages,
                diff_to_comp_messages,
                comp_to_diff_messages,
            })
        }
        
        /// Runs the message passing algorithm
        pub fn run_message_passing(&mut self, max_iterations: usize, convergence_epsilon: f64) -> Result<()> {
            // Phase 1: Light arrows (top to bottom)
            self.update_skill_to_performance_messages()?;
            self.update_performance_marginals()?;
            self.update_performance_to_team_messages()?;
            self.update_team_performance_marginals()?;
            
            // Phase 2: Iterative loop for team performances and differences
            for iteration in 0..max_iterations {
                let old_team_performances = self.team_performances.clone();
                
                self.update_team_to_difference_messages()?;
                self.update_difference_marginals()?;
                self.update_difference_to_comparison_messages()?;
                self.update_comparison_to_difference_messages()?;
                self.update_difference_to_team_messages()?;
                self.update_team_performance_marginals()?;
                
                // Check convergence
                let mut max_change: f64 = 0.0;
                for (old, new) in old_team_performances.iter().zip(self.team_performances.iter()) {
                    let mean_change = (old.mean() - new.mean()).abs();
                    let var_change = (old.variance() - new.variance()).abs();
                    max_change = max_change.max(mean_change).max(var_change);
                }
                
                if max_change < convergence_epsilon {
                    break;
                }
            }
            
            // Phase 3: Dark arrows (bottom to top)
            self.update_team_to_performance_messages()?;
            self.update_performance_marginals()?;
            self.update_performance_to_skill_messages()?;
            self.update_skill_marginals()?;
            
            Ok(())
        }
        
        /// Updates messages from skills to performances (likelihood factors)
        fn update_skill_to_performance_messages(&mut self) -> Result<()> {
            for i in 0..self.num_players {
                // Likelihood factor: p_i ~ N(s_i, β²)
                // Message from skill to performance is just the skill marginal
                self.skill_to_perf_messages[i] = self.skills[i].clone();
            }
            Ok(())
        }
        
        /// Updates performance marginals
        fn update_performance_marginals(&mut self) -> Result<()> {
            for i in 0..self.num_players {
                // Performance marginal = skill_to_perf * perf_to_skill^-1 + β² noise
                let cavity = self.skill_to_perf_messages[i].divide(&self.perf_to_skill_messages[i])
                    .unwrap_or_else(|_| self.skill_to_perf_messages[i].clone());
                
                // Add performance variance (β²)
                let performance_precision = 1.0 / self.beta_squared;
                let noise = GaussianMessage::new(performance_precision, 0.0);
                
                self.performances[i] = cavity.multiply(&noise);
            }
            Ok(())
        }
        
        /// Updates messages from performances to team performances
        fn update_performance_to_team_messages(&mut self) -> Result<()> {
            for team_idx in 0..self.num_teams {
                let mut player_idx_in_team = 0;
                for player_idx in 0..self.num_players {
                    if self.team_assignments[player_idx] == team_idx {
                        // Message is the performance marginal divided by incoming message
                        let cavity = self.performances[player_idx].divide(&self.team_to_perf_messages[team_idx][player_idx_in_team])
                            .unwrap_or_else(|_| self.performances[player_idx].clone());
                        self.perf_to_team_messages[team_idx][player_idx_in_team] = cavity;
                        player_idx_in_team += 1;
                    }
                }
            }
            Ok(())
        }
        
        /// Updates team performance marginals (sum factors)
        fn update_team_performance_marginals(&mut self) -> Result<()> {
            for team_idx in 0..self.num_teams {
                let mut team_precision = 0.0;
                let mut team_precision_adjusted_mean = 0.0;
                
                // Sum all incoming messages from team members
                for msg in &self.perf_to_team_messages[team_idx] {
                    team_precision += msg.precision;
                    team_precision_adjusted_mean += msg.precision_adjusted_mean;
                }
                
                self.team_performances[team_idx] = GaussianMessage::new(
                    team_precision,
                    team_precision_adjusted_mean,
                );
            }
            Ok(())
        }
        
        /// Updates messages from team performances to differences
        fn update_team_to_difference_messages(&mut self) -> Result<()> {
            for comp_idx in 0..self.performance_diffs.len() {
                // For each comparison, we have team_winner vs team_loser
                let team_winner = self.get_winning_team(comp_idx);
                let team_loser = self.get_losing_team(comp_idx);
                
                // Message from team_winner to difference (positive contribution)
                let winner_cavity = self.team_performances[team_winner].divide(&self.diff_to_team_messages[comp_idx][0])
                    .unwrap_or_else(|_| self.team_performances[team_winner].clone());
                self.team_to_diff_messages[comp_idx][0] = winner_cavity;
                
                // Message from team_loser to difference (negative contribution)
                let loser_cavity = self.team_performances[team_loser].divide(&self.diff_to_team_messages[comp_idx][1])
                    .unwrap_or_else(|_| self.team_performances[team_loser].clone());
                self.team_to_diff_messages[comp_idx][1] = loser_cavity;
            }
            Ok(())
        }
        
        /// Updates difference marginals
        fn update_difference_marginals(&mut self) -> Result<()> {
            for comp_idx in 0..self.performance_diffs.len() {
                // Difference = team_winner - team_loser
                let winner_msg = &self.team_to_diff_messages[comp_idx][0];
                let loser_msg = &self.team_to_diff_messages[comp_idx][1];
                
                // For difference d = t1 - t2, precision and precision-adjusted mean combine
                let diff_precision = winner_msg.precision + loser_msg.precision;
                let diff_precision_adjusted_mean = winner_msg.precision_adjusted_mean - loser_msg.precision_adjusted_mean;
                
                self.performance_diffs[comp_idx] = GaussianMessage::new(
                    diff_precision,
                    diff_precision_adjusted_mean,
                );
            }
            Ok(())
        }
        
        /// Updates messages from differences to comparisons
        fn update_difference_to_comparison_messages(&mut self) -> Result<()> {
            for comp_idx in 0..self.performance_diffs.len() {
                let cavity = self.performance_diffs[comp_idx].divide(&self.comp_to_diff_messages[comp_idx])
                    .unwrap_or_else(|_| self.performance_diffs[comp_idx].clone());
                self.diff_to_comp_messages[comp_idx] = cavity;
            }
            Ok(())
        }
        
        /// Updates messages from comparisons to differences (using V and W functions)
        fn update_comparison_to_difference_messages(&mut self) -> Result<()> {
            for comp_idx in 0..self.performance_diffs.len() {
                let cavity = &self.diff_to_comp_messages[comp_idx];
                let is_draw = self.is_draw(comp_idx);
                
                if cavity.variance() <= 0.0 {
                    continue; // Skip invalid cavity distributions
                }
                
                let t = cavity.mean() / cavity.variance().sqrt();
                let epsilon_effective = self.epsilon / cavity.variance().sqrt();
                
                let (v, w) = if is_draw {
                    (v_draw(t, epsilon_effective), w_draw(t, epsilon_effective))
                } else {
                    (v_win(t, epsilon_effective), w_win(t, epsilon_effective))
                };
                
                // Update using moment matching
                let new_precision = cavity.precision + w;
                let new_precision_adjusted_mean = cavity.precision_adjusted_mean + v;
                
                self.comp_to_diff_messages[comp_idx] = GaussianMessage::new(
                    new_precision.max(f64::EPSILON), // Ensure positive precision
                    new_precision_adjusted_mean,
                );
            }
            Ok(())
        }
        
        /// Updates messages from differences back to team performances
        fn update_difference_to_team_messages(&mut self) -> Result<()> {
            for comp_idx in 0..self.performance_diffs.len() {
                let diff_marginal = &self.performance_diffs[comp_idx];
                
                // Message to winner team (positive coefficient)
                self.diff_to_team_messages[comp_idx][0] = diff_marginal.clone();
                
                // Message to loser team (negative coefficient)
                // For negative coefficient, we flip the sign of precision-adjusted mean
                self.diff_to_team_messages[comp_idx][1] = GaussianMessage::new(
                    diff_marginal.precision,
                    -diff_marginal.precision_adjusted_mean,
                );
            }
            Ok(())
        }
        
        /// Updates messages from team performances back to performances
        fn update_team_to_performance_messages(&mut self) -> Result<()> {
            for team_idx in 0..self.num_teams {
                let team_marginal = &self.team_performances[team_idx];
                let team_size = self.perf_to_team_messages[team_idx].len();
                
                for member_idx in 0..team_size {
                    let cavity = team_marginal.divide(&self.perf_to_team_messages[team_idx][member_idx])
                        .unwrap_or_else(|_| team_marginal.clone());
                    self.team_to_perf_messages[team_idx][member_idx] = cavity;
                }
            }
            Ok(())
        }
        
        /// Updates messages from performances back to skills
        fn update_performance_to_skill_messages(&mut self) -> Result<()> {
            for i in 0..self.num_players {
                let team_idx = self.team_assignments[i];
                let member_idx = self.get_member_index_in_team(i, team_idx);
                
                let perf_marginal = &self.performances[i];
                let team_to_perf = &self.team_to_perf_messages[team_idx][member_idx];
                
                // Remove performance noise (β²) and team contribution
                let performance_noise = GaussianMessage::new(1.0 / self.beta_squared, 0.0);
                let without_noise = perf_marginal.divide(&performance_noise)
                    .unwrap_or_else(|_| perf_marginal.clone());
                
                self.perf_to_skill_messages[i] = without_noise.divide(team_to_perf)
                    .unwrap_or_else(|_| without_noise);
            }
            Ok(())
        }
        
        /// Updates final skill marginals
        fn update_skill_marginals(&mut self) -> Result<()> {
            for i in 0..self.num_players {
                // Skill posterior = prior * perf_to_skill
                self.skills[i] = self.skills[i].multiply(&self.perf_to_skill_messages[i]);
            }
            Ok(())
        }
        
        /// Helper: Gets the winning team for a comparison
        fn get_winning_team(&self, comp_idx: usize) -> usize {
            // Find teams with ranks comp_idx+1 and comp_idx+2
            for (team_idx, &rank) in self.ranks.iter().enumerate() {
                if rank == comp_idx + 1 {
                    return team_idx;
                }
            }
            0 // Fallback
        }
        
        /// Helper: Gets the losing team for a comparison
        fn get_losing_team(&self, comp_idx: usize) -> usize {
            // Find teams with ranks comp_idx+1 and comp_idx+2
            for (team_idx, &rank) in self.ranks.iter().enumerate() {
                if rank == comp_idx + 2 {
                    return team_idx;
                }
            }
            1 // Fallback
        }
        
        /// Helper: Checks if a comparison is a draw
        fn is_draw(&self, comp_idx: usize) -> bool {
            let winner_rank = comp_idx + 1;
            let loser_rank = comp_idx + 2;
            
            // Check if any teams have the same rank (indicating a draw)
            let winner_count = self.ranks.iter().filter(|&&r| r == winner_rank).count();
            let loser_count = self.ranks.iter().filter(|&&r| r == loser_rank).count();
            
            winner_count > 1 || loser_count > 1
        }
        
        /// Helper: Gets the index of a player within their team
        fn get_member_index_in_team(&self, player_idx: usize, team_idx: usize) -> usize {
            let mut member_idx = 0;
            for i in 0..player_idx {
                if self.team_assignments[i] == team_idx {
                    member_idx += 1;
                }
            }
            member_idx
        }
        
        /// Returns the updated skill ratings
        pub fn get_updated_skills(&self) -> Vec<TrueSkillRating> {
            self.skills
                .iter()
                .map(|skill| TrueSkillRating {
                    mean: skill.mean(),
                    variance: skill.variance(),
                })
                .collect()
        }
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
        if rating_groups.is_empty() {
            return Err(Error::InvalidInput("No rating groups provided".to_string()));
        }
        
        if !outcome.is_valid_for_team_count(rating_groups.len()) {
            return Err(Error::InvalidInput("Invalid outcome for given teams".to_string()));
        }
        
        // Flatten all players and create team assignments
        let mut all_players = Vec::new();
        let mut team_assignments = Vec::new();
        
        for (team_idx, team) in rating_groups.iter().enumerate() {
            for player in team.player_ratings() {
                all_players.push(player.clone());
                team_assignments.push(team_idx);
            }
        }
        
        // Create and run factor graph
        let mut graph = factor_graph::FactorGraph::new(
            &all_players,
            team_assignments.clone(),
            outcome.ranks().to_vec(),
            self.beta_squared,
            self.draw_margin,
        )?;
        
        graph.run_message_passing(10, 0.0001)?;
        let updated_skills = graph.get_updated_skills();
        
        // Add dynamics variance for next game
        let updated_skills_with_dynamics: Vec<TrueSkillRating> = updated_skills
            .into_iter()
            .map(|mut skill| {
                skill.variance += self.gamma_squared;
                skill
            })
            .collect();
        
        // Reconstruct team structure
        let mut result_teams = Vec::new();
        let mut player_idx = 0;
        
        for team in rating_groups {
            let team_size = team.player_ratings().len();
            let team_players = updated_skills_with_dynamics[player_idx..player_idx + team_size].to_vec();
            result_teams.push(TrueSkillTeam::from_player_ratings(team_players));
            player_idx += team_size;
        }
        
        Ok(result_teams)
    }
    
    fn calculate_match_quality(&self, rating_groups: &[Self::TeamRating]) -> Result<f64> {
        if rating_groups.len() != 2 {
            return Err(Error::InvalidInput(
                "Match quality calculation currently only supports 2 teams".to_string(),
            ));
        }
        
        // For now, implement pairwise match quality between two teams
        let team1 = &rating_groups[0];
        let team2 = &rating_groups[1];
        
        if team1.player_ratings().len() != 1 || team2.player_ratings().len() != 1 {
            return Err(Error::InvalidInput(
                "Match quality calculation currently only supports 1v1 matches".to_string(),
            ));
        }
        
        let player1 = &team1.player_ratings()[0];
        let player2 = &team2.player_ratings()[0];
        
        // Pairwise match quality formula from TrueSkill paper
        let mu_diff = player1.mean() - player2.mean();
        let variance_sum = 2.0 * self.beta_squared + player1.variance() + player2.variance();
        
        let coefficient = (2.0 * self.beta_squared / variance_sum).sqrt();
        let exponential = (-mu_diff * mu_diff / (2.0 * variance_sum)).exp();
        
        Ok(coefficient * exponential)
    }
}