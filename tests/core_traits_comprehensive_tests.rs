/// Comprehensive tests for Phase 1 core abstractions
/// Tests the fundamental traits and data structures that form the foundation of the library

use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating, Outcome},
    error::{Error, Result},
};

/// Mock rating implementation for testing core traits
#[derive(Clone, Debug, PartialEq)]
pub struct MockRating {
    mean: f64,
    variance: f64,
}

impl MockRating {
    pub fn new(mean: f64, variance: f64) -> Self {
        Self { mean, variance }
    }
}

impl Rating for MockRating {
    fn mean(&self) -> f64 {
        self.mean
    }

    fn variance(&self) -> f64 {
        self.variance
    }

    fn standard_deviation(&self) -> f64 {
        self.variance.sqrt()
    }

    fn conservative_rating(&self) -> f64 {
        self.mean - 3.0 * self.standard_deviation()
    }
}

/// Mock team rating implementation for testing core traits
#[derive(Clone, Debug, PartialEq)]
pub struct MockTeamRating {
    players: Vec<MockRating>,
}

impl MockTeamRating {
    pub fn new(players: Vec<MockRating>) -> Self {
        Self { players }
    }
}

impl TeamRating for MockTeamRating {
    type PlayerRating = MockRating;

    fn player_ratings(&self) -> &[Self::PlayerRating] {
        &self.players
    }

    fn from_player_ratings(ratings: Vec<Self::PlayerRating>) -> Self {
        Self { players: ratings }
    }
}

/// Mock rating system implementation for testing core traits
#[derive(Clone, Debug)]
pub struct MockRatingSystem {
    default_mean: f64,
    default_variance: f64,
}

impl MockRatingSystem {
    pub fn new(default_mean: f64, default_variance: f64) -> Self {
        Self {
            default_mean,
            default_variance,
        }
    }
}

impl RatingSystem for MockRatingSystem {
    type PlayerRating = MockRating;
    type TeamRating = MockTeamRating;
    type Outcome = GameOutcome;

    fn create_rating(&self) -> Self::PlayerRating {
        MockRating::new(self.default_mean, self.default_variance)
    }

    fn create_rating_with_values(&self, mean: f64, variance: f64) -> Self::PlayerRating {
        MockRating::new(mean, variance)
    }

    fn rate(
        &self,
        _rating_groups: &[Self::TeamRating],
        _outcome: &Self::Outcome,
    ) -> Result<Vec<Self::TeamRating>> {
        // Mock implementation - just return the input unchanged
        Ok(_rating_groups.to_vec())
    }

    fn calculate_match_quality(&self, _rating_groups: &[Self::TeamRating]) -> Result<f64> {
        // Mock implementation - return 0.5 for balanced match
        Ok(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rating_trait_basic_functionality() {
        let rating = MockRating::new(100.0, 25.0);
        
        assert_eq!(rating.mean(), 100.0);
        assert_eq!(rating.variance(), 25.0);
        assert_eq!(rating.standard_deviation(), 5.0);
        assert_eq!(rating.conservative_rating(), 85.0); // 100 - 3*5
    }

    #[test]
    fn test_rating_trait_edge_cases() {
        // Test with zero variance
        let zero_var_rating = MockRating::new(50.0, 0.0);
        assert_eq!(zero_var_rating.variance(), 0.0);
        assert_eq!(zero_var_rating.standard_deviation(), 0.0);
        assert_eq!(zero_var_rating.conservative_rating(), 50.0);

        // Test with very high variance
        let high_var_rating = MockRating::new(100.0, 10000.0);
        assert_eq!(high_var_rating.variance(), 10000.0);
        assert_eq!(high_var_rating.standard_deviation(), 100.0);
        assert_eq!(high_var_rating.conservative_rating(), -200.0);

        // Test with negative mean
        let negative_mean_rating = MockRating::new(-50.0, 16.0);
        assert_eq!(negative_mean_rating.mean(), -50.0);
        assert_eq!(negative_mean_rating.conservative_rating(), -62.0); // -50 - 3*4
    }

    #[test]
    fn test_team_rating_trait_functionality() {
        let player1 = MockRating::new(100.0, 25.0);
        let player2 = MockRating::new(120.0, 36.0);
        let player3 = MockRating::new(80.0, 16.0);
        
        let players = vec![player1.clone(), player2.clone(), player3.clone()];
        let team = MockTeamRating::from_player_ratings(players.clone());
        
        assert_eq!(team.player_ratings().len(), 3);
        assert_eq!(team.player_ratings()[0], player1);
        assert_eq!(team.player_ratings()[1], player2);
        assert_eq!(team.player_ratings()[2], player3);
        
        // Test accessing individual player ratings
        let ratings = team.player_ratings();
        assert_eq!(ratings[0].mean(), 100.0);
        assert_eq!(ratings[1].mean(), 120.0);
        assert_eq!(ratings[2].mean(), 80.0);
    }

    #[test]
    fn test_team_rating_empty_team() {
        let empty_team = MockTeamRating::from_player_ratings(vec![]);
        assert_eq!(empty_team.player_ratings().len(), 0);
    }

    #[test]
    fn test_team_rating_single_player() {
        let player = MockRating::new(150.0, 49.0);
        let team = MockTeamRating::from_player_ratings(vec![player.clone()]);
        
        assert_eq!(team.player_ratings().len(), 1);
        assert_eq!(team.player_ratings()[0], player);
    }

    #[test]
    fn test_rating_system_trait_functionality() {
        let system = MockRatingSystem::new(25.0, 64.0);
        
        // Test create_rating
        let default_rating = system.create_rating();
        assert_eq!(default_rating.mean(), 25.0);
        assert_eq!(default_rating.variance(), 64.0);
        
        // Test create_rating_with_values
        let custom_rating = system.create_rating_with_values(100.0, 225.0);
        assert_eq!(custom_rating.mean(), 100.0);
        assert_eq!(custom_rating.variance(), 225.0);
        
        // Test match quality calculation
        let team1 = MockTeamRating::from_player_ratings(vec![default_rating.clone()]);
        let team2 = MockTeamRating::from_player_ratings(vec![custom_rating]);
        
        let quality = system.calculate_match_quality(&[team1, team2]).unwrap();
        assert_eq!(quality, 0.5);
    }

    #[test]
    fn test_rating_system_rate_functionality() {
        let system = MockRatingSystem::new(25.0, 64.0);
        let rating1 = system.create_rating();
        let rating2 = system.create_rating_with_values(30.0, 49.0);
        
        let team1 = MockTeamRating::from_player_ratings(vec![rating1.clone()]);
        let team2 = MockTeamRating::from_player_ratings(vec![rating2.clone()]);
        
        let outcome = GameOutcome::win(0, 2);
        let updated_teams = system.rate(&[team1.clone(), team2.clone()], &outcome).unwrap();
        
        // Mock implementation returns unchanged ratings
        assert_eq!(updated_teams.len(), 2);
        assert_eq!(updated_teams[0].player_ratings()[0], rating1);
        assert_eq!(updated_teams[1].player_ratings()[0], rating2);
    }

    #[test]
    fn test_game_outcome_creation_and_validation() {
        // Test basic creation
        let outcome = GameOutcome::new(vec![1, 2, 3]);
        assert_eq!(outcome.ranks(), &[1, 2, 3]);
        assert!(outcome.is_valid_for_team_count(3));
        assert!(!outcome.is_valid_for_team_count(2));
        assert!(!outcome.is_valid_for_team_count(4));
        
        // Test win scenario
        let win_outcome = GameOutcome::win(0, 3);
        assert_eq!(win_outcome.ranks(), &[1, 2, 2]);
        assert!(win_outcome.is_valid_for_team_count(3));
        
        // Test different winner indices
        let win_outcome_2 = GameOutcome::win(1, 3);
        assert_eq!(win_outcome_2.ranks(), &[2, 1, 2]);
        
        let win_outcome_3 = GameOutcome::win(2, 3);
        assert_eq!(win_outcome_3.ranks(), &[2, 2, 1]);
    }

    #[test]
    fn test_game_outcome_draw_scenarios() {
        // Test draw with multiple teams
        let draw_outcome = GameOutcome::draw(4);
        assert_eq!(draw_outcome.ranks(), &[1, 1, 1, 1]);
        assert!(draw_outcome.is_valid_for_team_count(4));
        
        // Test single team "draw"
        let single_draw = GameOutcome::draw(1);
        assert_eq!(single_draw.ranks(), &[1]);
        assert!(single_draw.is_valid_for_team_count(1));
        
        // Test two team draw
        let two_team_draw = GameOutcome::draw(2);
        assert_eq!(two_team_draw.ranks(), &[1, 1]);
        assert!(two_team_draw.is_valid_for_team_count(2));
    }

    #[test]
    fn test_game_outcome_edge_cases() {
        // Test empty outcome
        let empty_outcome = GameOutcome::new(vec![]);
        assert_eq!(empty_outcome.ranks().len(), 0);
        assert!(!empty_outcome.is_valid_for_team_count(0));
        assert!(!empty_outcome.is_valid_for_team_count(1));
        
        // Test complex ranking scenarios
        let complex_outcome = GameOutcome::new(vec![1, 1, 3, 3, 5]);
        assert_eq!(complex_outcome.ranks(), &[1, 1, 3, 3, 5]);
        assert!(complex_outcome.is_valid_for_team_count(5));
        assert!(!complex_outcome.is_valid_for_team_count(4));
    }

    #[test]
    fn test_outcome_trait_implementation() {
        let outcome = GameOutcome::new(vec![1, 2]);
        
        // Test that GameOutcome properly implements Outcome trait
        assert!(outcome.is_valid_for_team_count(2));
        assert!(!outcome.is_valid_for_team_count(3));
        assert!(!outcome.is_valid_for_team_count(1));
        assert!(!outcome.is_valid_for_team_count(0));
    }

    #[test]
    fn test_trait_clone_and_debug() {
        let rating = MockRating::new(100.0, 25.0);
        let cloned_rating = rating.clone();
        assert_eq!(rating, cloned_rating);
        
        let team = MockTeamRating::from_player_ratings(vec![rating.clone()]);
        let cloned_team = team.clone();
        assert_eq!(team, cloned_team);
        
        let outcome = GameOutcome::new(vec![1, 2]);
        let cloned_outcome = outcome.clone();
        assert_eq!(outcome, cloned_outcome);
        
        // Test debug formatting (just ensure it doesn't panic)
        let _debug_rating = format!("{:?}", rating);
        let _debug_team = format!("{:?}", team);
        let _debug_outcome = format!("{:?}", outcome);
    }

    #[test]
    fn test_large_team_scenarios() {
        // Test with many players per team
        let players: Vec<MockRating> = (0..100)
            .map(|i| MockRating::new(1000.0 + i as f64, 100.0))
            .collect();
        
        let large_team = MockTeamRating::from_player_ratings(players.clone());
        assert_eq!(large_team.player_ratings().len(), 100);
        
        // Verify player ratings are preserved correctly
        for (i, player) in large_team.player_ratings().iter().enumerate() {
            assert_eq!(player.mean(), 1000.0 + i as f64);
            assert_eq!(player.variance(), 100.0);
        }
        
        // Test match with many teams
        let many_teams: Vec<MockTeamRating> = (0..50)
            .map(|i| {
                let player = MockRating::new(1500.0 + i as f64, 64.0);
                MockTeamRating::from_player_ratings(vec![player])
            })
            .collect();
        
        let system = MockRatingSystem::new(25.0, 64.0);
        let quality = system.calculate_match_quality(&many_teams).unwrap();
        assert_eq!(quality, 0.5); // Mock implementation returns 0.5
        
        // Test complex outcome with many teams
        let ranks: Vec<usize> = (1..=50).collect();
        let complex_outcome = GameOutcome::new(ranks);
        assert!(complex_outcome.is_valid_for_team_count(50));
        assert!(!complex_outcome.is_valid_for_team_count(49));
    }
}