use ladder_rs::{
    core::{RatingSystem},
    trueskill::{TrueSkill, TrueSkillImplementation, TrueSkillTeam},
};

#[test]
fn debug_trueskill_parameters() {
    // Test negative mean - should this really fail?
    let result = TrueSkill::with_parameters(
        -25.0,
        100.0,
        25.0,
        1.0,
        0.1,
        TrueSkillImplementation::Simplified,
    );
    println!("Negative mean result: {:?}", result.is_err());

    // Test zero draw probability - should this really fail?
    let result = TrueSkill::with_parameters(
        25.0,
        100.0,
        25.0,
        1.0,
        0.0,
        TrueSkillImplementation::Simplified,
    );
    println!("Zero draw probability result: {:?}", result.is_err());

    // Test draw probability 1.0 - should this really fail?
    let result = TrueSkill::with_parameters(
        25.0,
        100.0,
        25.0,
        1.0,
        1.0,
        TrueSkillImplementation::Simplified,
    );
    println!("Draw probability 1.0 result: {:?}", result.is_err());
}

#[test]
fn debug_match_quality() {
    let ts = TrueSkill::new_simplified();

    let team1 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);
    let team2 = TrueSkillTeam::from_player_ratings(vec![ts.create_rating()]);

    let result = ts.calculate_match_quality(&[team1, team2]);
    println!("Match quality result: {:?}", result.is_err());
    match result {
        Ok(quality) => println!("Quality: {}", quality),
        Err(e) => println!("Error: {:?}", e),
    }
}
