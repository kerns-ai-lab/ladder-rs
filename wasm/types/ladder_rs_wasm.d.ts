
// Utility types for ladder-rs WASM
export type PlayerId = string;
export type RatingValue = number;
export type MatchOutcome = 0 | 1 | 2; // 0=draw, 1=team1 wins, 2=team2 wins
export type Probability = number; // 0.0 to 1.0

// Configuration types for rating systems
export interface EloConfig {
  k_factor?: number;
}

export interface GlickoConfig {
  initial_volatility?: number;
}

export interface TrueSkillConfig {
  beta?: number;
  tau?: number;
}

export type RatingSystemConfigType = EloConfig | GlickoConfig | TrueSkillConfig;

// Error types for better error handling
export interface LadderRsError {
  message: string;
  code?: string;
}

// Match result types
export interface MatchResult {
  player_id: string;
  old_rating: number;
  new_rating: number;
  rating_change: number;
}

// Tournament types
export interface TournamentMatch {
  player1_id: PlayerId;
  player2_id: PlayerId;
  result: MatchOutcome;
  timestamp?: number;
}

// Statistics types
export interface PlayerStatistics {
  total_matches: number;
  wins: number;
  losses: number;
  draws: number;
  win_rate: number;
  current_streak: number;
  average_rating: number;
  peak_rating: number;
}

/* tslint:disable */
/* eslint-disable */
/**
 * JavaScript-friendly match outcome
 */
export enum JsOutcome {
  Win = 0,
  Loss = 1,
  Draw = 2,
}
/**
 * Match outcome for 1v1 games
 */
export enum MatchOutcome {
  Player1Win = 0,
  Player2Win = 1,
  Draw = 2,
}
/**
 * WASM-friendly Elo rating wrapper
 */
export class EloRating {
  free(): void;
  /**
   * Creates a new Elo rating with the specified value
   */
  constructor(value: number);
  /**
   * Serializes the rating to a string
   */
  serialize(): string;
  /**
   * Deserializes a rating from a string
   */
  static deserialize(data: string): EloRating;
  /**
   * Converts to JSON string for JavaScript interop
   */
  to_json(): string;
  /**
   * Creates from JSON string
   */
  static from_json(json: string): EloRating;
  /**
   * Gets the rating value
   */
  readonly value: number;
}
/**
 * WASM-friendly Elo system wrapper
 */
export class EloSystem {
  free(): void;
  /**
   * Creates a new Elo system with default parameters
   * Default: k_factor = 32.0, initial_rating = 1500.0
   */
  constructor();
  /**
   * Creates a new Elo system with custom parameters
   */
  static with_parameters(k_factor: number, initial_rating: number): EloSystem;
  /**
   * Creates a new rating with the default value
   */
  create_rating(): EloRating;
  /**
   * Creates a rating with a specific value
   */
  create_rating_with_value(value: number): EloRating;
  /**
   * Processes a 1v1 match and returns updated ratings
   */
  process_1v1(player1: EloRating, player2: EloRating, outcome: MatchOutcome): MatchResult;
  /**
   * Calculates the win probability for player1
   */
  win_probability(player1: EloRating, player2: EloRating): number;
  /**
   * Calculates match quality (0-1, higher is better)
   */
  match_quality(player1: EloRating, player2: EloRating): number;
  /**
   * Serializes the system configuration
   */
  serialize(): string;
  /**
   * Deserializes system configuration
   */
  static deserialize(data: string): EloSystem;
  /**
   * Processes a 1v1 match and returns updated ratings as JSON
   * Returns: {"player1": 1520, "player2": 1480}
   */
  process_1v1_json(player1_rating: number, player2_rating: number, outcome: MatchOutcome): string;
  /**
   * Gets the k-factor
   */
  readonly k_factor: number;
  /**
   * Gets the initial rating
   */
  readonly initial_rating: number;
}
/**
 * Utility functions for batch operations
 */
export class EloUtils {
  private constructor();
  free(): void;
  /**
   * Processes multiple matches in batch
   * Takes JSON strings: ratings array and matches array
   * Match data format: [[player1_idx, player2_idx, outcome], ...]
   * Returns updated ratings as JSON string
   */
  static batch_process(system: EloSystem, ratings_json: string, matches_json: string): string;
  /**
   * Creates a leaderboard from ratings JSON
   * Returns JSON array of [index, rating] sorted by rating descending
   */
  static create_leaderboard(ratings_json: string): string;
  /**
   * Helper to create a ratings array from values
   */
  static create_ratings_from_values(values_json: string): string;
}
/**
 * Result of a 1v1 match processing in Glicko
 */
export class GlickoMatchResult {
  private constructor();
  free(): void;
  /**
   * Gets the updated rating for player 1
   */
  readonly player1_rating: number;
  /**
   * Gets the updated RD for player 1
   */
  readonly player1_rd: number;
  /**
   * Gets the updated rating for player 2
   */
  readonly player2_rating: number;
  /**
   * Gets the updated RD for player 2
   */
  readonly player2_rd: number;
}
/**
 * WASM-friendly Glicko rating wrapper
 */
export class GlickoRating {
  free(): void;
  /**
   * Creates a new Glicko rating with the specified values
   */
  constructor(mu: number, rd: number);
  /**
   * Gets the conservative rating (μ - 2*RD)
   */
  conservative_rating(): number;
  /**
   * Serializes the rating to a string
   */
  serialize(): string;
  /**
   * Deserializes a rating from a string
   */
  static deserialize(data: string): GlickoRating;
  /**
   * Converts to JSON string for JavaScript interop
   */
  to_json(): string;
  /**
   * Creates from JSON string
   */
  static from_json(json: string): GlickoRating;
  /**
   * Gets the rating mean (μ)
   */
  readonly mu: number;
  /**
   * Gets the rating deviation (RD)
   */
  readonly rd: number;
}
/**
 * WASM-friendly Glicko system wrapper
 */
export class GlickoSystem {
  free(): void;
  /**
   * Creates a new Glicko system with default parameters
   * Default: c = 15.8, initial_rating = 1500.0, initial_rd = 350.0
   */
  constructor();
  /**
   * Creates a new Glicko system with custom parameters
   */
  static with_parameters(c: number, initial_rating: number, initial_rd: number): GlickoSystem;
  /**
   * Creates a new rating with the default values
   */
  create_rating(): GlickoRating;
  /**
   * Creates a rating with specific values
   */
  create_rating_with_values(mu: number, rd: number): GlickoRating;
  /**
   * Processes a 1v1 match and returns updated ratings
   */
  process_1v1(player1: GlickoRating, player2: GlickoRating, outcome: MatchOutcome): GlickoMatchResult;
  /**
   * Applies rating periods without matches (increases RD)
   */
  apply_rating_period(rating: GlickoRating, periods: number): GlickoRating;
  /**
   * Calculates the win probability for player1
   */
  win_probability(player1: GlickoRating, player2: GlickoRating): number;
  /**
   * Calculates match quality (0-1, higher is better)
   */
  match_quality(player1: GlickoRating, player2: GlickoRating): number;
  /**
   * Serializes the system configuration
   */
  serialize(): string;
  /**
   * Deserializes system configuration
   */
  static deserialize(data: string): GlickoSystem;
  /**
   * Processes a 1v1 match and returns updated ratings as JSON
   * Returns: {"player1": {"mu": 1520, "rd": 180}, "player2": {"mu": 1480, "rd": 190}}
   */
  process_1v1_json(player1_mu: number, player1_rd: number, player2_mu: number, player2_rd: number, outcome: MatchOutcome): string;
  /**
   * Gets the c parameter
   */
  readonly c: number;
  /**
   * Gets the initial rating
   */
  readonly initial_rating: number;
  /**
   * Gets the initial RD
   */
  readonly initial_rd: number;
}
/**
 * Utility functions for batch operations with Glicko
 */
export class GlickoUtils {
  private constructor();
  free(): void;
  /**
   * Processes multiple matches in batch
   * Takes JSON strings: ratings array and matches array
   * Match data format: [[player1_idx, player2_idx, outcome], ...]
   * Returns updated ratings as JSON string
   */
  static batch_process(system: GlickoSystem, ratings_json: string, matches_json: string): string;
  /**
   * Creates a leaderboard from ratings JSON
   * Returns JSON array of [index, rating, rd] sorted by rating descending
   */
  static create_leaderboard(ratings_json: string): string;
  /**
   * Helper to create a ratings array from values
   */
  static create_ratings_from_values(values_json: string): string;
}
/**
 * Configuration for Elo algorithm
 */
export class JsEloConfig {
  free(): void;
  /**
   * Create Elo configuration
   */
  constructor(k_factor: number, initial_rating: number, initial_variance: number);
  /**
   * Get K-factor
   */
  readonly kFactor: number;
  /**
   * Get initial rating
   */
  readonly initialRating: number;
  /**
   * Get initial variance
   */
  readonly initialVariance: number;
}
/**
 * Error type for WASM operations
 */
export class JsError {
  free(): void;
  /**
   * Create an error
   */
  constructor(message: string, error_type: string);
  /**
   * Convert to string
   */
  toString(): string;
  /**
   * Get error message
   */
  readonly message: string;
  /**
   * Get error type
   */
  readonly errorType: string;
}
/**
 * Configuration for Glicko algorithm
 */
export class JsGlickoConfig {
  free(): void;
  /**
   * Create Glicko configuration
   */
  constructor(initial_rating: number, initial_deviation: number, c: number);
  /**
   * Get initial rating
   */
  readonly initialRating: number;
  /**
   * Get initial deviation
   */
  readonly initialDeviation: number;
  /**
   * Get c constant
   */
  readonly c: number;
}
/**
 * Match configuration for different algorithms
 */
export class JsMatchConfig {
  free(): void;
  /**
   * Create match configuration
   */
  constructor(algorithm: string, params: any);
  /**
   * Get algorithm name
   */
  readonly algorithm: string;
  /**
   * Get parameters
   */
  readonly params: any;
}
/**
 * Match result between two players
 */
export class JsMatchResult {
  free(): void;
  /**
   * Create match result
   */
  constructor(winner: string | null | undefined, ratings: JsRating[]);
  /**
   * Convert to JSON string
   */
  toJSON(): string;
  /**
   * Create from JSON string
   */
  static fromJSON(json: string): JsMatchResult;
  /**
   * Get winner ID
   */
  readonly winner: string | undefined;
  /**
   * Get updated ratings
   */
  readonly ratings: JsRating[];
}
/**
 * JavaScript-friendly player representation
 */
export class JsPlayer {
  free(): void;
  /**
   * Create a new player
   */
  constructor(id: string, name: string | null | undefined, rating: JsRating);
  /**
   * Convert to JSON string
   */
  toJSON(): string;
  /**
   * Create from JSON string
   */
  static fromJSON(json: string): JsPlayer;
  /**
   * Get player ID
   */
  readonly id: string;
  /**
   * Get player name
   */
  readonly name: string | undefined;
  /**
   * Get player rating
   */
  readonly rating: JsRating;
}
/**
 * JavaScript-friendly rating representation
 */
export class JsRating {
  free(): void;
  /**
   * Create a new rating
   */
  constructor(mean: number, variance: number);
  /**
   * Create a new rating (for internal use, not exposed to JS)
   */
  static new_unchecked(mean: number, variance: number): JsRating;
  /**
   * Convert to JSON string
   */
  toJSON(): string;
  /**
   * Create from JSON string
   */
  static fromJSON(json: string): JsRating;
  /**
   * Get the mean value
   */
  readonly mean: number;
  /**
   * Get the variance value
   */
  readonly variance: number;
}
/**
 * Configuration for TrueSkill algorithm
 */
export class JsTrueSkillConfig {
  free(): void;
  /**
   * Create TrueSkill configuration
   */
  constructor(initial_mean: number, initial_std_dev: number, beta: number, tau: number, draw_probability: number);
  /**
   * Get initial mean
   */
  readonly initialMean: number;
  /**
   * Get initial standard deviation
   */
  readonly initialStdDev: number;
  /**
   * Get beta
   */
  readonly beta: number;
  /**
   * Get tau
   */
  readonly tau: number;
  /**
   * Get draw probability
   */
  readonly drawProbability: number;
}
/**
 * Result of a 1v1 match processing
 */
export class MatchResult {
  private constructor();
  free(): void;
  /**
   * Gets the updated rating for player 1
   */
  readonly player1_rating: number;
  /**
   * Gets the updated rating for player 2
   */
  readonly player2_rating: number;
}
/**
 * WASM-friendly TrueSkill rating wrapper
 */
export class TrueSkillRating {
  free(): void;
  /**
   * Creates a new TrueSkill rating with the specified values
   */
  constructor(mean: number, variance: number);
  /**
   * Gets the standard deviation (σ)
   */
  std_dev(): number;
  /**
   * Gets the conservative rating (μ - 3σ)
   */
  conservative_rating(): number;
  /**
   * Serializes the rating to a string
   */
  serialize(): string;
  /**
   * Deserializes a rating from a string
   */
  static deserialize(data: string): TrueSkillRating;
  /**
   * Converts to JSON string for JavaScript interop
   */
  to_json(): string;
  /**
   * Creates from JSON string
   */
  static from_json(json: string): TrueSkillRating;
  /**
   * Gets the rating mean (μ)
   */
  readonly mean: number;
  /**
   * Gets the rating variance (σ²)
   */
  readonly variance: number;
}
/**
 * WASM-friendly TrueSkill system wrapper
 */
export class TrueSkillSystem {
  free(): void;
  /**
   * Creates a new TrueSkill system with default parameters
   * Default: mu = 25.0, sigma = 8.333, beta = 4.166, tau = 0.0833, draw_prob = 0.1
   */
  constructor();
  /**
   * Creates a new TrueSkill system with custom parameters
   */
  static with_parameters(mu: number, sigma: number, beta: number, tau: number, draw_probability: number): TrueSkillSystem;
  /**
   * Gets the calculated draw margin
   */
  draw_margin(): number;
  /**
   * Creates a new rating with the default values
   */
  create_rating(): TrueSkillRating;
  /**
   * Creates a rating with specific values
   */
  create_rating_with_values(mean: number, variance: number): TrueSkillRating;
  /**
   * Processes a match and returns updated teams
   * ranks: array where ranks[i] is the rank of team i (1 = first place, 2 = second, etc.)
   */
  process_match(teams: Array<any>, ranks: Array<any>): Array<any>;
  /**
   * Calculates win probabilities for each team
   */
  win_probability(teams: Array<any>): Array<any>;
  /**
   * Calculates match quality (0-1, higher is better)
   */
  match_quality(teams: Array<any>): number;
  /**
   * Serializes the system configuration
   */
  serialize(): string;
  /**
   * Deserializes system configuration
   */
  static deserialize(data: string): TrueSkillSystem;
  /**
   * Gets the initial mean (μ)
   */
  readonly mu: number;
  /**
   * Gets the initial standard deviation (σ)
   */
  readonly sigma: number;
  /**
   * Gets the performance variance parameter (β)
   */
  readonly beta: number;
  /**
   * Gets the dynamics factor (τ)
   */
  readonly tau: number;
  /**
   * Gets the draw probability
   */
  readonly draw_probability: number;
}
/**
 * WASM-friendly TrueSkill team wrapper
 */
export class TrueSkillTeam {
  private constructor();
  free(): void;
  /**
   * Creates a team from player ratings
   */
  static from_ratings(ratings: TrueSkillRating[]): TrueSkillTeam;
  /**
   * Creates a team with partial play weights
   */
  static from_ratings_with_weights(ratings: TrueSkillRating[], weights: Float64Array): TrueSkillTeam;
  /**
   * Gets the number of players in the team
   */
  size(): number;
  /**
   * Gets the sum of all player means
   */
  mean_sum(): number;
  /**
   * Gets the sum of all player variances
   */
  variance_sum(): number;
  /**
   * Gets a copy of the ratings
   */
  ratings(): TrueSkillRating[];
  /**
   * Converts to JSON for serialization
   */
  to_json(): string;
}
/**
 * Utility functions for batch operations with TrueSkill
 */
export class TrueSkillUtils {
  private constructor();
  free(): void;
  /**
   * Processes multiple matches in batch
   * Takes JSON strings: ratings array and matches array
   * Match format: {"teams": [[player_indices], ...], "ranks": [1, 2, ...]}
   * Returns updated ratings as JSON string
   */
  static batch_process(system: TrueSkillSystem, ratings_json: string, matches_json: string): string;
  /**
   * Creates a leaderboard from ratings JSON
   * Returns JSON array of [index, mean, variance, conservative_rating] sorted by conservative rating
   * If use_conservative is true, sorts by conservative rating; otherwise by mean
   */
  static create_leaderboard(ratings_json: string, use_conservative: boolean): string;
  /**
   * Helper to create a ratings array from mean/variance pairs
   */
  static create_ratings_from_values(values_json: string): string;
  /**
   * Simulates a tournament and returns expected results
   * Takes ratings and number of simulations
   */
  static simulate_tournament(_system: TrueSkillSystem, teams_json: string, _num_simulations: number): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_elorating_free: (a: number, b: number) => void;
  readonly elorating_new: (a: number) => number;
  readonly elorating_value: (a: number) => number;
  readonly elorating_serialize: (a: number) => [number, number];
  readonly elorating_deserialize: (a: number, b: number) => [number, number, number];
  readonly elorating_to_json: (a: number) => [number, number];
  readonly elorating_from_json: (a: number, b: number) => [number, number, number];
  readonly __wbg_matchresult_free: (a: number, b: number) => void;
  readonly matchresult_player1_rating: (a: number) => number;
  readonly matchresult_player2_rating: (a: number) => number;
  readonly __wbg_elosystem_free: (a: number, b: number) => void;
  readonly elosystem_new: () => number;
  readonly elosystem_with_parameters: (a: number, b: number) => number;
  readonly elosystem_k_factor: (a: number) => number;
  readonly elosystem_initial_rating: (a: number) => number;
  readonly elosystem_create_rating: (a: number) => number;
  readonly elosystem_create_rating_with_value: (a: number, b: number) => number;
  readonly elosystem_process_1v1: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly elosystem_win_probability: (a: number, b: number, c: number) => number;
  readonly elosystem_match_quality: (a: number, b: number, c: number) => number;
  readonly elosystem_serialize: (a: number) => [number, number];
  readonly elosystem_deserialize: (a: number, b: number) => [number, number, number];
  readonly elosystem_process_1v1_json: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly __wbg_eloutils_free: (a: number, b: number) => void;
  readonly eloutils_batch_process: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
  readonly eloutils_create_leaderboard: (a: number, b: number) => [number, number, number, number];
  readonly eloutils_create_ratings_from_values: (a: number, b: number) => [number, number, number, number];
  readonly __wbg_trueskillrating_free: (a: number, b: number) => void;
  readonly trueskillrating_new: (a: number, b: number) => [number, number, number];
  readonly trueskillrating_mean: (a: number) => number;
  readonly trueskillrating_variance: (a: number) => number;
  readonly trueskillrating_std_dev: (a: number) => number;
  readonly trueskillrating_conservative_rating: (a: number) => number;
  readonly trueskillrating_serialize: (a: number) => [number, number];
  readonly trueskillrating_deserialize: (a: number, b: number) => [number, number, number];
  readonly trueskillrating_to_json: (a: number) => [number, number];
  readonly trueskillrating_from_json: (a: number, b: number) => [number, number, number];
  readonly __wbg_trueskillteam_free: (a: number, b: number) => void;
  readonly trueskillteam_from_ratings: (a: number, b: number) => [number, number, number];
  readonly trueskillteam_from_ratings_with_weights: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly trueskillteam_size: (a: number) => number;
  readonly trueskillteam_mean_sum: (a: number) => number;
  readonly trueskillteam_variance_sum: (a: number) => number;
  readonly trueskillteam_ratings: (a: number) => [number, number];
  readonly trueskillteam_to_json: (a: number) => [number, number, number, number];
  readonly __wbg_trueskillsystem_free: (a: number, b: number) => void;
  readonly trueskillsystem_new: () => number;
  readonly trueskillsystem_with_parameters: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
  readonly trueskillsystem_mu: (a: number) => number;
  readonly trueskillsystem_sigma: (a: number) => number;
  readonly trueskillsystem_beta: (a: number) => number;
  readonly trueskillsystem_tau: (a: number) => number;
  readonly trueskillsystem_draw_probability: (a: number) => number;
  readonly trueskillsystem_draw_margin: (a: number) => number;
  readonly trueskillsystem_create_rating: (a: number) => number;
  readonly trueskillsystem_create_rating_with_values: (a: number, b: number, c: number) => [number, number, number];
  readonly trueskillsystem_process_match: (a: number, b: any, c: any) => [number, number, number];
  readonly trueskillsystem_win_probability: (a: number, b: any) => [number, number, number];
  readonly trueskillsystem_match_quality: (a: number, b: any) => [number, number, number];
  readonly trueskillsystem_serialize: (a: number) => [number, number];
  readonly trueskillsystem_deserialize: (a: number, b: number) => [number, number, number];
  readonly __wbg_trueskillutils_free: (a: number, b: number) => void;
  readonly trueskillutils_batch_process: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
  readonly trueskillutils_create_leaderboard: (a: number, b: number, c: number) => [number, number, number, number];
  readonly trueskillutils_create_ratings_from_values: (a: number, b: number) => [number, number, number, number];
  readonly trueskillutils_simulate_tournament: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly __wbg_glickorating_free: (a: number, b: number) => void;
  readonly glickorating_new: (a: number, b: number) => [number, number, number];
  readonly glickorating_mu: (a: number) => number;
  readonly glickorating_rd: (a: number) => number;
  readonly glickorating_conservative_rating: (a: number) => number;
  readonly glickorating_serialize: (a: number) => [number, number];
  readonly glickorating_deserialize: (a: number, b: number) => [number, number, number];
  readonly glickorating_to_json: (a: number) => [number, number];
  readonly glickorating_from_json: (a: number, b: number) => [number, number, number];
  readonly __wbg_glickomatchresult_free: (a: number, b: number) => void;
  readonly glickomatchresult_player1_rating: (a: number) => number;
  readonly glickomatchresult_player1_rd: (a: number) => number;
  readonly glickomatchresult_player2_rating: (a: number) => number;
  readonly glickomatchresult_player2_rd: (a: number) => number;
  readonly __wbg_glickosystem_free: (a: number, b: number) => void;
  readonly glickosystem_new: () => number;
  readonly glickosystem_with_parameters: (a: number, b: number, c: number) => [number, number, number];
  readonly glickosystem_c: (a: number) => number;
  readonly glickosystem_initial_rating: (a: number) => number;
  readonly glickosystem_initial_rd: (a: number) => number;
  readonly glickosystem_create_rating: (a: number) => number;
  readonly glickosystem_create_rating_with_values: (a: number, b: number, c: number) => [number, number, number];
  readonly glickosystem_process_1v1: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly glickosystem_apply_rating_period: (a: number, b: number, c: number) => [number, number, number];
  readonly glickosystem_win_probability: (a: number, b: number, c: number) => number;
  readonly glickosystem_match_quality: (a: number, b: number, c: number) => number;
  readonly glickosystem_serialize: (a: number) => [number, number];
  readonly glickosystem_deserialize: (a: number, b: number) => [number, number, number];
  readonly glickosystem_process_1v1_json: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
  readonly __wbg_glickoutils_free: (a: number, b: number) => void;
  readonly glickoutils_batch_process: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
  readonly glickoutils_create_leaderboard: (a: number, b: number) => [number, number, number, number];
  readonly glickoutils_create_ratings_from_values: (a: number, b: number) => [number, number, number, number];
  readonly __wbg_jsrating_free: (a: number, b: number) => void;
  readonly jsrating_new: (a: number, b: number) => [number, number, number];
  readonly jsrating_new_unchecked: (a: number, b: number) => number;
  readonly jsrating_mean: (a: number) => number;
  readonly jsrating_variance: (a: number) => number;
  readonly jsrating_toJSON: (a: number) => [number, number, number, number];
  readonly jsrating_fromJSON: (a: number, b: number) => [number, number, number];
  readonly __wbg_jsplayer_free: (a: number, b: number) => void;
  readonly jsplayer_new: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly jsplayer_id: (a: number) => [number, number];
  readonly jsplayer_name: (a: number) => [number, number];
  readonly jsplayer_rating: (a: number) => number;
  readonly jsplayer_toJSON: (a: number) => [number, number, number, number];
  readonly jsplayer_fromJSON: (a: number, b: number) => [number, number, number];
  readonly __wbg_jsmatchconfig_free: (a: number, b: number) => void;
  readonly jsmatchconfig_new: (a: number, b: number, c: any) => number;
  readonly jsmatchconfig_algorithm: (a: number) => [number, number];
  readonly jsmatchconfig_params: (a: number) => any;
  readonly __wbg_jsmatchresult_free: (a: number, b: number) => void;
  readonly jsmatchresult_new: (a: number, b: number, c: number, d: number) => number;
  readonly jsmatchresult_winner: (a: number) => [number, number];
  readonly jsmatchresult_ratings: (a: number) => [number, number];
  readonly jsmatchresult_toJSON: (a: number) => [number, number, number, number];
  readonly jsmatchresult_fromJSON: (a: number, b: number) => [number, number, number];
  readonly __wbg_jseloconfig_free: (a: number, b: number) => void;
  readonly jseloconfig_new: (a: number, b: number, c: number) => number;
  readonly jseloconfig_kFactor: (a: number) => number;
  readonly jseloconfig_initialRating: (a: number) => number;
  readonly jseloconfig_initialVariance: (a: number) => number;
  readonly __wbg_jsglickoconfig_free: (a: number, b: number) => void;
  readonly jsglickoconfig_new: (a: number, b: number, c: number) => number;
  readonly jsglickoconfig_initialRating: (a: number) => number;
  readonly jsglickoconfig_initialDeviation: (a: number) => number;
  readonly jsglickoconfig_c: (a: number) => number;
  readonly __wbg_jstrueskillconfig_free: (a: number, b: number) => void;
  readonly jstrueskillconfig_new: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly jstrueskillconfig_initialMean: (a: number) => number;
  readonly jstrueskillconfig_initialStdDev: (a: number) => number;
  readonly jstrueskillconfig_beta: (a: number) => number;
  readonly jstrueskillconfig_tau: (a: number) => number;
  readonly jstrueskillconfig_drawProbability: (a: number) => number;
  readonly __wbg_jserror_free: (a: number, b: number) => void;
  readonly jserror_new: (a: number, b: number, c: number, d: number) => number;
  readonly jserror_message: (a: number) => [number, number];
  readonly jserror_errorType: (a: number) => [number, number];
  readonly jserror_toString: (a: number) => [number, number];
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __externref_drop_slice: (a: number, b: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

// Type assertions for runtime validation
export declare function isWasmRating(obj: any): obj is WasmRating;
export declare function isValidPlayerId(id: any): id is PlayerId;
export declare function isValidProbability(p: any): p is Probability;
export declare function isMatchOutcome(outcome: any): outcome is MatchOutcome;

// Enhanced async types for WebAssembly initialization
export interface WasmInitOptions {
  module?: WebAssembly.Module | BufferSource | Response | Promise<WebAssembly.Module | BufferSource | Response>;
  memory?: WebAssembly.Memory;
  instantiateStreaming?: boolean;
}

export interface WasmInitResult extends InitOutput {
  initialized: boolean;
  moduleSize: number;
}

// Promise-based initialization wrapper
export declare function initializeWasm(options?: WasmInitOptions): Promise<WasmInitResult>;

// Backward compatibility aliases
export type JsRatingValue = WasmRating;
export type JsTeamValue = WasmTeam;
export type JsSystemValue = WasmRatingSystem;

// Legacy interface support
export interface LegacyEloConfig {
  k_factor?: number;
  initial_rating?: number;
}

// Convert legacy config to new format
export declare function convertLegacyConfig(legacy: LegacyEloConfig): EloConfig;
