use ladder_rs::{
    core::{GameOutcome, RatingSystem, TeamRating},
    glicko::{Glicko2, Glicko2Rating, Glicko2TeamRating},
};

#[test]
fn debug_glicko2_behavior() {
    let system = Glicko2::new();

    let player1 = Glicko2Rating::new(1500.0, 200.0, 0.06);
    let player2 = Glicko2Rating::new(1400.0, 30.0, 0.06);

    println!(
        "Before: P1 μ={}, RD={}, σ={}",
        player1.mu, player1.rd, player1.volatility
    );
    println!(
        "Before: P2 μ={}, RD={}, σ={}",
        player2.mu, player2.rd, player2.volatility
    );

    let team1 = Glicko2TeamRating::from_player_ratings(vec![player1]);
    let team2 = Glicko2TeamRating::from_player_ratings(vec![player2]);

    // Player 1 wins
    let outcome = GameOutcome::win(0, 2);
    let result = system.rate(&[team1, team2], &outcome).unwrap();

    let new_player1 = &result[0].player_ratings()[0];
    let new_player2 = &result[1].player_ratings()[0];

    println!(
        "After: P1 μ={:.3}, RD={:.3}, σ={:.6}",
        new_player1.mu, new_player1.rd, new_player1.volatility
    );
    println!(
        "After: P2 μ={:.3}, RD={:.3}, σ={:.6}",
        new_player2.mu, new_player2.rd, new_player2.volatility
    );
}
