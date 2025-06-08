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

export class WasmRating {
    player_id: string;
    rating: number;
    uncertainty?: number;
    volatility?: number;
}

export class WasmTeam {
    constructor(score: number);
    add_player(player: WasmRating): void;
    readonly player_count: number;
    score: number;
    players: WasmRating[];
}

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
