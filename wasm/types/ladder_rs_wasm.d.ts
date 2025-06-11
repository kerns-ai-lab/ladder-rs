
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

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_jsrating_free: (a: number, b: number) => void;
  readonly jsrating_new: (a: number, b: number, c: number) => void;
  readonly jsrating_new_unchecked: (a: number, b: number) => number;
  readonly jsrating_mean: (a: number) => number;
  readonly jsrating_variance: (a: number) => number;
  readonly jsrating_toJSON: (a: number, b: number) => void;
  readonly jsrating_fromJSON: (a: number, b: number, c: number) => void;
  readonly __wbg_jsplayer_free: (a: number, b: number) => void;
  readonly jsplayer_new: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly jsplayer_id: (a: number, b: number) => void;
  readonly jsplayer_name: (a: number, b: number) => void;
  readonly jsplayer_rating: (a: number) => number;
  readonly jsplayer_toJSON: (a: number, b: number) => void;
  readonly jsplayer_fromJSON: (a: number, b: number, c: number) => void;
  readonly __wbg_jsmatchconfig_free: (a: number, b: number) => void;
  readonly jsmatchconfig_new: (a: number, b: number, c: number) => number;
  readonly jsmatchconfig_algorithm: (a: number, b: number) => void;
  readonly jsmatchconfig_params: (a: number) => number;
  readonly __wbg_jsmatchresult_free: (a: number, b: number) => void;
  readonly jsmatchresult_new: (a: number, b: number, c: number, d: number) => number;
  readonly jsmatchresult_winner: (a: number, b: number) => void;
  readonly jsmatchresult_ratings: (a: number, b: number) => void;
  readonly jsmatchresult_toJSON: (a: number, b: number) => void;
  readonly jsmatchresult_fromJSON: (a: number, b: number, c: number) => void;
  readonly __wbg_jseloconfig_free: (a: number, b: number) => void;
  readonly jseloconfig_new: (a: number, b: number, c: number) => number;
  readonly jseloconfig_kFactor: (a: number) => number;
  readonly jseloconfig_initialRating: (a: number) => number;
  readonly jseloconfig_initialVariance: (a: number) => number;
  readonly __wbg_jstrueskillconfig_free: (a: number, b: number) => void;
  readonly jstrueskillconfig_new: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly jstrueskillconfig_initialMean: (a: number) => number;
  readonly jstrueskillconfig_initialStdDev: (a: number) => number;
  readonly jstrueskillconfig_beta: (a: number) => number;
  readonly jstrueskillconfig_tau: (a: number) => number;
  readonly jstrueskillconfig_drawProbability: (a: number) => number;
  readonly __wbg_jserror_free: (a: number, b: number) => void;
  readonly jserror_new: (a: number, b: number, c: number, d: number) => number;
  readonly jserror_message: (a: number, b: number) => void;
  readonly jserror_errorType: (a: number, b: number) => void;
  readonly jserror_toString: (a: number, b: number) => void;
  readonly __wbg_elorating_free: (a: number, b: number) => void;
  readonly elorating_new: (a: number) => number;
  readonly elorating_value: (a: number) => number;
  readonly elorating_serialize: (a: number, b: number) => void;
  readonly elorating_deserialize: (a: number, b: number, c: number) => void;
  readonly __wbg_matchresult_free: (a: number, b: number) => void;
  readonly __wbg_elosystem_free: (a: number, b: number) => void;
  readonly elosystem_new: () => number;
  readonly elosystem_with_parameters: (a: number, b: number) => number;
  readonly elosystem_k_factor: (a: number) => number;
  readonly elosystem_initial_rating: (a: number) => number;
  readonly elosystem_create_rating: (a: number) => number;
  readonly elosystem_create_rating_with_value: (a: number, b: number) => number;
  readonly elosystem_process_1v1: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly elosystem_win_probability: (a: number, b: number, c: number) => number;
  readonly elosystem_match_quality: (a: number, b: number, c: number) => number;
  readonly elosystem_serialize: (a: number, b: number) => void;
  readonly elosystem_deserialize: (a: number, b: number, c: number) => void;
  readonly elosystem_process_1v1_json: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wbg_eloutils_free: (a: number, b: number) => void;
  readonly eloutils_batch_process: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly eloutils_create_leaderboard: (a: number, b: number, c: number) => void;
  readonly eloutils_create_ratings_from_values: (a: number, b: number, c: number) => void;
  readonly elorating_from_json: (a: number, b: number, c: number) => void;
  readonly jsglickoconfig_new: (a: number, b: number, c: number) => number;
  readonly elorating_to_json: (a: number, b: number) => void;
  readonly __wbg_jsglickoconfig_free: (a: number, b: number) => void;
  readonly jsglickoconfig_c: (a: number) => number;
  readonly jsglickoconfig_initialDeviation: (a: number) => number;
  readonly jsglickoconfig_initialRating: (a: number) => number;
  readonly matchresult_player2_rating: (a: number) => number;
  readonly matchresult_player1_rating: (a: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_0: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_1: (a: number, b: number) => number;
  readonly __wbindgen_export_2: (a: number, b: number, c: number, d: number) => number;
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
