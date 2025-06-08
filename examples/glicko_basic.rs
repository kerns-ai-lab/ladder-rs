use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    glicko::{Glicko, GlickoRating, GlickoTeamRating},
};

fn main() {
    println!("=== Basic Glicko Rating System Example ===\n");

    // Create a Glicko rating system with default parameters
    let glicko_system = Glicko::new();
    println!("Created Glicko system with default parameters:");
    println!("- c (rating period variance): 15.8");
    println!("- q (conversion factor): {:.6}\n", (10.0_f64).ln() / 400.0);

    // Create two players with different initial ratings and uncertainties
    let alice_rating = GlickoRating::new(1500.0, 200.0); // Average skill, high uncertainty
    let bob_rating = GlickoRating::new(1400.0, 50.0); // Lower skill, low uncertainty

    println!("Initial ratings:");
    println!(
        "Alice: μ={:.1}, RD={:.1} (conservative: {:.1})",
        alice_rating.mean(),
        alice_rating.standard_deviation(),
        alice_rating.conservative_rating()
    );
    println!(
        "Bob: μ={:.1}, RD={:.1} (conservative: {:.1})\n",
        bob_rating.mean(),
        bob_rating.standard_deviation(),
        bob_rating.conservative_rating()
    );

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
    println!(
        "Alice: μ={:.1}, RD={:.1} (conservative: {:.1})",
        alice_new.mean(),
        alice_new.standard_deviation(),
        alice_new.conservative_rating()
    );
    println!(
        "       Change: μ{:+.1}, RD{:+.1}",
        alice_new.mean() - 1500.0,
        alice_new.standard_deviation() - 200.0
    );
    println!(
        "Bob: μ={:.1}, RD={:.1} (conservative: {:.1})",
        bob_new.mean(),
        bob_new.standard_deviation(),
        bob_new.conservative_rating()
    );
    println!(
        "     Change: μ{:+.1}, RD{:+.1}\n",
        bob_new.mean() - 1400.0,
        bob_new.standard_deviation() - 50.0
    );

    println!("Note: Rating uncertainty (RD) typically decreases after playing games");
}
