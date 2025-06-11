//! Performance benchmarks for WASM bindings
//! 
//! This module provides Criterion-based benchmarks for the WASM module
//! to track performance regressions in native Rust (pre-WASM) code.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ladder_rs::{
    core::{GameOutcome, RatingSystem},
    elo::{EloRating, EloSystem, EloTeamRating},
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
};
use serde_json;
use std::collections::HashMap;

/// Benchmark WASM-specific serialization overhead
fn bench_wasm_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_serialization");
    
    // Create test data
    let elo_rating = EloRating::new(1500.0);
    let trueskill_rating = TrueSkillRating::new(25.0, (25.0 / 3.0).powi(2)).unwrap();
    
    // Benchmark JSON serialization (used by WASM bindings)
    group.bench_function("elo_to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&elo_rating).unwrap();
            black_box(json)
        })
    });
    
    group.bench_function("elo_from_json", |b| {
        let json = serde_json::to_string(&elo_rating).unwrap();
        b.iter(|| {
            let rating: EloRating = serde_json::from_str(&json).unwrap();
            black_box(rating)
        })
    });
    
    group.bench_function("trueskill_to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&trueskill_rating).unwrap();
            black_box(json)
        })
    });
    
    group.bench_function("trueskill_from_json", |b| {
        let json = serde_json::to_string(&trueskill_rating).unwrap();
        b.iter(|| {
            let rating: TrueSkillRating = serde_json::from_str(&json).unwrap();
            black_box(rating)
        })
    });
    
    // Benchmark batch serialization
    let players: Vec<(String, EloRating)> = (0..100)
        .map(|i| (format!("player{}", i), EloRating::new(1500.0 + i as f64)))
        .collect();
    
    group.bench_function("batch_100_players_to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&players).unwrap();
            black_box(json)
        })
    });
    
    group.finish();
}

/// Benchmark WASM interface patterns
fn bench_wasm_interface_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_interface");
    
    // Simulate WASM-style player management
    #[derive(Clone)]
    struct WasmPlayer {
        id: String,
        rating: f64,
        metadata: HashMap<String, String>,
    }
    
    impl WasmPlayer {
        fn new(id: String, rating: f64) -> Self {
            Self {
                id,
                rating,
                metadata: HashMap::new(),
            }
        }
    }
    
    // Benchmark player pool operations
    group.bench_function("create_player_pool", |b| {
        b.iter(|| {
            let players: Vec<WasmPlayer> = (0..100)
                .map(|i| WasmPlayer::new(format!("player{}", i), 1500.0))
                .collect();
            black_box(players)
        })
    });
    
    // Benchmark match result processing
    group.bench_function("process_match_results", |b| {
        let elo_system = EloSystem::new();
        let players: Vec<(String, EloRating)> = (0..20)
            .map(|i| (format!("player{}", i), EloRating::new(1500.0)))
            .collect();
        
        b.iter(|| {
            for i in 0..10 {
                let p1_idx = i * 2;
                let p2_idx = i * 2 + 1;
                
                let team1 = EloTeamRating::new(players[p1_idx].1.clone());
                let team2 = EloTeamRating::new(players[p2_idx].1.clone());
                let outcome = GameOutcome::win(0, 2);
                
                let _ = black_box(elo_system.rate(&[team1, team2], &outcome).unwrap());
            }
        })
    });
    
    group.finish();
}

/// Benchmark memory allocation patterns in WASM context
fn bench_wasm_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_memory");
    
    // Benchmark string allocations (common in WASM interfaces)
    group.bench_function("string_id_allocations", |b| {
        b.iter(|| {
            let ids: Vec<String> = (0..1000)
                .map(|i| format!("player_{}_with_long_identifier", i))
                .collect();
            black_box(ids)
        })
    });
    
    // Benchmark HashMap operations (used for metadata)
    group.bench_function("metadata_operations", |b| {
        b.iter(|| {
            let mut metadata = HashMap::new();
            for i in 0..100 {
                metadata.insert(format!("key{}", i), format!("value{}", i));
            }
            black_box(metadata)
        })
    });
    
    // Benchmark vector operations (team compositions)
    group.bench_function("team_composition_operations", |b| {
        let player_ids: Vec<String> = (0..100).map(|i| format!("player{}", i)).collect();
        
        b.iter(|| {
            let teams: Vec<Vec<String>> = player_ids
                .chunks(5)
                .map(|chunk| chunk.to_vec())
                .collect();
            black_box(teams)
        })
    });
    
    group.finish();
}

/// Benchmark WASM-specific algorithm adaptations
fn bench_wasm_algorithm_adaptations(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_algorithms");
    
    // Test performance with different batch sizes
    for &batch_size in &[10, 100, 1000] {
        // Elo batch processing
        let elo_system = EloSystem::new();
        let matches: Vec<(usize, usize, GameOutcome)> = (0..batch_size)
            .map(|i| {
                let p1 = i % 20;
                let p2 = (i + 1) % 20;
                let outcome = if i % 2 == 0 { 
                    GameOutcome::win(0, 2) 
                } else { 
                    GameOutcome::win(1, 2) 
                };
                (p1, p2, outcome)
            })
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("elo_batch", batch_size),
            &batch_size,
            |b, _| {
                let mut ratings: Vec<EloRating> = (0..20)
                    .map(|i| EloRating::new(1500.0 + i as f64 * 10.0))
                    .collect();
                
                b.iter(|| {
                    for &(p1_idx, p2_idx, ref outcome) in &matches {
                        let team1 = EloTeamRating::new(ratings[p1_idx].clone());
                        let team2 = EloTeamRating::new(ratings[p2_idx].clone());
                        
                        let updated = elo_system.rate(&[team1, team2], outcome).unwrap();
                        
                        ratings[p1_idx] = updated[0].players()[0].clone();
                        ratings[p2_idx] = updated[1].players()[0].clone();
                    }
                    black_box(&ratings)
                })
            },
        );
        
        // TrueSkill batch processing
        let ts_system = TrueSkill::new_simplified();
        
        group.bench_with_input(
            BenchmarkId::new("trueskill_batch", batch_size),
            &batch_size,
            |b, _| {
                let mut ratings: Vec<TrueSkillRating> = (0..20)
                    .map(|i| {
                        TrueSkillRating::new(
                            25.0 + i as f64 * 0.5,
                            (25.0 / 3.0).powi(2),
                        )
                        .unwrap()
                    })
                    .collect();
                
                b.iter(|| {
                    for &(p1_idx, p2_idx, ref outcome) in &matches {
                        let team1 = TrueSkillTeam::from_player_ratings(vec![ratings[p1_idx].clone()]);
                        let team2 = TrueSkillTeam::from_player_ratings(vec![ratings[p2_idx].clone()]);
                        
                        let updated = ts_system.rate(&[team1, team2], outcome).unwrap();
                        
                        ratings[p1_idx] = updated[0].players()[0].clone();
                        ratings[p2_idx] = updated[1].players()[0].clone();
                    }
                    black_box(&ratings)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark performance critical paths identified in WASM usage
fn bench_wasm_critical_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_critical_paths");
    
    // Benchmark tournament bracket generation
    group.bench_function("tournament_bracket_32", |b| {
        let players: Vec<String> = (0..32).map(|i| format!("player{}", i)).collect();
        
        b.iter(|| {
            let mut bracket = Vec::new();
            for i in (0..32).step_by(2) {
                bracket.push((players[i].clone(), players[i + 1].clone()));
            }
            black_box(bracket)
        })
    });
    
    // Benchmark leaderboard sorting
    group.bench_function("leaderboard_sort_1000", |b| {
        let mut players: Vec<(String, f64)> = (0..1000)
            .map(|i| (format!("player{}", i), 1200.0 + (i as f64 * 3.7) % 800.0))
            .collect();
        
        b.iter(|| {
            players.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            black_box(&players)
        })
    });
    
    // Benchmark match quality calculations
    group.bench_function("match_quality_calculations", |b| {
        let ts_system = TrueSkill::new_simplified();
        let ratings: Vec<TrueSkillRating> = (0..20)
            .map(|i| {
                TrueSkillRating::new(
                    20.0 + i as f64,
                    (25.0 / 3.0).powi(2) * (1.0 + i as f64 * 0.1),
                )
                .unwrap()
            })
            .collect();
        
        b.iter(|| {
            let mut qualities = Vec::new();
            for i in 0..20 {
                for j in (i + 1)..20 {
                    let team1 = TrueSkillTeam::from_player_ratings(vec![ratings[i].clone()]);
                    let team2 = TrueSkillTeam::from_player_ratings(vec![ratings[j].clone()]);
                    
                    let quality = ts_system.match_quality(&[team1, team2]).unwrap();
                    qualities.push(quality);
                }
            }
            black_box(qualities)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_wasm_serialization,
    bench_wasm_interface_patterns,
    bench_wasm_memory_patterns,
    bench_wasm_algorithm_adaptations,
    bench_wasm_critical_paths
);
criterion_main!(benches);