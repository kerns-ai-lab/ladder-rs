use ladder_rs::{
    elo::{EloSystem, EloRating, EloTeamRating},
    glicko::{Glicko, GlickoRating, GlickoTeamRating, Glicko2, Glicko2Rating, Glicko2TeamRating},
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
    core::{RatingSystem, GameOutcome, Rating},
};

#[derive(Clone)]
struct PlayerAcrossSystem {
    name: String,
    elo_rating: EloRating,
    glicko_rating: GlickoRating,
    glicko2_rating: Glicko2Rating,
    trueskill_rating: TrueSkillRating,
}

impl PlayerAcrossSystem {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            elo_rating: EloRating::new(1500.0),
            glicko_rating: GlickoRating::new(1500.0, 200.0),
            glicko2_rating: Glicko2Rating::new(1500.0, 200.0, 0.06),
            trueskill_rating: TrueSkillRating::new(25.0, (25.0/3.0).powi(2)).unwrap(),
        }
    }

    fn display_ratings(&self) -> String {
        format!(
            "{}: \n  Elo: {:.1}, \n  Glicko: {:.1}±{:.1}, \n  Glicko-2: {:.1}±{:.1}(σ:{:.3}), \n  TrueSkill: {:.1}±{:.1}",
            self.name,
            self.elo_rating.rating(),
            self.glicko_rating.mean(), self.glicko_rating.standard_deviation(),
            self.glicko2_rating.mean(), self.glicko2_rating.standard_deviation(), self.glicko2_rating.volatility,
            self.trueskill_rating.mean(), self.trueskill_rating.standard_deviation()
        )
    }

    fn display_conservative(&self) -> String {
        format!(
            "{} Conservative: Elo:{:.1}, Glicko:{:.1}, Glicko-2:{:.1}, TrueSkill:{:.1}",
            self.name,
            self.elo_rating.conservative_rating(),
            self.glicko_rating.conservative_rating(),
            self.glicko2_rating.conservative_rating(),
            self.trueskill_rating.conservative_rating()
        )
    }
}

fn main() {
    println!("=== Rating System Comparison ===\n");

    // Initialize all rating systems
    let elo_system = EloSystem::new();
    let glicko_system = Glicko::new();
    let glicko2_system = Glicko2::new();
    let trueskill_system = TrueSkill::new_simplified();

    println!("Comparing four major rating systems:");
    println!("1. Elo: Simple, deterministic, widely used");
    println!("2. Glicko: Adds uncertainty (RD), time-based degradation");
    println!("3. Glicko-2: Adds volatility (σ), accounts for performance consistency");
    println!("4. TrueSkill: Bayesian, supports teams, Microsoft's system\n");

    // Create two players
    let mut alice = PlayerAcrossSystem::new("Alice");
    let mut bob = PlayerAcrossSystem::new("Bob");

    println!("Initial ratings:");
    println!("{}", alice.display_ratings());
    println!("{}\n", bob.display_ratings());

    // Simulate a series of matches with the same outcomes across all systems
    let match_outcomes = vec![
        ("Alice wins", 0),
        ("Bob wins", 1), 
        ("Alice wins", 0),
        ("Alice wins", 0),
        ("Draw", 2),
        ("Alice wins", 0),
        ("Bob wins", 1),
        ("Draw", 2),
        ("Alice wins", 0),
        ("Bob wins", 1),
    ];

    println!("=== Match Simulation ===\n");

    for (i, (description, winner)) in match_outcomes.iter().enumerate() {
        println!("Match {}: {}", i + 1, description);

        let outcome = if *winner == 2 {
            GameOutcome::draw(2)
        } else {
            GameOutcome::win(*winner, 2)
        };

        // Update Elo ratings
        let elo_team1 = EloTeamRating::new(alice.elo_rating.clone());
        let elo_team2 = EloTeamRating::new(bob.elo_rating.clone());
        let elo_result = elo_system.rate(&[elo_team1, elo_team2], &outcome).unwrap();
        alice.elo_rating = elo_result[0].player_ratings()[0].clone();
        bob.elo_rating = elo_result[1].player_ratings()[0].clone();

        // Update Glicko ratings
        let glicko_team1 = GlickoTeamRating::from_player_ratings(vec![alice.glicko_rating.clone()]);
        let glicko_team2 = GlickoTeamRating::from_player_ratings(vec![bob.glicko_rating.clone()]);
        let glicko_result = glicko_system.rate(&[glicko_team1, glicko_team2], &outcome).unwrap();
        alice.glicko_rating = glicko_result[0].player_ratings()[0].clone();
        bob.glicko_rating = glicko_result[1].player_ratings()[0].clone();

        // Update Glicko-2 ratings
        let glicko2_team1 = Glicko2TeamRating::from_player_ratings(vec![alice.glicko2_rating.clone()]);
        let glicko2_team2 = Glicko2TeamRating::from_player_ratings(vec![bob.glicko2_rating.clone()]);
        let glicko2_result = glicko2_system.rate(&[glicko2_team1, glicko2_team2], &outcome).unwrap();
        alice.glicko2_rating = glicko2_result[0].player_ratings()[0].clone();
        bob.glicko2_rating = glicko2_result[1].player_ratings()[0].clone();

        // Update TrueSkill ratings
        let trueskill_team1 = TrueSkillTeam::from_player_ratings(vec![alice.trueskill_rating.clone()]);
        let trueskill_team2 = TrueSkillTeam::from_player_ratings(vec![bob.trueskill_rating.clone()]);
        let trueskill_result = trueskill_system.rate(&[trueskill_team1, trueskill_team2], &outcome).unwrap();
        alice.trueskill_rating = trueskill_result[0].player_ratings()[0].clone();
        bob.trueskill_rating = trueskill_result[1].player_ratings()[0].clone();

        // Show every 3rd match for brevity
        if (i + 1) % 3 == 0 {
            println!("After match {}:", i + 1);
            println!("  {}", alice.display_conservative());
            println!("  {}\n", bob.display_conservative());
        }
    }

    println!("=== Final Ratings ===");
    println!("{}", alice.display_ratings());
    println!("{}\n", bob.display_ratings());

    println!("=== Final Conservative Ratings ===");
    println!("{}", alice.display_conservative());
    println!("{}\n", bob.display_conservative());

    // Compare match qualities for a hypothetical next match
    println!("=== Match Quality Comparison ===");
    
    // Elo match quality
    let elo_team1 = EloTeamRating::new(alice.elo_rating.clone());
    let elo_team2 = EloTeamRating::new(bob.elo_rating.clone());
    let elo_quality = elo_system.calculate_match_quality(&[elo_team1, elo_team2]).unwrap();
    println!("Elo match quality: {:.3}", elo_quality);

    // Glicko match quality
    let glicko_team1 = GlickoTeamRating::from_player_ratings(vec![alice.glicko_rating.clone()]);
    let glicko_team2 = GlickoTeamRating::from_player_ratings(vec![bob.glicko_rating.clone()]);
    let glicko_quality = glicko_system.calculate_match_quality(&[glicko_team1, glicko_team2]).unwrap();
    println!("Glicko match quality: {:.3}", glicko_quality);

    // Glicko-2 match quality
    let glicko2_team1 = Glicko2TeamRating::from_player_ratings(vec![alice.glicko2_rating.clone()]);
    let glicko2_team2 = Glicko2TeamRating::from_player_ratings(vec![bob.glicko2_rating.clone()]);
    let glicko2_quality = glicko2_system.calculate_match_quality(&[glicko2_team1, glicko2_team2]).unwrap();
    println!("Glicko-2 match quality: {:.3}", glicko2_quality);

    println!("TrueSkill match quality: Not implemented yet\n");

    // Analysis
    println!("=== System Comparison Analysis ===");
    
    println!("1. Rating Convergence:");
    println!("   - Elo: Simple linear adjustment, fastest convergence");
    println!("   - Glicko: Uncertainty-aware, moderate convergence");
    println!("   - Glicko-2: Volatility-adjusted, sophisticated convergence");
    println!("   - TrueSkill: Bayesian updates, handles uncertainty well");
    println!();

    println!("2. Uncertainty Representation:");
    println!("   - Elo: No explicit uncertainty (variance = 0)");
    println!("   - Glicko: Rating Deviation (RD) shows uncertainty");
    println!("   - Glicko-2: RD + Volatility for performance consistency");
    println!("   - TrueSkill: Full Gaussian distribution with μ and σ");
    println!();

    println!("3. Conservative Ratings (for leaderboards):");
    println!("   - Elo: Same as rating (no uncertainty penalty)");
    println!("   - Glicko: μ - 2σ (modest uncertainty penalty)");
    println!("   - Glicko-2: μ - 2σ (with volatility-adjusted RD)");
    println!("   - TrueSkill: μ - 3σ (conservative uncertainty penalty)");
    println!();

    println!("4. Best Use Cases:");
    println!("   - Elo: Simple games, quick implementation, chess/Go");
    println!("   - Glicko: Periodic tournaments, longer rating periods");
    println!("   - Glicko-2: Competitive gaming with varying performance");
    println!("   - TrueSkill: Team games, Xbox Live, complex matchmaking");
    println!();

    println!("5. Computational Complexity:");
    println!("   - Elo: O(1) - very fast");
    println!("   - Glicko: O(n) where n = opponents");
    println!("   - Glicko-2: O(n) with iterative volatility calculation");
    println!("   - TrueSkill: O(players × teams) - most complex");
}