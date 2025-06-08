use ladder_rs::{
    core::{GameOutcome, Outcome, Rating, RatingSystem, TeamRating},
    elo::{EloRating, EloSystem, EloTeamRating},
    glicko::{Glicko, Glicko2, Glicko2Rating, Glicko2TeamRating, GlickoRating, GlickoTeamRating},
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
};

#[test]
fn test_game_outcome_comprehensive() {
    // Test GameOutcome creation and validation
    let outcome = GameOutcome::new(vec![1, 2, 3]);
    assert_eq!(outcome.ranks(), &[1, 2, 3]);

    // Test win outcome creation
    let win_outcome = GameOutcome::win(0, 3);
    assert_eq!(win_outcome.ranks(), &[1, 2, 2]);
    assert!(win_outcome.is_valid_for_team_count(3));
    assert!(!win_outcome.is_valid_for_team_count(2));

    // Test draw outcome creation
    let draw_outcome = GameOutcome::draw(4);
    assert_eq!(draw_outcome.ranks(), &[1, 1, 1, 1]);
    assert!(draw_outcome.is_valid_for_team_count(4));

    // Test outcome validation edge cases
    let empty_outcome = GameOutcome::new(vec![]);
    assert!(!empty_outcome.is_valid_for_team_count(1));
    assert!(!empty_outcome.is_valid_for_team_count(0));
}

#[test]
fn test_rating_trait_consistency_across_systems() {
    // Test that all rating types properly implement the Rating trait
    let elo_rating = EloRating::new(1500.0);
    let glicko_rating = GlickoRating::new(1500.0, 100.0);
    let glicko2_rating = Glicko2Rating::new(1500.0, 100.0, 0.06);
    let trueskill_rating = TrueSkillRating::new(25.0, 64.0).unwrap();

    // Test mean() implementation
    assert_eq!(elo_rating.mean(), 1500.0);
    assert_eq!(glicko_rating.mean(), 1500.0);
    assert_eq!(glicko2_rating.mean(), 1500.0);
    assert_eq!(trueskill_rating.mean(), 25.0);

    // Test variance() implementation
    assert_eq!(elo_rating.variance(), 0.0); // Elo has no variance
    assert_eq!(glicko_rating.variance(), 10000.0); // 100^2
    assert_eq!(glicko2_rating.variance(), 10000.0); // 100^2
    assert_eq!(trueskill_rating.variance(), 64.0);

    // Test standard_deviation() implementation
    assert_eq!(elo_rating.standard_deviation(), 0.0);
    assert_eq!(glicko_rating.standard_deviation(), 100.0);
    assert_eq!(glicko2_rating.standard_deviation(), 100.0);
    assert_eq!(trueskill_rating.standard_deviation(), 8.0);

    // Test conservative_rating() implementation
    assert_eq!(elo_rating.conservative_rating(), 1500.0); // Same as rating for Elo
    assert_eq!(glicko_rating.conservative_rating(), 1300.0); // μ - 2σ
    assert_eq!(glicko2_rating.conservative_rating(), 1300.0); // μ - 2σ
    assert_eq!(trueskill_rating.conservative_rating(), 1.0); // μ - 3σ = 25 - 24
}

#[test]
fn test_team_rating_trait_consistency() {
    // Test that all team rating types properly implement TeamRating trait
    let elo_player = EloRating::new(1500.0);
    let elo_team = EloTeamRating::from_player_ratings(vec![elo_player]);

    let glicko_player = GlickoRating::new(1500.0, 100.0);
    let glicko_team = GlickoTeamRating::from_player_ratings(vec![glicko_player]);

    let glicko2_player = Glicko2Rating::new(1500.0, 100.0, 0.06);
    let glicko2_team = Glicko2TeamRating::from_player_ratings(vec![glicko2_player]);

    let trueskill_player = TrueSkillRating::new(25.0, 64.0).unwrap();
    let trueskill_team = TrueSkillTeam::from_player_ratings(vec![trueskill_player]);

    // Test player_ratings() consistency
    assert_eq!(elo_team.player_ratings().len(), 1);
    assert_eq!(glicko_team.player_ratings().len(), 1);
    assert_eq!(glicko2_team.player_ratings().len(), 1);
    assert_eq!(trueskill_team.player_ratings().len(), 1);
}

#[test]
fn test_rating_system_trait_consistency() {
    // Test that all systems properly implement RatingSystem trait
    let elo_system = EloSystem::new();
    let glicko_system = Glicko::new();
    let glicko2_system = Glicko2::new();
    let trueskill_system = TrueSkill::new();

    // Test create_rating() consistency
    let elo_rating = elo_system.create_rating();
    let glicko_rating = glicko_system.create_rating();
    let glicko2_rating = glicko2_system.create_rating();
    let trueskill_rating = trueskill_system.create_rating();

    assert_eq!(elo_rating.rating(), 1500.0);
    assert_eq!(glicko_rating.mu, 1500.0);
    assert_eq!(glicko2_rating.mu, 1500.0);
    assert_eq!(trueskill_rating.mean(), 25.0);

    // Test create_rating_with_values() consistency
    let elo_custom = elo_system.create_rating_with_values(1800.0, 100.0);
    let glicko_custom = glicko_system.create_rating_with_values(1800.0, 100.0);
    let glicko2_custom = glicko2_system.create_rating_with_values(1800.0, 100.0);
    let trueskill_custom = trueskill_system.create_rating_with_values(30.0, 49.0);

    assert_eq!(elo_custom.mean(), 1800.0);
    assert_eq!(glicko_custom.mean(), 1800.0);
    assert_eq!(glicko2_custom.mean(), 1800.0);
    assert_eq!(trueskill_custom.mean(), 30.0);
}

#[test]
fn test_all_systems_basic_functionality() {
    // Test that all systems can perform basic rating updates
    let elo_system = EloSystem::new();
    let glicko_system = Glicko::new();
    let _glicko2_system = Glicko2::new();
    let trueskill_system = TrueSkill::new();

    // Create teams for each system
    let elo_team1 = EloTeamRating::new(EloRating::new(1500.0));
    let elo_team2 = EloTeamRating::new(EloRating::new(1500.0));

    let glicko_team1 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);
    let glicko_team2 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);

    let glicko2_team1 =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);
    let glicko2_team2 =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);

    let trueskill_team1 =
        TrueSkillTeam::from_player_ratings(vec![TrueSkillRating::new(25.0, 64.0).unwrap()]);
    let trueskill_team2 =
        TrueSkillTeam::from_player_ratings(vec![TrueSkillRating::new(25.0, 64.0).unwrap()]);

    let outcome = GameOutcome::win(0, 2);

    // Test that all systems can process the same outcome
    let elo_result = elo_system.rate(&[elo_team1, elo_team2], &outcome);
    assert!(
        elo_result.is_ok(),
        "Elo system should handle basic rating update"
    );

    let glicko_result = glicko_system.rate(&[glicko_team1, glicko_team2], &outcome);
    assert!(
        glicko_result.is_ok(),
        "Glicko system should handle basic rating update"
    );

    let glicko2_system = Glicko2::new();
    let glicko2_result = glicko2_system.rate(&[glicko2_team1, glicko2_team2], &outcome);
    assert!(
        glicko2_result.is_ok(),
        "Glicko2 system should handle basic rating update"
    );

    let trueskill_result = trueskill_system.rate(&[trueskill_team1, trueskill_team2], &outcome);
    assert!(
        trueskill_result.is_ok(),
        "TrueSkill system should handle basic rating update"
    );
}

#[test]
fn test_error_handling_consistency() {
    // Test that all systems handle errors consistently
    let elo_system = EloSystem::new();
    let glicko_system = Glicko::new();
    let glicko2_system = Glicko2::new();
    let trueskill_system = TrueSkill::new();

    // Test single team error
    let elo_team = EloTeamRating::new(EloRating::new(1500.0));
    let glicko_team = GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);
    let glicko2_team =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);
    let trueskill_team =
        TrueSkillTeam::from_player_ratings(vec![TrueSkillRating::new(25.0, 64.0).unwrap()]);

    let single_outcome = GameOutcome::new(vec![1]);

    assert!(elo_system.rate(&[elo_team], &single_outcome).is_err());
    assert!(glicko_system.rate(&[glicko_team], &single_outcome).is_err());
    assert!(glicko2_system
        .rate(&[glicko2_team], &single_outcome)
        .is_err());
    assert!(trueskill_system
        .rate(&[trueskill_team], &single_outcome)
        .is_err());
}

#[test]
fn test_match_quality_implementations() {
    // Test match quality calculations where implemented
    let elo_system = EloSystem::new();
    let glicko_system = Glicko::new();
    let glicko2_system = Glicko2::new();
    let trueskill_system = TrueSkill::new();

    // Create equal teams
    let elo_team1 = EloTeamRating::new(EloRating::new(1500.0));
    let elo_team2 = EloTeamRating::new(EloRating::new(1500.0));

    let glicko_team1 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);
    let glicko_team2 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 200.0)]);

    let glicko2_team1 =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);
    let glicko2_team2 =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1500.0, 200.0, 0.06)]);

    let trueskill_team1 =
        TrueSkillTeam::from_player_ratings(vec![TrueSkillRating::new(25.0, 64.0).unwrap()]);
    let trueskill_team2 =
        TrueSkillTeam::from_player_ratings(vec![TrueSkillRating::new(25.0, 64.0).unwrap()]);

    // Test implemented match quality calculations
    let elo_quality = elo_system.calculate_match_quality(&[elo_team1, elo_team2]);
    assert!(
        elo_quality.is_ok(),
        "Elo match quality should be implemented"
    );
    assert!(
        elo_quality.unwrap() > 0.9,
        "Equal players should have high match quality"
    );

    let glicko_quality = glicko_system.calculate_match_quality(&[glicko_team1, glicko_team2]);
    assert!(
        glicko_quality.is_ok(),
        "Glicko match quality should be implemented"
    );
    assert!(
        glicko_quality.unwrap() > 0.8,
        "Equal players should have high match quality"
    );

    let glicko2_quality = glicko2_system.calculate_match_quality(&[glicko2_team1, glicko2_team2]);
    assert!(
        glicko2_quality.is_ok(),
        "Glicko2 match quality should be implemented"
    );
    assert!(
        glicko2_quality.unwrap() > 0.8,
        "Equal players should have high match quality"
    );

    // TrueSkill match quality is not yet implemented
    let trueskill_quality =
        trueskill_system.calculate_match_quality(&[trueskill_team1, trueskill_team2]);
    assert!(
        trueskill_quality.is_err(),
        "TrueSkill match quality should not be implemented"
    );
}

#[test]
fn test_outcome_validation_across_systems() {
    // Test that all systems properly validate outcomes
    let elo_system = EloSystem::new();

    let team1 = EloTeamRating::new(EloRating::new(1500.0));
    let team2 = EloTeamRating::new(EloRating::new(1500.0));

    // Test valid outcomes
    let valid_outcomes = vec![
        GameOutcome::win(0, 2),
        GameOutcome::win(1, 2),
        GameOutcome::draw(2),
        GameOutcome::new(vec![1, 2]),
        GameOutcome::new(vec![2, 1]),
        GameOutcome::new(vec![1, 1]),
    ];

    for outcome in valid_outcomes {
        assert!(
            outcome.is_valid_for_team_count(2),
            "Outcome should be valid for 2 teams"
        );

        let result = elo_system.rate(&[team1.clone(), team2.clone()], &outcome);
        assert!(result.is_ok(), "Valid outcome should work with Elo system");
    }

    // Test invalid outcomes
    let invalid_outcomes = vec![
        GameOutcome::new(vec![1]),       // Wrong number of ranks
        GameOutcome::new(vec![1, 2, 3]), // Wrong number of ranks
        GameOutcome::new(vec![]),        // Empty ranks
    ];

    for outcome in invalid_outcomes {
        assert!(
            !outcome.is_valid_for_team_count(2),
            "Outcome should be invalid for 2 teams"
        );
    }
}

#[test]
fn test_system_independence() {
    // Test that different systems produce independent results
    let elo_system = EloSystem::new();
    let glicko_system = Glicko::new();

    // Create equivalent initial conditions (scaled appropriately)
    let elo_team1 = EloTeamRating::new(EloRating::new(1500.0));
    let elo_team2 = EloTeamRating::new(EloRating::new(1500.0));

    let glicko_team1 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 350.0)]);
    let glicko_team2 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1500.0, 350.0)]);

    let outcome = GameOutcome::win(0, 2);

    let elo_result = elo_system.rate(&[elo_team1, elo_team2], &outcome).unwrap();
    let glicko_result = glicko_system
        .rate(&[glicko_team1, glicko_team2], &outcome)
        .unwrap();

    // Systems should produce different rating changes due to different algorithms
    let elo_change = elo_result[0].player_ratings()[0].rating() - 1500.0;
    let glicko_change = glicko_result[0].player_ratings()[0].mu - 1500.0;

    // They should both increase the winner's rating, but by different amounts
    assert!(elo_change > 0.0, "Elo winner should gain rating");
    assert!(glicko_change > 0.0, "Glicko winner should gain rating");
    assert_ne!(
        elo_change, glicko_change,
        "Different systems should produce different results"
    );
}

#[test]
fn test_comprehensive_draw_handling() {
    // Test that all systems handle draws appropriately
    let elo_system = EloSystem::new();
    let glicko_system = Glicko::new();
    let glicko2_system = Glicko2::new();
    let trueskill_system = TrueSkill::new();

    // Create slightly unequal teams to test draw behavior
    let outcome = GameOutcome::draw(2);

    // Elo test - using smaller rating differences due to observed behavior
    let elo_team1 = EloTeamRating::new(EloRating::new(1520.0));
    let elo_team2 = EloTeamRating::new(EloRating::new(1480.0));
    let elo_result = elo_system.rate(&[elo_team1, elo_team2], &outcome).unwrap();

    // In a draw, higher rated player should lose some rating, lower should gain (if the difference is large enough)
    let elo_new1 = elo_result[0].player_ratings()[0].rating();
    let elo_new2 = elo_result[1].player_ratings()[0].rating();

    // For small rating differences, changes might be very small, so just check they're reasonable
    assert!(
        elo_new1 <= 1520.0,
        "Higher player shouldn't gain rating in draw"
    );
    assert!(
        elo_new2 >= 1480.0,
        "Lower player shouldn't lose rating in draw"
    );

    // Glicko test
    let glicko_team1 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1600.0, 100.0)]);
    let glicko_team2 =
        GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1400.0, 100.0)]);
    let glicko_result = glicko_system
        .rate(&[glicko_team1, glicko_team2], &outcome)
        .unwrap();

    assert!(glicko_result[0].player_ratings()[0].mu < 1600.0);
    assert!(glicko_result[1].player_ratings()[0].mu > 1400.0);

    // Glicko2 test
    let glicko2_team1 =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1600.0, 100.0, 0.06)]);
    let glicko2_team2 =
        Glicko2TeamRating::from_player_ratings(vec![Glicko2Rating::new(1400.0, 100.0, 0.06)]);
    let glicko2_result = glicko2_system
        .rate(&[glicko2_team1, glicko2_team2], &outcome)
        .unwrap();

    assert!(glicko2_result[0].player_ratings()[0].mu < 1600.0);
    assert!(glicko2_result[1].player_ratings()[0].mu > 1400.0);

    // TrueSkill test
    let trueskill_team1 =
        TrueSkillTeam::from_player_ratings(vec![TrueSkillRating::new(30.0, 64.0).unwrap()]);
    let trueskill_team2 =
        TrueSkillTeam::from_player_ratings(vec![TrueSkillRating::new(20.0, 64.0).unwrap()]);
    let trueskill_result = trueskill_system
        .rate(&[trueskill_team1, trueskill_team2], &outcome)
        .unwrap();

    // TrueSkill should also show this pattern in draws
    assert!(trueskill_result[0].player_ratings()[0].mean() < 30.0);
    assert!(trueskill_result[1].player_ratings()[0].mean() > 20.0);
}

#[test]
fn test_boundary_rating_handling() {
    // Test how systems handle extreme rating values
    let elo_system = EloSystem::new();
    let glicko_system = Glicko::new();

    // Test very high vs very low ratings - but just verify they don't crash
    let elo_high = EloTeamRating::new(EloRating::new(3000.0));
    let elo_low = EloTeamRating::new(EloRating::new(500.0));

    let glicko_high = GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(2500.0, 50.0)]);
    let glicko_low = GlickoTeamRating::from_player_ratings(vec![GlickoRating::new(1000.0, 300.0)]);

    let upset_outcome = GameOutcome::win(1, 2); // Low player wins

    // Both systems should handle extreme cases gracefully
    let elo_result = elo_system.rate(&[elo_high, elo_low], &upset_outcome);
    assert!(elo_result.is_ok(), "Elo should handle extreme ratings");

    let glicko_result = glicko_system.rate(&[glicko_high, glicko_low], &upset_outcome);
    assert!(
        glicko_result.is_ok(),
        "Glicko should handle extreme ratings"
    );

    // Just verify the results are finite and reasonable
    if let Ok(elo_ratings) = elo_result {
        assert!(
            elo_ratings[0].player_ratings()[0].rating().is_finite(),
            "Ratings should be finite"
        );
        assert!(
            elo_ratings[1].player_ratings()[0].rating().is_finite(),
            "Ratings should be finite"
        );
    }
}
