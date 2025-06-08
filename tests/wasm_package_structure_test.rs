//! Integration tests for WASM package structure setup
//!
//! This test validates that Task 1.1.1 (Create WASM Package Structure) is completed correctly.
//!
//! Requirements tested:
//! - wasm/ directory exists in project root
//! - wasm/Cargo.toml exists with proper WASM configuration
//! - Proper crate-type and target specifications
//! - Required WASM dependencies are configured
//! - Feature flags for optional functionality are set up

use std::fs;
use std::path::Path;

#[test]
fn test_wasm_directory_exists() {
    let wasm_dir = Path::new("wasm");
    assert!(
        wasm_dir.exists() && wasm_dir.is_dir(),
        "wasm/ directory should exist in project root"
    );
}

#[test]
fn test_wasm_cargo_toml_exists() {
    let cargo_toml = Path::new("wasm/Cargo.toml");
    assert!(
        cargo_toml.exists() && cargo_toml.is_file(),
        "wasm/Cargo.toml should exist"
    );
}

#[test]
fn test_wasm_cargo_toml_configuration() {
    let cargo_toml_path = Path::new("wasm/Cargo.toml");

    // Read the Cargo.toml content
    let content =
        fs::read_to_string(cargo_toml_path).expect("Should be able to read wasm/Cargo.toml");

    // Parse as TOML to validate syntax
    let toml_value: toml::Value =
        toml::from_str(&content).expect("wasm/Cargo.toml should be valid TOML");

    // Check package configuration
    let package = toml_value
        .get("package")
        .expect("wasm/Cargo.toml should have [package] section");

    assert!(package.get("name").is_some(), "Package should have a name");

    assert!(
        package.get("version").is_some(),
        "Package should have a version"
    );

    assert!(
        package.get("edition").is_some(),
        "Package should specify Rust edition"
    );

    // Check lib configuration for WASM
    let lib = toml_value
        .get("lib")
        .expect("wasm/Cargo.toml should have [lib] section for WASM");

    let crate_type = lib
        .get("crate-type")
        .expect("Should specify crate-type for WASM")
        .as_array()
        .expect("crate-type should be an array");

    assert!(
        crate_type.iter().any(|t| t.as_str() == Some("cdylib")),
        "Should include 'cdylib' crate type for WASM"
    );
}

#[test]
fn test_wasm_dependencies() {
    let cargo_toml_path = Path::new("wasm/Cargo.toml");
    let content =
        fs::read_to_string(cargo_toml_path).expect("Should be able to read wasm/Cargo.toml");

    let toml_value: toml::Value =
        toml::from_str(&content).expect("wasm/Cargo.toml should be valid TOML");

    let dependencies = toml_value
        .get("dependencies")
        .expect("wasm/Cargo.toml should have [dependencies] section");

    // Check for required WASM dependencies
    let required_deps = [
        "wasm-bindgen",
        "wasm-bindgen-futures",
        "js-sys",
        "web-sys",
        "serde",
        "serde-wasm-bindgen",
        "console_error_panic_hook",
    ];

    for dep in required_deps {
        assert!(
            dependencies.get(dep).is_some(),
            "Should include {} dependency",
            dep
        );
    }

    // Check for ladder-rs dependency with proper path
    let ladder_rs_dep = dependencies
        .get("ladder-rs")
        .expect("Should include ladder-rs dependency");

    if let Some(dep_table) = ladder_rs_dep.as_table() {
        assert!(
            dep_table.get("path").is_some(),
            "ladder-rs dependency should use local path"
        );
    }
}

#[test]
fn test_optional_dependencies() {
    let cargo_toml_path = Path::new("wasm/Cargo.toml");
    let content =
        fs::read_to_string(cargo_toml_path).expect("Should be able to read wasm/Cargo.toml");

    let toml_value: toml::Value =
        toml::from_str(&content).expect("wasm/Cargo.toml should be valid TOML");

    let dependencies = toml_value
        .get("dependencies")
        .expect("wasm/Cargo.toml should have [dependencies] section");

    // Check for wee_alloc as optional dependency
    if let Some(wee_alloc) = dependencies.get("wee_alloc") {
        if let Some(dep_table) = wee_alloc.as_table() {
            assert!(
                dep_table.get("optional").and_then(|v| v.as_bool()) == Some(true),
                "wee_alloc should be marked as optional"
            );
        }
    }
}

#[test]
fn test_feature_flags() {
    let cargo_toml_path = Path::new("wasm/Cargo.toml");
    let content =
        fs::read_to_string(cargo_toml_path).expect("Should be able to read wasm/Cargo.toml");

    let toml_value: toml::Value =
        toml::from_str(&content).expect("wasm/Cargo.toml should be valid TOML");

    // Check for features section
    if let Some(features) = toml_value.get("features") {
        // Should have default features defined
        assert!(
            features.get("default").is_some(),
            "Should define default features"
        );

        // Should have wee_alloc feature for size optimization
        if features.get("wee_alloc").is_some() {
            let wee_alloc_features = features
                .get("wee_alloc")
                .and_then(|f| f.as_array())
                .expect("wee_alloc feature should be an array");

            assert!(
                wee_alloc_features
                    .iter()
                    .any(|f| f.as_str() == Some("dep:wee_alloc")),
                "wee_alloc feature should enable the wee_alloc dependency"
            );
        }
    }
}

#[test]
fn test_wasm_src_directory_structure() {
    let src_dir = Path::new("wasm/src");
    assert!(
        src_dir.exists() && src_dir.is_dir(),
        "wasm/src/ directory should exist"
    );

    let lib_rs = Path::new("wasm/src/lib.rs");
    assert!(
        lib_rs.exists() && lib_rs.is_file(),
        "wasm/src/lib.rs should exist"
    );
}

#[test]
fn test_wasm_lib_rs_basic_structure() {
    let lib_rs_path = Path::new("wasm/src/lib.rs");
    let content = fs::read_to_string(lib_rs_path).expect("Should be able to read wasm/src/lib.rs");

    // Check for basic WASM setup code
    assert!(
        content.contains("wasm_bindgen"),
        "lib.rs should import wasm_bindgen"
    );

    assert!(
        content.contains("console_error_panic_hook"),
        "lib.rs should set up panic hook for debugging"
    );
}

#[test]
fn test_cargo_workspace_integration() {
    // Check that the main Cargo.toml doesn't conflict with WASM package
    let main_cargo_toml = Path::new("Cargo.toml");
    let content =
        fs::read_to_string(main_cargo_toml).expect("Should be able to read root Cargo.toml");

    // Parse to ensure it's still valid after adding WASM package
    let _toml_value: toml::Value =
        toml::from_str(&content).expect("Root Cargo.toml should remain valid TOML");

    // The wasm package should be separate, not a workspace member initially
    // This ensures clean separation during development
}

// Helper function to validate WASM target compatibility
#[test]
fn test_wasm_target_compatibility() {
    let cargo_toml_path = Path::new("wasm/Cargo.toml");
    let content =
        fs::read_to_string(cargo_toml_path).expect("Should be able to read wasm/Cargo.toml");

    let toml_value: toml::Value =
        toml::from_str(&content).expect("wasm/Cargo.toml should be valid TOML");

    // Ensure no conflicting target-specific configurations
    assert!(
        !content.contains("[target.") || content.contains("wasm32"),
        "Any target-specific configuration should be WASM-compatible"
    );

    // Check that lib.name is set for predictable output
    if let Some(lib) = toml_value.get("lib") {
        if let Some(name) = lib.get("name") {
            assert!(
                name.as_str().is_some(),
                "If lib.name is specified, it should be a valid string"
            );
        }
    }
}
