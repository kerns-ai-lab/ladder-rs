
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
export function wasm_main(): void;
export function greet(name: string): void;
/**
 * Rating system type enumeration
 */
/**
 * Rating system type enumeration
 * 
 * Defines the available rating system algorithms.
 * Each has different characteristics and use cases.
 * 
 * - Elo: Simple, fast, good for 1v1 games
 * - Glicko: Includes rating reliability/uncertainty
 * - TrueSkill: Microsoft's system, supports teams and draws
 */
export enum RatingSystemType {
  Elo = 0,
  Glicko = 1,
  Glicko2 = 2,
  TrueSkill = 3,
}
/**
 * JavaScript-friendly game outcome representation
 */
export class JsGameOutcome {
  free(): void;
  /**
   * Creates a new game outcome from ranks
   */
  constructor();
  /**
   * Sets ranks from a JavaScript array
   */
  set_ranks(ranks: Uint32Array | number[]): void;
  /**
   * Creates a win outcome (team at index wins)
   */
  static win(winner_index: number, total_teams: number): JsGameOutcome;
  /**
   * Creates a draw outcome between all teams
   */
  static draw(total_teams: number): JsGameOutcome;
  /**
   * Gets the rank for a specific team
   */
  get_rank(team_index: number): number | undefined;
  /**
   * Creates a string representation
   */
  toString(): string;
  /**
   * Gets the number of teams
   */
  readonly team_count: number;
}
/**
 * JavaScript-friendly rating representation
 */
export class JsRating {
  free(): void;
  /**
   * Creates a new rating with the given mean and variance
   */
  constructor(mean: number, variance: number);
  /**
   * Creates a string representation
   */
  toString(): string;
  /**
   * Gets the mean skill value
   */
  readonly mean: number;
  /**
   * Gets the variance
   */
  readonly variance: number;
  /**
   * Gets the standard deviation (σ)
   */
  readonly standard_deviation: number;
  /**
   * Gets a conservative skill estimate (μ - 3σ)
   */
  readonly conservative_rating: number;
}
/**
 * JavaScript-friendly team representation
 */
export class JsTeam {
  free(): void;
  /**
   * Creates a new team from an array of ratings
   */
  constructor();
  /**
   * Adds a player rating to the team
   */
  add_player(rating: JsRating): void;
  /**
   * Gets a player rating at the specified index
   */
  get_player(index: number): JsRating | undefined;
  /**
   * Creates a string representation
   */
  toString(): string;
  /**
   * Gets the number of players in the team
   */
  readonly player_count: number;
  /**
   * Gets the team's total mean (sum of player means)
   */
  readonly team_mean: number;
  /**
   * Gets the team's total variance (sum of player variances)
   */
  readonly team_variance: number;
}
/**
 * Configuration for rating systems
 */
export class RatingSystemConfig {
  free(): void;
  /**
   * Creates a new configuration with default parameters
   */
  constructor(system_type: RatingSystemType);
  /**
   * Sets custom parameters as a JSON string
   */
  set_parameters(params: string): void;
  /**
   * Type of rating system
   */
  system_type: RatingSystemType;
  /**
   * Gets the rating system type
   */
  readonly systemType: RatingSystemType;
}
/**
 * Result type for rating updates
 */
export class RatingUpdate {
  private constructor();
  free(): void;
  /**
   * Gets an updated team by index
   */
  get_team(index: number): JsTeam | undefined;
  /**
   * Optional match quality (0-1, higher is better)
   */
  get match_quality(): number | undefined;
  /**
   * Optional match quality (0-1, higher is better)
   */
  set match_quality(value: number | null | undefined);
  /**
   * Gets the number of teams updated
   */
  readonly team_count: number;
  /**
   * Gets the match quality if available
   */
  readonly matchQuality: number | undefined;
}
/**
 * Player rating for JavaScript
 */
/**
 * Player rating representation for JavaScript
 * 
 * Represents a player's rating value with their unique identifier.
 * This is the primary data structure for tracking player skill levels.
 * 
 * @example
 * ```typescript
 * const rating: WasmRating = system.create_player("player_1");
 * console.log(`Player ${rating.player_id} has rating ${rating.rating}`);
 * ```
 */
export class WasmRating {
  private constructor();
  free(): void;
  player_id: string;
  rating: number;
}
/**
 * Optimized Elo rating system for JavaScript
 *
 * This provides a minimal API surface for Elo rating calculations
 * to achieve the smallest possible WASM bundle size.
 */
/**
 * Optimized Elo rating system for JavaScript
 * 
 * This class provides a complete Elo rating system implementation optimized for
 * WebAssembly performance. It supports player management, rating calculations,
 * and match processing with minimal bundle size overhead.
 * 
 * @example
 * ```typescript
 * const system = new WasmRatingSystem({ k_factor: 32 });
 * const alice = system.create_player("alice");
 * const bob = system.create_player("bob");
 * const results = system.update_match("alice", "bob", true);
 * console.log(`Alice's new rating: ${results[0].rating}`);
 * ```
 */
export class WasmRatingSystem {
  free(): void;
  /**
   * Creates a new Elo rating system
   *
   * # Arguments
   * * `config` - JSON configuration object with optional k_factor
   *
   * # Returns
   * A new WasmRatingSystem instance
   */
  constructor(config?: EloConfig | any);
  /**
   * Creates a new player with default Elo rating (1500)
   *
   * # Arguments
   * * `player_id` - Unique identifier for the player
   *
   * # Returns
   * A WasmRating object representing the new player's rating
   */
  create_player(player_id: string): WasmRating;
  /**
   * Updates ratings for a 1v1 match
   *
   * # Arguments
   * * `player1_id` - ID of first player
   * * `player2_id` - ID of second player  
   * * `player1_wins` - true if player1 wins, false if player2 wins
   *
   * # Returns
   * Array with updated ratings for both players
   */
  update_match(player1_id: string, player2_id: string, player1_wins: boolean): WasmRating[];
  /**
   * Calculates expected win probability for player1 vs player2
   *
   * # Arguments
   * * `player1_id` - ID of first player
   * * `player2_id` - ID of second player
   *
   * # Returns
   * Probability (0.0 to 1.0) that player1 wins
   */
  get_win_probability(player1_id: string, player2_id: string): number;
  /**
   * Gets a player's current rating
   *
   * # Arguments
   * * `player_id` - ID of the player
   *
   * # Returns
   * Current rating value, or default (1500) if player not found
   */
  get_rating(player_id: string): number;
  /**
   * Returns all players sorted by rating (highest first)
   *
   * # Returns
   * Array of WasmRating objects sorted by rating descending
   */
  get_leaderboard(): WasmRating[];
  /**
   * Gets the number of tracked players
   */
  player_count(): number;
}
/**
 * Team representation for JavaScript  
 */
/**
 * Team representation for JavaScript
 * 
 * Represents a team of players with a score for match processing.
 * Used primarily for team-based game modes and tournaments.
 * 
 * @example
 * ```typescript
 * const team = new WasmTeam(100);
 * team.add_player(alice_rating);
 * team.add_player(bob_rating);
 * ```
 */
export class WasmTeam {
  free(): void;
  constructor(score: number);
  add_player(player: WasmRating): void;
  score: number;
  readonly player_count: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly wasm_main: () => void;
  readonly greet: (a: number, b: number) => void;
  readonly __wbg_wasmrating_free: (a: number, b: number) => void;
  readonly __wbg_get_wasmrating_player_id: (a: number, b: number) => void;
  readonly __wbg_set_wasmrating_player_id: (a: number, b: number, c: number) => void;
  readonly __wbg_get_wasmrating_rating: (a: number) => number;
  readonly __wbg_set_wasmrating_rating: (a: number, b: number) => void;
  readonly __wbg_wasmteam_free: (a: number, b: number) => void;
  readonly wasmteam_new: (a: number) => number;
  readonly wasmteam_add_player: (a: number, b: number) => void;
  readonly wasmteam_player_count: (a: number) => number;
  readonly __wbg_wasmratingsystem_free: (a: number, b: number) => void;
  readonly wasmratingsystem_new: (a: number, b: number) => void;
  readonly wasmratingsystem_create_player: (a: number, b: number, c: number, d: number) => void;
  readonly wasmratingsystem_update_match: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly wasmratingsystem_get_win_probability: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wasmratingsystem_get_rating: (a: number, b: number, c: number) => number;
  readonly wasmratingsystem_get_leaderboard: (a: number, b: number) => void;
  readonly wasmratingsystem_player_count: (a: number) => number;
  readonly __wbg_jsrating_free: (a: number, b: number) => void;
  readonly jsrating_new: (a: number, b: number, c: number) => void;
  readonly jsrating_mean: (a: number) => number;
  readonly jsrating_variance: (a: number) => number;
  readonly jsrating_standard_deviation: (a: number) => number;
  readonly jsrating_conservative_rating: (a: number) => number;
  readonly jsrating_toString: (a: number, b: number) => void;
  readonly __wbg_jsteam_free: (a: number, b: number) => void;
  readonly jsteam_new: () => number;
  readonly jsteam_add_player: (a: number, b: number) => void;
  readonly jsteam_player_count: (a: number) => number;
  readonly jsteam_team_mean: (a: number) => number;
  readonly jsteam_team_variance: (a: number) => number;
  readonly jsteam_get_player: (a: number, b: number) => number;
  readonly jsteam_toString: (a: number, b: number) => void;
  readonly __wbg_jsgameoutcome_free: (a: number, b: number) => void;
  readonly jsgameoutcome_new: () => number;
  readonly jsgameoutcome_set_ranks: (a: number, b: number, c: number, d: number) => void;
  readonly jsgameoutcome_team_count: (a: number) => number;
  readonly jsgameoutcome_win: (a: number, b: number, c: number) => void;
  readonly jsgameoutcome_draw: (a: number, b: number) => void;
  readonly jsgameoutcome_get_rank: (a: number, b: number) => number;
  readonly jsgameoutcome_toString: (a: number, b: number) => void;
  readonly __wbg_ratingsystemconfig_free: (a: number, b: number) => void;
  readonly __wbg_get_ratingsystemconfig_system_type: (a: number) => number;
  readonly __wbg_set_ratingsystemconfig_system_type: (a: number, b: number) => void;
  readonly ratingsystemconfig_new: (a: number) => number;
  readonly ratingsystemconfig_set_parameters: (a: number, b: number, c: number) => void;
  readonly ratingsystemconfig_get_system_type: (a: number) => number;
  readonly __wbg_ratingupdate_free: (a: number, b: number) => void;
  readonly __wbg_get_ratingupdate_match_quality: (a: number, b: number) => void;
  readonly __wbg_set_ratingupdate_match_quality: (a: number, b: number, c: number) => void;
  readonly ratingupdate_team_count: (a: number) => number;
  readonly ratingupdate_get_team: (a: number, b: number) => number;
  readonly ratingupdate_get_match_quality: (a: number, b: number) => void;
  readonly __wbg_set_wasmteam_score: (a: number, b: number) => void;
  readonly __wbg_get_wasmteam_score: (a: number) => number;
  readonly __wbindgen_export_0: (a: number, b: number) => number;
  readonly __wbindgen_export_1: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_2: (a: number, b: number, c: number) => void;
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
export function isWasmRating(obj: any): obj is WasmRating {
  return obj && typeof obj.player_id === 'string' && typeof obj.rating === 'number';
}

export function isValidPlayerId(id: any): id is PlayerId {
  return typeof id === 'string' && id.length > 0;
}

export function isValidProbability(p: any): p is Probability {
  return typeof p === 'number' && p >= 0 && p <= 1;
}

export function isMatchOutcome(outcome: any): outcome is MatchOutcome {
  return typeof outcome === 'number' && (outcome === 0 || outcome === 1 || outcome === 2);
}

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
export function initializeWasm(options?: WasmInitOptions): Promise<WasmInitResult>;

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
export function convertLegacyConfig(legacy: LegacyEloConfig): EloConfig;
