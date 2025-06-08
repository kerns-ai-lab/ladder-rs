use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    glicko::{Glicko, GlickoRating, GlickoTeamRating, Glicko2, Glicko2Rating, Glicko2TeamRating},
};

#[test]
fn debug_glicko_behavior() {
    let system = Glicko::new();

    let player1 = GlickoRating::new(1500.0, 350.0);
    let player2 = GlickoRating::new(1500.0, 350.0);

    println!("=== GLICKO TEST ===");
    println!("Before: P1 μ={}, RD={}", player1.mu, player1.rd);
    println!("Before: P2 μ={}, RD={}", player2.mu, player2.rd);

    let team1 = GlickoTeamRating::from_player_ratings(vec![player1]);
    let team2 = GlickoTeamRating::from_player_ratings(vec![player2]);

    // Player 1 wins
    let outcome = GameOutcome::win(0, 2);
    println!("Outcome ranks: {:?}", outcome.ranks());
    let result = system.rate(&[team1, team2], &outcome).unwrap();

    let new_player1 = &result[0].player_ratings()[0];
    let new_player2 = &result[1].player_ratings()[0];

    println!(
        "After: P1 μ={:.3}, RD={:.3}",
        new_player1.mu, new_player1.rd
    );
    println!(
        "After: P2 μ={:.3}, RD={:.3}",
        new_player2.mu, new_player2.rd
    );
    
    // Check that winner's rating increased
    assert!(new_player1.mu > 1500.0, "Winner should have higher rating");
    assert!(new_player2.mu < 1500.0, "Loser should have lower rating");
}

#[test]
fn debug_glicko_order_matters() {
    let system = Glicko::new();

    // Test 1: 7 wins then 3 losses
    let mut p1a = system.create_rating();
    let mut p2a = system.create_rating();
    
    for i in 0..10 {
        let outcome = if i < 7 {
            GameOutcome::win(0, 2) // P1 wins
        } else {
            GameOutcome::win(1, 2) // P2 wins
        };
        
        let team1 = GlickoTeamRating::from_player_ratings(vec![p1a]);
        let team2 = GlickoTeamRating::from_player_ratings(vec![p2a]);
        let result = system.rate(&[team1, team2], &outcome).unwrap();
        p1a = result[0].player_ratings()[0].clone();
        p2a = result[1].player_ratings()[0].clone();
    }
    
    println!("7 wins then 3 losses: P1={:.3}, P2={:.3}", p1a.mean(), p2a.mean());
    
    // Test 2: alternating wins/losses  
    let mut p1b = system.create_rating();
    let mut p2b = system.create_rating();
    
    let wins = [0, 0, 0, 1, 0, 0, 1, 0, 1, 0]; // same 7-3 record
    for &winner in &wins {
        let outcome = GameOutcome::win(winner, 2);
        
        let team1 = GlickoTeamRating::from_player_ratings(vec![p1b]);
        let team2 = GlickoTeamRating::from_player_ratings(vec![p2b]);
        let result = system.rate(&[team1, team2], &outcome).unwrap();
        p1b = result[0].player_ratings()[0].clone();
        p2b = result[1].player_ratings()[0].clone();
    }
    
    println!("Alternating pattern: P1={:.3}, P2={:.3}", p1b.mean(), p2b.mean());
}

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
