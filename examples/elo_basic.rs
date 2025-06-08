use ladder_rs::{
    elo::{EloSystem, EloTeamRating},
    core::{RatingSystem, GameOutcome, TeamRating},
};

fn main() {
    println!("=== Basic Elo Rating System Example ===\n");

    // Create an Elo rating system with default parameters
    let elo_system = EloSystem::new();
    println!("Created Elo system with default parameters:");
    println!("- K-factor: 20.0");
    println!("- Alpha: 0.1");
    println!("- Beta: 200.0");
    println!("- Default rating: 1500.0\n");

    // Create two players with default ratings
    let alice_rating = elo_system.create_rating();
    let bob_rating = elo_system.create_rating();
    
    println!("Initial ratings:");
    println!("Alice: {:.1}", alice_rating.rating());
    println!("Bob: {:.1}\n", bob_rating.rating());

    // Create teams (in Elo, each team has exactly one player)
    let alice_team = EloTeamRating::new(alice_rating);
    let bob_team = EloTeamRating::new(bob_rating);

    // Calculate initial match quality
    let initial_quality = elo_system
        .calculate_match_quality(&[alice_team.clone(), bob_team.clone()])
        .unwrap();
    println!("Initial match quality: {:.3}\n", initial_quality);

    // Simulate a match where Alice wins
    println!("=== Match 1: Alice vs Bob (Alice wins) ===");
    let outcome = GameOutcome::win(0, 2); // Team 0 (Alice) wins
    let updated_ratings = elo_system
        .rate(&[alice_team, bob_team], &outcome)
        .unwrap();

    println!("Updated ratings:");
    println!("Alice: {:.1} (gained: {:.1})", 
             updated_ratings[0].player_ratings()[0].rating(),
             updated_ratings[0].player_ratings()[0].rating() - 1500.0);
    println!("Bob: {:.1} (lost: {:.1})\n", 
             updated_ratings[1].player_ratings()[0].rating(),
             1500.0 - updated_ratings[1].player_ratings()[0].rating());

    // Simulate another match where Bob wins
    println!("=== Match 2: Alice vs Bob (Bob wins) ===");
    let alice_team = updated_ratings[0].clone();
    let bob_team = updated_ratings[1].clone();
    let outcome = GameOutcome::win(1, 2); // Team 1 (Bob) wins
    let updated_ratings = elo_system
        .rate(&[alice_team, bob_team], &outcome)
        .unwrap();

    println!("Updated ratings:");
    println!("Alice: {:.1}", updated_ratings[0].player_ratings()[0].rating());
    println!("Bob: {:.1}\n", updated_ratings[1].player_ratings()[0].rating());

    // Simulate a draw
    println!("=== Match 3: Alice vs Bob (Draw) ===");
    let alice_team = updated_ratings[0].clone();
    let bob_team = updated_ratings[1].clone();
    let outcome = GameOutcome::draw(2); // Draw between both teams
    let updated_ratings = elo_system
        .rate(&[alice_team, bob_team], &outcome)
        .unwrap();

    println!("Updated ratings:");
    println!("Alice: {:.1}", updated_ratings[0].player_ratings()[0].rating());
    println!("Bob: {:.1}\n", updated_ratings[1].player_ratings()[0].rating());

    // Final match quality
    let final_quality = elo_system
        .calculate_match_quality(&updated_ratings)
        .unwrap();
    println!("Final match quality: {:.3}", final_quality);
}