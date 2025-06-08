#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_bindgen::JsValue;
use serde_json::json;
use web_sys::window;

extern crate ladder_rs_wasm;
use ladder_rs_wasm::{WasmRatingSystem, WasmTeam};

wasm_bindgen_test_configure!(run_in_browser);

/// Test browser-specific functionality like localStorage and Performance API
#[wasm_bindgen_test]
fn test_browser_storage_integration() {
    console_error_panic_hook::set_once();
    
    // Create a rating system
    let config = json!({
        "type": "elo",
        "k_factor": 32.0
    });
    
    let mut system = WasmRatingSystem::new("elo", JsValue::from_serde(&config).unwrap())
        .expect("Failed to create Elo system");
    
    // Create players
    let player1 = system.create_player("browser_player1".to_string())
        .expect("Failed to create player1");
    let player2 = system.create_player("browser_player2".to_string())
        .expect("Failed to create player2");
    
    // Access browser localStorage (if available)
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            // Store some test data
            let player_data = format!("{{\"id\": \"{}\", \"rating\": {}}}", 
                                    player1.player_id, player1.rating);
            
            let _ = storage.set_item("test_player", &player_data);
            
            // Retrieve and verify
            if let Ok(Some(stored_data)) = storage.get_item("test_player") {
                assert!(stored_data.contains(&player1.player_id));
                web_sys::console::log_1(&"Successfully stored and retrieved player data from localStorage".into());
            }
            
            // Clean up
            let _ = storage.remove_item("test_player");
        }
    }
    
    web_sys::console::log_1(&"Browser storage integration test completed!".into());
}

/// Test performance measurement capabilities
#[wasm_bindgen_test]
fn test_browser_performance_integration() {
    console_error_panic_hook::set_once();
    
    if let Some(window) = window() {
        if let Ok(performance) = window.performance() {
            let start_time = performance.now();
            
            // Create and run multiple rating calculations
            let config = json!({
                "type": "trueskill",
                "beta": 4.166666666666667,
                "tau": 0.08333333333333333
            });
            
            let mut system = WasmRatingSystem::new("trueskill", JsValue::from_serde(&config).unwrap())
                .expect("Failed to create TrueSkill system");
            
            // Perform multiple operations to measure performance
            for i in 0..10 {
                let player1 = system.create_player(format!("perf_player_{}_1", i))
                    .expect("Failed to create player1");
                let player2 = system.create_player(format!("perf_player_{}_2", i))
                    .expect("Failed to create player2");
                
                let mut team1 = WasmTeam::new(1.0);
                let mut team2 = WasmTeam::new(2.0);
                
                team1.add_player(player1);
                team2.add_player(player2);
                
                let _ = system.update_ratings(vec![team1, team2])
                    .expect("Failed to update ratings");
            }
            
            let end_time = performance.now();
            let duration = end_time - start_time;
            
            let msg = format!("Performed 10 TrueSkill calculations in {:.2}ms", duration);
            web_sys::console::log_1(&msg.into());
            
            // Performance should be reasonable (less than 100ms for 10 calculations)
            assert!(duration < 100.0, "Performance test failed: {}ms > 100ms", duration);
        }
    }
    
    web_sys::console::log_1(&"Browser performance integration test completed!".into());
}

/// Test DOM manipulation capabilities (basic)
#[wasm_bindgen_test]
fn test_browser_dom_integration() {
    console_error_panic_hook::set_once();
    
    if let Some(window) = window() {
        if let Ok(document) = window.document() {
            // Create a div element to display rating information
            if let Ok(div) = document.create_element("div") {
                let _ = div.set_attribute("id", "rating-display");
                
                // Create rating system and player
                let config = json!({
                    "type": "glicko",
                    "initial_volatility": 0.06
                });
                
                let mut system = WasmRatingSystem::new("glicko", JsValue::from_serde(&config).unwrap())
                    .expect("Failed to create Glicko system");
                
                let player = system.create_player("dom_player".to_string())
                    .expect("Failed to create player");
                
                // Display rating information in the div
                let rating_text = format!("Player: {} | Rating: {:.1} | Uncertainty: {:.1}", 
                                        player.player_id, 
                                        player.rating,
                                        player.uncertainty.unwrap_or(0.0));
                
                div.set_text_content(Some(&rating_text));
                
                // Verify the content was set
                if let Some(content) = div.text_content() {
                    assert!(content.contains(&player.player_id));
                    assert!(content.contains("1500"));
                    web_sys::console::log_1(&"Successfully updated DOM with rating information".into());
                }
            }
        }
    }
    
    web_sys::console::log_1(&"Browser DOM integration test completed!".into());
}