#![allow(dead_code)] // This is a utility module for benchmarks

use ladder_rs::{core::GameOutcome, elo::EloRating, trueskill::TrueSkillRating};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::Duration;

/// Skill level distributions for realistic testing
#[derive(Debug, Clone, Copy)]
pub enum SkillDistribution {
    Uniform,  // Equal skill players
    Normal,   // Bell curve distribution
    Bimodal,  // Beginners + experts
    PowerLaw, // Few experts, many novices
    Extreme,  // Very wide skill gaps
}

/// Game outcome patterns
#[derive(Debug, Clone, Copy)]
pub enum OutcomePattern {
    Balanced, // 50/50 win rates
    Skill,    // Outcomes match skill
    Upset,    // Frequent upsets
    Draw,     // High draw probability
}

/// Benchmark scenario configuration
#[derive(Debug, Clone)]
pub struct BenchmarkScenario {
    pub player_count: usize,
    pub team_size: usize,
    pub game_count: usize,
    pub skill_distribution: SkillDistribution,
    pub outcome_pattern: OutcomePattern,
}

/// Performance metrics for benchmark analysis
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub throughput: f64,                     // Games per second
    pub latency_p50: Duration,               // Median latency
    pub latency_p99: Duration,               // 99th percentile
    pub memory_peak: usize,                  // Peak memory usage
    pub convergence_iterations: Option<f64>, // Avg iterations (TrueSkill)
}

/// Test data generator for benchmarks
pub struct TestDataGenerator {
    rng: StdRng,
}

impl TestDataGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Generate Elo ratings based on skill distribution
    pub fn generate_elo_ratings(
        &mut self,
        count: usize,
        distribution: SkillDistribution,
    ) -> Vec<EloRating> {
        match distribution {
            SkillDistribution::Uniform => (0..count).map(|_| EloRating::new(1500.0)).collect(),
            SkillDistribution::Normal => (0..count)
                .map(|_| {
                    let rating = self.rng.gen_range(800.0..2200.0);
                    EloRating::new(rating)
                })
                .collect(),
            SkillDistribution::Bimodal => {
                (0..count)
                    .map(|_| {
                        let rating = if self.rng.gen_bool(0.3) {
                            self.rng.gen_range(1800.0..2200.0) // Experts
                        } else {
                            self.rng.gen_range(800.0..1200.0) // Beginners
                        };
                        EloRating::new(rating)
                    })
                    .collect()
            }
            SkillDistribution::PowerLaw => {
                (0..count)
                    .map(|_| {
                        let percentile = self.rng.gen::<f64>().powf(2.0); // Power law distribution
                        let rating = 800.0 + (2200.0 - 800.0) * percentile;
                        EloRating::new(rating)
                    })
                    .collect()
            }
            SkillDistribution::Extreme => {
                (0..count)
                    .map(|_| {
                        let rating = if self.rng.gen_bool(0.1) {
                            self.rng.gen_range(2800.0..3200.0) // Super experts
                        } else {
                            self.rng.gen_range(400.0..800.0) // Very weak
                        };
                        EloRating::new(rating)
                    })
                    .collect()
            }
        }
    }

    /// Generate TrueSkill ratings based on skill distribution
    pub fn generate_trueskill_ratings(
        &mut self,
        count: usize,
        distribution: SkillDistribution,
    ) -> Vec<TrueSkillRating> {
        match distribution {
            SkillDistribution::Uniform => (0..count)
                .map(|_| TrueSkillRating::new(25.0, (25.0_f64 / 3.0).powi(2)).unwrap())
                .collect(),
            SkillDistribution::Normal => (0..count)
                .map(|_| {
                    let mean = self.rng.gen_range(10.0..40.0);
                    let variance = self.rng.gen_range(16.0..100.0);
                    TrueSkillRating::new(mean, variance).unwrap()
                })
                .collect(),
            SkillDistribution::Bimodal => {
                (0..count)
                    .map(|_| {
                        let (mean, variance) = if self.rng.gen_bool(0.3) {
                            (
                                self.rng.gen_range(35.0..45.0),
                                self.rng.gen_range(25.0..64.0),
                            ) // Experts
                        } else {
                            (
                                self.rng.gen_range(5.0..15.0),
                                self.rng.gen_range(64.0..144.0),
                            ) // Beginners
                        };
                        TrueSkillRating::new(mean, variance).unwrap()
                    })
                    .collect()
            }
            SkillDistribution::PowerLaw => (0..count)
                .map(|_| {
                    let percentile = self.rng.gen::<f64>().powf(2.0);
                    let mean = 5.0 + (45.0 - 5.0) * percentile;
                    let variance = 25.0 + (100.0 - 25.0) * (1.0 - percentile);
                    TrueSkillRating::new(mean, variance).unwrap()
                })
                .collect(),
            SkillDistribution::Extreme => {
                (0..count)
                    .map(|_| {
                        let (mean, variance) = if self.rng.gen_bool(0.1) {
                            (
                                self.rng.gen_range(45.0..60.0),
                                self.rng.gen_range(16.0..36.0),
                            ) // Super experts
                        } else {
                            (
                                self.rng.gen_range(1.0..5.0),
                                self.rng.gen_range(100.0..400.0),
                            ) // Very weak
                        };
                        TrueSkillRating::new(mean, variance).unwrap()
                    })
                    .collect()
            }
        }
    }

    /// Generate game outcomes based on pattern and player skills
    pub fn generate_outcomes(
        &mut self,
        games: usize,
        players_per_game: usize,
        pattern: OutcomePattern,
    ) -> Vec<GameOutcome> {
        (0..games)
            .map(|_| {
                match pattern {
                    OutcomePattern::Balanced => {
                        // Random outcomes regardless of skill
                        let mut ranks: Vec<usize> = (1..=players_per_game).collect();
                        self.shuffle_vec(&mut ranks);
                        GameOutcome::new(ranks)
                    }
                    OutcomePattern::Skill => {
                        // Outcomes based on skill (higher skill = better rank)
                        let ranks: Vec<usize> = (1..=players_per_game).collect();
                        GameOutcome::new(ranks)
                    }
                    OutcomePattern::Upset => {
                        // Frequent upsets (reverse skill order 30% of time)
                        let mut ranks: Vec<usize> = (1..=players_per_game).collect();
                        if self.rng.gen_bool(0.3) {
                            ranks.reverse();
                        }
                        GameOutcome::new(ranks)
                    }
                    OutcomePattern::Draw => {
                        // High probability of draws/ties
                        let rank = if self.rng.gen_bool(0.4) {
                            1 // Everyone tied for first
                        } else {
                            self.rng.gen_range(1..=players_per_game)
                        };
                        GameOutcome::new(vec![rank; players_per_game])
                    }
                }
            })
            .collect()
    }

    /// Generate team compositions for team games
    pub fn generate_teams(&mut self, team_count: usize, team_size: usize) -> Vec<Vec<usize>> {
        let total_players = team_count * team_size;
        let mut players: Vec<usize> = (0..total_players).collect();
        self.shuffle_vec(&mut players);

        players
            .chunks(team_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    fn shuffle_vec<T>(&mut self, vec: &mut [T]) {
        use rand::seq::SliceRandom;
        vec.shuffle(&mut self.rng);
    }
}

/// Benchmark scenario presets for common testing patterns
impl BenchmarkScenario {
    /// Quick micro-benchmark scenario
    pub fn micro() -> Self {
        Self {
            player_count: 2,
            team_size: 1,
            game_count: 1000,
            skill_distribution: SkillDistribution::Normal,
            outcome_pattern: OutcomePattern::Skill,
        }
    }

    /// Small tournament scenario
    pub fn small_tournament() -> Self {
        Self {
            player_count: 16,
            team_size: 1,
            game_count: 1000,
            skill_distribution: SkillDistribution::Normal,
            outcome_pattern: OutcomePattern::Skill,
        }
    }

    /// Large tournament scenario
    pub fn large_tournament() -> Self {
        Self {
            player_count: 128,
            team_size: 1,
            game_count: 10000,
            skill_distribution: SkillDistribution::Normal,
            outcome_pattern: OutcomePattern::Skill,
        }
    }

    /// Team-based scenario
    pub fn team_games() -> Self {
        Self {
            player_count: 20,
            team_size: 5,
            game_count: 1000,
            skill_distribution: SkillDistribution::Normal,
            outcome_pattern: OutcomePattern::Skill,
        }
    }

    /// Stress test scenario
    pub fn stress_test() -> Self {
        Self {
            player_count: 1000,
            team_size: 1,
            game_count: 100000,
            skill_distribution: SkillDistribution::PowerLaw,
            outcome_pattern: OutcomePattern::Upset,
        }
    }
}

/// Utility functions for benchmark setup and teardown
pub mod benchmark_helpers {
    use super::TestDataGenerator;
    use criterion::Criterion;
    use std::time::{Duration, Instant};

    /// Configure Criterion for consistent benchmarking
    pub fn criterion_config() -> Criterion {
        Criterion::default()
            .sample_size(50) // Balance between accuracy and speed
            .measurement_time(Duration::from_secs(5))
            .warm_up_time(Duration::from_secs(2))
    }

    /// Measure throughput for a batch operation
    pub fn measure_throughput<F>(operation: F, iterations: usize) -> f64
    where
        F: Fn(),
    {
        let start = Instant::now();
        for _ in 0..iterations {
            operation();
        }
        let elapsed = start.elapsed();
        iterations as f64 / elapsed.as_secs_f64()
    }

    /// Create a seeded test data generator for reproducible benchmarks
    pub fn seeded_generator() -> TestDataGenerator {
        TestDataGenerator::new(42) // Fixed seed for reproducibility
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_data_generator() {
        let mut gen = TestDataGenerator::new(42);

        // Test Elo rating generation
        let elo_ratings = gen.generate_elo_ratings(10, SkillDistribution::Normal);
        assert_eq!(elo_ratings.len(), 10);

        // Test TrueSkill rating generation
        let ts_ratings = gen.generate_trueskill_ratings(10, SkillDistribution::Normal);
        assert_eq!(ts_ratings.len(), 10);

        // Test outcome generation
        let outcomes = gen.generate_outcomes(5, 2, OutcomePattern::Balanced);
        assert_eq!(outcomes.len(), 5);
    }

    #[test]
    fn test_benchmark_scenarios() {
        let micro = BenchmarkScenario::micro();
        assert_eq!(micro.player_count, 2);
        assert_eq!(micro.game_count, 1000);

        let tournament = BenchmarkScenario::large_tournament();
        assert_eq!(tournament.player_count, 128);
        assert_eq!(tournament.game_count, 10000);
    }
}
