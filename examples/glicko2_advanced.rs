use ladder_rs::{
    glicko::{Glicko2, Glicko2Rating, Glicko2TeamRating, Glicko2Config},
    core::{RatingSystem, GameOutcome, Rating},
};

#[derive(Clone)]
struct Player {
    name: String,
    rating: Glicko2Rating,
    games_played: usize,
}

impl Player {
    fn new(name: &str, mu: f64, rd: f64, volatility: f64) -> Self {
        Self {
            name: name.to_string(),
            rating: Glicko2Rating::new(mu, rd, volatility),
            games_played: 0,
        }
    }

    fn display_rating(&self) -> String {
        format!(
            "μ={:.1}, RD={:.1}, σ={:.3} (conservative: {:.1})",
            self.rating.mean(),
            self.rating.standard_deviation(),
            self.rating.volatility,
            self.rating.conservative_rating()
        )
    }
}

fn main() {
    println!("=== Advanced Glicko-2 Rating System Example ===\n");

    // Create a custom Glicko-2 system with modified parameters
    let config = Glicko2Config {
        tau: 0.3,     // Lower tau for less volatility change
        epsilon: 0.000001,
    };
    let glicko2_system = Glicko2::with_config(config);
    
    println!("Created Glicko-2 system with custom parameters:");
    println!("- tau (volatility change): 0.3");
    println!("- epsilon (convergence): 0.000001\n");

    // Create players with different skill levels and experience
    let mut players = vec![
        Player::new("Veteran Alice", 1800.0, 80.0, 0.05),    // Experienced, stable
        Player::new("Rising Bob", 1600.0, 150.0, 0.08),      // Improving, moderate uncertainty
        Player::new("Newcomer Charlie", 1500.0, 350.0, 0.06), // New, high uncertainty
        Player::new("Inconsistent Dave", 1550.0, 120.0, 0.12), // Volatile performance
    ];

    println!("Initial player ratings:");
    for (i, player) in players.iter().enumerate() {
        println!("{}. {}: {}", i + 1, player.name, player.display_rating());
    }
    println!();

    // Simulate a rating period with multiple games
    println!("=== Rating Period Simulation ===\n");

    // Match 1: Alice (high skill) vs Charlie (newcomer)
    println!("Match 1: {} vs {}", players[0].name, players[2].name);
    let team1 = Glicko2TeamRating::from_player_ratings(vec![players[0].rating.clone()]);
    let team2 = Glicko2TeamRating::from_player_ratings(vec![players[2].rating.clone()]);
    
    let quality = glicko2_system.calculate_match_quality(&[team1.clone(), team2.clone()]).unwrap();
    println!("Match quality: {:.3}", quality);
    
    // Alice wins (expected result)
    let outcome = GameOutcome::win(0, 2);
    let result = glicko2_system.rate(&[team1, team2], &outcome).unwrap();
    
    players[0].rating = result[0].player_ratings()[0].clone();
    players[2].rating = result[1].player_ratings()[0].clone();
    players[0].games_played += 1;
    players[2].games_played += 1;
    
    println!("Result: {} wins", players[0].name);
    println!("  {}: {}", players[0].name, players[0].display_rating());
    println!("  {}: {}\n", players[2].name, players[2].display_rating());

    // Match 2: Bob vs Dave (closer skill match)
    println!("Match 2: {} vs {}", players[1].name, players[3].name);
    let team1 = Glicko2TeamRating::from_player_ratings(vec![players[1].rating.clone()]);
    let team2 = Glicko2TeamRating::from_player_ratings(vec![players[3].rating.clone()]);
    
    let quality = glicko2_system.calculate_match_quality(&[team1.clone(), team2.clone()]).unwrap();
    println!("Match quality: {:.3}", quality);
    
    // Bob wins
    let outcome = GameOutcome::win(0, 2);
    let result = glicko2_system.rate(&[team1, team2], &outcome).unwrap();
    
    players[1].rating = result[0].player_ratings()[0].clone();
    players[3].rating = result[1].player_ratings()[0].clone();
    players[1].games_played += 1;
    players[3].games_played += 1;
    
    println!("Result: {} wins", players[1].name);
    println!("  {}: {}", players[1].name, players[1].display_rating());
    println!("  {}: {}\n", players[3].name, players[3].display_rating());

    // Match 3: Charlie (newcomer) vs Dave (upset potential)
    println!("Match 3: {} vs {}", players[2].name, players[3].name);
    let team1 = Glicko2TeamRating::from_player_ratings(vec![players[2].rating.clone()]);
    let team2 = Glicko2TeamRating::from_player_ratings(vec![players[3].rating.clone()]);
    
    let quality = glicko2_system.calculate_match_quality(&[team1.clone(), team2.clone()]).unwrap();
    println!("Match quality: {:.3}", quality);
    
    // Charlie wins (potential upset given Dave's higher rating)
    let outcome = GameOutcome::win(0, 2);
    let result = glicko2_system.rate(&[team1, team2], &outcome).unwrap();
    
    players[2].rating = result[0].player_ratings()[0].clone();
    players[3].rating = result[1].player_ratings()[0].clone();
    players[2].games_played += 1;
    players[3].games_played += 1;
    
    println!("Result: {} wins (upset!)", players[2].name);
    println!("  {}: {}", players[2].name, players[2].display_rating());
    println!("  {}: {}\n", players[3].name, players[3].display_rating());

    // Match 4: Alice vs Bob (top players)
    println!("Match 4: {} vs {}", players[0].name, players[1].name);
    let team1 = Glicko2TeamRating::from_player_ratings(vec![players[0].rating.clone()]);
    let team2 = Glicko2TeamRating::from_player_ratings(vec![players[1].rating.clone()]);
    
    let quality = glicko2_system.calculate_match_quality(&[team1.clone(), team2.clone()]).unwrap();
    println!("Match quality: {:.3}", quality);
    
    // Draw between top players
    let outcome = GameOutcome::draw(2);
    let result = glicko2_system.rate(&[team1, team2], &outcome).unwrap();
    
    players[0].rating = result[0].player_ratings()[0].clone();
    players[1].rating = result[1].player_ratings()[0].clone();
    players[0].games_played += 1;
    players[1].games_played += 1;
    
    println!("Result: Draw");
    println!("  {}: {}", players[0].name, players[0].display_rating());
    println!("  {}: {}\n", players[1].name, players[1].display_rating());

    // Final standings
    players.sort_by(|a, b| b.rating.conservative_rating().partial_cmp(&a.rating.conservative_rating()).unwrap());
    
    println!("=== Final Standings (by conservative rating) ===");
    for (i, player) in players.iter().enumerate() {
        println!("{}. {}: {}", i + 1, player.name, player.display_rating());
        println!("   Games played: {}", player.games_played);
    }
    println!();

    // Demonstrate key Glicko-2 features
    println!("=== Glicko-2 Key Features Demonstrated ===");
    
    println!("1. Rating Deviation (RD) decreases with more games:");
    for player in &players {
        println!("   {}: RD = {:.1} (started at varying levels)", 
                 player.name, player.rating.standard_deviation());
    }
    println!();
    
    println!("2. Volatility (σ) adapts to performance consistency:");
    for player in &players {
        println!("   {}: σ = {:.3}", player.name, player.rating.volatility);
    }
    println!();
    
    println!("3. Conservative rating accounts for uncertainty:");
    for player in &players {
        let diff = player.rating.mean() - player.rating.conservative_rating();
        println!("   {}: μ - conservative = {:.1}", player.name, diff);
    }
    println!();

    // Show what happens if a player doesn't play
    println!("=== Effect of Inactivity ===");
    let inactive_alice = Glicko2Rating::new(
        players[0].rating.mean(),
        (players[0].rating.standard_deviation().powi(2) + players[0].rating.volatility.powi(2)).sqrt(),
        players[0].rating.volatility
    );
    
    println!("If {} doesn't play for a rating period:", players[0].name);
    println!("Before: μ={:.1}, RD={:.1}, σ={:.3}", 
             players[0].rating.mean(), players[0].rating.standard_deviation(), players[0].rating.volatility);
    println!("After:  μ={:.1}, RD={:.1}, σ={:.3}", 
             inactive_alice.mean(), inactive_alice.standard_deviation(), inactive_alice.volatility);
    println!("(Rating uncertainty increases due to inactivity)");
}