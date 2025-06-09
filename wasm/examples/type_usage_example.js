/**
 * Example demonstrating the usage of ladder-rs-wasm types from JavaScript
 * 
 * This example shows:
 * - Creating players with ratings
 * - Creating matches between players
 * - Using different algorithm configurations
 * - Error handling
 */

// Import the WASM module (assuming it's been built and available)
// In a real application, you would import from the published npm package
// import * as wasm from '../pkg/ladder_rs_wasm.js';

// For this example, we'll demonstrate the API usage

// 1. Creating Players
console.log("=== Creating Players ===");

// Create a rating for player 1
// Note: JsRating constructor validates variance > 0 and can throw
const player1Rating = new wasm.JsRating(1500.0, 200.0);
console.log(`Player 1 Rating - Mean: ${player1Rating.mean}, Variance: ${player1Rating.variance}`);

// Create player 1
const player1 = new wasm.JsPlayer("player1", "Alice", player1Rating);
console.log(`Created Player: ${player1.id} (${player1.name})`);

// Create player 2 with no name
const player2Rating = new wasm.JsRating(1600.0, 150.0);
const player2 = new wasm.JsPlayer("player2", null, player2Rating);
console.log(`Created Player: ${player2.id} (name: ${player2.name || "unnamed"})`);

// 2. JSON Serialization
console.log("\n=== JSON Serialization ===");

// Serialize player to JSON
const playerJson = player1.toJSON();
console.log("Player 1 as JSON:", playerJson);

// Deserialize from JSON
const deserializedPlayer = wasm.JsPlayer.fromJSON(playerJson);
console.log("Deserialized player ID:", deserializedPlayer.id);

// 3. Configuration Types
console.log("\n=== Algorithm Configurations ===");

// Elo configuration
const eloConfig = new wasm.JsEloConfig(32.0, 1500.0, 300.0);
console.log(`Elo Config - K-factor: ${eloConfig.kFactor}, Initial: ${eloConfig.initialRating}`);

// Glicko configuration
const glickoConfig = new wasm.JsGlickoConfig(1500.0, 350.0, 15.0);
console.log(`Glicko Config - Initial: ${glickoConfig.initialRating}, Deviation: ${glickoConfig.initialDeviation}`);

// TrueSkill configuration
const trueSkillConfig = new wasm.JsTrueSkillConfig(25.0, 8.333, 4.166, 0.083, 0.1);
console.log(`TrueSkill Config - Mean: ${trueSkillConfig.initialMean}, Beta: ${trueSkillConfig.beta}`);

// 4. Match Configuration
console.log("\n=== Match Configuration ===");

// Create a match configuration for Elo
const eloMatchConfig = new wasm.JsMatchConfig("elo", { k_factor: 32.0 });
console.log(`Match Config - Algorithm: ${eloMatchConfig.algorithm}`);
console.log("Match Config - Params:", eloMatchConfig.params);

// 5. Match Results
console.log("\n=== Match Results ===");

// Simulate a match result
const updatedRatings = [
    new wasm.JsRating(1520.0, 190.0),  // Winner's new rating
    new wasm.JsRating(1480.0, 210.0)   // Loser's new rating
];

// Note: In JavaScript, we pass null for a draw, or a string for the winner
const matchResult = new wasm.JsMatchResult("player1", updatedRatings);  // Winner: player1
// For a draw: const drawResult = new wasm.JsMatchResult(null, updatedRatings);
console.log(`Match Winner: ${matchResult.winner}`);
console.log(`Updated Ratings Count: ${matchResult.ratings.length}`);
console.log(`Winner's new rating: Mean=${matchResult.ratings[0].mean}, Variance=${matchResult.ratings[0].variance}`);

// Serialize match result
const matchResultJson = matchResult.toJSON();
console.log("Match result as JSON:", matchResultJson);

// 6. Match Outcomes
console.log("\n=== Match Outcomes ===");
console.log("Available outcomes:");
console.log(`- Win: ${wasm.JsOutcome.Win}`);
console.log(`- Loss: ${wasm.JsOutcome.Loss}`);
console.log(`- Draw: ${wasm.JsOutcome.Draw}`);

// 7. Error Handling
console.log("\n=== Error Handling ===");

// Create an error
const error = new wasm.JsError("Invalid player ID", "ValidationError");
console.log(`Error: ${error.toString()}`);
console.log(`Error Message: ${error.message}`);
console.log(`Error Type: ${error.errorType}`);

// Example of error handling in practice
try {
    // This would fail in real usage
    const invalidJson = '{"invalid": "json"}';
    // const player = wasm.JsPlayer.fromJSON(invalidJson); // Would throw
    console.log("Error handling example: Would throw on invalid JSON");
} catch (e) {
    console.error("Caught error:", e);
}

// Example of rating validation
try {
    // This will throw because variance must be positive
    const invalidRating = new wasm.JsRating(1500.0, -100.0);
} catch (e) {
    console.log("Caught variance validation error:", e);  // "Variance must be positive"
}

try {
    // Zero variance is also invalid
    const zeroVarianceRating = new wasm.JsRating(1500.0, 0.0);
} catch (e) {
    console.log("Caught zero variance error:", e);  // "Variance must be positive"
}

// 8. Working with Collections
console.log("\n=== Working with Collections ===");

// Create multiple players
const players = [
    new wasm.JsPlayer("p1", "Alice", new wasm.JsRating(1500, 200)),
    new wasm.JsPlayer("p2", "Bob", new wasm.JsRating(1600, 180)),
    new wasm.JsPlayer("p3", "Charlie", new wasm.JsRating(1400, 220))
];

console.log("Created players:");
players.forEach(p => {
    console.log(`  - ${p.id}: ${p.name} (Rating: ${p.rating.mean})`);
});

// 9. Memory Management
console.log("\n=== Memory Management ===");
console.log("Note: WASM objects should be freed when no longer needed");
console.log("In modern JavaScript, this is handled automatically by the garbage collector");
console.log("But for performance-critical applications, you can call .free() explicitly");

// Example of explicit cleanup (optional)
// player1.free();
// player2.free();
// eloConfig.free();

console.log("\n=== Example Complete ===");
console.log("This example demonstrated all the core type definitions exposed by ladder-rs-wasm");