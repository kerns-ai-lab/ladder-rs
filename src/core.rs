use crate::error::Result;

/// Trait for individual player skill representation.
pub trait Rating: Clone + std::fmt::Debug {
    /// Returns the mean (μ) of the player's skill distribution.
    fn mean(&self) -> f64;

    /// Returns the variance (σ²) of the player's skill distribution.
    fn variance(&self) -> f64;

    /// Returns the standard deviation (σ) of the player's skill distribution.
    fn standard_deviation(&self) -> f64 {
        self.variance().sqrt()
    }

    /// Returns a conservative skill estimate suitable for leaderboard display.
    /// Default implementation returns μ - 3σ.
    fn conservative_rating(&self) -> f64 {
        self.mean() - 3.0 * self.standard_deviation()
    }
}

/// Trait for team-specific rating properties.
pub trait TeamRating: Clone + std::fmt::Debug {
    /// The type of individual player ratings in this team.
    type PlayerRating: Rating;

    /// Returns a list of player ratings in this team.
    fn player_ratings(&self) -> &[Self::PlayerRating];

    /// Creates a new team rating from a list of player ratings.
    fn from_player_ratings(ratings: Vec<Self::PlayerRating>) -> Self;
}

/// Trait to represent game outcomes.
pub trait Outcome: Clone + std::fmt::Debug {
    /// Validates whether the outcome is valid for the given number of teams.
    fn is_valid_for_team_count(&self, team_count: usize) -> bool;
}

/// Trait defining common operations for all rating systems.
pub trait RatingSystem {
    /// The type representing individual player ratings.
    type PlayerRating: Rating;

    /// The type representing team ratings.
    type TeamRating: TeamRating<PlayerRating = Self::PlayerRating>;

    /// The type representing game outcomes.
    type Outcome: Outcome;

    /// Creates a new rating with default values.
    fn create_rating(&self) -> Self::PlayerRating;

    /// Creates a new rating with the specified mean and variance.
    fn create_rating_with_values(&self, mean: f64, variance: f64) -> Self::PlayerRating;

    /// Updates ratings based on a match outcome.
    ///
    /// # Arguments
    /// * `rating_groups` - List of team ratings, where each team is a list of player ratings.
    /// * `outcome` - The game outcome (typically ranks).
    ///
    /// # Returns
    /// Updated ratings for all players.
    fn rate(
        &self,
        rating_groups: &[Self::TeamRating],
        outcome: &Self::Outcome,
    ) -> Result<Vec<Self::TeamRating>>;

    /// Calculates the quality of a match between two teams.
    ///
    /// # Arguments
    /// * `rating_groups` - List of team ratings to evaluate.
    ///
    /// # Returns
    /// A value between 0 and 1 representing match quality (higher is better).
    fn calculate_match_quality(&self, rating_groups: &[Self::TeamRating]) -> Result<f64>;
}

/// Standard implementation of game outcomes as ranks.
/// Ranks are represented as a vector of usize, where each element is a rank
/// (1st, 2nd, etc.), and the index corresponds to the team's index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameOutcome {
    ranks: Vec<usize>,
}

impl GameOutcome {
    /// Creates a new game outcome from a list of ranks.
    pub fn new(ranks: Vec<usize>) -> Self {
        Self { ranks }
    }

    /// Returns the ranks of teams.
    pub fn ranks(&self) -> &[usize] {
        &self.ranks
    }

    /// Creates an outcome where the team at index `winner_idx` wins.
    pub fn win(winner_idx: usize, team_count: usize) -> Self {
        assert!(
            winner_idx < team_count,
            "winner_idx ({}) must be less than team_count ({})",
            winner_idx,
            team_count
        );
        let mut ranks = vec![2; team_count];
        ranks[winner_idx] = 1;
        Self { ranks }
    }

    /// Creates an outcome representing a draw between all teams.
    pub fn draw(team_count: usize) -> Self {
        Self {
            ranks: vec![1; team_count],
        }
    }
}

impl Outcome for GameOutcome {
    fn is_valid_for_team_count(&self, team_count: usize) -> bool {
        self.ranks.len() == team_count && !self.ranks.is_empty()
    }
}
