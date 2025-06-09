//! Async operations interface for JavaScript
//!
//! Provides Promise-based async operations for rating calculations.

use crate::js_interface::core::*;
use crate::js_interface::systems::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use js_sys::*;

/// Async extension for rating system interface
#[wasm_bindgen]
impl JsRatingSystemInterface {
    /// Update ratings asynchronously (returns Promise)
    #[wasm_bindgen(js_name = "updateRatingsAsync")]
    pub fn update_ratings_async(
        &self,
        teams: Vec<JsTeamInterface>,
        outcome: JsGameOutcomeInterface,
    ) -> Promise {
        let system_type = self.get_system_type();
        
        future_to_promise(async move {
            // Simulate async processing
            let result = Self::process_async_rating_update(system_type, teams, outcome).await;
            result.map(|array| JsValue::from(array))
                .map_err(|e| JsValue::from(e))
        })
    }
    
    /// Process batch of matches asynchronously
    #[wasm_bindgen(js_name = "processBatchAsync")]
    pub fn process_batch_async(&self, batch: JsMatchBatchInterface) -> Promise {
        let system_type = self.get_system_type();
        
        future_to_promise(async move {
            let results = Array::new();
            
            for i in 0..batch.size() {
                if let (Some(teams), Some(outcome)) = (batch.get_teams(i), batch.get_outcome(i)) {
                    let match_result = Self::process_async_rating_update(
                        system_type.clone(), 
                        teams, 
                        outcome
                    ).await?;
                    
                    results.push(&JsValue::from(match_result));
                }
            }
            
            Ok(JsValue::from(results))
        })
    }
    
    /// Calculate match quality asynchronously
    #[wasm_bindgen(js_name = "calculateMatchQualityAsync")]
    pub fn calculate_match_quality_async(&self, teams: Vec<JsTeamInterface>) -> Promise {
        let quality = self.calculate_match_quality(teams);
        
        future_to_promise(async move {
            // Simulate async processing delay
            Ok(JsValue::from_f64(quality))
        })
    }
}

impl JsRatingSystemInterface {
    /// Internal async processing method
    async fn process_async_rating_update(
        system_type: String,
        teams: Vec<JsTeamInterface>,
        outcome: JsGameOutcomeInterface,
    ) -> Result<Array, JsValue> {
        // Simulate processing time
        let _ = js_sys::Promise::resolve(&JsValue::from(0));
        
        let updated_teams = Array::new();
        
        for (i, team) in teams.iter().enumerate() {
            let rank = outcome.get_rank(i);
            let mut updated_team = JsTeamInterface::new();
            
            // Algorithm-specific rating adjustments
            let (rating_change, variance_change) = match system_type.as_str() {
                "elo" => Self::calculate_elo_changes(rank),
                "glicko" => Self::calculate_glicko_changes(rank),
                "trueskill" => Self::calculate_trueskill_changes(rank),
                _ => (0.0, 0.0),
            };
            
            for j in 0..team.size() {
                if let Some(player) = team.get_player(j) {
                    let updated_player = player
                        .adjust_mean(rating_change)
                        .adjust_variance(variance_change);
                    
                    updated_team.add_player(updated_player);
                }
            }
            
            updated_teams.push(&JsValue::from(updated_team));
        }
        
        Ok(updated_teams)
    }
    
    fn calculate_elo_changes(rank: u32) -> (f64, f64) {
        let rating_change = if rank == 1 { 25.0 } else { -25.0 };
        let variance_change = -5.0; // Reduce uncertainty
        (rating_change, variance_change)
    }
    
    fn calculate_glicko_changes(rank: u32) -> (f64, f64) {
        let rating_change = if rank == 1 { 30.0 } else { -30.0 };
        let variance_change = -1000.0; // Reduce deviation squared
        (rating_change, variance_change)
    }
    
    fn calculate_trueskill_changes(rank: u32) -> (f64, f64) {
        let rating_change = if rank == 1 { 2.0 } else { -2.0 };
        let variance_change = -2.0; // Reduce variance
        (rating_change, variance_change)
    }
}

/// Batch processing interface for multiple matches
#[wasm_bindgen(js_name = "MatchBatch")]
pub struct JsMatchBatchInterface {
    teams: Vec<Vec<JsTeamInterface>>,
    outcomes: Vec<JsGameOutcomeInterface>,
}

#[wasm_bindgen(js_class = "MatchBatch")]
impl JsMatchBatchInterface {
    /// Creates a new match batch
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            teams: Vec::new(),
            outcomes: Vec::new(),
        }
    }
    
    /// Adds a match to the batch
    #[wasm_bindgen(js_name = "addMatch")]
    pub fn add_match(&mut self, teams: Vec<JsTeamInterface>, outcome: JsGameOutcomeInterface) {
        self.teams.push(teams);
        self.outcomes.push(outcome);
    }
    
    /// Gets the number of matches in the batch
    pub fn size(&self) -> usize {
        self.teams.len()
    }
    
    /// Gets teams for a match by index
    #[wasm_bindgen(js_name = "getTeams")]
    pub fn get_teams(&self, index: usize) -> Option<Vec<JsTeamInterface>> {
        self.teams.get(index).cloned()
    }
    
    /// Gets outcome for a match by index
    #[wasm_bindgen(js_name = "getOutcome")]
    pub fn get_outcome(&self, index: usize) -> Option<JsGameOutcomeInterface> {
        self.outcomes.get(index).cloned()
    }
    
    /// Clear all matches
    pub fn clear(&mut self) {
        self.teams.clear();
        self.outcomes.clear();
    }
}

/// Promise utilities for JavaScript integration
#[wasm_bindgen(js_name = "PromiseUtils")]
pub struct JsPromiseUtils;

#[wasm_bindgen(js_class = "PromiseUtils")]
impl JsPromiseUtils {
    /// Create a resolved promise with a value
    #[wasm_bindgen(js_name = "resolve")]
    pub fn resolve(value: JsValue) -> Promise {
        Promise::resolve(&value)
    }
    
    /// Create a rejected promise with an error
    #[wasm_bindgen(js_name = "reject")]
    pub fn reject(error: JsValue) -> Promise {
        Promise::reject(&error)
    }
    
    /// Create a promise that resolves after a delay
    #[wasm_bindgen(js_name = "delay")]
    pub fn delay(_ms: u32) -> Promise {
        future_to_promise(async move {
            // In a real implementation, this would use setTimeout
            Ok(JsValue::undefined())
        })
    }
    
    /// Combine multiple promises into one (Promise.all equivalent)
    #[wasm_bindgen(js_name = "all")]
    pub fn all(promises: Array) -> Promise {
        Promise::all(&promises)
    }
}

/// Async iterator interface for large datasets
#[wasm_bindgen(js_name = "AsyncRatingIterator")]
pub struct JsAsyncRatingIterator {
    ratings: Vec<JsRatingInterface>,
    current_index: usize,
    batch_size: usize,
}

#[wasm_bindgen(js_class = "AsyncRatingIterator")]
impl JsAsyncRatingIterator {
    /// Creates a new async iterator
    #[wasm_bindgen(constructor)]
    pub fn new(ratings: Vec<JsRatingInterface>, batch_size: usize) -> Self {
        Self {
            ratings,
            current_index: 0,
            batch_size,
        }
    }
    
    /// Gets the next batch of ratings asynchronously
    #[wasm_bindgen(js_name = "nextBatch")]
    pub fn next_batch(&mut self) -> Promise {
        let start = self.current_index;
        let end = (start + self.batch_size).min(self.ratings.len());
        
        if start >= self.ratings.len() {
            return Promise::resolve(&JsValue::null());
        }
        
        let batch = Array::new();
        for i in start..end {
            if let Some(rating) = self.ratings.get(i) {
                let js_rating = JsRatingInterface::new(rating.mean(), rating.variance()).unwrap();
                batch.push(&JsValue::from(js_rating));
            }
        }
        
        self.current_index = end;
        
        future_to_promise(async move {
            Ok(JsValue::from(batch))
        })
    }
    
    /// Check if there are more items
    #[wasm_bindgen(js_name = "hasMore")]
    pub fn has_more(&self) -> bool {
        self.current_index < self.ratings.len()
    }
    
    /// Reset iterator to beginning
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_match_batch_creation() {
        let mut batch = JsMatchBatchInterface::new();
        assert_eq!(batch.size(), 0);
        
        let team1 = JsTeamInterface::new();
        let team2 = JsTeamInterface::new();
        let outcome = JsGameOutcomeInterface::create_win(0, 2).unwrap();
        
        batch.add_match(vec![team1, team2], outcome);
        assert_eq!(batch.size(), 1);
    }
    
    #[test]
    fn test_async_iterator() {
        let ratings = vec![
            JsRatingInterface::new(1500.0, 200.0).unwrap(),
            JsRatingInterface::new(1600.0, 180.0).unwrap(),
        ];
        
        let mut iterator = JsAsyncRatingIterator::new(ratings, 1);
        assert!(iterator.has_more());
        
        iterator.reset();
        assert_eq!(iterator.current_index, 0);
    }
}