
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

// TypeScript definitions for ladder-rs WASM bindings
// Generated for Task 1.1.4

export function wasm_main(): void;
export function greet(name: string): void;

export class JsRating {
    constructor(mean: number, variance: number);
    readonly mean: number;
    readonly variance: number;
    readonly standard_deviation: number;
    readonly conservative_rating: number;
    toString(): string;
}

export class JsTeam {
    constructor();
    add_player(rating: JsRating): void;
    readonly player_count: number;
    readonly team_mean: number;
    readonly team_variance: number;
    get_player(index: number): JsRating | undefined;
    toString(): string;
}

export class JsGameOutcome {
    constructor();
    set_ranks(ranks: number[]): void;
    readonly team_count: number;
    static win(winnerIndex: number, totalTeams: number): JsGameOutcome;
    static draw(totalTeams: number): JsGameOutcome;
    get_rank(teamIndex: number): number | undefined;
    toString(): string;
}

export const enum RatingSystemType {
    Elo = 0,
    Glicko = 1,
    Glicko2 = 2,
    TrueSkill = 3,
}

export class RatingSystemConfig {
    constructor(systemType: RatingSystemType);
    set_parameters(params: string): void;
    readonly systemType: RatingSystemType;
}

export class RatingUpdate {
    readonly team_count: number;
    get_team(index: number): JsTeam | undefined;
    readonly matchQuality?: number;
}

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
    player_id: string;
    rating: number;
    uncertainty?: number;
    volatility?: number;
}

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
    constructor(score: number);
    add_player(player: WasmRating): void;
    readonly player_count: number;
    score: number;
    players: WasmRating[];
}

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
    constructor(system_type: string, config: any);
    create_player(player_id: string): WasmRating;
    update_ratings(teams: WasmTeam[]): WasmTeam[];
    get_match_quality(teams: WasmTeam[]): number;
    get_leaderboard(): WasmRating[];
}

export interface PlayerProfile {
    id: string;
    name?: string | null;
    email?: string | null;
    created_at: number;
    updated_at: number;
    is_active: boolean;
}

export interface MatchRecord {
    id: string;
    team1_players: string[];
    team2_players: string[];
    outcome: number;
    timestamp: number;
    notes?: string | null;
}

export interface PlayerStats {
    player_id: string;
    total_matches: number;
    wins: number;
    losses: number;
    draws: number;
    win_rate: number;
    current_streak: number;
    longest_win_streak: number;
    longest_loss_streak: number;
}

export interface HeadToHeadRecord {
    player1_id: string;
    player2_id: string;
    total_matches: number;
    player1_wins: number;
    player2_wins: number;
    draws: number;
}

export class PlayerManager {
    constructor();
    register_player(id: string, name?: string | null, email?: string | null): PlayerProfile;
    get_player_profile(idOrAlias: string): PlayerProfile;
    update_player_profile(id: string, name?: string | null, email?: string | null): PlayerProfile;
    deactivate_player(id: string): void;
    reactivate_player(id: string): void;
    is_player_active(id: string): boolean;
    add_match_record(team1: string[], team2: string[], outcome: number, notes?: string | null): string;
    get_player_match_history(player_id: string, limit?: number, offset?: number): MatchRecord[];
    get_player_stats(player_id: string): PlayerStats;
    player_count(): number;
    get_active_players(): PlayerProfile[];
    search_players(query: string): PlayerProfile[];
    bulk_import_players(json_data: string): number;
    export_players(include_inactive: boolean): string;
    get_player_head_to_head(player1_id: string, player2_id: string): HeadToHeadRecord;
    merge_players(from_id: string, to_id: string): void;
    add_player_alias(player_id: string, alias: string): void;
    get_player_aliases(player_id: string): string[];
}

export default function init(module?: WebAssembly.Module | BufferSource | Response | Promise<WebAssembly.Module | BufferSource | Response>): Promise<void>;

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
