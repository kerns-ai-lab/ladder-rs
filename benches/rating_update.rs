use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ladder_rs::{
    core::{GameOutcome, RatingSystem, TeamRating},
    elo::{EloSystem, EloRating, EloTeamRating},
    glicko::{Glicko, Glicko2, GlickoRating, Glicko2Rating, GlickoTeamRating, Glicko2TeamRating},
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
};

mod benchmark_utils;
use benchmark_utils::{TestDataGenerator, SkillDistribution, OutcomePattern};

/// Micro-benchmarks for rating system creation and initialization
pub fn bench_rating_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("rating_creation");
    
    group.bench_function("elo_system", |b| {
        b.iter(|| black_box(EloSystem::new()))
    });
    
    group.bench_function("elo_rating", |b| {
        b.iter(|| black_box(EloRating::new(1500.0)))
    });
    
    group.bench_function("glicko_system", |b| {
        b.iter(|| black_box(Glicko::new()))
    });
    
    group.bench_function("glicko2_system", |b| {
        b.iter(|| black_box(Glicko2::new()))
    });
    
    group.bench_function("glicko_rating", |b| {
        b.iter(|| black_box(GlickoRating::new(1500.0, 350.0)))
    });
    
    group.bench_function("glicko2_rating", |b| {
        b.iter(|| black_box(Glicko2Rating::new(1500.0, 350.0, 0.06)))
    });
    
    group.bench_function("trueskill_simplified", |b| {
        b.iter(|| black_box(TrueSkill::new_simplified()))
    });
    
    group.bench_function("trueskill_factor_graph", |b| {
        b.iter(|| black_box(TrueSkill::new_factor_graph()))
    });
    
    group.bench_function("trueskill_rating", |b| {
        b.iter(|| black_box(TrueSkillRating::new(25.0, (25.0_f64/3.0).powi(2)).unwrap()))
    });
    
    group.finish();
}

/// Micro-benchmarks for single game rating updates
pub fn bench_single_game_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_game_update");
    
    // Elo single game
    let elo_system = EloSystem::new();
    let elo_p1 = EloRating::new(1500.0);
    let elo_p2 = EloRating::new(1500.0);
    let elo_team1 = EloTeamRating::new(elo_p1);
    let elo_team2 = EloTeamRating::new(elo_p2);
    let outcome = GameOutcome::win(0, 2);
    
    group.bench_function("elo", |b| {
        b.iter(|| {
            black_box(elo_system.rate(&[elo_team1.clone(), elo_team2.clone()], &outcome).unwrap())
        })
    });
    
    // Glicko single game
    let glicko_system = Glicko::new();
    let glicko_p1 = GlickoRating::new(1500.0, 350.0);
    let glicko_p2 = GlickoRating::new(1500.0, 350.0);
    let glicko_team1 = GlickoTeamRating::from_player_ratings(vec![glicko_p1]);
    let glicko_team2 = GlickoTeamRating::from_player_ratings(vec![glicko_p2]);
    
    group.bench_function("glicko", |b| {
        b.iter(|| {
            black_box(glicko_system.rate(&[glicko_team1.clone(), glicko_team2.clone()], &outcome).unwrap())
        })
    });
    
    // Glicko2 single game
    let glicko2_system = Glicko2::new();
    let glicko2_p1 = Glicko2Rating::new(1500.0, 350.0, 0.06);
    let glicko2_p2 = Glicko2Rating::new(1500.0, 350.0, 0.06);
    let glicko2_team1 = Glicko2TeamRating::from_player_ratings(vec![glicko2_p1]);
    let glicko2_team2 = Glicko2TeamRating::from_player_ratings(vec![glicko2_p2]);
    
    group.bench_function("glicko2", |b| {
        b.iter(|| {
            black_box(glicko2_system.rate(&[glicko2_team1.clone(), glicko2_team2.clone()], &outcome).unwrap())
        })
    });
    
    // TrueSkill simplified single game
    let ts_simplified = TrueSkill::new_simplified();
    let ts_p1 = TrueSkillRating::new(25.0, (25.0_f64/3.0).powi(2)).unwrap();
    let ts_p2 = TrueSkillRating::new(25.0, (25.0_f64/3.0).powi(2)).unwrap();
    let ts_team1 = TrueSkillTeam::from_player_ratings(vec![ts_p1]);
    let ts_team2 = TrueSkillTeam::from_player_ratings(vec![ts_p2]);
    
    group.bench_function("trueskill_simplified", |b| {
        b.iter(|| {
            black_box(ts_simplified.rate(&[ts_team1.clone(), ts_team2.clone()], &outcome).unwrap())
        })
    });
    
    // TrueSkill factor graph single game
    let ts_factor_graph = TrueSkill::new_factor_graph();
    
    group.bench_function("trueskill_factor_graph", |b| {
        b.iter(|| {
            black_box(ts_factor_graph.rate(&[ts_team1.clone(), ts_team2.clone()], &outcome).unwrap())
        })
    });
    
    group.finish();
}

/// Benchmarks for batch processing performance
pub fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");
    
    let mut data_gen = TestDataGenerator::new(42);
    
    for &game_count in &[100, 1000, 10000] {
        // Elo batch processing
        let elo_system = EloSystem::new();
        let elo_ratings = data_gen.generate_elo_ratings(100, SkillDistribution::Normal);
        let outcomes = data_gen.generate_outcomes(game_count, 2, OutcomePattern::Skill);
        
        group.bench_with_input(
            BenchmarkId::new("elo", game_count),
            &game_count,
            |b, _| {
                b.iter(|| {
                    for outcome in &outcomes {
                        let team1 = EloTeamRating::new(elo_ratings[0].clone());
                        let team2 = EloTeamRating::new(elo_ratings[1].clone());
                        black_box(elo_system.rate(&[team1, team2], outcome).unwrap());
                    }
                })
            },
        );
        
        // TrueSkill simplified batch processing
        let ts_system = TrueSkill::new_simplified();
        let ts_ratings = data_gen.generate_trueskill_ratings(100, SkillDistribution::Normal);
        
        group.bench_with_input(
            BenchmarkId::new("trueskill_simplified", game_count),
            &game_count,
            |b, _| {
                b.iter(|| {
                    for outcome in &outcomes {
                        let team1 = TrueSkillTeam::from_player_ratings(vec![ts_ratings[0].clone()]);
                        let team2 = TrueSkillTeam::from_player_ratings(vec![ts_ratings[1].clone()]);
                        black_box(ts_system.rate(&[team1, team2], outcome).unwrap());
                    }
                })
            },
        );
        
        // TrueSkill factor graph batch processing
        let ts_fg_system = TrueSkill::new_factor_graph();
        
        group.bench_with_input(
            BenchmarkId::new("trueskill_factor_graph", game_count),
            &game_count,
            |b, _| {
                b.iter(|| {
                    for outcome in &outcomes {
                        let team1 = TrueSkillTeam::from_player_ratings(vec![ts_ratings[0].clone()]);
                        let team2 = TrueSkillTeam::from_player_ratings(vec![ts_ratings[1].clone()]);
                        black_box(ts_fg_system.rate(&[team1, team2], outcome).unwrap());
                    }
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmarks for different player counts
pub fn bench_player_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("player_scaling");
    
    let mut data_gen = TestDataGenerator::new(42);
    
    for &player_count in &[2, 4, 8, 16, 32] {
        // Only test systems that support multi-player
        let elo_system = EloSystem::new();
        let elo_ratings = data_gen.generate_elo_ratings(player_count, SkillDistribution::Normal);
        let outcome = GameOutcome::new((1..=player_count).collect());
        
        // Note: Current Elo implementation only supports 2 players
        if player_count == 2 {
            group.bench_with_input(
                BenchmarkId::new("elo", player_count),
                &player_count,
                |b, _| {
                    b.iter(|| {
                        let team1 = EloTeamRating::new(elo_ratings[0].clone());
                        let team2 = EloTeamRating::new(elo_ratings[1].clone());
                        black_box(elo_system.rate(&[team1, team2], &outcome).unwrap())
                    })
                },
            );
        }
        
        // TrueSkill can handle multiple players (though current implementation is limited to 2)
        if player_count == 2 {
            let ts_system = TrueSkill::new_simplified();
            let ts_ratings = data_gen.generate_trueskill_ratings(player_count, SkillDistribution::Normal);
            
            group.bench_with_input(
                BenchmarkId::new("trueskill", player_count),
                &player_count,
                |b, _| {
                    b.iter(|| {
                        let team1 = TrueSkillTeam::from_player_ratings(vec![ts_ratings[0].clone()]);
                        let team2 = TrueSkillTeam::from_player_ratings(vec![ts_ratings[1].clone()]);
                        black_box(ts_system.rate(&[team1, team2], &outcome).unwrap())
                    })
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmarks for memory allocation patterns
pub fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Benchmark rating object creation (potential allocations)
    group.bench_function("rating_object_creation", |b| {
        b.iter(|| {
            let _elo = black_box(EloRating::new(1500.0));
            let _glicko = black_box(GlickoRating::new(1500.0, 350.0));
            let _glicko2 = black_box(Glicko2Rating::new(1500.0, 350.0, 0.06));
            let _trueskill = black_box(TrueSkillRating::new(25.0, (25.0_f64/3.0).powi(2)).unwrap());
        })
    });
    
    // Benchmark team creation (collections)
    group.bench_function("team_creation", |b| {
        let ratings = vec![
            TrueSkillRating::new(25.0, (25.0_f64/3.0).powi(2)).unwrap(),
            TrueSkillRating::new(24.0, (25.0_f64/3.0).powi(2)).unwrap(),
            TrueSkillRating::new(26.0, (25.0_f64/3.0).powi(2)).unwrap(),
        ];
        
        b.iter(|| {
            black_box(TrueSkillTeam::from_player_ratings(ratings.clone()))
        })
    });
    
    // Benchmark outcome creation
    group.bench_function("outcome_creation", |b| {
        b.iter(|| {
            black_box(GameOutcome::new(vec![1, 2, 3, 4]))
        })
    });
    
    group.finish();
}

criterion_group!(
    benches, 
    bench_rating_creation,
    bench_single_game_update,
    bench_batch_processing,
    bench_player_scaling,
    bench_memory_usage
);
criterion_main!(benches);
