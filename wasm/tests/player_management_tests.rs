use ladder_rs_wasm::{PlayerManager, PlayerProfile, MatchRecord, PlayerStats};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_player_manager() {
    let manager = PlayerManager::new();
    assert_eq!(manager.player_count(), 0);
}

#[wasm_bindgen_test]
fn test_register_player() {
    let mut manager = PlayerManager::new();
    
    let profile = manager.register_player(
        "player1".to_string(),
        Some("Alice".to_string()),
        None,
    ).unwrap();
    
    assert_eq!(profile.id, "player1");
    assert_eq!(profile.name.unwrap(), "Alice");
    assert_eq!(manager.player_count(), 1);
}

#[wasm_bindgen_test]
fn test_register_duplicate_player() {
    let mut manager = PlayerManager::new();
    
    manager.register_player("player1".to_string(), None, None).unwrap();
    let result = manager.register_player("player1".to_string(), None, None);
    
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_get_player_profile() {
    let mut manager = PlayerManager::new();
    
    manager.register_player(
        "player1".to_string(),
        Some("Alice".to_string()),
        Some("alice@example.com".to_string()),
    ).unwrap();
    
    let profile = manager.get_player_profile("player1").unwrap();
    assert_eq!(profile.id, "player1");
    assert_eq!(profile.name.unwrap(), "Alice");
    assert_eq!(profile.email.unwrap(), "alice@example.com");
}

#[wasm_bindgen_test]
fn test_update_player_profile() {
    let mut manager = PlayerManager::new();
    
    manager.register_player("player1".to_string(), None, None).unwrap();
    
    let updated = manager.update_player_profile(
        "player1",
        Some("Bob".to_string()),
        Some("bob@example.com".to_string()),
    ).unwrap();
    
    assert_eq!(updated.name.unwrap(), "Bob");
    assert_eq!(updated.email.unwrap(), "bob@example.com");
}

#[wasm_bindgen_test]
fn test_deactivate_player() {
    let mut manager = PlayerManager::new();
    
    manager.register_player("player1".to_string(), None, None).unwrap();
    assert!(manager.is_player_active("player1"));
    
    manager.deactivate_player("player1").unwrap();
    assert!(!manager.is_player_active("player1"));
}

#[wasm_bindgen_test]
fn test_reactivate_player() {
    let mut manager = PlayerManager::new();
    
    manager.register_player("player1".to_string(), None, None).unwrap();
    manager.deactivate_player("player1").unwrap();
    manager.reactivate_player("player1").unwrap();
    
    assert!(manager.is_player_active("player1"));
}

#[wasm_bindgen_test]
fn test_add_match_record() {
    let mut manager = PlayerManager::new();
    
    manager.register_player("player1".to_string(), None, None).unwrap();
    manager.register_player("player2".to_string(), None, None).unwrap();
    
    let match_id = manager.add_match_record(
        vec!["player1".to_string()],
        vec!["player2".to_string()],
        1, // team1 wins
        None,
    ).unwrap();
    
    assert!(!match_id.is_empty());
}

#[wasm_bindgen_test]
fn test_get_player_match_history() {
    let mut manager = PlayerManager::new();
    
    manager.register_player("player1".to_string(), None, None).unwrap();
    manager.register_player("player2".to_string(), None, None).unwrap();
    
    // Add multiple matches
    manager.add_match_record(
        vec!["player1".to_string()],
        vec!["player2".to_string()],
        1,
        None,
    ).unwrap();
    
    manager.add_match_record(
        vec!["player2".to_string()],
        vec!["player1".to_string()],
        1,
        None,
    ).unwrap();
    
    let history = manager.get_player_match_history("player1", None, None).unwrap();
    assert_eq!(history.len(), 2);
}

#[wasm_bindgen_test]
fn test_get_player_stats() {
    let mut manager = PlayerManager::new();
    
    manager.register_player("player1".to_string(), None, None).unwrap();
    manager.register_player("player2".to_string(), None, None).unwrap();
    
    // Player1 wins
    manager.add_match_record(
        vec!["player1".to_string()],
        vec!["player2".to_string()],
        1,
        None,
    ).unwrap();
    
    // Player2 wins
    manager.add_match_record(
        vec!["player2".to_string()],
        vec!["player1".to_string()],
        1,
        None,
    ).unwrap();
    
    // Draw
    manager.add_match_record(
        vec!["player1".to_string()],
        vec!["player2".to_string()],
        0,
        None,
    ).unwrap();
    
    let stats = manager.get_player_stats("player1").unwrap();
    assert_eq!(stats.total_matches, 3);
    assert_eq!(stats.wins, 1);
    assert_eq!(stats.losses, 1);
    assert_eq!(stats.draws, 1);
}

#[wasm_bindgen_test]
fn test_get_active_players() {
    let mut manager = PlayerManager::new();
    
    manager.register_player("player1".to_string(), None, None).unwrap();
    manager.register_player("player2".to_string(), None, None).unwrap();
    manager.register_player("player3".to_string(), None, None).unwrap();
    
    manager.deactivate_player("player2").unwrap();
    
    let active = manager.get_active_players().unwrap();
    assert_eq!(active.len(), 2);
    assert!(active.iter().any(|p| p.id == "player1"));
    assert!(active.iter().any(|p| p.id == "player3"));
    assert!(!active.iter().any(|p| p.id == "player2"));
}

#[wasm_bindgen_test]
fn test_search_players() {
    let mut manager = PlayerManager::new();
    
    manager.register_player(
        "alice123".to_string(),
        Some("Alice Smith".to_string()),
        None,
    ).unwrap();
    
    manager.register_player(
        "bob456".to_string(),
        Some("Bob Jones".to_string()),
        None,
    ).unwrap();
    
    manager.register_player(
        "charlie789".to_string(),
        Some("Charlie Smith".to_string()),
        None,
    ).unwrap();
    
    // Search by name
    let results = manager.search_players("Smith").unwrap();
    assert_eq!(results.len(), 2);
    
    // Search by ID
    let results = manager.search_players("bob").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "bob456");
}

#[wasm_bindgen_test]
fn test_bulk_import_players() {
    let mut manager = PlayerManager::new();
    
    let players_json = r#"[
        {"id": "player1", "name": "Alice", "email": "alice@example.com"},
        {"id": "player2", "name": "Bob"},
        {"id": "player3"}
    ]"#;
    
    let imported = manager.bulk_import_players(players_json).unwrap();
    assert_eq!(imported, 3);
    assert_eq!(manager.player_count(), 3);
}

#[wasm_bindgen_test]
fn test_export_players() {
    let mut manager = PlayerManager::new();
    
    manager.register_player(
        "player1".to_string(),
        Some("Alice".to_string()),
        Some("alice@example.com".to_string()),
    ).unwrap();
    
    manager.register_player(
        "player2".to_string(),
        Some("Bob".to_string()),
        None,
    ).unwrap();
    
    let export = manager.export_players(true).unwrap();
    assert!(export.contains("player1"));
    assert!(export.contains("Alice"));
    assert!(export.contains("player2"));
    assert!(export.contains("Bob"));
}

#[wasm_bindgen_test]
fn test_get_player_head_to_head() {
    let mut manager = PlayerManager::new();
    
    manager.register_player("player1".to_string(), None, None).unwrap();
    manager.register_player("player2".to_string(), None, None).unwrap();
    
    // Player1 wins twice
    manager.add_match_record(
        vec!["player1".to_string()],
        vec!["player2".to_string()],
        1,
        None,
    ).unwrap();
    
    manager.add_match_record(
        vec!["player1".to_string()],
        vec!["player2".to_string()],
        1,
        None,
    ).unwrap();
    
    // Player2 wins once
    manager.add_match_record(
        vec!["player2".to_string()],
        vec!["player1".to_string()],
        1,
        None,
    ).unwrap();
    
    let h2h = manager.get_player_head_to_head("player1", "player2").unwrap();
    assert_eq!(h2h.total_matches, 3);
    assert_eq!(h2h.player1_wins, 2);
    assert_eq!(h2h.player2_wins, 1);
    assert_eq!(h2h.draws, 0);
}

#[wasm_bindgen_test]
fn test_merge_duplicate_players() {
    let mut manager = PlayerManager::new();
    
    // Create two players that are actually the same person
    manager.register_player(
        "alice_old".to_string(),
        Some("Alice".to_string()),
        None,
    ).unwrap();
    
    manager.register_player(
        "alice_new".to_string(),
        Some("Alice Smith".to_string()),
        Some("alice@example.com".to_string()),
    ).unwrap();
    
    // Add some match history to the old ID
    manager.register_player("opponent".to_string(), None, None).unwrap();
    manager.add_match_record(
        vec!["alice_old".to_string()],
        vec!["opponent".to_string()],
        1,
        None,
    ).unwrap();
    
    // Merge old into new
    manager.merge_players("alice_old", "alice_new").unwrap();
    
    // Old player should be deactivated
    assert!(!manager.is_player_active("alice_old"));
    
    // New player should have the match history
    let history = manager.get_player_match_history("alice_new", None, None).unwrap();
    assert_eq!(history.len(), 1);
}

#[wasm_bindgen_test]
fn test_player_aliases() {
    let mut manager = PlayerManager::new();
    
    manager.register_player("player1".to_string(), None, None).unwrap();
    
    // Add aliases
    manager.add_player_alias("player1", "TheChampion").unwrap();
    manager.add_player_alias("player1", "ChessKing").unwrap();
    
    // Should be able to find player by alias
    let profile = manager.get_player_profile("TheChampion").unwrap();
    assert_eq!(profile.id, "player1");
    
    let profile = manager.get_player_profile("ChessKing").unwrap();
    assert_eq!(profile.id, "player1");
    
    // Get all aliases
    let aliases = manager.get_player_aliases("player1").unwrap();
    assert_eq!(aliases.len(), 2);
    assert!(aliases.contains(&"TheChampion".to_string()));
    assert!(aliases.contains(&"ChessKing".to_string()));
}