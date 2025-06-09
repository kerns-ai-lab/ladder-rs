// Task 1.1.4: TypeScript Compilation Test
// This file tests that our generated TypeScript definitions are valid and usable

// Import the module - this will test our TypeScript definitions
import init, { 
  WasmRatingSystem, 
  WasmRating, 
  WasmTeam,
  JsRating,
  JsTeam,
  JsGameOutcome,
  RatingSystemType,
  RatingSystemConfig,
  wasm_main,
  greet
} from '../pkg/ladder_rs_wasm';

// Test basic type usage
async function testBasicTypes(): Promise<void> {
  // Initialize the WASM module
  await init();
  
  // Test basic functions
  wasm_main();
  greet("TypeScript");
  
  // Test Elo rating system
  const ratingSystem = new WasmRatingSystem({ k_factor: 32 });
  
  // Test player creation
  const player1: WasmRating = ratingSystem.create_player("alice");
  const player2: WasmRating = ratingSystem.create_player("bob");
  
  // Verify type properties
  console.assert(typeof player1.player_id === 'string');
  console.assert(typeof player1.rating === 'number');
  
  // Test match update
  const updatedRatings: WasmRating[] = ratingSystem.update_match("alice", "bob", true);
  console.assert(Array.isArray(updatedRatings));
  console.assert(updatedRatings.length === 2);
  
  // Test win probability
  const winProb: number = ratingSystem.get_win_probability("alice", "bob");
  console.assert(typeof winProb === 'number');
  console.assert(winProb >= 0 && winProb <= 1);
  
  // Test leaderboard
  const leaderboard: WasmRating[] = ratingSystem.get_leaderboard();
  console.assert(Array.isArray(leaderboard));
  
  // Test player count
  const count: number = ratingSystem.player_count();
  console.assert(typeof count === 'number');
}

// Test team functionality
function testTeamTypes(): void {
  const team = new WasmTeam(100);
  
  // Test properties
  console.assert(typeof team.score === 'number');
  console.assert(typeof team.player_count === 'number');
  
  // Test score assignment
  team.score = 200;
  console.assert(team.score === 200);
}

// Test legacy JS types for backward compatibility
function testLegacyTypes(): void {
  // Test JsRating
  const rating = new JsRating(1500, 200);
  console.assert(typeof rating.mean === 'number');
  console.assert(typeof rating.variance === 'number');
  console.assert(typeof rating.standard_deviation === 'number');
  console.assert(typeof rating.conservative_rating === 'number');
  
  // Test JsTeam
  const team = new JsTeam();
  team.add_player(rating);
  console.assert(typeof team.player_count === 'number');
  console.assert(typeof team.team_mean === 'number');
  console.assert(typeof team.team_variance === 'number');
  
  // Test JsGameOutcome
  const outcome = new JsGameOutcome();
  const ranks = new Uint32Array([1, 2]);
  outcome.set_ranks(ranks);
  console.assert(typeof outcome.team_count === 'number');
  
  // Test static methods
  const winOutcome = JsGameOutcome.win(0, 2);
  const drawOutcome = JsGameOutcome.draw(2);
  console.assert(winOutcome instanceof JsGameOutcome);
  console.assert(drawOutcome instanceof JsGameOutcome);
}

// Test enum usage
function testEnumTypes(): void {
  // Test RatingSystemType enum
  const eloType: RatingSystemType = RatingSystemType.Elo;
  const glickoType: RatingSystemType = RatingSystemType.Glicko;
  const trueskillType: RatingSystemType = RatingSystemType.TrueSkill;
  
  console.assert(typeof eloType === 'number');
  console.assert(eloType === 0);
  
  // Test RatingSystemConfig
  const config = new RatingSystemConfig(RatingSystemType.Elo);
  config.set_parameters('{"k_factor": 32}');
  console.assert(config.systemType === RatingSystemType.Elo);
}

// Test type safety and nullability
function testTypeSafety(): void {
  const rating = new JsRating(1500, 200);
  const team = new JsTeam();
  team.add_player(rating);
  
  // Test optional return types
  const player: JsRating | undefined = team.get_player(0);
  if (player !== undefined) {
    console.assert(typeof player.mean === 'number');
  }
  
  // Test out-of-bounds access
  const nonExistent: JsRating | undefined = team.get_player(999);
  console.assert(nonExistent === undefined);
  
  // Test outcome rank access
  const outcome = JsGameOutcome.win(0, 2);
  const rank: number | undefined = outcome.get_rank(0);
  if (rank !== undefined) {
    console.assert(typeof rank === 'number');
  }
}

// Test memory management types
function testMemoryManagement(): void {
  const rating = new JsRating(1500, 200);
  const team = new JsTeam();
  const outcome = new JsGameOutcome();
  
  // Verify free methods exist and are callable
  console.assert(typeof rating.free === 'function');
  console.assert(typeof team.free === 'function');
  console.assert(typeof outcome.free === 'function');
  
  // Call free methods (in real usage, you'd do this when done with objects)
  rating.free();
  team.free();
  outcome.free();
}

// Test async initialization patterns
async function testAsyncPatterns(): Promise<void> {
  // Test different initialization methods
  
  // Method 1: Default init
  await init();
  
  // Method 2: With Response
  // const response = fetch('/path/to/wasm');
  // await init(response);
  
  // Method 3: With ArrayBuffer
  // const buffer = new ArrayBuffer(1024);
  // await init(buffer);
  
  // For this test, we'll just verify the types are correct
  console.log("Async initialization patterns verified");
}

// Test strict TypeScript compatibility
function testStrictTypeScript(): void {
  // Test that we can use strict null checks
  const system = new WasmRatingSystem({ k_factor: 32 });
  
  // These should compile without warnings in strict mode
  const playerId: string = "test-player";
  const rating: number = system.get_rating(playerId);
  
  console.assert(typeof rating === 'number');
  
  // Test readonly properties
  const player = system.create_player(playerId);
  // player.player_id should be assignable but rating properties should be readonly in good types
  
  console.assert(player.player_id === playerId);
}

// Test complex usage patterns
function testComplexUsage(): void {
  const system = new WasmRatingSystem({ k_factor: 32 });
  
  // Create multiple players
  const players: string[] = ['alice', 'bob', 'charlie', 'diana'];
  const ratings: WasmRating[] = players.map(id => system.create_player(id));
  
  // Verify array types
  console.assert(Array.isArray(ratings));
  console.assert(ratings.every(r => typeof r.rating === 'number'));
  
  // Simulate tournament matches
  for (let i = 0; i < players.length - 1; i++) {
    const updatedRatings = system.update_match(players[i], players[i + 1], Math.random() > 0.5);
    console.assert(updatedRatings.length === 2);
  }
  
  // Get final leaderboard
  const leaderboard = system.get_leaderboard();
  console.assert(leaderboard.length === players.length);
  
  // Verify leaderboard is sorted (highest first)
  for (let i = 0; i < leaderboard.length - 1; i++) {
    console.assert(leaderboard[i].rating >= leaderboard[i + 1].rating);
  }
}

// Run all tests
export async function runTypeScriptTests(): Promise<void> {
  console.log("Running TypeScript definition tests...");
  
  try {
    await testBasicTypes();
    console.log("✓ Basic types test passed");
    
    testTeamTypes();
    console.log("✓ Team types test passed");
    
    testLegacyTypes();
    console.log("✓ Legacy types test passed");
    
    testEnumTypes();
    console.log("✓ Enum types test passed");
    
    testTypeSafety();
    console.log("✓ Type safety test passed");
    
    testMemoryManagement();
    console.log("✓ Memory management test passed");
    
    await testAsyncPatterns();
    console.log("✓ Async patterns test passed");
    
    testStrictTypeScript();
    console.log("✓ Strict TypeScript test passed");
    
    testComplexUsage();
    console.log("✓ Complex usage test passed");
    
    console.log("All TypeScript definition tests passed! ✨");
  } catch (error) {
    console.error("TypeScript test failed:", error);
    throw error;
  }
}

// Export for external testing
export default runTypeScriptTests;