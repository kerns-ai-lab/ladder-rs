//! Utility interfaces for JavaScript

use crate::js_interface::core::*;
use wasm_bindgen::prelude::*;
use js_sys::*;

/// Utility functions for ratings and teams
#[wasm_bindgen(js_name = "Utils")]
pub struct JsUtilsInterface;

#[wasm_bindgen(js_class = "Utils")]
impl JsUtilsInterface {
    /// Compare two ratings (-1, 0, 1)
    #[wasm_bindgen(js_name = "compareRatings")]
    pub fn compare_ratings(rating1: &JsRatingInterface, rating2: &JsRatingInterface) -> i32 {
        rating1.compare_to(rating2)
    }
    
    /// Balance players into teams
    #[wasm_bindgen(js_name = "balanceTeams")]
    pub fn balance_teams(players: Vec<JsRatingInterface>, team_count: usize) -> Array {
        let teams = Array::new();
        
        // Create initial empty teams
        let mut team_objects = Vec::new();
        for _ in 0..team_count {
            let team = JsTeamInterface::new();
            team_objects.push(team);
        }
        
        // Simple round-robin assignment
        for (i, player) in players.into_iter().enumerate() {
            let team_index = i % team_count;
            if let Some(team) = team_objects.get_mut(team_index) {
                team.add_player(player);
            }
        }
        
        // Convert to JS Array
        for team in team_objects {
            teams.push(&JsValue::from(team));
        }
        
        teams
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rating_comparison() {
        let rating1 = JsRatingInterface::new(1500.0, 200.0).unwrap();
        let rating2 = JsRatingInterface::new(1600.0, 180.0).unwrap();
        
        let comparison = JsUtilsInterface::compare_ratings(&rating1, &rating2);
        assert_eq!(comparison, -1);
    }
}