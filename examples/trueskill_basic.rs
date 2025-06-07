use ladder_rs::{
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam, TrueSkillImplementation},
    core::{RatingSystem, GameOutcome, Rating},
};

fn main() {
    println!("=== Basic TrueSkill Rating System Example ===\n");

    // Create a TrueSkill rating system with default parameters
    let trueskill_system = TrueSkill::new_simplified();
    println!("Created TrueSkill system with default parameters:");
    println!("- μ₀ (initial mean): 25.0");
    println!("- σ₀² (initial variance): {:.2}", (25.0/3.0).powi(2));
    println!("- β² (performance variance): {:.2}", (25.0/3.0/2.0).powi(2));
    println!("- γ² (dynamics variance): {:.4}", (25.0/3.0/100.0).powi(2));
    println!("- Draw probability: 10%");
    println!("- Implementation: Simplified\n");

    // Create two players with default ratings
    let alice_rating = trueskill_system.create_rating();
    let bob_rating = trueskill_system.create_rating();
    
    println!("Initial ratings:");
    println!("Alice: μ={:.1}, σ={:.2} (conservative: {:.1})", 
             alice_rating.mean(), alice_rating.standard_deviation(), alice_rating.conservative_rating());
    println!("Bob: μ={:.1}, σ={:.2} (conservative: {:.1})\n", 
             bob_rating.mean(), bob_rating.standard_deviation(), bob_rating.conservative_rating());

    // Create teams (each with one player for 1v1 match)
    let alice_team = TrueSkillTeam::from_player_ratings(vec![alice_rating]);
    let bob_team = TrueSkillTeam::from_player_ratings(vec![bob_rating]);

    // Simulate a match where Alice wins
    println!("=== Match 1: Alice vs Bob (Alice wins) ===");
    let outcome = GameOutcome::win(0, 2); // Team 0 (Alice) wins
    let updated_ratings = trueskill_system
        .rate(&[alice_team, bob_team], &outcome)
        .unwrap();

    let alice_new = &updated_ratings[0].player_ratings()[0];
    let bob_new = &updated_ratings[1].player_ratings()[0];

    println!("Updated ratings:");
    println!("Alice: μ={:.1}, σ={:.2} (conservative: {:.1})", 
             alice_new.mean(), alice_new.standard_deviation(), alice_new.conservative_rating());
    println!("       Change: μ{:+.1}, σ{:+.2}", 
             alice_new.mean() - 25.0, alice_new.standard_deviation() - 25.0/3.0);
    println!("Bob: μ={:.1}, σ={:.2} (conservative: {:.1})", 
             bob_new.mean(), bob_new.standard_deviation(), bob_new.conservative_rating());
    println!("     Change: μ{:+.1}, σ{:+.2}\n", 
             bob_new.mean() - 25.0, bob_new.standard_deviation() - 25.0/3.0);

    // Simulate another match where Bob wins (comeback)
    println!("=== Match 2: Alice vs Bob (Bob wins) ===");
    let alice_team = updated_ratings[0].clone();
    let bob_team = updated_ratings[1].clone();
    let outcome = GameOutcome::win(1, 2); // Team 1 (Bob) wins
    let updated_ratings = trueskill_system
        .rate(&[alice_team, bob_team], &outcome)
        .unwrap();

    let alice_final = &updated_ratings[0].player_ratings()[0];
    let bob_final = &updated_ratings[1].player_ratings()[0];

    println!("Updated ratings:");
    println!("Alice: μ={:.1}, σ={:.2} (conservative: {:.1})", 
             alice_final.mean(), alice_final.standard_deviation(), alice_final.conservative_rating());
    println!("Bob: μ={:.1}, σ={:.2} (conservative: {:.1})", 
             bob_final.mean(), bob_final.standard_deviation(), bob_final.conservative_rating());
    println!();

    // Simulate a draw
    println!("=== Match 3: Alice vs Bob (Draw) ===");
    let alice_team = updated_ratings[0].clone();
    let bob_team = updated_ratings[1].clone();
    let outcome = GameOutcome::draw(2); // Draw between both teams
    let updated_ratings = trueskill_system
        .rate(&[alice_team, bob_team], &outcome)
        .unwrap();

    let alice_draw = &updated_ratings[0].player_ratings()[0];
    let bob_draw = &updated_ratings[1].player_ratings()[0];

    println!("Updated ratings:");
    println!("Alice: μ={:.1}, σ={:.2} (conservative: {:.1})", 
             alice_draw.mean(), alice_draw.standard_deviation(), alice_draw.conservative_rating());
    println!("Bob: μ={:.1}, σ={:.2} (conservative: {:.1})\n", 
             bob_draw.mean(), bob_draw.standard_deviation(), bob_draw.conservative_rating());

    // Demonstrate creating players with custom ratings
    println!("=== Custom Initial Ratings ===");
    let expert_rating = trueskill_system.create_rating_with_values(35.0, (5.0).powi(2));
    let novice_rating = trueskill_system.create_rating_with_values(15.0, (10.0).powi(2));

    println!("Expert player: μ={:.1}, σ={:.1} (conservative: {:.1})", 
             expert_rating.mean(), expert_rating.standard_deviation(), expert_rating.conservative_rating());
    println!("Novice player: μ={:.1}, σ={:.1} (conservative: {:.1})", 
             novice_rating.mean(), novice_rating.standard_deviation(), novice_rating.conservative_rating());

    // Match between expert and novice
    println!("\n=== Expert vs Novice (Expert wins) ===");
    let expert_team = TrueSkillTeam::from_player_ratings(vec![expert_rating]);
    let novice_team = TrueSkillTeam::from_player_ratings(vec![novice_rating]);
    let outcome = GameOutcome::win(0, 2); // Expert wins

    let result = trueskill_system.rate(&[expert_team, novice_team], &outcome).unwrap();
    
    let expert_after = &result[0].player_ratings()[0];
    let novice_after = &result[1].player_ratings()[0];

    println!("After match:");
    println!("Expert: μ={:.1}, σ={:.1} (conservative: {:.1}) - Change: μ{:+.1}", 
             expert_after.mean(), expert_after.standard_deviation(), 
             expert_after.conservative_rating(), expert_after.mean() - 35.0);
    println!("Novice: μ={:.1}, σ={:.1} (conservative: {:.1}) - Change: μ{:+.1}", 
             novice_after.mean(), novice_after.standard_deviation(), 
             novice_after.conservative_rating(), novice_after.mean() - 15.0);
    
    println!("\nNote: Expert gains little from beating novice, novice loses more rating");
    println!("Both players' uncertainty (σ) decreases after playing a game");
    
    // Show what TrueSkill values represent
    println!("\n=== Understanding TrueSkill Values ===");
    println!("μ (mu): The estimated skill level of the player");
    println!("σ (sigma): The uncertainty in the skill estimate");
    println!("Conservative rating (μ - 3σ): A 99.7% confidence lower bound on skill");
    println!("- Use conservative rating for leaderboards and matchmaking");
    println!("- Higher σ means less certainty about the player's true skill");
    println!("- σ decreases as a player plays more games");
}