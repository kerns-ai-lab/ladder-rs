use ladder_rs::{
    core::{GameOutcome, RatingSystem, TeamRating},
    elo::{EloSystem, EloRating, EloTeamRating},
};

#[test]
fn debug_elo_behavior() {
    let system = EloSystem::new();
    
    // Test extreme rating difference
    let high_player = EloRating::new(2800.0);
    let low_player = EloRating::new(800.0);
    
    let team1 = EloTeamRating::new(high_player);
    let team2 = EloTeamRating::new(low_player);
    
    // Low-rated player wins (upset)
    let upset_outcome = GameOutcome::win(1, 2);
    let upset_result = system.rate(&[team1, team2], &upset_outcome).unwrap();
    
    let low_gain = upset_result[1].player_ratings()[0].rating() - 800.0;
    let high_loss = 2800.0 - upset_result[0].player_ratings()[0].rating();
    
    println!("Low player gain: {}", low_gain);
    println!("High player loss: {}", high_loss);
    
    // Test custom parameters
    let high_k_system = EloSystem::with_parameters(50.0, 0.2, 300.0, 1200.0);
    
    let player1 = EloRating::new(1200.0);
    let player2 = EloRating::new(1200.0);
    
    let team1 = EloTeamRating::new(player1);
    let team2 = EloTeamRating::new(player2);
    
    let outcome = GameOutcome::win(0, 2);
    let result = high_k_system.rate(&[team1, team2], &outcome).unwrap();
    
    let rating_change = result[0].player_ratings()[0].rating() - 1200.0;
    println!("High K-factor rating change: {}", rating_change);
    
    // Test default K-factor
    let default_system = EloSystem::new();
    let def_player1 = EloRating::new(1500.0);
    let def_player2 = EloRating::new(1500.0);
    
    let def_team1 = EloTeamRating::new(def_player1);
    let def_team2 = EloTeamRating::new(def_player2);
    
    let def_result = default_system.rate(&[def_team1, def_team2], &outcome).unwrap();
    let def_change = def_result[0].player_ratings()[0].rating() - 1500.0;
    println!("Default K-factor rating change: {}", def_change);
}