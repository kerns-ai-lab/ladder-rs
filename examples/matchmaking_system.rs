use ladder_rs::{
    elo::{EloSystem, EloRating, EloTeamRating},
    glicko::{Glicko2, Glicko2Rating, Glicko2TeamRating},
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
    core::{RatingSystem, GameOutcome, Rating},
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct Player {
    id: u32,
    name: String,
    elo_rating: EloRating,
    glicko2_rating: Glicko2Rating,
    trueskill_rating: TrueSkillRating,
    games_played: usize,
    last_played: usize, // Simulation time units since last game
}

impl Player {
    fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            elo_rating: EloRating::new(1500.0),
            glicko2_rating: Glicko2Rating::new(1500.0, 350.0, 0.06),
            trueskill_rating: TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap(),
            games_played: 0,
            last_played: 0,
        }
    }

    fn elo_conservative(&self) -> f64 {
        self.elo_rating.conservative_rating()
    }

    fn glicko2_conservative(&self) -> f64 {
        self.glicko2_rating.conservative_rating()
    }

    fn trueskill_conservative(&self) -> f64 {
        self.trueskill_rating.conservative_rating()
    }
}

struct MatchmakingSystem {
    elo_system: EloSystem,
    glicko2_system: Glicko2,
    trueskill_system: TrueSkill,
    players: HashMap<u32, Player>,
    match_history: Vec<MatchResult>,
    current_time: usize,
}

#[derive(Debug)]
struct MatchResult {
    player1_id: u32,
    player2_id: u32,
    winner_id: Option<u32>, // None for draw
    match_quality_elo: f64,
    match_quality_glicko2: f64,
    time: usize,
}

impl MatchmakingSystem {
    fn new() -> Self {
        Self {
            elo_system: EloSystem::new(),
            glicko2_system: Glicko2::new(),
            trueskill_system: TrueSkill::new_simplified(),
            players: HashMap::new(),
            match_history: Vec::new(),
            current_time: 0,
        }
    }

    fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    fn find_best_match_elo(&self, player_id: u32) -> Option<(u32, f64)> {
        let player = self.players.get(&player_id)?;
        let mut best_match: Option<(u32, f64)> = None;

        for (&other_id, other_player) in &self.players {
            if other_id == player_id {
                continue;
            }

            let team1 = EloTeamRating::new(player.elo_rating.clone());
            let team2 = EloTeamRating::new(other_player.elo_rating.clone());
            
            if let Ok(quality) = self.elo_system.calculate_match_quality(&[team1, team2]) {
                match best_match {
                    None => best_match = Some((other_id, quality)),
                    Some((_, best_quality)) => {
                        if quality > best_quality {
                            best_match = Some((other_id, quality));
                        }
                    }
                }
            }
        }

        best_match
    }

    fn find_best_match_glicko2(&self, player_id: u32) -> Option<(u32, f64)> {
        let player = self.players.get(&player_id)?;
        let mut best_match: Option<(u32, f64)> = None;

        for (&other_id, other_player) in &self.players {
            if other_id == player_id {
                continue;
            }

            let team1 = Glicko2TeamRating::from_player_ratings(vec![player.glicko2_rating.clone()]);
            let team2 = Glicko2TeamRating::from_player_ratings(vec![other_player.glicko2_rating.clone()]);
            
            if let Ok(quality) = self.glicko2_system.calculate_match_quality(&[team1, team2]) {
                match best_match {
                    None => best_match = Some((other_id, quality)),
                    Some((_, best_quality)) => {
                        if quality > best_quality {
                            best_match = Some((other_id, quality));
                        }
                    }
                }
            }
        }

        best_match
    }

    fn simulate_match(&mut self, player1_id: u32, player2_id: u32) -> Option<MatchResult> {
        let player1 = self.players.get(&player1_id)?.clone();
        let player2 = self.players.get(&player2_id)?.clone();

        // Calculate pre-match qualities
        let elo_team1 = EloTeamRating::new(player1.elo_rating.clone());
        let elo_team2 = EloTeamRating::new(player2.elo_rating.clone());
        let elo_quality = self.elo_system.calculate_match_quality(&[elo_team1, elo_team2]).ok()?;

        let glicko2_team1 = Glicko2TeamRating::from_player_ratings(vec![player1.glicko2_rating.clone()]);
        let glicko2_team2 = Glicko2TeamRating::from_player_ratings(vec![player2.glicko2_rating.clone()]);
        let glicko2_quality = self.glicko2_system.calculate_match_quality(&[glicko2_team1, glicko2_team2]).ok()?;

        // Simulate outcome based on skill difference (using TrueSkill as ground truth)
        let skill_diff = player1.trueskill_rating.mean() - player2.trueskill_rating.mean();
        let win_prob = 1.0 / (1.0 + (-skill_diff / 4.0).exp()); // Sigmoid function
        
        let rand_val: f64 = rand::random();
        let outcome = if rand_val < win_prob * 0.8 {
            GameOutcome::win(0, 2) // Player 1 wins
        } else if rand_val < win_prob * 0.8 + (1.0 - win_prob) * 0.8 {
            GameOutcome::win(1, 2) // Player 2 wins  
        } else {
            GameOutcome::draw(2) // Draw (20% chance regardless of skill)
        };

        let winner_id = match outcome.ranks() {
            [1, 2] => Some(player1_id),
            [2, 1] => Some(player2_id),
            [1, 1] => None,
            _ => return None,
        };

        // Update all rating systems
        self.update_ratings(player1_id, player2_id, &outcome);

        let match_result = MatchResult {
            player1_id,
            player2_id,
            winner_id,
            match_quality_elo: elo_quality,
            match_quality_glicko2: glicko2_quality,
            time: self.current_time,
        };

        self.match_history.push(match_result.clone());
        Some(match_result)
    }

    fn update_ratings(&mut self, player1_id: u32, player2_id: u32, outcome: &GameOutcome) {
        let player1 = self.players.get(&player1_id).unwrap().clone();
        let player2 = self.players.get(&player2_id).unwrap().clone();

        // Update Elo
        let elo_team1 = EloTeamRating::new(player1.elo_rating.clone());
        let elo_team2 = EloTeamRating::new(player2.elo_rating.clone());
        if let Ok(elo_result) = self.elo_system.rate(&[elo_team1, elo_team2], outcome) {
            self.players.get_mut(&player1_id).unwrap().elo_rating = elo_result[0].player_ratings()[0].clone();
            self.players.get_mut(&player2_id).unwrap().elo_rating = elo_result[1].player_ratings()[0].clone();
        }

        // Update Glicko-2
        let glicko2_team1 = Glicko2TeamRating::from_player_ratings(vec![player1.glicko2_rating.clone()]);
        let glicko2_team2 = Glicko2TeamRating::from_player_ratings(vec![player2.glicko2_rating.clone()]);
        if let Ok(glicko2_result) = self.glicko2_system.rate(&[glicko2_team1, glicko2_team2], outcome) {
            self.players.get_mut(&player1_id).unwrap().glicko2_rating = glicko2_result[0].player_ratings()[0].clone();
            self.players.get_mut(&player2_id).unwrap().glicko2_rating = glicko2_result[1].player_ratings()[0].clone();
        }

        // Update TrueSkill
        let trueskill_team1 = TrueSkillTeam::from_player_ratings(vec![player1.trueskill_rating.clone()]);
        let trueskill_team2 = TrueSkillTeam::from_player_ratings(vec![player2.trueskill_rating.clone()]);
        if let Ok(trueskill_result) = self.trueskill_system.rate(&[trueskill_team1, trueskill_team2], outcome) {
            self.players.get_mut(&player1_id).unwrap().trueskill_rating = trueskill_result[0].player_ratings()[0].clone();
            self.players.get_mut(&player2_id).unwrap().trueskill_rating = trueskill_result[1].player_ratings()[0].clone();
        }

        // Update game stats
        self.players.get_mut(&player1_id).unwrap().games_played += 1;
        self.players.get_mut(&player2_id).unwrap().games_played += 1;
        self.players.get_mut(&player1_id).unwrap().last_played = self.current_time;
        self.players.get_mut(&player2_id).unwrap().last_played = self.current_time;
    }

    fn get_leaderboard_elo(&self) -> Vec<(u32, String, f64)> {
        let mut players: Vec<_> = self.players.values()
            .map(|p| (p.id, p.name.clone(), p.elo_conservative()))
            .collect();
        players.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        players
    }

    fn get_leaderboard_glicko2(&self) -> Vec<(u32, String, f64)> {
        let mut players: Vec<_> = self.players.values()
            .map(|p| (p.id, p.name.clone(), p.glicko2_conservative()))
            .collect();
        players.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        players
    }

    fn get_leaderboard_trueskill(&self) -> Vec<(u32, String, f64)> {
        let mut players: Vec<_> = self.players.values()
            .map(|p| (p.id, p.name.clone(), p.trueskill_conservative()))
            .collect();
        players.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        players
    }

    fn advance_time(&mut self) {
        self.current_time += 1;
    }
}

fn main() {
    println!("=== Comprehensive Matchmaking System ===\n");

    let mut mm_system = MatchmakingSystem::new();

    // Add players with different skill levels (using TrueSkill as ground truth)
    let players = vec![
        Player { id: 1, name: "Grandmaster Alice".to_string(), trueskill_rating: TrueSkillRating::new(35.0, (3.0).powi(2)).unwrap(), ..Player::new(1, "Grandmaster Alice") },
        Player { id: 2, name: "Expert Bob".to_string(), trueskill_rating: TrueSkillRating::new(30.0, (4.0).powi(2)).unwrap(), ..Player::new(2, "Expert Bob") },
        Player { id: 3, name: "Advanced Charlie".to_string(), trueskill_rating: TrueSkillRating::new(28.0, (4.0).powi(2)).unwrap(), ..Player::new(3, "Advanced Charlie") },
        Player { id: 4, name: "Intermediate Dave".to_string(), trueskill_rating: TrueSkillRating::new(25.0, (5.0).powi(2)).unwrap(), ..Player::new(4, "Intermediate Dave") },
        Player { id: 5, name: "Beginner Eve".to_string(), trueskill_rating: TrueSkillRating::new(20.0, (6.0).powi(2)).unwrap(), ..Player::new(5, "Beginner Eve") },
        Player { id: 6, name: "Novice Frank".to_string(), trueskill_rating: TrueSkillRating::new(15.0, (7.0).powi(2)).unwrap(), ..Player::new(6, "Novice Frank") },
    ];

    for player in players {
        mm_system.add_player(player);
    }

    println!("Added 6 players with different skill levels (hidden ground truth)");
    println!("All systems start with default ratings and will learn through play\n");

    // Simulate matchmaking over time
    println!("=== Matchmaking Simulation ===\n");

    for round in 1..=20 {
        println!("Round {}", round);
        mm_system.advance_time();

        // Find best matches using different systems
        let alice_elo_match = mm_system.find_best_match_elo(1);
        let alice_glicko2_match = mm_system.find_best_match_glicko2(1);

        if let (Some((elo_opponent, elo_quality)), Some((glicko2_opponent, glicko2_quality))) = 
            (alice_elo_match, alice_glicko2_match) {
            
            println!("  Best match for Alice:");
            println!("    Elo suggests: {} (quality: {:.3})", 
                     mm_system.players.get(&elo_opponent).unwrap().name, elo_quality);
            println!("    Glicko-2 suggests: {} (quality: {:.3})", 
                     mm_system.players.get(&glicko2_opponent).unwrap().name, glicko2_quality);

            // Use Glicko-2 recommendation for this simulation
            let opponent_id = glicko2_opponent;
            if let Some(match_result) = mm_system.simulate_match(1, opponent_id) {
                let winner_name = match match_result.winner_id {
                    Some(id) => mm_system.players.get(&id).unwrap().name.clone(),
                    None => "Draw".to_string(),
                };
                println!("    Result: {}", winner_name);
            }
        }

        // Show leaderboards every 5 rounds
        if round % 5 == 0 {
            println!("\n  === Leaderboards after {} rounds ===", round);
            
            println!("  Elo Leaderboard:");
            for (i, (_, name, rating)) in mm_system.get_leaderboard_elo().iter().enumerate() {
                println!("    {}. {}: {:.1}", i + 1, name, rating);
            }
            
            println!("  Glicko-2 Leaderboard (conservative):");
            for (i, (_, name, rating)) in mm_system.get_leaderboard_glicko2().iter().enumerate() {
                println!("    {}. {}: {:.1}", i + 1, name, rating);
            }
            
            println!("  TrueSkill Leaderboard (conservative):");
            for (i, (_, name, rating)) in mm_system.get_leaderboard_trueskill().iter().enumerate() {
                println!("    {}. {}: {:.1}", i + 1, name, rating);
            }
            println!();
        }
    }

    // Analysis
    println!("=== Match Quality Analysis ===");
    let avg_elo_quality: f64 = mm_system.match_history.iter()
        .map(|m| m.match_quality_elo)
        .sum::<f64>() / mm_system.match_history.len() as f64;
    
    let avg_glicko2_quality: f64 = mm_system.match_history.iter()
        .map(|m| m.match_quality_glicko2)
        .sum::<f64>() / mm_system.match_history.len() as f64;

    println!("Average match quality:");
    println!("  Elo: {:.3}", avg_elo_quality);
    println!("  Glicko-2: {:.3}", avg_glicko2_quality);
    println!();

    // Show final player stats
    println!("=== Final Player Statistics ===");
    for player in mm_system.players.values() {
        println!("{} (games: {}):", player.name, player.games_played);
        println!("  Elo: {:.1}", player.elo_rating.rating());
        println!("  Glicko-2: μ={:.1}, σ={:.1} (conservative: {:.1})", 
                 player.glicko2_rating.mean(), player.glicko2_rating.standard_deviation(), player.glicko2_conservative());
        println!("  TrueSkill: μ={:.1}, σ={:.1} (conservative: {:.1})", 
                 player.trueskill_rating.mean(), player.trueskill_rating.standard_deviation(), player.trueskill_conservative());
        println!("  Ground Truth: μ={:.1}, σ={:.1}", 
                 player.trueskill_rating.mean(), player.trueskill_rating.standard_deviation());
    }

    println!("\n=== Matchmaking Insights ===");
    println!("1. Match Quality: Higher values indicate more balanced, competitive matches");
    println!("2. Conservative Ratings: Used for leaderboards, account for uncertainty");
    println!("3. Uncertainty Reduction: Players' σ values decrease as they play more games");
    println!("4. System Convergence: All systems eventually learn the true skill ranking");
    println!("5. Matchmaking Trade-offs: Balance between match quality and queue times");
}