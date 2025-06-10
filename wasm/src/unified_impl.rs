//! Implementation details for the unified rating system

use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem as CoreRatingSystem, TeamRating},
    elo::{EloRating, EloSystem, EloTeamRating},
    glicko::{GlickoRating, Glicko, GlickoTeamRating},
    trueskill::{TrueSkillRating, TrueSkill, TrueSkillTeamRating},
};

use crate::unified::{UnifiedRatingSystem, PlayerInfo, RatingStorage};
use crate::errors::{to_js_result};

impl UnifiedRatingSystem {
    /// Process a match using the Elo rating system
    pub(crate) fn process_elo_match(
        &mut self,
        team1_players: &[String],
        team2_players: &[String],
        winner_team: u32
    ) -> Result<Vec<PlayerInfo>, JsValue> {
        let system = self.elo_system.as_ref().unwrap();
        
        // Build teams
        let team1_ratings: Vec<EloRating> = team1_players.iter()
            .map(|id| match &self.players[id].0 {
                RatingStorage::Elo(r) => r.clone(),
                _ => unreachable!()
            })
            .collect();
        
        let team2_ratings: Vec<EloRating> = team2_players.iter()
            .map(|id| match &self.players[id].0 {
                RatingStorage::Elo(r) => r.clone(),
                _ => unreachable!()
            })
            .collect();
        
        let team1 = EloTeamRating::from_player_ratings(team1_ratings);
        let team2 = EloTeamRating::from_player_ratings(team2_ratings);
        
        // Create outcome
        let outcome = if winner_team == 1 {
            GameOutcome::new(vec![1, 2])
        } else {
            GameOutcome::new(vec![2, 1])
        };
        
        // Rate match
        let updated_teams = to_js_result(system.rate(&[team1, team2], &outcome))?;
        
        // Update storage and collect results
        let mut updated_players = Vec::new();
        
        for (i, player_id) in team1_players.iter().enumerate() {
            let new_rating = updated_teams[0].player_ratings()[i].clone();
            let (_, matches) = self.players.get_mut(player_id).unwrap();
            *matches += 1;
            self.players.insert(player_id.clone(), (RatingStorage::Elo(new_rating.clone()), *matches));
            updated_players.push(RatingStorage::Elo(new_rating).to_player_info(player_id, *matches));
        }
        
        for (i, player_id) in team2_players.iter().enumerate() {
            let new_rating = updated_teams[1].player_ratings()[i].clone();
            let (_, matches) = self.players.get_mut(player_id).unwrap();
            *matches += 1;
            self.players.insert(player_id.clone(), (RatingStorage::Elo(new_rating.clone()), *matches));
            updated_players.push(RatingStorage::Elo(new_rating).to_player_info(player_id, *matches));
        }
        
        Ok(updated_players)
    }
    
    /// Process a match using the Glicko rating system
    pub(crate) fn process_glicko_match(
        &mut self,
        team1_players: &[String],
        team2_players: &[String],
        winner_team: u32
    ) -> Result<Vec<PlayerInfo>, JsValue> {
        let system = self.glicko_system.as_ref().unwrap();
        
        // Build teams
        let team1_ratings: Vec<GlickoRating> = team1_players.iter()
            .map(|id| match &self.players[id].0 {
                RatingStorage::Glicko(r) => r.clone(),
                _ => unreachable!()
            })
            .collect();
        
        let team2_ratings: Vec<GlickoRating> = team2_players.iter()
            .map(|id| match &self.players[id].0 {
                RatingStorage::Glicko(r) => r.clone(),
                _ => unreachable!()
            })
            .collect();
        
        let team1 = GlickoTeamRating::from_player_ratings(team1_ratings);
        let team2 = GlickoTeamRating::from_player_ratings(team2_ratings);
        
        // Create outcome
        let outcome = if winner_team == 1 {
            GameOutcome::new(vec![1, 2])
        } else {
            GameOutcome::new(vec![2, 1])
        };
        
        // Rate match
        let updated_teams = to_js_result(system.rate(&[team1, team2], &outcome))?;
        
        // Update storage and collect results
        let mut updated_players = Vec::new();
        
        for (i, player_id) in team1_players.iter().enumerate() {
            let new_rating = updated_teams[0].player_ratings()[i].clone();
            let (_, matches) = self.players.get_mut(player_id).unwrap();
            *matches += 1;
            self.players.insert(player_id.clone(), (RatingStorage::Glicko(new_rating.clone()), *matches));
            updated_players.push(RatingStorage::Glicko(new_rating).to_player_info(player_id, *matches));
        }
        
        for (i, player_id) in team2_players.iter().enumerate() {
            let new_rating = updated_teams[1].player_ratings()[i].clone();
            let (_, matches) = self.players.get_mut(player_id).unwrap();
            *matches += 1;
            self.players.insert(player_id.clone(), (RatingStorage::Glicko(new_rating.clone()), *matches));
            updated_players.push(RatingStorage::Glicko(new_rating).to_player_info(player_id, *matches));
        }
        
        Ok(updated_players)
    }
    
    /// Process a match using the TrueSkill rating system
    pub(crate) fn process_trueskill_match(
        &mut self,
        team1_players: &[String],
        team2_players: &[String],
        winner_team: u32
    ) -> Result<Vec<PlayerInfo>, JsValue> {
        let system = self.trueskill_system.as_ref().unwrap();
        
        // Build teams
        let team1_ratings: Vec<TrueSkillRating> = team1_players.iter()
            .map(|id| match &self.players[id].0 {
                RatingStorage::TrueSkill(r) => r.clone(),
                _ => unreachable!()
            })
            .collect();
        
        let team2_ratings: Vec<TrueSkillRating> = team2_players.iter()
            .map(|id| match &self.players[id].0 {
                RatingStorage::TrueSkill(r) => r.clone(),
                _ => unreachable!()
            })
            .collect();
        
        let team1 = TrueSkillTeamRating::from_player_ratings(team1_ratings);
        let team2 = TrueSkillTeamRating::from_player_ratings(team2_ratings);
        
        // Create outcome
        let outcome = if winner_team == 1 {
            GameOutcome::new(vec![1, 2])
        } else {
            GameOutcome::new(vec![2, 1])
        };
        
        // Rate match
        let updated_teams = to_js_result(system.rate(&[team1, team2], &outcome))?;
        
        // Update storage and collect results
        let mut updated_players = Vec::new();
        
        for (i, player_id) in team1_players.iter().enumerate() {
            let new_rating = updated_teams[0].player_ratings()[i].clone();
            let (_, matches) = self.players.get_mut(player_id).unwrap();
            *matches += 1;
            self.players.insert(player_id.clone(), (RatingStorage::TrueSkill(new_rating.clone()), *matches));
            updated_players.push(RatingStorage::TrueSkill(new_rating).to_player_info(player_id, *matches));
        }
        
        for (i, player_id) in team2_players.iter().enumerate() {
            let new_rating = updated_teams[1].player_ratings()[i].clone();
            let (_, matches) = self.players.get_mut(player_id).unwrap();
            *matches += 1;
            self.players.insert(player_id.clone(), (RatingStorage::TrueSkill(new_rating.clone()), *matches));
            updated_players.push(RatingStorage::TrueSkill(new_rating).to_player_info(player_id, *matches));
        }
        
        Ok(updated_players)
    }
}