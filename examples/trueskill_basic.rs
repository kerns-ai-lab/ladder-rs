use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem},
    trueskill::{TrueSkill, TrueSkillTeam},
};

fn main() {
    println!("=== Basic TrueSkill Rating System Example ===\n");

    // Create a TrueSkill rating system with default parameters
    let trueskill_system = TrueSkill::new_simplified();
    println!("Created TrueSkill system with default parameters:");
    println!("- μ₀ (initial mean): 25.0");
    println!("- σ₀² (initial variance): {:.2}", (25.0_f64 / 3.0).powi(2));
    println!(
        "- β² (performance variance): {:.2}",
        (25.0_f64 / 3.0 / 2.0).powi(2)
    );
    println!(
        "- γ² (dynamics variance): {:.4}",
        (25.0_f64 / 3.0 / 100.0).powi(2)
    );
    println!("- Draw probability: 10%");
    println!("- Implementation: Simplified\n");

    // Create two players with default ratings
    let alice_rating = trueskill_system.create_rating();
    let bob_rating = trueskill_system.create_rating();

    println!("Initial ratings:");
    println!(
        "Alice: μ={:.1}, σ={:.2} (conservative: {:.1})",
        alice_rating.mean(),
        alice_rating.standard_deviation(),
        alice_rating.conservative_rating()
    );
    println!(
        "Bob: μ={:.1}, σ={:.2} (conservative: {:.1})\n",
        bob_rating.mean(),
        bob_rating.standard_deviation(),
        bob_rating.conservative_rating()
    );

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
    println!(
        "Alice: μ={:.1}, σ={:.2} (conservative: {:.1})",
        alice_new.mean(),
        alice_new.standard_deviation(),
        alice_new.conservative_rating()
    );
    println!(
        "       Change: μ{:+.1}, σ{:+.2}",
        alice_new.mean() - 25.0,
        alice_new.standard_deviation() - 25.0 / 3.0
    );
    println!(
        "Bob: μ={:.1}, σ={:.2} (conservative: {:.1})",
        bob_new.mean(),
        bob_new.standard_deviation(),
        bob_new.conservative_rating()
    );
    println!(
        "     Change: μ{:+.1}, σ{:+.2}\n",
        bob_new.mean() - 25.0,
        bob_new.standard_deviation() - 25.0 / 3.0
    );

    println!("Note: Both players' uncertainty (σ) decreases after playing a game");
    println!("Conservative rating (μ - 3σ) provides a 99.7% confidence lower bound");
}
