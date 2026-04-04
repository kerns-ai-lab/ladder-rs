use crate::core::{GameOutcome, Outcome, Rating, RatingSystem, TeamRating};
use crate::error::Result;
#[cfg(feature = "full-deps")]
use rayon::prelude::*;
use std::f64::consts::PI;

const GLICKO_SCALE: f64 = 173.7178;

/// Glicko rating structure with mean (μ) and Rating Deviation (RD).
#[derive(Clone, Debug, PartialEq)]
pub struct GlickoRating {
    /// The mean (μ) of the player's skill distribution
    pub mu: f64,
    /// The Rating Deviation (RD), analogous to standard deviation
    pub rd: f64,
}

impl GlickoRating {
    /// Creates a new Glicko rating with specified values.
    pub fn new(mu: f64, rd: f64) -> Self {
        Self { mu, rd }
    }

    /// Creates a default Glicko rating (μ=1500, RD=350).
    pub fn with_default_values() -> Self {
        Self::new(1500.0, 350.0)
    }

    /// Converts rating to Glicko-2 scale (divide by 173.7178).
    pub fn to_glicko2_scale(&self) -> (f64, f64) {
        (self.mu / GLICKO_SCALE, self.rd / GLICKO_SCALE)
    }

    /// Creates rating from Glicko-2 scale (multiply by 173.7178).
    pub fn from_glicko2_scale(mu_scaled: f64, rd_scaled: f64) -> Self {
        Self::new(mu_scaled * GLICKO_SCALE, rd_scaled * GLICKO_SCALE)
    }
}

impl Rating for GlickoRating {
    fn mean(&self) -> f64 {
        self.mu
    }

    fn variance(&self) -> f64 {
        self.rd * self.rd
    }

    fn standard_deviation(&self) -> f64 {
        self.rd
    }

    fn conservative_rating(&self) -> f64 {
        self.mu - 2.0 * self.rd
    }
}

/// Glicko-2 rating structure extending Glicko with Rating Volatility (σ_volatility).
#[derive(Clone, Debug, PartialEq)]
pub struct Glicko2Rating {
    /// The mean (μ) of the player's skill distribution
    pub mu: f64,
    /// The Rating Deviation (RD), analogous to standard deviation  
    pub rd: f64,
    /// The Rating Volatility (σ_volatility)
    pub volatility: f64,
}

impl Glicko2Rating {
    /// Creates a new Glicko-2 rating with specified values.
    pub fn new(mu: f64, rd: f64, volatility: f64) -> Self {
        Self { mu, rd, volatility }
    }

    /// Creates a default Glicko-2 rating (μ=1500, RD=350, σ_volatility=0.06).
    pub fn with_default_values() -> Self {
        Self::new(1500.0, 350.0, 0.06)
    }

    /// Converts rating to Glicko-2 scale.
    pub fn to_glicko2_scale(&self) -> (f64, f64, f64) {
        (
            self.mu / GLICKO_SCALE,
            self.rd / GLICKO_SCALE,
            self.volatility,
        )
    }

    /// Creates rating from Glicko-2 scale.
    pub fn from_glicko2_scale(mu_scaled: f64, rd_scaled: f64, volatility: f64) -> Self {
        Self::new(
            mu_scaled * GLICKO_SCALE,
            rd_scaled * GLICKO_SCALE,
            volatility,
        )
    }
}

impl Rating for Glicko2Rating {
    fn mean(&self) -> f64 {
        self.mu
    }

    fn variance(&self) -> f64 {
        self.rd * self.rd
    }

    fn standard_deviation(&self) -> f64 {
        self.rd
    }

    fn conservative_rating(&self) -> f64 {
        self.mu - 2.0 * self.rd
    }
}

/// Team rating for Glicko system.
#[derive(Clone, Debug)]
pub struct GlickoTeamRating {
    player_ratings: Vec<GlickoRating>,
}

impl TeamRating for GlickoTeamRating {
    type PlayerRating = GlickoRating;

    fn player_ratings(&self) -> &[Self::PlayerRating] {
        &self.player_ratings
    }

    fn from_player_ratings(ratings: Vec<Self::PlayerRating>) -> Self {
        Self {
            player_ratings: ratings,
        }
    }
}

/// Team rating for Glicko-2 system.
#[derive(Clone, Debug)]
pub struct Glicko2TeamRating {
    player_ratings: Vec<Glicko2Rating>,
}

impl TeamRating for Glicko2TeamRating {
    type PlayerRating = Glicko2Rating;

    fn player_ratings(&self) -> &[Self::PlayerRating] {
        &self.player_ratings
    }

    fn from_player_ratings(ratings: Vec<Self::PlayerRating>) -> Self {
        Self {
            player_ratings: ratings,
        }
    }
}

/// Glicko rating system configuration.
#[derive(Clone, Debug)]
pub struct GlickoConfig {
    /// System constant for rating period variance increase
    pub c: f64,
    /// Conversion factor  
    pub q: f64,
}

impl Default for GlickoConfig {
    fn default() -> Self {
        Self {
            c: 15.8, // Typical value for rating period variance increase
            q: (10.0_f64).ln() / 400.0,
        }
    }
}

/// Glicko rating system implementation.
#[derive(Clone, Debug)]
pub struct Glicko {
    config: GlickoConfig,
}

impl Glicko {
    /// Creates a new Glicko rating system with default configuration.
    pub fn new() -> Self {
        Self {
            config: GlickoConfig::default(),
        }
    }

    /// Creates a new Glicko rating system with custom configuration.
    pub fn with_config(config: GlickoConfig) -> Self {
        Self { config }
    }

    /// Calculates the g function for Glicko algorithm.
    fn g(&self, rd: f64) -> f64 {
        1.0 / (1.0 + 3.0 * self.config.q * self.config.q * rd * rd / (PI * PI)).sqrt()
    }

    /// Calculates expected score for Glicko algorithm.
    fn expected_score(&self, mu_i: f64, mu_j: f64, rd_j: f64) -> f64 {
        1.0 / (1.0 + 10.0_f64.powf(-self.g(rd_j) * (mu_i - mu_j) / 400.0))
    }

    /// Updates a single player's rating based on match results.
    fn update_rating(
        &self,
        rating: &GlickoRating,
        opponents: &[(GlickoRating, f64)],
    ) -> GlickoRating {
        if opponents.is_empty() {
            // No games played, just increase RD due to time passage
            let new_rd = (rating.rd * rating.rd + self.config.c * self.config.c).sqrt();
            return GlickoRating::new(rating.mu, new_rd);
        }

        #[cfg(feature = "full-deps")]
        let (d_squared_inv_sum, delta_sum) = opponents
            .par_iter()
            .map(|(opponent, score)| {
                let g_rd = self.g(opponent.rd);
                let expected = self.expected_score(rating.mu, opponent.mu, opponent.rd);

                (
                    g_rd * g_rd * expected * (1.0 - expected),
                    g_rd * (score - expected),
                )
            })
            .reduce(|| (0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));

        #[cfg(not(feature = "full-deps"))]
        let (d_squared_inv_sum, delta_sum): (f64, f64) = opponents
            .iter()
            .map(|(opponent, score)| {
                let g_rd = self.g(opponent.rd);
                let expected = self.expected_score(rating.mu, opponent.mu, opponent.rd);

                (
                    g_rd * g_rd * expected * (1.0 - expected),
                    g_rd * (score - expected),
                )
            })
            .fold((0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));

        let d_squared_inv = d_squared_inv_sum * self.config.q * self.config.q;

        // Calculate new rating deviation
        let rd_new_inv_squared = 1.0 / (rating.rd * rating.rd) + d_squared_inv;
        let rd_new = 1.0 / rd_new_inv_squared.sqrt();

        // Calculate new rating
        let mu_new = rating.mu + self.config.q * rd_new * rd_new * delta_sum;

        GlickoRating::new(mu_new, rd_new)
    }
}

impl Default for Glicko {
    fn default() -> Self {
        Self::new()
    }
}

impl RatingSystem for Glicko {
    type PlayerRating = GlickoRating;
    type TeamRating = GlickoTeamRating;
    type Outcome = GameOutcome;

    fn create_rating(&self) -> Self::PlayerRating {
        GlickoRating::with_default_values()
    }

    fn create_rating_with_values(&self, mean: f64, variance: f64) -> Self::PlayerRating {
        GlickoRating::new(mean, variance.sqrt())
    }

    fn rate(
        &self,
        rating_groups: &[Self::TeamRating],
        outcome: &Self::Outcome,
    ) -> Result<Vec<Self::TeamRating>> {
        if rating_groups.len() != 2 {
            return Err(crate::error::Error::InvalidInput(
                "Glicko only supports 1v1 matches".to_string(),
            ));
        }

        if !outcome.is_valid_for_team_count(rating_groups.len()) {
            return Err(crate::error::Error::InvalidInput(
                "Invalid outcome for team count".to_string(),
            ));
        }

        let team1 = &rating_groups[0];
        let team2 = &rating_groups[1];

        if team1.player_ratings().len() != 1 || team2.player_ratings().len() != 1 {
            return Err(crate::error::Error::InvalidInput(
                "Glicko only supports 1v1 matches".to_string(),
            ));
        }

        let player1 = &team1.player_ratings()[0];
        let player2 = &team2.player_ratings()[0];

        // Determine scores based on outcome
        let (score1, score2) = match outcome.ranks() {
            [1, 2] => (1.0, 0.0), // Player 1 wins
            [2, 1] => (0.0, 1.0), // Player 2 wins
            [1, 1] => (0.5, 0.5), // Draw
            _ => {
                return Err(crate::error::Error::InvalidInput(
                    "Invalid outcome ranks".to_string(),
                ))
            }
        };

        // Update ratings
        let new_player1 = self.update_rating(player1, &[(player2.clone(), score1)]);
        let new_player2 = self.update_rating(player2, &[(player1.clone(), score2)]);

        Ok(vec![
            GlickoTeamRating::from_player_ratings(vec![new_player1]),
            GlickoTeamRating::from_player_ratings(vec![new_player2]),
        ])
    }

    fn calculate_match_quality(&self, rating_groups: &[Self::TeamRating]) -> Result<f64> {
        if rating_groups.len() != 2 {
            return Err(crate::error::Error::InvalidInput(
                "Glicko match quality only supports 2 teams".to_string(),
            ));
        }

        let team1 = &rating_groups[0];
        let team2 = &rating_groups[1];

        if team1.player_ratings().len() != 1 || team2.player_ratings().len() != 1 {
            return Err(crate::error::Error::InvalidInput(
                "Glicko match quality only supports 1v1 matches".to_string(),
            ));
        }

        let player1 = &team1.player_ratings()[0];
        let player2 = &team2.player_ratings()[0];

        // Match quality is higher when expected score is closer to 0.5
        let expected = self.expected_score(player1.mu, player2.mu, player2.rd);
        let quality = 1.0 - 2.0 * (expected - 0.5).abs();

        Ok(quality)
    }
}

/// Glicko-2 rating system configuration.
#[derive(Clone, Debug)]
pub struct Glicko2Config {
    /// System constant for volatility change (tau)
    pub tau: f64,
    /// Convergence tolerance for volatility calculation
    pub epsilon: f64,
}

impl Default for Glicko2Config {
    fn default() -> Self {
        Self {
            tau: 0.5, // Typical value for tau
            epsilon: 0.000001,
        }
    }
}

/// Glicko-2 rating system implementation.
#[derive(Clone, Debug)]
pub struct Glicko2 {
    config: Glicko2Config,
}

impl Glicko2 {
    /// Creates a new Glicko-2 rating system with default configuration.
    pub fn new() -> Self {
        Self {
            config: Glicko2Config::default(),
        }
    }

    /// Creates a new Glicko-2 rating system with custom configuration.
    pub fn with_config(config: Glicko2Config) -> Self {
        Self { config }
    }

    /// Calculates the g function for Glicko-2 algorithm.
    fn g(&self, phi: f64) -> f64 {
        1.0 / (1.0 + 3.0 * phi * phi / (PI * PI)).sqrt()
    }

    /// Calculates expected score for Glicko-2 algorithm.
    fn expected_score(&self, mu: f64, mu_j: f64, phi_j: f64) -> f64 {
        1.0 / (1.0 + (-self.g(phi_j) * (mu - mu_j)).exp())
    }

    /// Updates a single player's rating based on match results.
    fn update_rating(
        &self,
        rating: &Glicko2Rating,
        opponents: &[(Glicko2Rating, f64)],
    ) -> Glicko2Rating {
        let (mu, phi, sigma) = rating.to_glicko2_scale();

        if opponents.is_empty() {
            // No games played, just increase phi due to time passage
            let new_phi = (phi * phi + sigma * sigma).sqrt();
            return Glicko2Rating::from_glicko2_scale(mu, new_phi, sigma);
        }

        #[cfg(feature = "full-deps")]
        let (nu_inv_sum, delta_sum) = opponents
            .par_iter()
            .map(|(opponent, score)| {
                let (mu_j, phi_j, _) = opponent.to_glicko2_scale();
                let g_phi = self.g(phi_j);
                let expected = self.expected_score(mu, mu_j, phi_j);

                (
                    g_phi * g_phi * expected * (1.0 - expected),
                    g_phi * (score - expected),
                )
            })
            .reduce(|| (0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));

        #[cfg(not(feature = "full-deps"))]
        let (nu_inv_sum, delta_sum): (f64, f64) = opponents
            .iter()
            .map(|(opponent, score)| {
                let (mu_j, phi_j, _) = opponent.to_glicko2_scale();
                let g_phi = self.g(phi_j);
                let expected = self.expected_score(mu, mu_j, phi_j);

                (
                    g_phi * g_phi * expected * (1.0 - expected),
                    g_phi * (score - expected),
                )
            })
            .fold((0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));

        let nu = 1.0 / nu_inv_sum;
        let delta = nu * delta_sum;

        // Calculate new volatility using iterative method
        let new_sigma = self.calculate_new_volatility(sigma, delta, phi, nu);

        // Calculate new phi
        let phi_star = (phi * phi + new_sigma * new_sigma).sqrt();
        let new_phi = 1.0 / (1.0 / (phi_star * phi_star) + 1.0 / nu).sqrt();

        // Calculate new mu
        let new_mu = mu + new_phi * new_phi * delta_sum;

        Glicko2Rating::from_glicko2_scale(new_mu, new_phi, new_sigma)
    }

    /// Calculates new volatility using Illinois algorithm.
    fn calculate_new_volatility(&self, sigma: f64, delta: f64, phi: f64, nu: f64) -> f64 {
        let a = (sigma * sigma).ln();
        let tau = self.config.tau;

        let f = |x: f64| -> f64 {
            let ex = x.exp();
            let phi2 = phi * phi;
            let delta2 = delta * delta;

            (ex * (delta2 - phi2 - nu - ex)) / (2.0 * (phi2 + nu + ex).powi(2))
                - (x - a) / (tau * tau)
        };

        // Find bounds
        let mut k = 1.0;
        while f(a - k * tau) < 0.0 {
            k += 1.0;
        }

        let mut b = a - k * tau;
        let mut a_val = a;

        // Illinois algorithm
        for _ in 0..1000 {
            let c = a_val + (a_val - b) * f(a_val) / (f(b) - f(a_val));
            let fc = f(c);

            if fc.abs() < self.config.epsilon {
                return (c / 2.0).exp();
            }

            if f(a_val) * fc < 0.0 {
                b = a_val;
                a_val = c;
            } else {
                a_val = c;
            }
        }

        (a_val / 2.0).exp()
    }
}

impl Default for Glicko2 {
    fn default() -> Self {
        Self::new()
    }
}

impl RatingSystem for Glicko2 {
    type PlayerRating = Glicko2Rating;
    type TeamRating = Glicko2TeamRating;
    type Outcome = GameOutcome;

    fn create_rating(&self) -> Self::PlayerRating {
        Glicko2Rating::with_default_values()
    }

    fn create_rating_with_values(&self, mean: f64, variance: f64) -> Self::PlayerRating {
        Glicko2Rating::new(mean, variance.sqrt(), 0.06)
    }

    fn rate(
        &self,
        rating_groups: &[Self::TeamRating],
        outcome: &Self::Outcome,
    ) -> Result<Vec<Self::TeamRating>> {
        if rating_groups.len() != 2 {
            return Err(crate::error::Error::InvalidInput(
                "Glicko-2 only supports 1v1 matches".to_string(),
            ));
        }

        if !outcome.is_valid_for_team_count(rating_groups.len()) {
            return Err(crate::error::Error::InvalidInput(
                "Invalid outcome for team count".to_string(),
            ));
        }

        let team1 = &rating_groups[0];
        let team2 = &rating_groups[1];

        if team1.player_ratings().len() != 1 || team2.player_ratings().len() != 1 {
            return Err(crate::error::Error::InvalidInput(
                "Glicko-2 only supports 1v1 matches".to_string(),
            ));
        }

        let player1 = &team1.player_ratings()[0];
        let player2 = &team2.player_ratings()[0];

        // Determine scores based on outcome
        let (score1, score2) = match outcome.ranks() {
            [1, 2] => (1.0, 0.0), // Player 1 wins
            [2, 1] => (0.0, 1.0), // Player 2 wins
            [1, 1] => (0.5, 0.5), // Draw
            _ => {
                return Err(crate::error::Error::InvalidInput(
                    "Invalid outcome ranks".to_string(),
                ))
            }
        };

        // Update ratings
        let new_player1 = self.update_rating(player1, &[(player2.clone(), score1)]);
        let new_player2 = self.update_rating(player2, &[(player1.clone(), score2)]);

        Ok(vec![
            Glicko2TeamRating::from_player_ratings(vec![new_player1]),
            Glicko2TeamRating::from_player_ratings(vec![new_player2]),
        ])
    }

    fn calculate_match_quality(&self, rating_groups: &[Self::TeamRating]) -> Result<f64> {
        if rating_groups.len() != 2 {
            return Err(crate::error::Error::InvalidInput(
                "Glicko-2 match quality only supports 2 teams".to_string(),
            ));
        }

        let team1 = &rating_groups[0];
        let team2 = &rating_groups[1];

        if team1.player_ratings().len() != 1 || team2.player_ratings().len() != 1 {
            return Err(crate::error::Error::InvalidInput(
                "Glicko-2 match quality only supports 1v1 matches".to_string(),
            ));
        }

        let player1 = &team1.player_ratings()[0];
        let player2 = &team2.player_ratings()[0];

        let (mu1, _phi1, _) = player1.to_glicko2_scale();
        let (mu2, phi2, _) = player2.to_glicko2_scale();

        // Match quality is higher when expected score is closer to 0.5
        let expected = self.expected_score(mu1, mu2, phi2);
        let quality = 1.0 - 2.0 * (expected - 0.5).abs();

        Ok(quality)
    }
}

// Maintain Phase 6 module structure for backward compatibility

pub mod glicko2 {
    pub use super::Glicko2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glicko_rating_creation() {
        let rating = GlickoRating::with_default_values();
        assert_eq!(rating.mean(), 1500.0);
        assert_eq!(rating.standard_deviation(), 350.0);
    }

    #[test]
    fn test_glicko2_rating_creation() {
        let rating = Glicko2Rating::with_default_values();
        assert_eq!(rating.mean(), 1500.0);
        assert_eq!(rating.standard_deviation(), 350.0);
        assert_eq!(rating.volatility, 0.06);
    }

    #[test]
    fn test_glicko_match_update() {
        let glicko = Glicko::new();
        let player1 = GlickoRating::new(1500.0, 200.0);
        let player2 = GlickoRating::new(1400.0, 30.0);

        let team1 = GlickoTeamRating::from_player_ratings(vec![player1]);
        let team2 = GlickoTeamRating::from_player_ratings(vec![player2]);

        let outcome = GameOutcome::win(0, 2); // Player 1 wins

        let result = glicko.rate(&[team1, team2], &outcome).unwrap();

        // Player 1 should have increased rating
        assert!(result[0].player_ratings()[0].mean() > 1500.0);
        // Player 2 should have decreased rating
        assert!(result[1].player_ratings()[0].mean() < 1400.0);
    }

    #[test]
    fn test_glicko_match_quality() {
        let glicko = Glicko::new();
        let player1 = GlickoRating::new(1500.0, 200.0);
        let player2 = GlickoRating::new(1500.0, 200.0);

        let team1 = GlickoTeamRating::from_player_ratings(vec![player1]);
        let team2 = GlickoTeamRating::from_player_ratings(vec![player2]);

        let quality = glicko.calculate_match_quality(&[team1, team2]).unwrap();

        // Equal players should have high match quality
        assert!(quality > 0.8);
    }

    #[test]
    fn test_glicko2_match_update() {
        let glicko2 = Glicko2::new();
        let player1 = Glicko2Rating::new(1500.0, 200.0, 0.06);
        let player2 = Glicko2Rating::new(1400.0, 30.0, 0.06);

        let team1 = Glicko2TeamRating::from_player_ratings(vec![player1]);
        let team2 = Glicko2TeamRating::from_player_ratings(vec![player2]);

        let outcome = GameOutcome::win(0, 2); // Player 1 wins

        let result = glicko2.rate(&[team1, team2], &outcome).unwrap();

        // Player 1 should have increased rating
        assert!(result[0].player_ratings()[0].mean() > 1500.0);
        // Player 2 should have decreased rating
        assert!(result[1].player_ratings()[0].mean() < 1400.0);
    }

    #[test]
    fn test_rating_trait_implementation() {
        let glicko_rating = GlickoRating::new(1600.0, 100.0);
        assert_eq!(glicko_rating.mean(), 1600.0);
        assert_eq!(glicko_rating.variance(), 10000.0);
        assert_eq!(glicko_rating.standard_deviation(), 100.0);
        assert_eq!(glicko_rating.conservative_rating(), 1400.0); // 1600 - 2*100

        let glicko2_rating = Glicko2Rating::new(1600.0, 100.0, 0.05);
        assert_eq!(glicko2_rating.mean(), 1600.0);
        assert_eq!(glicko2_rating.variance(), 10000.0);
        assert_eq!(glicko2_rating.standard_deviation(), 100.0);
        assert_eq!(glicko2_rating.conservative_rating(), 1400.0); // 1600 - 2*100
    }
}
