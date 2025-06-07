use ladder_rs::{
    glicko::{Glicko, GlickoRating, GlickoTeamRating},
    core::{RatingSystem, GameOutcome, Rating},
};

fn main() {
    println!("=== Basic Glicko Rating System Example ===\n");

    // Create a Glicko rating system with default parameters
    let glicko_system = Glicko::new();
    println!("Created Glicko system with default parameters:");
    println!("- c (rating period variance): 15.8");
    println!("- q (conversion factor): {:.6}\n", (10.0_f64).ln() / 400.0);

    // Create two players with different initial ratings and uncertainties
    let alice_rating = GlickoRating::new(1500.0, 200.0);  // Average skill, high uncertainty
    let bob_rating = GlickoRating::new(1400.0, 50.0);     // Lower skill, low uncertainty
    
    println!("Initial ratings:");
    println!("Alice: μ={:.1}, RD={:.1} (conservative: {:.1})", 
             alice_rating.mean(), alice_rating.standard_deviation(), alice_rating.conservative_rating());
    println!("Bob: μ={:.1}, RD={:.1} (conservative: {:.1})\n", 
             bob_rating.mean(), bob_rating.standard_deviation(), bob_rating.conservative_rating());

    // Create teams (in Glicko, each team has exactly one player)
    let alice_team = GlickoTeamRating::from_player_ratings(vec![alice_rating]);
    let bob_team = GlickoTeamRating::from_player_ratings(vec![bob_rating]);

    // Calculate initial match quality
    let initial_quality = glicko_system
        .calculate_match_quality(&[alice_team.clone(), bob_team.clone()])
        .unwrap();
    println!("Initial match quality: {:.3}\n", initial_quality);

    // Simulate a match where Alice wins
    println!("=== Match 1: Alice vs Bob (Alice wins) ===");
    let outcome = GameOutcome::win(0, 2); // Team 0 (Alice) wins
    let updated_ratings = glicko_system
        .rate(&[alice_team, bob_team], &outcome)
        .unwrap();

    let alice_new = &updated_ratings[0].player_ratings()[0];
    let bob_new = &updated_ratings[1].player_ratings()[0];

    println!("Updated ratings:");
    println!("Alice: μ={:.1}, RD={:.1} (conservative: {:.1})", 
             alice_new.mean(), alice_new.standard_deviation(), alice_new.conservative_rating());
    println!("       Change: μ{:+.1}, RD{:+.1}", 
             alice_new.mean() - 1500.0, alice_new.standard_deviation() - 200.0);
    println!("Bob: μ={:.1}, RD={:.1} (conservative: {:.1})", 
             bob_new.mean(), bob_new.standard_deviation(), bob_new.conservative_rating());
    println!("     Change: μ{:+.1}, RD{:+.1}\n", 
             bob_new.mean() - 1400.0, bob_new.standard_deviation() - 50.0);

    // Simulate another match where Bob wins (upset!)
    println!("=== Match 2: Alice vs Bob (Bob wins - upset!) ===");
    let alice_team = updated_ratings[0].clone();
    let bob_team = updated_ratings[1].clone();
    let outcome = GameOutcome::win(1, 2); // Team 1 (Bob) wins
    let updated_ratings = glicko_system
        .rate(&[alice_team, bob_team], &outcome)
        .unwrap();

    let alice_final = &updated_ratings[0].player_ratings()[0];
    let bob_final = &updated_ratings[1].player_ratings()[0];

    println!("Updated ratings:");
    println!("Alice: μ={:.1}, RD={:.1} (conservative: {:.1})", 
             alice_final.mean(), alice_final.standard_deviation(), alice_final.conservative_rating());
    println!("Bob: μ={:.1}, RD={:.1} (conservative: {:.1})", 
             bob_final.mean(), bob_final.standard_deviation(), bob_final.conservative_rating());
    println!();

    // Simulate a draw
    println!("=== Match 3: Alice vs Bob (Draw) ===");
    let alice_team = updated_ratings[0].clone();
    let bob_team = updated_ratings[1].clone();
    let outcome = GameOutcome::draw(2); // Draw between both teams
    let updated_ratings = glicko_system
        .rate(&[alice_team, bob_team], &outcome)
        .unwrap();

    let alice_draw = &updated_ratings[0].player_ratings()[0];
    let bob_draw = &updated_ratings[1].player_ratings()[0];

    println!("Updated ratings:");
    println!("Alice: μ={:.1}, RD={:.1} (conservative: {:.1})", 
             alice_draw.mean(), alice_draw.standard_deviation(), alice_draw.conservative_rating());
    println!("Bob: μ={:.1}, RD={:.1} (conservative: {:.1})\n", 
             bob_draw.mean(), bob_draw.standard_deviation(), bob_draw.conservative_rating());

    // Final match quality
    let final_quality = glicko_system
        .calculate_match_quality(&updated_ratings)
        .unwrap();
    println!("Final match quality: {:.3}", final_quality);

    // Demonstrate the effect of time passage (no games played)
    println!("\n=== Time Passage (Rating Period without Games) ===");
    let no_opponents: Vec<(GlickoRating, f64)> = vec![];
    
    // Simulate Alice not playing any games (RD should increase)
    let alice_time_passed = GlickoRating::new(
        alice_draw.mean(),
        (alice_draw.standard_deviation().powi(2) + 15.8_f64.powi(2)).sqrt()
    );
    
    println!("Alice after time passage without games:");
    println!("Before: μ={:.1}, RD={:.1}", alice_draw.mean(), alice_draw.standard_deviation());
    println!("After:  μ={:.1}, RD={:.1}", alice_time_passed.mean(), alice_time_passed.standard_deviation());
    println!("(Rating uncertainty increases when not playing)");
}