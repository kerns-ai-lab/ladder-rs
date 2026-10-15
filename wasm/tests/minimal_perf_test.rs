//! Minimal performance test for WASM bindings
//!
//! This is the simplest possible test to ensure the WASM build works.

// Browser tests (wasm-pack test); on native targets they would be dead code.
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Basic smoke test
#[wasm_bindgen_test]
fn test_basic_wasm_functionality() {
    // Exercise the crate itself, not just the test harness
    let system = ladder_rs_wasm::EloSystem::new();
    assert_eq!(system.create_rating().value(), 1500.0);
}

/// Test that web_sys works
#[wasm_bindgen_test]
fn test_web_sys_available() {
    use web_sys::window;

    let win = window().expect("should have window");
    assert!(win.performance().is_some());
}
