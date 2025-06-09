//! Task 1.1.6: Package.json Configuration Test Suite
//!
//! This test suite validates the package.json configuration, npm scripts integration,
//! and compatibility with the development build scripts from Task 1.1.5.

use std::fs;
use std::path::Path;
use std::process::Command;
use serde_json::{Value, Map};

/// Test that package.json exists and is valid JSON
#[test]
fn test_package_json_exists_and_valid() {
    let package_json_path = "package.json";
    assert!(
        Path::new(package_json_path).exists(),
        "package.json should exist in the wasm directory"
    );
    
    let content = fs::read_to_string(package_json_path)
        .expect("Should be able to read package.json");
    
    let json: Value = serde_json::from_str(&content)
        .expect("package.json should be valid JSON");
    
    assert!(json.is_object(), "package.json should be a JSON object");
}

/// Test required metadata fields
#[test]
fn test_package_metadata_fields() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let obj = json.as_object().expect("Should be object");
    
    // Test required fields
    assert!(obj.contains_key("name"), "Should have 'name' field");
    assert_eq!(obj["name"], "ladder-rs-wasm", "Package name should be 'ladder-rs-wasm'");
    
    assert!(obj.contains_key("version"), "Should have 'version' field");
    assert!(obj.contains_key("description"), "Should have 'description' field");
    assert!(obj.contains_key("main"), "Should have 'main' field");
    assert!(obj.contains_key("types"), "Should have 'types' field");
    assert!(obj.contains_key("repository"), "Should have 'repository' field");
    assert!(obj.contains_key("keywords"), "Should have 'keywords' field");
    assert!(obj.contains_key("author"), "Should have 'author' field");
    assert!(obj.contains_key("license"), "Should have 'license' field");
    
    // Test types field points to TypeScript definitions
    assert!(
        obj["types"].as_str().unwrap().ends_with(".d.ts"),
        "Types field should point to .d.ts file"
    );
}

/// Test npm scripts configuration
#[test]
fn test_npm_scripts_configuration() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let scripts = json["scripts"].as_object()
        .expect("Should have 'scripts' object");
    
    // Test essential build scripts
    assert!(scripts.contains_key("build"), "Should have 'build' script");
    assert!(scripts.contains_key("build:dev"), "Should have 'build:dev' script");
    assert!(scripts.contains_key("build:release"), "Should have 'build:release' script");
    assert!(scripts.contains_key("build:all"), "Should have 'build:all' script");
    
    // Test target-specific build scripts
    assert!(scripts.contains_key("build:node"), "Should have 'build:node' script");
    assert!(scripts.contains_key("build:bundler"), "Should have 'build:bundler' script");
    
    // Test development scripts
    assert!(scripts.contains_key("dev"), "Should have 'dev' script");
    assert!(scripts.contains_key("watch"), "Should have 'watch' script");
    
    // Test quality scripts
    assert!(scripts.contains_key("test"), "Should have 'test' script");
    assert!(scripts.contains_key("test:node"), "Should have 'test:node' script");
    assert!(scripts.contains_key("lint"), "Should have 'lint' script");
    assert!(scripts.contains_key("fmt"), "Should have 'fmt' script");
    assert!(scripts.contains_key("check"), "Should have 'check' script");
    
    // Test utility scripts
    assert!(scripts.contains_key("clean"), "Should have 'clean' script");
    assert!(scripts.contains_key("size-report"), "Should have 'size-report' script");
}

/// Test integration with development build scripts from Task 1.1.5
#[test]
fn test_dev_scripts_integration() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let scripts = json["scripts"].as_object()
        .expect("Should have 'scripts' object");
    
    // Test that npm scripts integrate with dev.sh
    if let Some(dev_script) = scripts.get("dev:watch") {
        let script_value = dev_script.as_str().unwrap();
        assert!(
            script_value.contains("./scripts/dev.sh") || 
            script_value.contains("npm run dev"),
            "dev:watch should integrate with development scripts"
        );
    }
    
    // Test that npm scripts integrate with serve.sh
    if let Some(serve_script) = scripts.get("dev:serve") {
        let script_value = serve_script.as_str().unwrap();
        assert!(
            script_value.contains("./scripts/serve.sh") || 
            script_value.contains("serve"),
            "dev:serve should integrate with server scripts"
        );
    }
    
    // Test that npm scripts integrate with watch.sh
    if let Some(watch_script) = scripts.get("dev:watch:files") {
        let script_value = watch_script.as_str().unwrap();
        assert!(
            script_value.contains("./scripts/watch.sh") || 
            script_value.contains("watch"),
            "dev:watch:files should integrate with watch scripts"
        );
    }
}

/// Test files field configuration
#[test]
fn test_files_field_configuration() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let files = json["files"].as_array()
        .expect("Should have 'files' array");
    
    // Test that pkg directories are included
    let files_str: Vec<String> = files.iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    
    assert!(
        files_str.iter().any(|f| f.contains("pkg")),
        "Files should include pkg directory"
    );
}

/// Test exports field for modern module resolution
#[test]
fn test_exports_field_configuration() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    if let Some(exports) = json.get("exports") {
        let exports_obj = exports.as_object()
            .expect("Exports should be an object");
        
        // Test main export
        assert!(exports_obj.contains_key("."), "Should have main '.' export");
        
        // Test conditional exports
        if let Some(main_export) = exports_obj["."].as_object() {
            assert!(
                main_export.contains_key("types") || 
                main_export.contains_key("import") ||
                main_export.contains_key("require"),
                "Main export should have conditional exports"
            );
        }
        
        // Test target-specific exports
        assert!(
            exports_obj.contains_key("./web") ||
            exports_obj.contains_key("./node") ||
            exports_obj.contains_key("./bundler"),
            "Should have target-specific exports"
        );
    }
}

/// Test devDependencies configuration
#[test]
fn test_dev_dependencies() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    if let Some(dev_deps) = json.get("devDependencies") {
        let deps_obj = dev_deps.as_object()
            .expect("devDependencies should be an object");
        
        // Test for development tooling
        assert!(
            deps_obj.contains_key("@types/node") ||
            deps_obj.contains_key("typescript") ||
            deps_obj.contains_key("@wasm-tool/wasm-pack-plugin"),
            "Should have development tooling dependencies"
        );
    }
}

/// Test engines field
#[test]
fn test_engines_configuration() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    if let Some(engines) = json.get("engines") {
        let engines_obj = engines.as_object()
            .expect("engines should be an object");
        
        assert!(
            engines_obj.contains_key("node"),
            "Should specify Node.js version requirement"
        );
        
        if let Some(node_version) = engines_obj["node"].as_str() {
            assert!(
                node_version.contains(">="),
                "Node version should specify minimum version"
            );
        }
    }
}

/// Test publishing configuration
#[test]
fn test_publish_configuration() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    // Test publishConfig
    if let Some(publish_config) = json.get("publishConfig") {
        let config_obj = publish_config.as_object()
            .expect("publishConfig should be an object");
        
        assert!(
            config_obj.contains_key("registry") ||
            config_obj.contains_key("access"),
            "Should have publishing configuration"
        );
    }
    
    // Test prepublishOnly script
    let scripts = json["scripts"].as_object()
        .expect("Should have scripts");
    
    assert!(
        scripts.contains_key("prepublishOnly"),
        "Should have prepublishOnly script for publishing preparation"
    );
}

/// Test that npm scripts use the correct build scripts
#[test]
fn test_build_script_paths() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let scripts = json["scripts"].as_object()
        .expect("Should have scripts");
    
    // Check that build scripts reference the correct build.sh
    if let Some(build_script) = scripts["build"].as_str() {
        assert!(
            build_script.contains("./build.sh") || 
            build_script.contains("wasm-pack"),
            "Build script should use build.sh or wasm-pack"
        );
    }
}

/// Test development workflow scripts
#[test]
fn test_development_workflow_scripts() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let scripts = json["scripts"].as_object()
        .expect("Should have scripts");
    
    // Test for development workflow scripts that integrate with Task 1.1.5
    let dev_workflow_scripts = [
        "dev:all",           // Combined dev mode
        "dev:watch:build",   // Watch and build
        "dev:hot-reload",    // Hot reload server
        "dev:debug",         // Debug mode
    ];
    
    let has_dev_workflow = dev_workflow_scripts.iter()
        .any(|script| scripts.contains_key(*script));
    
    // Either have specific dev workflow scripts or a general dev script
    assert!(
        has_dev_workflow || scripts.contains_key("dev"),
        "Should have development workflow scripts"
    );
}

/// Test npm script execution (dry run)
#[test]
fn test_npm_script_syntax() {
    // This test verifies that npm scripts have valid syntax
    // by attempting to parse the package.json and check script values
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let scripts = json["scripts"].as_object()
        .expect("Should have scripts");
    
    for (name, value) in scripts {
        let script_str = value.as_str()
            .expect(&format!("Script '{}' should be a string", name));
        
        // Basic validation - scripts should not be empty
        assert!(
            !script_str.trim().is_empty(),
            "Script '{}' should not be empty", name
        );
        
        // Check for common issues
        assert!(
            !script_str.contains("undefined"),
            "Script '{}' contains 'undefined'", name
        );
    }
}

/// Test that package.json integrates with existing infrastructure
#[test]
fn test_infrastructure_integration() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    // Check that main entry points match build output
    let main = json["main"].as_str().unwrap();
    assert!(
        main.starts_with("pkg/"),
        "Main entry should point to pkg directory"
    );
    
    let types = json["types"].as_str().unwrap();
    assert!(
        types.starts_with("pkg/"),
        "Types entry should point to pkg directory"
    );
    
    // Check that clean script removes the correct directories
    let clean_script = json["scripts"]["clean"].as_str().unwrap();
    assert!(
        clean_script.contains("pkg"),
        "Clean script should remove pkg directory"
    );
}

/// Test concurrent script execution support
#[test]
fn test_concurrent_script_support() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let scripts = json["scripts"].as_object()
        .expect("Should have scripts");
    
    // Check for parallel build support
    assert!(
        scripts.contains_key("build:all-parallel") ||
        scripts.values().any(|v| v.as_str().unwrap_or("").contains("--parallel")),
        "Should support parallel builds for better performance"
    );
}

/// Test that all referenced scripts and files exist
#[test]
fn test_referenced_files_exist() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let scripts = json["scripts"].as_object()
        .expect("Should have scripts");
    
    // Check that build.sh exists if referenced
    for (_, value) in scripts {
        if let Some(script_str) = value.as_str() {
            if script_str.contains("./build.sh") {
                assert!(
                    Path::new("build.sh").exists(),
                    "build.sh should exist as it's referenced in npm scripts"
                );
            }
            if script_str.contains("./scripts/dev.sh") {
                assert!(
                    Path::new("scripts/dev.sh").exists(),
                    "scripts/dev.sh should exist as it's referenced in npm scripts"
                );
            }
        }
    }
}

/// Test npm lifecycle scripts
#[test]
fn test_lifecycle_scripts() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let scripts = json["scripts"].as_object()
        .expect("Should have scripts");
    
    // Test for useful lifecycle scripts
    let lifecycle_scripts = ["prepublishOnly", "postbuild", "pretest"];
    let has_lifecycle = lifecycle_scripts.iter()
        .any(|script| scripts.contains_key(*script));
    
    assert!(
        has_lifecycle,
        "Should have at least one lifecycle script for automation"
    );
}

/// Test validation script
#[test]
fn test_validation_script() {
    let content = fs::read_to_string("package.json")
        .expect("Should read package.json");
    let json: Value = serde_json::from_str(&content)
        .expect("Should parse JSON");
    
    let scripts = json["scripts"].as_object()
        .expect("Should have scripts");
    
    // Should have a validation or check:all script
    assert!(
        scripts.contains_key("validate") || 
        scripts.contains_key("check:all") ||
        scripts.contains_key("verify"),
        "Should have a comprehensive validation script"
    );
}