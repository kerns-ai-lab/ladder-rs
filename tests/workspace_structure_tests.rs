//! Workspace structure and compilation tests for ladder-rs.
//!
//! These tests verify that the Cargo workspace is correctly configured,
//! all crates compile, and the dependency graph is valid.

use std::fs;
use std::path::Path;
use std::process::Command;

const WORKSPACE_ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// Helper to read a Cargo.toml file and parse it as TOML
fn read_cargo_toml(path: &str) -> Result<toml::Value, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let value: toml::Value = toml::from_str(&content)?;
    Ok(value)
}

/// Helper to run a cargo command and check success
fn run_cargo(args: &[&str]) -> bool {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(WORKSPACE_ROOT)
        .output()
        .expect("Failed to execute cargo");
    output.status.success()
}

// ============================================================================
// Workspace Structure Tests
// ============================================================================

#[test]
fn test_workspace_root_cargo_toml_exists() {
    let path = Path::new(WORKSPACE_ROOT).join("Cargo.toml");
    assert!(path.exists(), "Workspace root Cargo.toml must exist");
}

#[test]
fn test_workspace_has_correct_members() {
    let cargo_toml = read_cargo_toml(&format!("{}/Cargo.toml", WORKSPACE_ROOT))
        .expect("Failed to parse workspace Cargo.toml");

    let workspace = cargo_toml
        .get("workspace")
        .expect("Cargo.toml must have [workspace] section");

    let members = workspace
        .get("members")
        .expect("[workspace] must have members array")
        .as_array()
        .expect("members must be an array");

    let member_strings: Vec<&str> = members.iter().filter_map(|m| m.as_str()).collect();

    // Verify all expected members are present
    assert!(
        member_strings
            .iter()
            .any(|m| m.contains("ladder-rs-persistence")),
        "Workspace must include ladder-rs-persistence crate"
    );
    assert!(
        member_strings
            .iter()
            .any(|m| m.contains("ladder-rs-server")),
        "Workspace must include ladder-rs-server crate"
    );
    assert!(
        member_strings.iter().any(|m| m.contains("wasm")),
        "Workspace must include wasm crate"
    );
}

#[test]
fn test_workspace_uses_resolver_v2() {
    let cargo_toml = read_cargo_toml(&format!("{}/Cargo.toml", WORKSPACE_ROOT))
        .expect("Failed to parse workspace Cargo.toml");

    let resolver = cargo_toml
        .get("workspace")
        .and_then(|w| w.get("resolver"))
        .and_then(|r| r.as_str());

    assert_eq!(
        resolver,
        Some("2"),
        "Workspace must use resolver = \"2\" for edition 2021"
    );
}

// ============================================================================
// Individual Crate Tests
// ============================================================================

#[test]
fn test_ladder_rs_persistence_cargo_toml_exists() {
    let path = Path::new(WORKSPACE_ROOT)
        .join("ladder-rs-persistence")
        .join("Cargo.toml");
    assert!(path.exists(), "ladder-rs-persistence Cargo.toml must exist");
}

#[test]
fn test_ladder_rs_persistence_has_required_dependencies() {
    let cargo_toml = read_cargo_toml(&format!(
        "{}/ladder-rs-persistence/Cargo.toml",
        WORKSPACE_ROOT
    ))
    .expect("Failed to parse ladder-rs-persistence Cargo.toml");

    let deps = cargo_toml
        .get("dependencies")
        .expect("ladder-rs-persistence must have [dependencies]");

    // Must depend on ladder-rs
    assert!(
        deps.get("ladder-rs").is_some(),
        "ladder-rs-persistence must depend on ladder-rs"
    );

    // Must use sqlx for database access
    assert!(
        deps.get("sqlx").is_some(),
        "ladder-rs-persistence must depend on sqlx"
    );

    // Must use tokio for async runtime
    assert!(
        deps.get("tokio").is_some(),
        "ladder-rs-persistence must depend on tokio"
    );

    // Must use thiserror for error handling
    assert!(
        deps.get("thiserror").is_some(),
        "ladder-rs-persistence must depend on thiserror"
    );

    // Must use serde for serialization
    assert!(
        deps.get("serde").is_some(),
        "ladder-rs-persistence must depend on serde"
    );
}

#[test]
fn test_ladder_rs_server_cargo_toml_exists() {
    let path = Path::new(WORKSPACE_ROOT)
        .join("ladder-rs-server")
        .join("Cargo.toml");
    assert!(path.exists(), "ladder-rs-server Cargo.toml must exist");
}

#[test]
fn test_ladder_rs_server_has_required_dependencies() {
    let cargo_toml = read_cargo_toml(&format!("{}/ladder-rs-server/Cargo.toml", WORKSPACE_ROOT))
        .expect("Failed to parse ladder-rs-server Cargo.toml");

    let deps = cargo_toml
        .get("dependencies")
        .expect("ladder-rs-server must have [dependencies]");

    // Must depend on ladder-rs-persistence
    assert!(
        deps.get("ladder-rs-persistence").is_some(),
        "ladder-rs-server must depend on ladder-rs-persistence"
    );

    // Must use axum for HTTP server
    assert!(
        deps.get("axum").is_some(),
        "ladder-rs-server must depend on axum"
    );

    // Must use tower for middleware
    assert!(
        deps.get("tower").is_some(),
        "ladder-rs-server must depend on tower"
    );

    // Must use tokio for async runtime
    assert!(
        deps.get("tokio").is_some(),
        "ladder-rs-server must depend on tokio"
    );
}

#[test]
fn test_wasm_crate_cargo_toml_exists() {
    let path = Path::new(WORKSPACE_ROOT).join("wasm").join("Cargo.toml");
    assert!(path.exists(), "wasm crate Cargo.toml must exist");
}

// ============================================================================
// Workspace Dependency Tests
// ============================================================================

#[test]
fn test_workspace_has_shared_dependencies() {
    let cargo_toml = read_cargo_toml(&format!("{}/Cargo.toml", WORKSPACE_ROOT))
        .expect("Failed to parse workspace Cargo.toml");

    let workspace_deps = cargo_toml
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .expect("[workspace.dependencies] must exist");

    // Key shared dependencies that should be in workspace
    let expected_deps = [
        "thiserror",
        "serde",
        "serde_json",
        "chrono",
        "tokio",
        "sqlx",
        "axum",
        "tower",
        "tower-http",
        "tracing",
        "uuid",
    ];

    for dep in &expected_deps {
        assert!(
            workspace_deps.get(dep).is_some(),
            "Workspace must define shared dependency: {}",
            dep
        );
    }
}

#[test]
fn test_persistence_uses_workspace_dependencies() {
    let cargo_toml = read_cargo_toml(&format!(
        "{}/ladder-rs-persistence/Cargo.toml",
        WORKSPACE_ROOT
    ))
    .expect("Failed to parse ladder-rs-persistence Cargo.toml");

    let deps = cargo_toml
        .get("dependencies")
        .expect("ladder-rs-persistence must have [dependencies]");

    // Check that key dependencies use workspace inheritance
    let workspace_deps = ["thiserror", "serde", "tokio", "sqlx", "uuid", "tracing"];

    for dep in &workspace_deps {
        let dep_value = deps
            .get(dep)
            .unwrap_or_else(|| panic!("ladder-rs-persistence must depend on {}", dep));

        // Should use .workspace = true
        assert!(
            dep_value.get("workspace").and_then(|v| v.as_bool()) == Some(true),
            "ladder-rs-persistence should use workspace dependency for {}",
            dep
        );
    }
}

#[test]
fn test_server_uses_workspace_dependencies() {
    let cargo_toml = read_cargo_toml(&format!("{}/ladder-rs-server/Cargo.toml", WORKSPACE_ROOT))
        .expect("Failed to parse ladder-rs-server Cargo.toml");

    let deps = cargo_toml
        .get("dependencies")
        .expect("ladder-rs-server must have [dependencies]");

    // Check that key dependencies use workspace inheritance
    let workspace_deps = [
        "thiserror",
        "serde",
        "tokio",
        "axum",
        "tower",
        "tower-http",
        "tracing",
        "uuid",
    ];

    for dep in &workspace_deps {
        let dep_value = deps
            .get(dep)
            .unwrap_or_else(|| panic!("ladder-rs-server must depend on {}", dep));

        // Should use .workspace = true
        assert!(
            dep_value.get("workspace").and_then(|v| v.as_bool()) == Some(true),
            "ladder-rs-server should use workspace dependency for {}",
            dep
        );
    }
}

// ============================================================================
// Compilation Tests
// ============================================================================

#[test]
fn test_workspace_compiles() {
    assert!(
        run_cargo(&["check", "--workspace"]),
        "Entire workspace must compile successfully"
    );
}

#[test]
fn test_persistence_crate_compiles() {
    assert!(
        run_cargo(&["check", "-p", "ladder-rs-persistence"]),
        "ladder-rs-persistence crate must compile successfully"
    );
}

#[test]
fn test_server_crate_compiles() {
    assert!(
        run_cargo(&["check", "-p", "ladder-rs-server"]),
        "ladder-rs-server crate must compile successfully"
    );
}

#[test]
fn test_workspace_tests_compile() {
    assert!(
        run_cargo(&["test", "--workspace", "--no-run"]),
        "All workspace tests must compile"
    );
}

// ============================================================================
// Feature Flag Tests
// ============================================================================

#[test]
fn test_default_features_compile() {
    assert!(
        run_cargo(&["check", "--workspace", "--features", "default"]),
        "Workspace must compile with default features"
    );
}

#[test]
fn test_elo_only_feature_compiles() {
    assert!(
        run_cargo(&["check", "-p", "ladder-rs", "--features", "elo-only"]),
        "ladder-rs must compile with elo-only feature"
    );
}

#[test]
fn test_all_algorithms_feature_compiles() {
    assert!(
        run_cargo(&["check", "-p", "ladder-rs", "--features", "all-algorithms"]),
        "ladder-rs must compile with all-algorithms feature"
    );
}

// ============================================================================
// Package Metadata Tests
// ============================================================================

#[test]
fn test_workspace_package_has_metadata() {
    let cargo_toml = read_cargo_toml(&format!("{}/Cargo.toml", WORKSPACE_ROOT))
        .expect("Failed to parse workspace Cargo.toml");

    let package = cargo_toml
        .get("package")
        .expect("Root Cargo.toml must have [package] section");

    assert!(package.get("name").is_some(), "Package must have a name");
    assert!(
        package.get("version").is_some(),
        "Package must have a version"
    );
    assert!(
        package.get("edition").is_some(),
        "Package must have an edition"
    );
    assert!(
        package.get("description").is_some(),
        "Package must have a description"
    );
}

#[test]
fn test_persistence_package_has_metadata() {
    let cargo_toml = read_cargo_toml(&format!(
        "{}/ladder-rs-persistence/Cargo.toml",
        WORKSPACE_ROOT
    ))
    .expect("Failed to parse ladder-rs-persistence Cargo.toml");

    let package = cargo_toml
        .get("package")
        .expect("ladder-rs-persistence Cargo.toml must have [package] section");

    assert_eq!(
        package.get("name").and_then(|n| n.as_str()),
        Some("ladder-rs-persistence"),
        "Package name must be ladder-rs-persistence"
    );
}

#[test]
fn test_server_package_has_metadata() {
    let cargo_toml = read_cargo_toml(&format!("{}/ladder-rs-server/Cargo.toml", WORKSPACE_ROOT))
        .expect("Failed to parse ladder-rs-server Cargo.toml");

    let package = cargo_toml
        .get("package")
        .expect("ladder-rs-server Cargo.toml must have [package] section");

    assert_eq!(
        package.get("name").and_then(|n| n.as_str()),
        Some("ladder-rs-server"),
        "Package name must be ladder-rs-server"
    );
}
