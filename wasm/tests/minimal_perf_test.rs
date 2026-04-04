//! Minimal performance test for WASM bindings
//!
//! This is the simplest possible test to ensure the WASM build works.

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Basic smoke test
#[wasm_bindgen_test]
fn test_basic_wasm_functionality() {
    // Just test that we can run a test
    assert_eq!(1 + 1, 2);
}

/// Test that web_sys works
#[wasm_bindgen_test]
fn test_web_sys_available() {
    use web_sys::window;

    let win = window().expect("should have window");
    assert!(win.performance().is_some());
}
