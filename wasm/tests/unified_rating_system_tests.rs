use ladder_rs_wasm::{WasmRating, WasmRatingSystem, WasmTeam};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_elo_system() {
    let config = JsValue::from_str(r#"{"type": "elo", "k_factor": 32.0}"#);
    let system = WasmRatingSystem::new("elo", config);
    assert!(system.is_ok());
}

#[wasm_bindgen_test]
fn test_create_glicko_system() {
    let config = JsValue::from_str(r#"{"type": "glicko", "initial_volatility": 0.06}"#);
    let system = WasmRatingSystem::new("glicko", config);
    assert!(system.is_ok());
}

#[wasm_bindgen_test]
fn test_create_trueskill_system() {
    let config = JsValue::from_str(
        r#"{"type": "trueskill", "beta": 4.166666666666667, "tau": 0.08333333333333334}"#,
    );
    let system = WasmRatingSystem::new("trueskill", config);
    assert!(system.is_ok());
}

#[wasm_bindgen_test]
fn test_invalid_system_type() {
    let config = JsValue::from_str(r#"{}"#);
    let system = WasmRatingSystem::new("invalid", config);
    assert!(system.is_err());
}

#[wasm_bindgen_test]
fn test_create_player_elo() {
    let config = JsValue::from_str(r#"{"type": "elo"}"#);
    let mut system = WasmRatingSystem::new("elo", config).unwrap();
    let player = system.create_player("player1".to_string()).unwrap();

    assert_eq!(player.player_id, "player1");
    assert_eq!(player.rating, 1500.0);
    assert!(player.uncertainty.is_none());
    assert!(player.volatility.is_none());
}

#[wasm_bindgen_test]
fn test_create_player_glicko() {
    let config = JsValue::from_str(r#"{"type": "glicko"}"#);
    let mut system = WasmRatingSystem::new("glicko", config).unwrap();
    let player = system.create_player("player1".to_string()).unwrap();

    assert_eq!(player.player_id, "player1");
    assert_eq!(player.rating, 1500.0);
    assert!(player.uncertainty.is_some());
    assert_eq!(player.uncertainty.unwrap(), 350.0);
    assert!(player.volatility.is_none());
}

#[wasm_bindgen_test]
fn test_create_player_trueskill() {
    let config = JsValue::from_str(r#"{"type": "trueskill"}"#);
    let mut system = WasmRatingSystem::new("trueskill", config).unwrap();
    let player = system.create_player("player1".to_string()).unwrap();

    assert_eq!(player.player_id, "player1");
    assert_eq!(player.rating, 25.0);
    assert!(player.uncertainty.is_some());
    assert!((player.uncertainty.unwrap() - 25.0 / 3.0).abs() < 0.01);
    assert!(player.volatility.is_none());
}

#[wasm_bindgen_test]
fn test_update_ratings_elo() {
    let config = JsValue::from_str(r#"{"type": "elo", "k_factor": 32.0}"#);
    let mut system = WasmRatingSystem::new("elo", config).unwrap();

    let player1 = system.create_player("player1".to_string()).unwrap();
    let player2 = system.create_player("player2".to_string()).unwrap();

    let mut team1 = WasmTeam::new(1.0);
    team1.add_player(player1);

    let mut team2 = WasmTeam::new(0.0);
    team2.add_player(player2);

    let result = system.update_ratings(vec![team1, team2]).unwrap();

    assert_eq!(result.len(), 2);
    assert!(result[0].players[0].rating > 1500.0);
    assert!(result[1].players[0].rating < 1500.0);
}

#[wasm_bindgen_test]
fn test_get_match_quality() {
    let config = JsValue::from_str(r#"{"type": "trueskill"}"#);
    let mut system = WasmRatingSystem::new("trueskill", config).unwrap();

    let player1 = system.create_player("player1".to_string()).unwrap();
    let player2 = system.create_player("player2".to_string()).unwrap();

    let mut team1 = WasmTeam::new(0.0);
    team1.add_player(player1);

    let mut team2 = WasmTeam::new(0.0);
    team2.add_player(player2);

    let quality = system.get_match_quality(vec![team1, team2]).unwrap();

    assert!(quality > 0.0 && quality <= 1.0);
    assert!(quality > 0.4); // New players should have decent match quality
}

#[wasm_bindgen_test]
fn test_get_leaderboard() {
    let config = JsValue::from_str(r#"{"type": "elo", "k_factor": 32.0}"#);
    let mut system = WasmRatingSystem::new("elo", config).unwrap();

    // Create players
    let player1 = system.create_player("player1".to_string()).unwrap();
    let player2 = system.create_player("player2".to_string()).unwrap();
    let player3 = system.create_player("player3".to_string()).unwrap();

    // Player1 beats player2
    let mut team1 = WasmTeam::new(1.0);
    team1.add_player(player1.clone());
    let mut team2 = WasmTeam::new(0.0);
    team2.add_player(player2.clone());
    let updated = system.update_ratings(vec![team1, team2]).unwrap();

    // Player3 beats player2
    let mut team3 = WasmTeam::new(1.0);
    team3.add_player(player3);
    let mut team4 = WasmTeam::new(0.0);
    team4.add_player(updated[1].players[0].clone());
    system.update_ratings(vec![team3, team4]).unwrap();

    let leaderboard = system.get_leaderboard().unwrap();

    assert_eq!(leaderboard.len(), 3);
    assert!(leaderboard[0].rating > leaderboard[1].rating);
    assert!(leaderboard[1].rating > leaderboard[2].rating);
}
