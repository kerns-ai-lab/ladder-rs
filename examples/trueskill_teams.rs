use ladder_rs::{
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
    core::{RatingSystem, GameOutcome, Rating},
};

#[derive(Clone)]
struct Player {
    name: String,
    rating: TrueSkillRating,
}

impl Player {
    fn new(name: &str, mu: f64, sigma: f64) -> Self {
        Self {
            name: name.to_string(),
            rating: TrueSkillRating::new(mu, sigma * sigma).unwrap(),
        }
    }

    fn display_rating(&self) -> String {
        format!(
            "μ={:.1}, σ={:.1} (conservative: {:.1})",
            self.rating.mean(),
            self.rating.standard_deviation(),
            self.rating.conservative_rating()
        )
    }
}

fn main() {
    println!("=== TrueSkill Team-Based Rating Example ===\n");

    // Create TrueSkill system with custom parameters optimized for team play
    let trueskill_system = TrueSkill::with_parameters(
        25.0,               // μ₀: initial mean
        (25.0/3.0).powi(2), // σ₀²: initial variance  
        (25.0/6.0).powi(2), // β²: smaller performance variance for team games
        (25.0/300.0).powi(2), // γ²: smaller dynamics variance
        0.05,               // 5% draw probability for competitive team games
        ladder_rs::trueskill::TrueSkillImplementation::Simplified,
    ).unwrap();
    
    println!("Created TrueSkill system optimized for team games:");
    println!("- Reduced performance variance (β²) for team coordination");
    println!("- Lower draw probability (5%) for competitive matches");
    println!("- Smaller dynamics variance for slower skill evolution\n");

    // Create players with varying skill levels
    let mut players = vec![
        Player::new("Alice", 30.0, 5.0),    // High skill, experienced
        Player::new("Bob", 25.0, 8.0),      // Average skill, moderate experience
        Player::new("Charlie", 20.0, 6.0),  // Below average, some experience
        Player::new("Dave", 15.0, 10.0),    // Low skill, new player
        Player::new("Eve", 28.0, 7.0),      // Above average, moderate experience
        Player::new("Frank", 22.0, 9.0),    // Slightly below average, less experienced
    ];

    println!("Initial player ratings:");
    for (i, player) in players.iter().enumerate() {
        println!("{}. {}: {}", i + 1, player.name, player.display_rating());
    }
    println!();

    // Note: Team examples are conceptual since current implementation only supports 1v1
    // In a full implementation, this would support multi-player teams
    
    println!("=== Individual Matches Leading to Team Formation ===\n");

    // Simulate several 1v1 matches to establish individual skills first
    println!("1v1 Training Matches:");

    // Alice vs Bob
    println!("Match 1: Alice vs Bob");
    let alice_team = TrueSkillTeam::from_player_ratings(vec![players[0].rating.clone()]);
    let bob_team = TrueSkillTeam::from_player_ratings(vec![players[1].rating.clone()]);
    let outcome = GameOutcome::win(0, 2); // Alice wins

    let result = trueskill_system.rate(&[alice_team, bob_team], &outcome).unwrap();
    players[0].rating = result[0].player_ratings()[0].clone();
    players[1].rating = result[1].player_ratings()[0].clone();

    println!("  Result: Alice wins");
    println!("  Alice: {} ", players[0].display_rating());
    println!("  Bob: {}\n", players[1].display_rating());

    // Charlie vs Dave
    println!("Match 2: Charlie vs Dave");
    let charlie_team = TrueSkillTeam::from_player_ratings(vec![players[2].rating.clone()]);
    let dave_team = TrueSkillTeam::from_player_ratings(vec![players[3].rating.clone()]);
    let outcome = GameOutcome::win(0, 2); // Charlie wins

    let result = trueskill_system.rate(&[charlie_team, dave_team], &outcome).unwrap();
    players[2].rating = result[0].player_ratings()[0].clone();
    players[3].rating = result[1].player_ratings()[0].clone();

    println!("  Result: Charlie wins");
    println!("  Charlie: {}", players[2].display_rating());
    println!("  Dave: {}\n", players[3].display_rating());

    // Eve vs Frank
    println!("Match 3: Eve vs Frank");
    let eve_team = TrueSkillTeam::from_player_ratings(vec![players[4].rating.clone()]);
    let frank_team = TrueSkillTeam::from_player_ratings(vec![players[5].rating.clone()]);
    let outcome = GameOutcome::draw(2); // Draw

    let result = trueskill_system.rate(&[eve_team, frank_team], &outcome).unwrap();
    players[4].rating = result[0].player_ratings()[0].clone();
    players[5].rating = result[1].player_ratings()[0].clone();

    println!("  Result: Draw");
    println!("  Eve: {}", players[4].display_rating());
    println!("  Frank: {}\n", players[5].display_rating());

    // Cross-matches for more data
    println!("Match 4: Alice vs Eve (top players)");
    let alice_team = TrueSkillTeam::from_player_ratings(vec![players[0].rating.clone()]);
    let eve_team = TrueSkillTeam::from_player_ratings(vec![players[4].rating.clone()]);
    let outcome = GameOutcome::win(1, 2); // Eve wins (upset!)

    let result = trueskill_system.rate(&[alice_team, eve_team], &outcome).unwrap();
    players[0].rating = result[0].player_ratings()[0].clone();
    players[4].rating = result[1].player_ratings()[0].clone();

    println!("  Result: Eve wins (upset!)");
    println!("  Alice: {}", players[0].display_rating());
    println!("  Eve: {}\n", players[4].display_rating());

    // Sort players by conservative rating for team selection
    players.sort_by(|a, b| b.rating.conservative_rating().partial_cmp(&a.rating.conservative_rating()).unwrap());

    println!("=== Current Player Rankings (by conservative rating) ===");
    for (i, player) in players.iter().enumerate() {
        println!("{}. {}: {}", i + 1, player.name, player.display_rating());
    }
    println!();

    // Demonstrate team concepts (even though implementation is 1v1 only)
    println!("=== Team Formation Concepts ===");
    
    // Balanced teams
    let team_a = vec![&players[0], &players[3]]; // Best + worst
    let team_b = vec![&players[1], &players[2]]; // Middle players
    
    println!("Balanced Team Formation:");
    println!("Team A: {} & {}", team_a[0].name, team_a[1].name);
    let team_a_avg = (team_a[0].rating.conservative_rating() + team_a[1].rating.conservative_rating()) / 2.0;
    println!("  Average conservative rating: {:.1}", team_a_avg);
    
    println!("Team B: {} & {}", team_b[0].name, team_b[1].name);
    let team_b_avg = (team_b[0].rating.conservative_rating() + team_b[1].rating.conservative_rating()) / 2.0;
    println!("  Average conservative rating: {:.1}", team_b_avg);
    
    println!("Team balance difference: {:.1}", (team_a_avg - team_b_avg).abs());
    println!();

    // Skill-based teams  
    let team_high = vec![&players[0], &players[1]]; // Top 2
    let team_low = vec![&players[2], &players[3]];  // Bottom 2
    
    println!("Skill-Based Team Formation:");
    println!("High Skill Team: {} & {}", team_high[0].name, team_high[1].name);
    let team_high_avg = (team_high[0].rating.conservative_rating() + team_high[1].rating.conservative_rating()) / 2.0;
    println!("  Average conservative rating: {:.1}", team_high_avg);
    
    println!("Lower Skill Team: {} & {}", team_low[0].name, team_low[1].name);
    let team_low_avg = (team_low[0].rating.conservative_rating() + team_low[1].rating.conservative_rating()) / 2.0;
    println!("  Average conservative rating: {:.1}", team_low_avg);
    
    println!("Skill gap: {:.1}", team_high_avg - team_low_avg);
    println!();

    println!("=== TrueSkill Team Game Advantages ===");
    println!("1. Multi-player support: TrueSkill can handle teams of any size");
    println!("2. Team skill calculation: Team strength = sum of individual skills");
    println!("3. Individual updates: Each player's rating updates based on team performance");
    println!("4. Uncertainty handling: Player uncertainty affects team uncertainty");
    println!("5. Match quality: Predicts how close/entertaining a team match will be");
    println!();

    println!("Note: This example uses 1v1 matches due to current implementation limits.");
    println!("A full TrueSkill implementation would support arbitrary team sizes and");
    println!("calculate ratings for all team members simultaneously.");
}