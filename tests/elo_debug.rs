use ladder_rs::{
    core::{GameOutcome, RatingSystem, TeamRating},
    elo::{EloRating, EloSystem, EloTeamRating},
};

#[test]
fn debug_elo_behavior() {
    let system = EloSystem::new();

    let mut player1 = EloRating::new(1500.0);
    let mut player2 = EloRating::new(1500.0);

    println!(
        "Initial ratings: P1={}, P2={}",
        player1.rating(),
        player2.rating()
    );

    // Player 1 wins 5 games in a row
    for i in 0..5 {
        let team1 = EloTeamRating::new(player1.clone());
        let team2 = EloTeamRating::new(player2.clone());

        let outcome = GameOutcome::win(0, 2);
        let result = system.rate(&[team1, team2], &outcome).unwrap();

        player1 = result[0].player_ratings()[0].clone();
        player2 = result[1].player_ratings()[0].clone();

        println!(
            "Game {}: P1={:.3}, P2={:.3}",
            i + 1,
            player1.rating(),
            player2.rating()
        );
    }

    // Test upset scenario
    let high_player = EloRating::new(2000.0);
    let low_player = EloRating::new(1200.0);

    let team1 = EloTeamRating::new(high_player);
    let team2 = EloTeamRating::new(low_player);

    // Low player wins (upset)
    let outcome = GameOutcome::win(1, 2);
    let result = system.rate(&[team1, team2], &outcome).unwrap();

    let new_high = result[0].player_ratings()[0].rating();
    let new_low = result[1].player_ratings()[0].rating();

    println!("Upset scenario:");
    println!("High player: 2000.0 -> {:.3}", new_high);
    println!("Low player: 1200.0 -> {:.3}", new_low);

    // Test match quality
    let equal1 = EloTeamRating::new(EloRating::new(1500.0));
    let equal2 = EloTeamRating::new(EloRating::new(1500.0));
    let quality = system.calculate_match_quality(&[equal1, equal2]).unwrap();
    println!("Match quality (equal): {:.6}", quality);

    let diff1 = EloTeamRating::new(EloRating::new(1600.0));
    let diff2 = EloTeamRating::new(EloRating::new(1400.0));
    let quality = system.calculate_match_quality(&[diff1, diff2]).unwrap();
    println!("Match quality (200 diff): {:.6}", quality);

    let extreme1 = EloTeamRating::new(EloRating::new(2500.0));
    let extreme2 = EloTeamRating::new(EloRating::new(500.0));
    let quality = system
        .calculate_match_quality(&[extreme1, extreme2])
        .unwrap();
    println!("Match quality (extreme): {:.6}", quality);
}
