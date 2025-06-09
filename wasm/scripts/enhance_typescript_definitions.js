#!/usr/bin/env node

/**
 * Task 1.1.4: TypeScript Definition Enhancement Script
 * 
 * This script post-processes the wasm-pack generated TypeScript definitions
 * to add enhancements, better documentation, and utility types.
 */

const fs = require('fs');
const path = require('path');

const PKG_DIR = path.join(__dirname, '..', 'pkg');
const TYPES_DIR = path.join(__dirname, '..', 'types');
const GENERATED_DEFS = path.join(PKG_DIR, 'ladder_rs_wasm.d.ts');
const ENHANCED_DEFS = path.join(PKG_DIR, 'ladder_rs_wasm.d.ts');

/**
 * Enhancement functions
 */

function addUtilityTypes() {
  return `
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

`;
}

function addDocumentationEnhancements(content) {
  // Enhanced JSDoc comments for key classes
  const enhancements = [
    {
      search: /export class WasmRatingSystem \{/,
      replace: `/**
 * Optimized Elo rating system for JavaScript
 * 
 * This class provides a complete Elo rating system implementation optimized for
 * WebAssembly performance. It supports player management, rating calculations,
 * and match processing with minimal bundle size overhead.
 * 
 * @example
 * \`\`\`typescript
 * const system = new WasmRatingSystem({ k_factor: 32 });
 * const alice = system.create_player("alice");
 * const bob = system.create_player("bob");
 * const results = system.update_match("alice", "bob", true);
 * console.log(\`Alice's new rating: \${results[0].rating}\`);
 * \`\`\`
 */
export class WasmRatingSystem {`
    },
    {
      search: /export class WasmRating \{/,
      replace: `/**
 * Player rating representation for JavaScript
 * 
 * Represents a player's rating value with their unique identifier.
 * This is the primary data structure for tracking player skill levels.
 * 
 * @example
 * \`\`\`typescript
 * const rating: WasmRating = system.create_player("player_1");
 * console.log(\`Player \${rating.player_id} has rating \${rating.rating}\`);
 * \`\`\`
 */
export class WasmRating {`
    },
    {
      search: /export class WasmTeam \{/,
      replace: `/**
 * Team representation for JavaScript
 * 
 * Represents a team of players with a score for match processing.
 * Used primarily for team-based game modes and tournaments.
 * 
 * @example
 * \`\`\`typescript
 * const team = new WasmTeam(100);
 * team.add_player(alice_rating);
 * team.add_player(bob_rating);
 * \`\`\`
 */
export class WasmTeam {`
    },
    {
      search: /export enum RatingSystemType \{/,
      replace: `/**
 * Rating system type enumeration
 * 
 * Defines the available rating system algorithms.
 * Each has different characteristics and use cases.
 * 
 * - Elo: Simple, fast, good for 1v1 games
 * - Glicko: Includes rating reliability/uncertainty
 * - TrueSkill: Microsoft's system, supports teams and draws
 */
export enum RatingSystemType {`
    }
  ];

  let enhanced = content;
  for (const enhancement of enhancements) {
    enhanced = enhanced.replace(enhancement.search, enhancement.replace);
  }

  return enhanced;
}

function addTypeAssertions(content) {
  // Add type assertions for better IntelliSense
  const assertions = `
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
`;

  return content + assertions;
}

function addPromiseTypes(content) {
  // Enhance async/Promise types for better async support
  const promiseEnhancement = `
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
`;

  return content + promiseEnhancement;
}

function fixTypeScriptIssues(content) {
  // Fix common TypeScript issues in generated definitions
  
  // Fix array types - ensure Uint32Array is properly typed
  content = content.replace(
    /set_ranks\(ranks: Uint32Array\): void;/g,
    'set_ranks(ranks: Uint32Array | number[]): void;'
  );

  // Add proper optional chaining for memory management
  content = content.replace(
    /free\(\): void;/g,
    'free?(): void;'
  );

  // Fix constructor parameter types to be more flexible
  content = content.replace(
    /constructor\(config: any\)/g,
    'constructor(config?: EloConfig | any)'
  );

  return content;
}

function addCompatibilityTypes(content) {
  // Add backward compatibility types for existing code
  const compatibility = `
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
`;

  return content + compatibility;
}

/**
 * Main enhancement function
 */
function enhanceTypeScriptDefinitions() {
  try {
    console.log('🔧 Enhancing TypeScript definitions...');

    // Read the generated definitions
    if (!fs.existsSync(GENERATED_DEFS)) {
      throw new Error(`Generated definitions not found at ${GENERATED_DEFS}`);
    }

    let content = fs.readFileSync(GENERATED_DEFS, 'utf8');
    console.log(`📖 Read ${content.length} characters from generated definitions`);

    // Apply enhancements
    console.log('✨ Adding utility types...');
    const utilityTypes = addUtilityTypes();

    console.log('📝 Enhancing documentation...');
    content = addDocumentationEnhancements(content);

    console.log('🔧 Fixing TypeScript issues...');
    content = fixTypeScriptIssues(content);

    console.log('🛡️ Adding type assertions...');
    content = addTypeAssertions(content);

    console.log('⚡ Adding Promise types...');
    content = addPromiseTypes(content);

    console.log('🔄 Adding compatibility types...');
    content = addCompatibilityTypes(content);

    // Combine all enhancements
    const enhanced = `${utilityTypes}${content}`;

    // Write enhanced definitions
    fs.writeFileSync(ENHANCED_DEFS, enhanced, 'utf8');
    console.log(`✅ Enhanced definitions written to ${ENHANCED_DEFS}`);

    // Also copy to types directory for reference
    fs.mkdirSync(TYPES_DIR, { recursive: true });
    fs.writeFileSync(path.join(TYPES_DIR, 'ladder_rs_wasm.d.ts'), enhanced, 'utf8');
    console.log(`📋 Copy written to ${TYPES_DIR}/ladder_rs_wasm.d.ts`);

    // Validate the enhanced definitions
    validateEnhancements(enhanced);

    console.log('🎉 TypeScript definition enhancement completed successfully!');
    return true;

  } catch (error) {
    console.error('❌ Error enhancing TypeScript definitions:', error.message);
    return false;
  }
}

function validateEnhancements(content) {
  console.log('🔍 Validating enhanced definitions...');

  const checks = [
    { test: () => content.includes('export type PlayerId'), desc: 'Utility types added' },
    { test: () => content.includes('export interface EloConfig'), desc: 'Config interfaces added' },
    { test: () => content.includes('@example'), desc: 'Enhanced documentation added' },
    { test: () => content.includes('isWasmRating'), desc: 'Type assertions added' },
    { test: () => content.includes('WasmInitOptions'), desc: 'Promise types added' },
    { test: () => content.includes('LegacyEloConfig'), desc: 'Compatibility types added' },
    { test: () => content.includes('free?(): void'), desc: 'Optional memory management fixed' },
  ];

  let passed = 0;
  for (const check of checks) {
    if (check.test()) {
      console.log(`  ✅ ${check.desc}`);
      passed++;
    } else {
      console.log(`  ❌ ${check.desc}`);
    }
  }

  console.log(`📊 Validation: ${passed}/${checks.length} checks passed`);
  
  if (passed < checks.length) {
    throw new Error('Some enhancement validations failed');
  }
}

// CLI execution
if (require.main === module) {
  const success = enhanceTypeScriptDefinitions();
  process.exit(success ? 0 : 1);
}

module.exports = { enhanceTypeScriptDefinitions };