
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
