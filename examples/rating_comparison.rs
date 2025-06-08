use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    elo::{EloSystem, EloTeamRating},
    glicko::{Glicko, GlickoTeamRating},
    trueskill::{TrueSkill, TrueSkillTeam},
};

fn main() {
    println!("=== Rating System Comparison ===\n");

    // Initialize all rating systems
    let elo_system = EloSystem::new();
    let glicko_system = Glicko::new();
    let trueskill_system = TrueSkill::new_simplified();

    println!("Comparing three major rating systems:");
    println!("1. Elo: Simple, deterministic, widely used");
    println!("2. Glicko: Adds uncertainty (RD), time-based degradation");
    println!("3. TrueSkill: Bayesian, supports teams, Microsoft's system\n");

    // Create initial ratings for Alice and Bob
    let alice_elo = elo_system.create_rating();
    let bob_elo = elo_system.create_rating();

    let alice_glicko = glicko_system.create_rating();
    let bob_glicko = glicko_system.create_rating();

    let alice_trueskill = trueskill_system.create_rating();
    let bob_trueskill = trueskill_system.create_rating();

    println!("Initial ratings:");
    println!(
        "Alice - Elo: {:.1}, Glicko: {:.1}±{:.1}, TrueSkill: {:.1}±{:.1}",
        alice_elo.rating(),
        alice_glicko.mean(),
        alice_glicko.standard_deviation(),
        alice_trueskill.mean(),
        alice_trueskill.standard_deviation()
    );

    println!(
        "Bob   - Elo: {:.1}, Glicko: {:.1}±{:.1}, TrueSkill: {:.1}±{:.1}\n",
        bob_elo.rating(),
        bob_glicko.mean(),
        bob_glicko.standard_deviation(),
        bob_trueskill.mean(),
        bob_trueskill.standard_deviation()
    );

    // Simulate Alice winning a match
    let outcome = GameOutcome::win(0, 2);

    // Update Elo ratings
    let elo_team1 = EloTeamRating::new(alice_elo);
    let elo_team2 = EloTeamRating::new(bob_elo);
    let elo_result = elo_system.rate(&[elo_team1, elo_team2], &outcome).unwrap();
    let alice_elo_new = &elo_result[0].player_ratings()[0];
    let bob_elo_new = &elo_result[1].player_ratings()[0];

    // Update Glicko ratings
    let glicko_team1 = GlickoTeamRating::from_player_ratings(vec![alice_glicko]);
    let glicko_team2 = GlickoTeamRating::from_player_ratings(vec![bob_glicko]);
    let glicko_result = glicko_system
        .rate(&[glicko_team1, glicko_team2], &outcome)
        .unwrap();
    let alice_glicko_new = &glicko_result[0].player_ratings()[0];
    let bob_glicko_new = &glicko_result[1].player_ratings()[0];

    // Update TrueSkill ratings
    let trueskill_team1 = TrueSkillTeam::from_player_ratings(vec![alice_trueskill]);
    let trueskill_team2 = TrueSkillTeam::from_player_ratings(vec![bob_trueskill]);
    let trueskill_result = trueskill_system
        .rate(&[trueskill_team1, trueskill_team2], &outcome)
        .unwrap();
    let alice_trueskill_new = &trueskill_result[0].player_ratings()[0];
    let bob_trueskill_new = &trueskill_result[1].player_ratings()[0];

    println!("After Alice wins:");
    println!(
        "Alice - Elo: {:.1}, Glicko: {:.1}±{:.1}, TrueSkill: {:.1}±{:.1}",
        alice_elo_new.rating(),
        alice_glicko_new.mean(),
        alice_glicko_new.standard_deviation(),
        alice_trueskill_new.mean(),
        alice_trueskill_new.standard_deviation()
    );

    println!(
        "Bob   - Elo: {:.1}, Glicko: {:.1}±{:.1}, TrueSkill: {:.1}±{:.1}\n",
        bob_elo_new.rating(),
        bob_glicko_new.mean(),
        bob_glicko_new.standard_deviation(),
        bob_trueskill_new.mean(),
        bob_trueskill_new.standard_deviation()
    );

    println!("=== Key Differences ===");
    println!("1. Elo: Simple point values, quick convergence");
    println!("2. Glicko: Includes uncertainty (±), more sophisticated");
    println!("3. TrueSkill: Full Bayesian approach, handles uncertainty well");
    println!("\nAll systems show Alice gaining rating and Bob losing rating after Alice's win.");
}
