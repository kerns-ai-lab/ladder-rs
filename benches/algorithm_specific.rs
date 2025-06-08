use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ladder_rs::{
    core::{GameOutcome, RatingSystem, TeamRating},
    elo::{EloSystem, EloRating, EloTeamRating},
    glicko::{Glicko, Glicko2, GlickoRating, Glicko2Rating, GlickoTeamRating, Glicko2TeamRating},
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam, TrueSkillImplementation},
};

/// Elo-specific component benchmarks
pub fn bench_elo_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("elo_components");
    
    let elo_system = EloSystem::new();
    let rating1 = EloRating::new(1600.0);
    let rating2 = EloRating::new(1400.0);
    
    // Benchmark expected score calculation
    group.bench_function("expected_score", |b| {
        b.iter(|| {
            let team1 = EloTeamRating::new(rating1.clone());
            let team2 = EloTeamRating::new(rating2.clone());
            let outcome = GameOutcome::win(0, 2);
            black_box(elo_system.rate(&[team1, team2], &outcome).unwrap())
        })
    });
    
    // Benchmark different K-factor scenarios
    for &k_factor in &[16.0, 32.0, 64.0] {
        let system = EloSystem::with_parameters(k_factor, 0.1, 200.0, 1500.0);
        group.bench_with_input(
            BenchmarkId::new("k_factor", k_factor as u32),
            &k_factor,
            |b, _| {
                b.iter(|| {
                    let team1 = EloTeamRating::new(rating1.clone());
                    let team2 = EloTeamRating::new(rating2.clone());
                    let outcome = GameOutcome::win(0, 2);
                    black_box(system.rate(&[team1, team2], &outcome).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

/// Glicko-specific component benchmarks
pub fn bench_glicko_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("glicko_components");
    
    let glicko_system = Glicko::new();
    let glicko2_system = Glicko2::new();
    
    // Test different rating deviation scenarios
    for &rd in &[50.0, 150.0, 350.0] {
        let rating1 = GlickoRating::new(1500.0, rd);
        let rating2 = GlickoRating::new(1500.0, rd);
        
        group.bench_with_input(
            BenchmarkId::new("glicko_rd", rd as u32),
            &rd,
            |b, _| {
                b.iter(|| {
                    let team1 = GlickoTeamRating::from_player_ratings(vec![rating1.clone()]);
                    let team2 = GlickoTeamRating::from_player_ratings(vec![rating2.clone()]);
                    let outcome = GameOutcome::win(0, 2);
                    black_box(glicko_system.rate(&[team1, team2], &outcome).unwrap())
                })
            },
        );
    }
    
    // Test Glicko2 volatility scenarios
    for &volatility in &[0.03, 0.06, 0.12] {
        let rating1 = Glicko2Rating::new(1500.0, 200.0, volatility);
        let rating2 = Glicko2Rating::new(1500.0, 200.0, volatility);
        
        group.bench_with_input(
            BenchmarkId::new("glicko2_volatility", (volatility * 1000.0) as u32),
            &volatility,
            |b, _| {
                b.iter(|| {
                    let team1 = Glicko2TeamRating::from_player_ratings(vec![rating1.clone()]);
                    let team2 = Glicko2TeamRating::from_player_ratings(vec![rating2.clone()]);
                    let outcome = GameOutcome::win(0, 2);
                    black_box(glicko2_system.rate(&[team1, team2], &outcome).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

/// TrueSkill-specific component benchmarks
pub fn bench_trueskill_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("trueskill_components");
    
    // Compare simplified vs factor graph implementations
    let ts_simplified = TrueSkill::new_simplified();
    let ts_factor_graph = TrueSkill::new_factor_graph();
    
    let rating1 = TrueSkillRating::new(25.0, (25.0_f64/3.0).powi(2)).unwrap();
    let rating2 = TrueSkillRating::new(25.0, (25.0_f64/3.0).powi(2)).unwrap();
    let outcome = GameOutcome::win(0, 2);
    
    group.bench_function("simplified_implementation", |b| {
        b.iter(|| {
            let team1 = TrueSkillTeam::from_player_ratings(vec![rating1.clone()]);
            let team2 = TrueSkillTeam::from_player_ratings(vec![rating2.clone()]);
            black_box(ts_simplified.rate(&[team1, team2], &outcome).unwrap())
        })
    });
    
    group.bench_function("factor_graph_implementation", |b| {
        b.iter(|| {
            let team1 = TrueSkillTeam::from_player_ratings(vec![rating1.clone()]);
            let team2 = TrueSkillTeam::from_player_ratings(vec![rating2.clone()]);
            black_box(ts_factor_graph.rate(&[team1, team2], &outcome).unwrap())
        })
    });
    
    // Test different skill gaps
    for &skill_gap in &[0.0, 5.0, 15.0, 25.0] {
        let rating_high = TrueSkillRating::new(25.0 + skill_gap, (25.0_f64/3.0).powi(2)).unwrap();
        let rating_low = TrueSkillRating::new(25.0 - skill_gap, (25.0_f64/3.0).powi(2)).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("skill_gap_simplified", skill_gap as u32),
            &skill_gap,
            |b, _| {
                b.iter(|| {
                    let team1 = TrueSkillTeam::from_player_ratings(vec![rating_high.clone()]);
                    let team2 = TrueSkillTeam::from_player_ratings(vec![rating_low.clone()]);
                    black_box(ts_simplified.rate(&[team1, team2], &outcome).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmarks for draw scenarios across algorithms
pub fn bench_draw_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_scenarios");
    
    // Elo draws
    let elo_system = EloSystem::new();
    let elo_rating = EloRating::new(1500.0);
    let draw_outcome = GameOutcome::new(vec![1, 1]); // Draw
    
    group.bench_function("elo_draw", |b| {
        b.iter(|| {
            let team1 = EloTeamRating::new(elo_rating.clone());
            let team2 = EloTeamRating::new(elo_rating.clone());
            black_box(elo_system.rate(&[team1, team2], &draw_outcome).unwrap())
        })
    });
    
    // Glicko draws
    let glicko_system = Glicko::new();
    let glicko_rating = GlickoRating::new(1500.0, 200.0);
    
    group.bench_function("glicko_draw", |b| {
        b.iter(|| {
            let team1 = GlickoTeamRating::from_player_ratings(vec![glicko_rating.clone()]);
            let team2 = GlickoTeamRating::from_player_ratings(vec![glicko_rating.clone()]);
            black_box(glicko_system.rate(&[team1, team2], &draw_outcome).unwrap())
        })
    });
    
    // TrueSkill draws
    let ts_system = TrueSkill::new_simplified();
    let ts_rating = TrueSkillRating::new(25.0, (25.0_f64/3.0).powi(2)).unwrap();
    
    group.bench_function("trueskill_draw", |b| {
        b.iter(|| {
            let team1 = TrueSkillTeam::from_player_ratings(vec![ts_rating.clone()]);
            let team2 = TrueSkillTeam::from_player_ratings(vec![ts_rating.clone()]);
            black_box(ts_system.rate(&[team1, team2], &draw_outcome).unwrap())
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_elo_components,
    bench_glicko_components,
    bench_trueskill_components,
    bench_draw_scenarios
);
criterion_main!(benches);