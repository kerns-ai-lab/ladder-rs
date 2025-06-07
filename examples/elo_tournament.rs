use ladder_rs::{
    elo::{EloSystem, EloRating, EloTeamRating},
    core::{RatingSystem, GameOutcome},
};
use std::collections::HashMap;

#[derive(Clone)]
struct Player {
    name: String,
    rating: EloRating,
}

impl Player {
    fn new(name: &str, initial_rating: f64) -> Self {
        Self {
            name: name.to_string(),
            rating: EloRating::new(initial_rating),
        }
    }
}

fn main() {
    println!("=== Elo Tournament Simulation ===\n");

    let elo_system = EloSystem::new();

    // Create a tournament with 4 players of different skill levels
    let mut players = vec![
        Player::new("Grandmaster Alice", 2000.0),
        Player::new("Expert Bob", 1800.0),
        Player::new("Intermediate Charlie", 1600.0),
        Player::new("Novice Dave", 1400.0),
    ];

    println!("Initial tournament standings:");
    for (i, player) in players.iter().enumerate() {
        println!("{}. {}: {:.1}", i + 1, player.name, player.rating.rating());
    }
    println!();

    // Simulate all possible matches in the tournament (round-robin)
    let mut match_number = 1;
    let mut match_results: Vec<(String, String, String)> = Vec::new();

    for i in 0..players.len() {
        for j in (i + 1)..players.len() {
            let player1 = &players[i];
            let player2 = &players[j];

            println!("=== Match {}: {} vs {} ===", match_number, player1.name, player2.name);

            // Calculate match quality before the match
            let team1 = EloTeamRating::new(player1.rating.clone());
            let team2 = EloTeamRating::new(player2.rating.clone());
            let quality = elo_system.calculate_match_quality(&[team1.clone(), team2.clone()]).unwrap();
            println!("Match quality: {:.3}", quality);

            // Simulate match outcome based on skill difference
            // Higher rated player has better chance to win
            let rating_diff = player1.rating.rating() - player2.rating.rating();
            let win_prob = 1.0 / (1.0 + 10.0_f64.powf(-rating_diff / 400.0)); // Standard Elo win probability
            
            let outcome = if rand::random::<f64>() < win_prob {
                println!("{} wins!", player1.name);
                GameOutcome::win(0, 2) // Player 1 wins
            } else {
                println!("{} wins!", player2.name);
                GameOutcome::win(1, 2) // Player 2 wins
            };

            // Update ratings
            let updated_ratings = elo_system.rate(&[team1, team2], &outcome).unwrap();
            
            let old_rating1 = players[i].rating.rating();
            let old_rating2 = players[j].rating.rating();
            let new_rating1 = updated_ratings[0].player_ratings()[0].rating();
            let new_rating2 = updated_ratings[1].player_ratings()[0].rating();

            players[i].rating = EloRating::new(new_rating1);
            players[j].rating = EloRating::new(new_rating2);

            println!("{}: {:.1} → {:.1} ({:+.1})",
                     player1.name, old_rating1, new_rating1, new_rating1 - old_rating1);
            println!("{}: {:.1} → {:.1} ({:+.1})",
                     player2.name, old_rating2, new_rating2, new_rating2 - old_rating2);
            println!();

            // Record match result
            let winner = if outcome.ranks()[0] < outcome.ranks()[1] {
                player1.name.clone()
            } else {
                player2.name.clone()
            };
            match_results.push((player1.name.clone(), player2.name.clone(), winner));

            match_number += 1;
        }
    }

    // Sort players by final rating
    players.sort_by(|a, b| b.rating.rating().partial_cmp(&a.rating.rating()).unwrap());

    println!("=== Final Tournament Standings ===");
    for (i, player) in players.iter().enumerate() {
        println!("{}. {}: {:.1}", i + 1, player.name, player.rating.rating());
    }
    println!();

    // Display match results summary
    println!("=== Match Results Summary ===");
    for (i, (player1, player2, winner)) in match_results.iter().enumerate() {
        println!("Match {}: {} vs {} → {} wins", i + 1, player1, player2, winner);
    }
    println!();

    // Calculate average match quality across all games
    let mut total_quality = 0.0;
    let mut quality_count = 0;
    
    // Re-calculate qualities for final state demonstration
    for i in 0..players.len() {
        for j in (i + 1)..players.len() {
            let team1 = EloTeamRating::new(players[i].rating.clone());
            let team2 = EloTeamRating::new(players[j].rating.clone());
            let quality = elo_system.calculate_match_quality(&[team1, team2]).unwrap();
            total_quality += quality;
            quality_count += 1;
        }
    }

    println!("Average match quality in final state: {:.3}", total_quality / quality_count as f64);
    println!("(Lower quality indicates more skill separation after tournament)");
}