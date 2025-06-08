use crate::core::{GameOutcome, Outcome, Rating, RatingSystem, TeamRating};
use crate::error::{Error, Result};
use std::f64::consts::PI;

/// Elo rating representation - single numerical value.
#[derive(Clone, Debug, PartialEq)]
pub struct EloRating {
    /// The Elo rating value (s)
    rating: f64,
}

impl EloRating {
    /// Creates a new Elo rating with the specified value.
    pub fn new(rating: f64) -> Self {
        Self { rating }
    }

    /// Returns the Elo rating value.
    pub fn rating(&self) -> f64 {
        self.rating
    }
}

impl Rating for EloRating {
    fn mean(&self) -> f64 {
        self.rating
    }

    fn variance(&self) -> f64 {
        // Elo doesn't have an explicit variance, but we can return 0
        // since it's a point estimate
        0.0
    }

    fn conservative_rating(&self) -> f64 {
        // For Elo, conservative rating is just the rating itself
        // since there's no uncertainty measure
        self.rating
    }
}

/// Team rating wrapper for Elo (for trait compatibility).
#[derive(Clone, Debug)]
pub struct EloTeamRating {
    players: Vec<EloRating>,
}

impl EloTeamRating {
    /// Creates a new team rating with a single player.
    pub fn new(player_rating: EloRating) -> Self {
        Self {
            players: vec![player_rating],
        }
    }
}

impl TeamRating for EloTeamRating {
    type PlayerRating = EloRating;

    fn player_ratings(&self) -> &[Self::PlayerRating] {
        &self.players
    }

    fn from_player_ratings(ratings: Vec<Self::PlayerRating>) -> Self {
        Self { players: ratings }
    }
}

/// Elo rating system with configurable parameters.
pub struct EloSystem {
    /// K-factor: scaling constant (e.g., 10-30)
    k_factor: f64,
    /// Alpha: learning rate (0 < α < 1, e.g., 0.05-0.1)
    alpha: f64,
    /// Performance variance for Elo context
    beta_elo: f64,
    /// Default starting rating
    default_rating: f64,
}

impl EloSystem {
    /// Creates a new Elo system with default parameters.
    pub fn new() -> Self {
        Self {
            k_factor: 20.0,
            alpha: 0.1,
            beta_elo: 200.0,
            default_rating: 1500.0,
        }
    }

    /// Creates a new Elo system with custom parameters.
    pub fn with_parameters(k_factor: f64, alpha: f64, beta_elo: f64, default_rating: f64) -> Self {
        Self {
            k_factor,
            alpha,
            beta_elo,
            default_rating,
        }
    }

    /// Calculates the win probability for player 1 against player 2.
    /// P(player 1 wins) = Φ((s1 - s2) / (√2 * β_elo))
    fn win_probability(&self, s1: f64, s2: f64) -> f64 {
        let diff = s1 - s2;
        let denominator = (2.0_f64).sqrt() * self.beta_elo;
        gaussian_cdf(diff / denominator)
    }

    /// Calculates the rating update magnitude Δ.
    /// Δ = (α * β_elo * √π / K-Factor) * ((y+1)/2 - Φ((s1-s2)/(√2*β_elo)))
    fn calculate_delta(&self, s1: f64, s2: f64, outcome: f64) -> f64 {
        let win_prob = self.win_probability(s1, s2);
        let expected_score = (outcome + 1.0) / 2.0;
        let multiplier = self.alpha * self.beta_elo * PI.sqrt() / self.k_factor;
        multiplier * (expected_score - win_prob)
    }
}

impl Default for EloSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RatingSystem for EloSystem {
    type PlayerRating = EloRating;
    type TeamRating = EloTeamRating;
    type Outcome = GameOutcome;

    fn create_rating(&self) -> Self::PlayerRating {
        EloRating::new(self.default_rating)
    }

    fn create_rating_with_values(&self, mean: f64, _variance: f64) -> Self::PlayerRating {
        // Elo doesn't use variance, so we just use the mean as the rating
        EloRating::new(mean)
    }

    fn rate(
        &self,
        rating_groups: &[Self::TeamRating],
        outcome: &Self::Outcome,
    ) -> Result<Vec<Self::TeamRating>> {
        // Elo focuses on 1v1 matches
        if rating_groups.len() != 2 {
            return Err(Error::InvalidInput(
                "Elo rating system only supports 1v1 matches (2 teams)".to_string(),
            ));
        }

        if !outcome.is_valid_for_team_count(2) {
            return Err(Error::InvalidInput(
                "Invalid outcome for 2 teams".to_string(),
            ));
        }

        // Each team should have exactly one player for Elo
        if rating_groups[0].player_ratings().len() != 1
            || rating_groups[1].player_ratings().len() != 1
        {
            return Err(Error::InvalidInput(
                "Each team must have exactly one player for Elo".to_string(),
            ));
        }

        let player1_rating = rating_groups[0].player_ratings()[0].rating();
        let player2_rating = rating_groups[1].player_ratings()[0].rating();

        let ranks = outcome.ranks();

        // Determine the outcome: y = +1 if player 1 wins, y = -1 if player 2 wins, y = 0 for draw
        let y = if ranks[0] < ranks[1] {
            1.0 // Player 1 wins
        } else if ranks[0] > ranks[1] {
            -1.0 // Player 2 wins
        } else {
            0.0 // Draw
        };

        // Calculate rating updates
        let delta = self.calculate_delta(player1_rating, player2_rating, y);

        let new_player1_rating = player1_rating + delta;
        let new_player2_rating = player2_rating - delta;

        let updated_team1 = EloTeamRating::new(EloRating::new(new_player1_rating));
        let updated_team2 = EloTeamRating::new(EloRating::new(new_player2_rating));

        Ok(vec![updated_team1, updated_team2])
    }

    fn calculate_match_quality(&self, rating_groups: &[Self::TeamRating]) -> Result<f64> {
        if rating_groups.len() != 2 {
            return Err(Error::InvalidInput(
                "Match quality calculation only supports 2 teams".to_string(),
            ));
        }

        if rating_groups[0].player_ratings().len() != 1
            || rating_groups[1].player_ratings().len() != 1
        {
            return Err(Error::InvalidInput(
                "Each team must have exactly one player".to_string(),
            ));
        }

        let s1 = rating_groups[0].player_ratings()[0].rating();
        let s2 = rating_groups[1].player_ratings()[0].rating();

        let win_prob = self.win_probability(s1, s2);

        // Match quality is higher when win probability is closer to 0.5 (more balanced)
        // We use 1 - 2 * |0.5 - win_prob| to get a value between 0 and 1
        let quality = 1.0 - 2.0 * (0.5 - win_prob).abs();

        Ok(quality)
    }
}

/// Gaussian cumulative distribution function approximation.
/// Uses the error function approximation.
fn gaussian_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / (2.0_f64).sqrt()))
}

/// Error function approximation using Abramowitz and Stegun formula.
fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elo_rating_creation() {
        let rating = EloRating::new(1500.0);
        assert_eq!(rating.rating(), 1500.0);
        assert_eq!(rating.mean(), 1500.0);
        assert_eq!(rating.variance(), 0.0);
        assert_eq!(rating.conservative_rating(), 1500.0);
    }

    #[test]
    fn test_elo_system_creation() {
        let system = EloSystem::new();
        let rating = system.create_rating();
        assert_eq!(rating.rating(), 1500.0);
    }

    #[test]
    fn test_elo_rating_update_win() {
        let system = EloSystem::new();
        let team1 = EloTeamRating::new(EloRating::new(1500.0));
        let team2 = EloTeamRating::new(EloRating::new(1500.0));
        let outcome = GameOutcome::win(0, 2); // Team 1 wins

        let result = system.rate(&[team1, team2], &outcome).unwrap();

        // Winner should gain rating, loser should lose rating
        assert!(result[0].player_ratings()[0].rating() > 1500.0);
        assert!(result[1].player_ratings()[0].rating() < 1500.0);
    }

    #[test]
    fn test_elo_rating_update_draw() {
        let system = EloSystem::new();
        let team1 = EloTeamRating::new(EloRating::new(1500.0));
        let team2 = EloTeamRating::new(EloRating::new(1500.0));
        let outcome = GameOutcome::draw(2); // Draw

        let result = system.rate(&[team1, team2], &outcome).unwrap();

        // In a draw between equal players, ratings should remain the same
        assert!((result[0].player_ratings()[0].rating() - 1500.0).abs() < 0.001);
        assert!((result[1].player_ratings()[0].rating() - 1500.0).abs() < 0.001);
    }

    #[test]
    fn test_match_quality() {
        let system = EloSystem::new();
        let team1 = EloTeamRating::new(EloRating::new(1500.0));
        let team2 = EloTeamRating::new(EloRating::new(1500.0));

        let quality = system.calculate_match_quality(&[team1, team2]).unwrap();

        // Equal players should have high match quality (close to 1.0)
        assert!(quality > 0.9);
        assert!(quality <= 1.0);
    }

    #[test]
    fn test_win_probability() {
        let system = EloSystem::new();

        // Equal players should have ~0.5 win probability
        let prob = system.win_probability(1500.0, 1500.0);
        assert!((prob - 0.5).abs() < 0.01);

        // Higher rated player should have > 0.5 win probability
        let prob = system.win_probability(1600.0, 1500.0);
        assert!(prob > 0.5);

        // Lower rated player should have < 0.5 win probability
        let prob = system.win_probability(1400.0, 1500.0);
        assert!(prob < 0.5);
    }

    #[test]
    fn test_invalid_team_count() {
        let system = EloSystem::new();
        let team1 = EloTeamRating::new(EloRating::new(1500.0));
        let outcome = GameOutcome::new(vec![1]);

        let result = system.rate(&[team1], &outcome);
        assert!(result.is_err());
    }
}
