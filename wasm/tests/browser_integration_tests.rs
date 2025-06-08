use wasm_bindgen_test::*;
use wasm_bindgen::prelude::*;
use web_sys::*;
use ladder_rs_wasm::*;
use crate::test_infrastructure::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Browser-specific integration tests for the WASM module.
/// These tests verify functionality that depends on browser APIs,
/// DOM manipulation, localStorage, and browser-specific features.

#[wasm_bindgen_test]
fn test_browser_storage_integration() {
    let logger = TestLogger::new();
    logger.log("Starting browser storage integration test");
    
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    
    // Add players and record matches
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    
    system.record_match(alice_id, bob_id, alice_id, None)
        .expect("Failed to record match");
    
    let alice_rating = system.get_player_rating(alice_id)
        .expect("Failed to get Alice's rating");
    
    // Test localStorage integration
    let window = web_sys::window().expect("Should have window in browser");
    let storage = window.local_storage()
        .expect("Should have localStorage")
        .expect("localStorage should be available");
    
    // Save system state to localStorage
    let state_key = "ladder_rs_test_state";
    let state_data = format!("alice_rating:{}", alice_rating);
    
    storage.set_item(state_key, &state_data)
        .expect("Should be able to save to localStorage");
    
    // Retrieve and verify
    let retrieved_data = storage.get_item(state_key)
        .expect("Should be able to retrieve from localStorage")
        .expect("Data should exist in localStorage");
    
    assert_eq!(retrieved_data, state_data, "Retrieved data should match saved data");
    
    // Clean up
    storage.remove_item(state_key)
        .expect("Should be able to remove from localStorage");
    
    logger.log("Browser storage integration test completed successfully");
}

#[wasm_bindgen_test]
fn test_dom_leaderboard_rendering() {
    let logger = TestLogger::new();
    logger.log("Starting DOM leaderboard rendering test");
    
    let system = WasmRatingSystem::new("glicko").expect("Failed to create Glicko system");
    
    // Add several players
    let players = vec!["Alice", "Bob", "Carol", "Dave"];
    let mut player_ids = Vec::new();
    
    for name in &players {
        let id = system.add_player(name).expect("Failed to add player");
        player_ids.push(id);
    }
    
    // Play some matches to create rating differences
    system.record_match(player_ids[0], player_ids[1], player_ids[0], None)
        .expect("Failed to record Alice vs Bob");
    system.record_match(player_ids[2], player_ids[3], player_ids[2], None)
        .expect("Failed to record Carol vs Dave");
    system.record_match(player_ids[0], player_ids[2], player_ids[0], None)
        .expect("Failed to record Alice vs Carol");
    
    // Get the document and create a test container
    let document = web_sys::window()
        .expect("Should have window")
        .document()
        .expect("Should have document");
    
    let container = document.create_element("div")
        .expect("Should be able to create div");
    container.set_id("leaderboard-test-container");
    
    // Get leaderboard data
    let leaderboard = system.get_leaderboard()
        .expect("Failed to get leaderboard");
    
    // Render leaderboard to DOM
    let mut table_html = String::from("<table><thead><tr><th>Rank</th><th>Player</th><th>Rating</th></tr></thead><tbody>");
    
    for (rank, player) in leaderboard.iter().enumerate() {
        table_html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{:.1}</td></tr>",
            rank + 1,
            player.name(),
            player.rating()
        ));
    }
    
    table_html.push_str("</tbody></table>");
    
    container.set_inner_html(&table_html);
    
    // Verify DOM structure
    let table = container.query_selector("table")
        .expect("Should be able to query table")
        .expect("Table should exist");
    
    let rows = table.query_selector_all("tbody tr")
        .expect("Should be able to query rows");
    
    assert_eq!(rows.length(), 4, "Should have 4 player rows");
    
    // Verify that Alice (who won 2 matches) is at the top
    let first_row = rows.get(0).expect("Should have first row");
    let first_row_html = first_row.unchecked_ref::<Element>().inner_html();
    assert!(first_row_html.contains("Alice"), "Alice should be in first row");
    assert!(first_row_html.contains("<td>1</td>"), "Should show rank 1");
    
    logger.log("DOM leaderboard rendering test completed successfully");
}

#[wasm_bindgen_test]
fn test_performance_api_integration() {
    let logger = TestLogger::new();
    logger.log("Starting Performance API integration test");
    
    let window = web_sys::window().expect("Should have window");
    let performance = window.performance()
        .expect("Should have Performance API");
    
    let system = WasmRatingSystem::new("trueskill").expect("Failed to create TrueSkill system");
    
    // Measure player creation performance
    let start_time = performance.now();
    
    let mut player_ids = Vec::new();
    for i in 0..50 {
        let name = format!("Player_{}", i);
        let id = system.add_player(&name).expect("Failed to add player");
        player_ids.push(id);
    }
    
    let creation_time = performance.now() - start_time;
    
    // Measure match recording performance
    let match_start_time = performance.now();
    
    for i in 0..25 {
        let player1_idx = i * 2;
        let player2_idx = i * 2 + 1;
        let winner_idx = if i % 2 == 0 { player1_idx } else { player2_idx };
        
        system.record_match(
            player_ids[player1_idx],
            player_ids[player2_idx],
            player_ids[winner_idx],
            None
        ).expect("Failed to record match");
    }
    
    let match_time = performance.now() - match_start_time;
    
    // Measure leaderboard generation performance
    let leaderboard_start_time = performance.now();
    let leaderboard = system.get_leaderboard().expect("Failed to get leaderboard");
    let leaderboard_time = performance.now() - leaderboard_start_time;
    
    logger.log(&format!(
        "Performance metrics - Creation: {:.2}ms, Matches: {:.2}ms, Leaderboard: {:.2}ms",
        creation_time, match_time, leaderboard_time
    ));
    
    // Verify performance is reasonable (these are loose bounds for browser tests)
    assert!(creation_time < 500.0, "Player creation should be reasonably fast");
    assert!(match_time < 500.0, "Match recording should be reasonably fast");
    assert!(leaderboard_time < 100.0, "Leaderboard generation should be fast");
    
    // Verify results are correct
    assert_eq!(leaderboard.len(), 50, "Should have all 50 players");
    
    logger.log("Performance API integration test completed successfully");
}

#[wasm_bindgen_test]
fn test_browser_event_handling() {
    let logger = TestLogger::new();
    logger.log("Starting browser event handling test");
    
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    
    // Get document for event testing
    let document = web_sys::window()
        .expect("Should have window")
        .document()
        .expect("Should have document");
    
    // Create a button for testing events
    let button = document.create_element("button")
        .expect("Should be able to create button");
    button.set_text_content(Some("Record Match"));
    button.set_id("test-match-button");
    
    // Create a results div
    let results_div = document.create_element("div")
        .expect("Should be able to create div");
    results_div.set_id("test-results");
    
    // Simulate button click by directly calling our function
    // (In real usage, this would be triggered by an event listener)
    let result = system.record_match(alice_id, bob_id, alice_id, None);
    assert!(result.is_ok(), "Match should be recorded successfully");
    
    // Update results div with match outcome
    let alice_rating = system.get_player_rating(alice_id)
        .expect("Failed to get Alice's rating");
    let bob_rating = system.get_player_rating(bob_id)
        .expect("Failed to get Bob's rating");
    
    let results_text = format!(
        "Match recorded! Alice: {:.1}, Bob: {:.1}",
        alice_rating, bob_rating
    );
    results_div.set_text_content(Some(&results_text));
    
    // Verify the results were updated
    let updated_text = results_div.text_content()
        .expect("Should have text content");
    assert!(updated_text.contains("Match recorded!"), "Results should be updated");
    assert!(updated_text.contains("Alice:"), "Should contain Alice's rating");
    assert!(updated_text.contains("Bob:"), "Should contain Bob's rating");
    
    logger.log("Browser event handling test completed successfully");
}

#[wasm_bindgen_test]
fn test_browser_url_handling() {
    let logger = TestLogger::new();
    logger.log("Starting browser URL handling test");
    
    let window = web_sys::window().expect("Should have window");
    let location = window.location();
    
    // Get current URL info (this will be the test page URL)
    let current_href = location.href().expect("Should be able to get href");
    logger.log(&format!("Current URL: {}", current_href));
    
    // Test URL fragment handling (commonly used for SPA routing)
    let test_fragment = "#leaderboard/elo";
    location.set_hash(test_fragment).expect("Should be able to set hash");
    
    let new_hash = location.hash().expect("Should be able to get hash");
    assert_eq!(new_hash, test_fragment, "Hash should be set correctly");
    
    // Parse the fragment to simulate routing
    let parts: Vec<&str> = test_fragment.trim_start_matches('#').split('/').collect();
    assert_eq!(parts.len(), 2, "Should have 2 URL parts");
    assert_eq!(parts[0], "leaderboard", "First part should be 'leaderboard'");
    assert_eq!(parts[1], "elo", "Second part should be 'elo'");
    
    // Create rating system based on URL parameter
    let system = WasmRatingSystem::new(parts[1]).expect("Should create system from URL");
    
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let alice_rating = system.get_player_rating(alice_id)
        .expect("Failed to get Alice's rating");
    
    // Verify the system type is correct based on URL
    if parts[1] == "elo" {
        assert_eq!(alice_rating, 1500.0, "Elo should start at 1500");
    }
    
    // Clean up by removing the hash
    location.set_hash("").expect("Should be able to clear hash");
    
    logger.log("Browser URL handling test completed successfully");
}

#[wasm_bindgen_test]
fn test_browser_responsive_behavior() {
    let logger = TestLogger::new();
    logger.log("Starting browser responsive behavior test");
    
    let window = web_sys::window().expect("Should have window");
    let document = window.document().expect("Should have document");
    
    let system = WasmRatingSystem::new("glicko").expect("Failed to create Glicko system");
    
    // Add multiple players
    for i in 0..10 {
        let name = format!("Player_{}", i);
        system.add_player(&name).expect("Failed to add player");
    }
    
    let leaderboard = system.get_leaderboard().expect("Failed to get leaderboard");
    
    // Create responsive table container
    let container = document.create_element("div")
        .expect("Should be able to create div");
    container.set_class_name("responsive-container");
    
    // Simulate different screen sizes by adjusting container width
    let test_widths = vec!["320px", "768px", "1024px"]; // Mobile, tablet, desktop
    
    for width in test_widths {
        container.style().set_property("width", width)
            .expect("Should be able to set width");
        
        // Generate appropriate content for this screen size
        let content = if width == "320px" {
            // Mobile: Show only top 5 players, compact format
            let mut mobile_html = String::from("<ul class='mobile-leaderboard'>");
            for (i, player) in leaderboard.iter().take(5).enumerate() {
                mobile_html.push_str(&format!(
                    "<li>{}. {} ({:.0})</li>",
                    i + 1,
                    player.name(),
                    player.rating()
                ));
            }
            mobile_html.push_str("</ul>");
            mobile_html
        } else {
            // Tablet/Desktop: Show full table
            let mut table_html = String::from("<table class='full-leaderboard'>");
            table_html.push_str("<thead><tr><th>Rank</th><th>Player</th><th>Rating</th></tr></thead><tbody>");
            for (i, player) in leaderboard.iter().enumerate() {
                table_html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{:.1}</td></tr>",
                    i + 1,
                    player.name(),
                    player.rating()
                ));
            }
            table_html.push_str("</tbody></table>");
            table_html
        };
        
        container.set_inner_html(&content);
        
        // Verify content was set correctly
        let html_content = container.inner_html();
        if width == "320px" {
            assert!(html_content.contains("<ul"), "Mobile should use list format");
            assert!(html_content.contains("Player_0"), "Should contain first player");
        } else {
            assert!(html_content.contains("<table"), "Desktop should use table format");
            assert!(html_content.contains("<thead"), "Should have table header");
        }
        
        logger.log(&format!("Responsive layout tested for width: {}", width));
    }
    
    logger.log("Browser responsive behavior test completed successfully");
}

#[wasm_bindgen_test]
fn test_browser_accessibility_features() {
    let logger = TestLogger::new();
    logger.log("Starting browser accessibility features test");
    
    let document = web_sys::window()
        .expect("Should have window")
        .document()
        .expect("Should have document");
    
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    
    system.record_match(alice_id, bob_id, alice_id, None)
        .expect("Failed to record match");
    
    let leaderboard = system.get_leaderboard().expect("Failed to get leaderboard");
    
    // Create accessible leaderboard table
    let table = document.create_element("table")
        .expect("Should be able to create table");
    
    // Set accessibility attributes
    table.set_attribute("role", "table")
        .expect("Should be able to set role");
    table.set_attribute("aria-label", "Player leaderboard")
        .expect("Should be able to set aria-label");
    
    // Create caption for screen readers
    let caption = document.create_element("caption")
        .expect("Should be able to create caption");
    caption.set_text_content(Some("Current player rankings with ratings"));
    table.append_child(&caption)
        .expect("Should be able to append caption");
    
    // Create accessible header
    let thead = document.create_element("thead")
        .expect("Should be able to create thead");
    let header_row = document.create_element("tr")
        .expect("Should be able to create tr");
    
    let headers = vec![
        ("Rank", "Player ranking position"),
        ("Player", "Player name"),
        ("Rating", "Current skill rating")
    ];
    
    for (text, description) in headers {
        let th = document.create_element("th")
            .expect("Should be able to create th");
        th.set_text_content(Some(text));
        th.set_attribute("scope", "col")
            .expect("Should be able to set scope");
        th.set_attribute("aria-describedby", &format!("{}-desc", text.to_lowercase()))
            .expect("Should be able to set aria-describedby");
        
        header_row.append_child(&th)
            .expect("Should be able to append th");
    }
    
    thead.append_child(&header_row)
        .expect("Should be able to append header row");
    table.append_child(&thead)
        .expect("Should be able to append thead");
    
    // Create accessible body
    let tbody = document.create_element("tbody")
        .expect("Should be able to create tbody");
    
    for (rank, player) in leaderboard.iter().enumerate() {
        let row = document.create_element("tr")
            .expect("Should be able to create tr");
        
        // Set ARIA attributes for the row
        row.set_attribute("role", "row")
            .expect("Should be able to set role");
        row.set_attribute("aria-rowindex", &(rank + 2).to_string()) // +2 because header is row 1
            .expect("Should be able to set aria-rowindex");
        
        let cells = vec![
            (rank + 1).to_string(),
            player.name().to_string(),
            format!("{:.1}", player.rating())
        ];
        
        for (i, cell_text) in cells.iter().enumerate() {
            let td = document.create_element("td")
                .expect("Should be able to create td");
            td.set_text_content(Some(cell_text));
            td.set_attribute("role", "cell")
                .expect("Should be able to set role");
            
            // Add specific attributes for different columns
            match i {
                0 => td.set_attribute("aria-label", &format!("Rank {}", cell_text))
                    .expect("Should be able to set rank aria-label"),
                1 => td.set_attribute("aria-label", &format!("Player name: {}", cell_text))
                    .expect("Should be able to set name aria-label"),
                2 => td.set_attribute("aria-label", &format!("Rating: {} points", cell_text))
                    .expect("Should be able to set rating aria-label"),
                _ => {}
            }
            
            row.append_child(&td)
                .expect("Should be able to append td");
        }
        
        tbody.append_child(&row)
            .expect("Should be able to append row");
    }
    
    table.append_child(&tbody)
        .expect("Should be able to append tbody");
    
    // Verify accessibility attributes are set
    assert_eq!(
        table.get_attribute("role").as_deref(),
        Some("table"),
        "Table should have role attribute"
    );
    assert_eq!(
        table.get_attribute("aria-label").as_deref(),
        Some("Player leaderboard"),
        "Table should have aria-label"
    );
    
    // Verify table structure
    let captions = table.query_selector_all("caption")
        .expect("Should be able to query captions");
    assert_eq!(captions.length(), 1, "Should have one caption");
    
    let th_elements = table.query_selector_all("th")
        .expect("Should be able to query th elements");
    assert_eq!(th_elements.length(), 3, "Should have three header cells");
    
    let td_elements = table.query_selector_all("td")
        .expect("Should be able to query td elements");
    assert_eq!(td_elements.length(), 6, "Should have six data cells (2 players × 3 columns)");
    
    logger.log("Browser accessibility features test completed successfully");
}

#[wasm_bindgen_test]
fn test_browser_error_display() {
    let logger = TestLogger::new();
    logger.log("Starting browser error display test");
    
    let document = web_sys::window()
        .expect("Should have window")
        .document()
        .expect("Should have document");
    
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    
    // Create error display container
    let error_container = document.create_element("div")
        .expect("Should be able to create div");
    error_container.set_id("error-display");
    error_container.set_class_name("error-container hidden");
    
    // Function to display error in browser
    let show_error = |message: &str| {
        error_container.set_class_name("error-container visible");
        error_container.set_inner_html(&format!(
            "<div class='error-message'><strong>Error:</strong> {}</div>",
            message
        ));
    };
    
    let hide_error = || {
        error_container.set_class_name("error-container hidden");
        error_container.set_inner_html("");
    };
    
    // Test various error scenarios
    
    // 1. Invalid player ID error
    let invalid_rating_result = system.get_player_rating(9999);
    if let Err(_) = invalid_rating_result {
        show_error("Player not found");
        
        // Verify error is displayed
        let error_html = error_container.inner_html();
        assert!(error_html.contains("Error:"), "Should display error message");
        assert!(error_html.contains("Player not found"), "Should show specific error");
        assert!(error_container.class_name().contains("visible"), "Error should be visible");
        
        hide_error();
    }
    
    // 2. Invalid match error
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let invalid_match_result = system.record_match(alice_id, alice_id, alice_id, None);
    if let Err(_) = invalid_match_result {
        show_error("Player cannot play against themselves");
        
        let error_html = error_container.inner_html();
        assert!(error_html.contains("cannot play against themselves"), "Should show self-match error");
        
        hide_error();
    }
    
    // 3. Success scenario (no error)
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    let valid_match_result = system.record_match(alice_id, bob_id, alice_id, None);
    if valid_match_result.is_ok() {
        // Error should remain hidden for successful operations
        assert!(error_container.class_name().contains("hidden"), "Error should be hidden for success");
        assert_eq!(error_container.inner_html(), "", "Error container should be empty");
    }
    
    logger.log("Browser error display test completed successfully");
}