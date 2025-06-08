use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating},
    elo::{EloSystem, EloRating, EloTeamRating},
    glicko::{Glicko, Glicko2, GlickoRating, Glicko2Rating, GlickoTeamRating, Glicko2TeamRating},
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam, TrueSkillImplementation},
};
use std::collections::HashMap;

mod benchmark_utils;
use benchmark_utils::{TestDataGenerator, BenchmarkScenario, SkillDistribution, OutcomePattern};

/// Simulate a Swiss tournament system
pub fn bench_swiss_tournament(c: &mut Criterion) {
    let mut group = c.benchmark_group("swiss_tournament");
    
    for &player_count in &[16, 64, 256] {
        let mut data_gen = TestDataGenerator::new(42);
        
        // Elo Swiss tournament
        group.bench_with_input(
            BenchmarkId::new("elo", player_count),
            &player_count,
            |b, &player_count| {
                b.iter(|| {
                    let elo_system = EloSystem::new();
                    let mut ratings = data_gen.generate_elo_ratings(player_count, SkillDistribution::Normal);
                    
                    // Simulate 7 rounds of Swiss pairings
                    for _round in 0..7 {
                        // Pair players by rating (simplified Swiss)
                        for i in (0..player_count).step_by(2) {
                            if i + 1 < player_count {
                                let team1 = EloTeamRating::new(ratings[i].clone());
                                let team2 = EloTeamRating::new(ratings[i + 1].clone());
                                
                                // Simulate game outcome based on rating difference
                                let outcome = if ratings[i].mean() > ratings[i + 1].mean() {
                                    GameOutcome::win(0, 2)
                                } else {
                                    GameOutcome::win(1, 2)
                                };
                                
                                let result = elo_system.rate(&[team1, team2], &outcome).unwrap();
                                ratings[i] = result[0].player_ratings()[0].clone();
                                ratings[i + 1] = result[1].player_ratings()[0].clone();
                            }
                        }
                    }
                    
                    black_box(ratings)
                })
            },
        );
        
        // TrueSkill Swiss tournament
        group.bench_with_input(
            BenchmarkId::new("trueskill", player_count),
            &player_count,
            |b, &player_count| {
                b.iter(|| {
                    let ts_system = TrueSkill::new_simplified();
                    let mut ratings = data_gen.generate_trueskill_ratings(player_count, SkillDistribution::Normal);
                    
                    // Simulate 7 rounds of Swiss pairings
                    for _round in 0..7 {
                        for i in (0..player_count).step_by(2) {
                            if i + 1 < player_count {
                                let team1 = TrueSkillTeam::from_player_ratings(vec![ratings[i].clone()]);
                                let team2 = TrueSkillTeam::from_player_ratings(vec![ratings[i + 1].clone()]);
                                
                                let outcome = if ratings[i].mean() > ratings[i + 1].mean() {
                                    GameOutcome::win(0, 2)
                                } else {
                                    GameOutcome::win(1, 2)
                                };
                                
                                let result = ts_system.rate(&[team1, team2], &outcome).unwrap();
                                ratings[i] = result[0].player_ratings()[0].clone();
                                ratings[i + 1] = result[1].player_ratings()[0].clone();
                            }
                        }
                    }
                    
                    black_box(ratings)
                })
            },
        );
    }
    
    group.finish();
}

/// Simulate round-robin tournament
pub fn bench_round_robin(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_robin");
    
    for &player_count in &[8, 16, 32] {
        let mut data_gen = TestDataGenerator::new(42);
        
        group.bench_with_input(
            BenchmarkId::new("elo", player_count),
            &player_count,
            |b, &player_count| {
                b.iter(|| {
                    let elo_system = EloSystem::new();
                    let mut ratings = data_gen.generate_elo_ratings(player_count, SkillDistribution::Normal);
                    
                    // Every player plays every other player
                    for i in 0..player_count {
                        for j in (i + 1)..player_count {
                            let team1 = EloTeamRating::new(ratings[i].clone());
                            let team2 = EloTeamRating::new(ratings[j].clone());
                            
                            let outcome = if ratings[i].mean() > ratings[j].mean() {
                                GameOutcome::win(0, 2)
                            } else {
                                GameOutcome::win(1, 2)
                            };
                            
                            let result = elo_system.rate(&[team1, team2], &outcome).unwrap();
                            ratings[i] = result[0].player_ratings()[0].clone();
                            ratings[j] = result[1].player_ratings()[0].clone();
                        }
                    }
                    
                    black_box(ratings)
                })
            },
        );
    }
    
    group.finish();
}

/// Simulate matchmaking system finding balanced matches
pub fn bench_matchmaking_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("matchmaking");
    
    let mut data_gen = TestDataGenerator::new(42);
    
    for &pool_size in &[100, 500, 1000] {
        // Elo-based matchmaking
        group.bench_with_input(
            BenchmarkId::new("elo_matchmaking", pool_size),
            &pool_size,
            |b, &pool_size| {
                b.iter(|| {
                    let elo_system = EloSystem::new();
                    let mut ratings = data_gen.generate_elo_ratings(pool_size, SkillDistribution::Normal);
                    
                    // Sort by rating for matchmaking
                    ratings.sort_by(|a, b| a.mean().partial_cmp(&b.mean()).unwrap());
                    
                    // Create 50 balanced matches
                    let mut match_results = Vec::new();
                    for i in (0..std::cmp::min(100, pool_size)).step_by(2) {
                        if i + 1 < ratings.len() {
                            let team1 = EloTeamRating::new(ratings[i].clone());
                            let team2 = EloTeamRating::new(ratings[i + 1].clone());
                            
                            // Simulate balanced match (50/50 outcome)
                            let outcome = if i % 2 == 0 {
                                GameOutcome::win(0, 2)
                            } else {
                                GameOutcome::win(1, 2)
                            };
                            
                            let result = elo_system.rate(&[team1, team2], &outcome).unwrap();
                            match_results.push(result);
                        }
                    }
                    
                    black_box(match_results)
                })
            },
        );
        
        // TrueSkill-based matchmaking
        group.bench_with_input(
            BenchmarkId::new("trueskill_matchmaking", pool_size),
            &pool_size,
            |b, &pool_size| {
                b.iter(|| {
                    let ts_system = TrueSkill::new_simplified();
                    let mut ratings = data_gen.generate_trueskill_ratings(pool_size, SkillDistribution::Normal);
                    
                    // Sort by conservative rating for matchmaking
                    ratings.sort_by(|a, b| a.conservative_rating().partial_cmp(&b.conservative_rating()).unwrap());
                    
                    let mut match_results = Vec::new();
                    for i in (0..std::cmp::min(100, pool_size)).step_by(2) {
                        if i + 1 < ratings.len() {
                            let team1 = TrueSkillTeam::from_player_ratings(vec![ratings[i].clone()]);
                            let team2 = TrueSkillTeam::from_player_ratings(vec![ratings[i + 1].clone()]);
                            
                            let outcome = if i % 2 == 0 {
                                GameOutcome::win(0, 2)
                            } else {
                                GameOutcome::win(1, 2)
                            };
                            
                            let result = ts_system.rate(&[team1, team2], &outcome).unwrap();
                            match_results.push(result);
                        }
                    }
                    
                    black_box(match_results)
                })
            },
        );
    }
    
    group.finish();
}

/// Simulate new player rating convergence
pub fn bench_rating_convergence(c: &mut Criterion) {
    let mut group = c.benchmark_group("rating_convergence");
    
    // Simulate a new player playing 50 games against established players
    group.bench_function("elo_new_player", |b| {
        b.iter(|| {
            let elo_system = EloSystem::new();
            let mut new_player = EloRating::new(1500.0); // Default rating
            let mut data_gen = TestDataGenerator::new(42);
            let opponents = data_gen.generate_elo_ratings(50, SkillDistribution::Normal);
            
            for opponent in &opponents {
                let team1 = EloTeamRating::new(new_player.clone());
                let team2 = EloTeamRating::new(opponent.clone());
                
                // New player wins 70% against weaker, 30% against stronger
                let outcome = if new_player.mean() > opponent.mean() {
                    if data_gen.generate_outcomes(1, 2, OutcomePattern::Skill)[0].ranks()[0] <= 1 {
                        GameOutcome::win(0, 2)
                    } else {
                        GameOutcome::win(1, 2)
                    }
                } else {
                    GameOutcome::win(1, 2)
                };
                
                let result = elo_system.rate(&[team1, team2], &outcome).unwrap();
                new_player = result[0].player_ratings()[0].clone();
            }
            
            black_box(new_player)
        })
    });
    
    group.bench_function("trueskill_new_player", |b| {
        b.iter(|| {
            let ts_system = TrueSkill::new_simplified();
            let mut new_player = ts_system.create_rating();
            let mut data_gen = TestDataGenerator::new(42);
            let opponents = data_gen.generate_trueskill_ratings(50, SkillDistribution::Normal);
            
            for opponent in &opponents {
                let team1 = TrueSkillTeam::from_player_ratings(vec![new_player.clone()]);
                let team2 = TrueSkillTeam::from_player_ratings(vec![opponent.clone()]);
                
                let outcome = if new_player.mean() > opponent.mean() {
                    GameOutcome::win(0, 2)
                } else {
                    GameOutcome::win(1, 2)
                };
                
                let result = ts_system.rate(&[team1, team2], &outcome).unwrap();
                new_player = result[0].player_ratings()[0].clone();
            }
            
            black_box(new_player)
        })
    });
    
    group.finish();
}

/// Simulate esports league season
pub fn bench_esports_season(c: &mut Criterion) {
    let mut group = c.benchmark_group("esports_season");
    
    let mut data_gen = TestDataGenerator::new(42);
    
    group.bench_function("regular_season", |b| {
        b.iter(|| {
            let ts_system = TrueSkill::new_simplified();
            let team_count = 20;
            let players_per_team = 5;
            
            // Initialize teams
            let mut teams: Vec<Vec<TrueSkillRating>> = (0..team_count)
                .map(|_| data_gen.generate_trueskill_ratings(players_per_team, SkillDistribution::Normal))
                .collect();
            
            // Regular season: each team plays 10 matches
            for team_idx in 0..team_count {
                for _match_num in 0..10 {
                    let opponent_idx = (team_idx + _match_num + 1) % team_count;
                    if opponent_idx != team_idx {
                        let team1 = TrueSkillTeam::from_player_ratings(teams[team_idx].clone());
                        let team2 = TrueSkillTeam::from_player_ratings(teams[opponent_idx].clone());
                        
                        // Team with higher average skill wins 70% of time
                        let team1_avg: f64 = teams[team_idx].iter().map(|r| r.mean()).sum::<f64>() / players_per_team as f64;
                        let team2_avg: f64 = teams[opponent_idx].iter().map(|r| r.mean()).sum::<f64>() / players_per_team as f64;
                        
                        let outcome = if team1_avg > team2_avg && data_gen.generate_outcomes(1, 2, OutcomePattern::Skill)[0].ranks()[0] == 1 {
                            GameOutcome::win(0, 2)
                        } else if team2_avg > team1_avg && data_gen.generate_outcomes(1, 2, OutcomePattern::Skill)[0].ranks()[0] == 2 {
                            GameOutcome::win(1, 2)
                        } else {
                            // Upset or close match
                            if _match_num % 2 == 0 { GameOutcome::win(0, 2) } else { GameOutcome::win(1, 2) }
                        };
                        
                        let result = ts_system.rate(&[team1, team2], &outcome).unwrap();
                        teams[team_idx] = result[0].player_ratings().to_vec();
                        teams[opponent_idx] = result[1].player_ratings().to_vec();
                    }
                }
            }
            
            black_box(teams)
        })
    });
    
    group.finish();
}

/// Benchmark massive parallel tournament processing
pub fn bench_concurrent_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_processing");
    
    // Simulate processing many independent matches
    for &match_count in &[1000, 10000, 100000] {
        group.bench_with_input(
            BenchmarkId::new("independent_matches", match_count),
            &match_count,
            |b, &match_count| {
                b.iter(|| {
                    let elo_system = EloSystem::new();
                    let mut data_gen = TestDataGenerator::new(42);
                    
                    // Generate many independent matches
                    let mut results = Vec::with_capacity(match_count);
                    for _ in 0..match_count {
                        let ratings = data_gen.generate_elo_ratings(2, SkillDistribution::Normal);
                        let team1 = EloTeamRating::new(ratings[0].clone());
                        let team2 = EloTeamRating::new(ratings[1].clone());
                        let outcome = data_gen.generate_outcomes(1, 2, OutcomePattern::Skill)[0].clone();
                        
                        let result = elo_system.rate(&[team1, team2], &outcome).unwrap();
                        results.push(result);
                    }
                    
                    black_box(results)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory-intensive scenarios
pub fn bench_memory_intensive(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_intensive");
    
    // Maintain large number of player ratings in memory
    for &player_count in &[10000, 100000, 1000000] {
        group.bench_with_input(
            BenchmarkId::new("large_player_base", player_count),
            &player_count,
            |b, &player_count| {
                b.iter(|| {
                    let mut data_gen = TestDataGenerator::new(42);
                    let ratings = data_gen.generate_elo_ratings(player_count, SkillDistribution::Normal);
                    
                    // Simulate looking up and updating random players
                    let elo_system = EloSystem::new();
                    let mut updated_count = 0;
                    
                    for _ in 0..1000 { // 1000 random matches
                        let idx1 = data_gen.generate_outcomes(1, player_count, OutcomePattern::Balanced)[0].ranks()[0] % player_count;
                        let idx2 = (idx1 + 1) % player_count;
                        
                        let team1 = EloTeamRating::new(ratings[idx1].clone());
                        let team2 = EloTeamRating::new(ratings[idx2].clone());
                        let outcome = GameOutcome::win(0, 2);
                        
                        let _result = elo_system.rate(&[team1, team2], &outcome).unwrap();
                        updated_count += 1;
                    }
                    
                    black_box((ratings.len(), updated_count))
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_swiss_tournament,
    bench_round_robin,
    bench_matchmaking_system,
    bench_rating_convergence,
    bench_esports_season,
    bench_concurrent_processing,
    bench_memory_intensive
);
criterion_main!(benches);