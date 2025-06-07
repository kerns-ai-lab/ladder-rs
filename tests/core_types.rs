use ladder_rs::core::{GameOutcome, Outcome, TeamRating};
use ladder_rs::trueskill::{TrueSkillRating, TrueSkillTeam};

#[test]
fn test_game_outcome_new() {
    let outcome = GameOutcome::new(vec![1, 2]);
    assert_eq!(outcome.ranks(), &[1, 2]);
    assert!(outcome.is_valid_for_team_count(2));
}

#[test]
fn test_game_outcome_win_and_draw() {
    let win_outcome = GameOutcome::win(0, 2);
    assert_eq!(win_outcome.ranks(), &[1, 2]);
    let draw_outcome = GameOutcome::draw(3);
    assert_eq!(draw_outcome.ranks(), &[1, 1, 1]);
}

#[test]
fn test_game_outcome_zero_teams_invalid() {
    let outcome = GameOutcome::new(Vec::new());
    assert!(!outcome.is_valid_for_team_count(0));
}

#[test]
#[should_panic]
fn test_game_outcome_invalid_winner_index() {
    GameOutcome::win(1, 1); // index 1 does not exist
}

#[test]
fn test_true_skill_team_player_ratings() {
    let ratings = vec![
        TrueSkillRating::new(25.0, 9.0).unwrap(),
        TrueSkillRating::new(30.0, 16.0).unwrap(),
    ];
    let team = TrueSkillTeam::from_player_ratings(ratings.clone());
    assert_eq!(team.player_ratings(), ratings.as_slice());
}
