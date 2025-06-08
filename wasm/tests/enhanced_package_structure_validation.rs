//! Enhanced Package Structure Validation Tests for Task 1.1.1 Completion
//! 
//! This test suite validates all improvements made to the WASM package structure
//! including the enhanced build system, size optimizations, and multi-target support.

use std::path::Path;
use std::fs;

#[cfg(test)]
mod enhanced_structure_tests {
    use super::*;

    #[test]
    fn test_enhanced_cargo_toml_configuration() {
        let cargo_path = Path::new("Cargo.toml");
        assert!(cargo_path.exists(), "Cargo.toml must exist");
        
        let content = fs::read_to_string(cargo_path)
            .expect("Should be able to read Cargo.toml");
        
        // Test enhanced optimization profiles
        assert!(content.contains("[profile.wasm-release]"), "Must have custom WASM release profile");
        assert!(content.contains("[profile.wasm-size]"), "Must have size-optimized profile");
        assert!(content.contains("lto = \"fat\""), "Must use fat LTO for maximum optimization");
        assert!(content.contains("overflow-checks = false"), "Must disable overflow checks for size");
        
        // Test feature flags for conditional compilation
        assert!(content.contains("[features]"), "Must have features section");
        assert!(content.contains("minimal = [\"wee_alloc\"]"), "Must have minimal feature flag");
        assert!(content.contains("elo-only"), "Must have algorithm-specific feature flags");
        
        // Test enhanced wasm-pack metadata
        assert!(content.contains("wasm-opt = [\"-Oz\""), "Must have advanced wasm-opt configuration");
        assert!(content.contains("enable-mutable-globals"), "Must enable mutable globals optimization");
        
        // Test size optimization dependencies
        assert!(content.contains("default-features = false"), "Must minimize dependencies with default-features = false");
    }

    #[test]
    fn test_wasm_pack_json_configuration() {
        let wasm_pack_path = Path::new(".wasm-pack.json");
        assert!(wasm_pack_path.exists(), ".wasm-pack.json must exist for consistent builds");
        
        let content = fs::read_to_string(wasm_pack_path)
            .expect("Should be able to read .wasm-pack.json");
        
        assert!(content.contains("\"out-dir\": \"pkg\""), "Must specify output directory");
        assert!(content.contains("\"target\": \"web\""), "Must specify default target");
        assert!(content.contains("\"mode\": \"normal\""), "Must specify build mode");
        assert!(content.contains("\"out-name\": \"ladder_rs_wasm\""), "Must specify output file name");
    }

    #[test]
    fn test_cargo_wasm_pack_metadata() {
        let cargo_content = fs::read_to_string("Cargo.toml")
            .expect("Should be able to read Cargo.toml");

        assert!(cargo_content.contains("[package.metadata.wasm-pack]"), "Cargo.toml must include wasm-pack metadata section");
        assert!(cargo_content.contains("out-dir = \"pkg\""), "Must set out-dir in metadata");
        assert!(cargo_content.contains("target = \"web\""), "Must set default target in metadata");
        assert!(cargo_content.contains("out-name = \"ladder_rs_wasm\""), "Must set out-name in metadata");
        assert!(cargo_content.contains("scope = \"@ladder-rs\""), "Must set npm scope in metadata");
    }

    #[test]
    fn test_enhanced_package_json_scripts() {
        let package_path = Path::new("package.json");
        let content = fs::read_to_string(package_path)
            .expect("Should be able to read package.json");
        
        // Test enhanced build scripts
        assert!(content.contains("\"build:all\""), "Must have build script for all targets");
        assert!(content.contains("\"build:all-parallel\""), "Must have parallel build script");
        assert!(content.contains("\"build:size-check\""), "Must have size checking script");
        assert!(content.contains("\"size-report\""), "Must have size reporting script");
        
        // Test validation scripts
        assert!(content.contains("\"test:structure\""), "Must have structure testing script");
        assert!(content.contains("\"test:all\""), "Must have comprehensive testing script");
        assert!(content.contains("\"validate\""), "Must have validation script");
        
        // Test development workflow scripts
        assert!(content.contains("\"check:all\""), "Must have comprehensive checking script");
        assert!(content.contains("\"fmt:check\""), "Must have format checking script");
        
        // Test multi-target exports
        assert!(content.contains("\"exports\""), "Must have exports configuration for multiple targets");
        assert!(content.contains("\"./web\""), "Must export web target");
        assert!(content.contains("\"./node\""), "Must export node target");
        assert!(content.contains("\"./bundler\""), "Must export bundler target");
    }

    #[test]
    fn test_enhanced_build_script() {
        let build_script_path = Path::new("build.sh");
        assert!(build_script_path.exists(), "Enhanced build.sh must exist");
        
        let content = fs::read_to_string(build_script_path)
            .expect("Should be able to read build.sh");
        
        // Test enhanced features
        assert!(content.contains("check_bundle_size"), "Must have bundle size checking function");
        assert!(content.contains("MAX_BUNDLE_SIZE=204800"), "Must have 200KB size target");
        assert!(content.contains("--all-targets"), "Must support building all targets");
        assert!(content.contains("--parallel"), "Must support parallel builds");
        assert!(content.contains("wasm-opt -Oz"), "Must use wasm-opt for size optimization");
        
        // Test build validation
        assert!(content.contains("run_post_build_tests"), "Must run validation tests after build");
        assert!(content.contains("log_success"), "Must have proper logging functions");
        
        // Test size monitoring
        assert!(content.contains("WARN_BUNDLE_SIZE"), "Must have size warning threshold");
        assert!(content.contains("Bundle size exceeds"), "Must check size limits");
    }

    #[test]
    fn test_bundle_size_optimization_features() {
        let cargo_content = fs::read_to_string("Cargo.toml")
            .expect("Should be able to read Cargo.toml");
        
        // Test aggressive size optimization features
        assert!(cargo_content.contains("profile.wasm-size"), "Must have ultra-aggressive size profile");
        assert!(cargo_content.contains("debug-assertions = false"), "Must disable debug assertions for size");
        assert!(cargo_content.contains("codegen-units = 1"), "Must use single codegen unit");
        
        // Test dependency optimization
        assert!(cargo_content.contains("default-features = false"), "Must minimize dependency features");
        
        // Test feature-based conditional compilation
        assert!(cargo_content.contains("elo-only = []"), "Must support algorithm-specific builds");
        assert!(cargo_content.contains("glicko-only = []"), "Must support algorithm-specific builds");
        assert!(cargo_content.contains("trueskill-only = []"), "Must support algorithm-specific builds");
    }

    #[test]
    fn test_multi_target_support() {
        let package_content = fs::read_to_string("package.json")
            .expect("Should be able to read package.json");
        
        // Test file exports for different targets
        assert!(package_content.contains("pkg/**/*"), "Must include web target files");
        assert!(package_content.contains("pkg-node/**/*"), "Must include node target files");
        assert!(package_content.contains("pkg-bundler/**/*"), "Must include bundler target files");
        
        // Test export maps
        assert!(package_content.contains("\"exports\""), "Must have export maps");
        assert!(package_content.contains("\"./web\""), "Must export web-specific entry");
        assert!(package_content.contains("\"./node\""), "Must export node-specific entry");
        assert!(package_content.contains("\"./bundler\""), "Must export bundler-specific entry");
    }

    #[test]
    fn test_development_workflow_enhancements() {
        let package_content = fs::read_to_string("package.json")
            .expect("Should be able to read package.json");
        
        // Test comprehensive workflow scripts
        assert!(package_content.contains("\"prepublishOnly\""), "Must have pre-publish validation");
        assert!(package_content.contains("\"postbuild\""), "Must have post-build reporting");
        
        // Test quality gates
        assert!(package_content.contains("npm run check:all"), "Must run all checks before publish");
        assert!(package_content.contains("npm run test:all"), "Must run all tests before publish");
        
        // Test development dependencies
        assert!(package_content.contains("cargo-watch"), "Must include cargo-watch for development");
    }

    #[test]
    fn test_typescript_integration_enhancements() {
        let package_content = fs::read_to_string("package.json")
            .expect("Should be able to read package.json");
        
        // Test TypeScript definitions for all targets
        assert!(package_content.contains("pkg/ladder_rs_wasm.d.ts"), "Must have web target TypeScript definitions");
        assert!(package_content.contains("pkg-node/ladder_rs_wasm.d.ts"), "Must have node target TypeScript definitions");
        assert!(package_content.contains("pkg-bundler/ladder_rs_wasm.d.ts"), "Must have bundler target TypeScript definitions");
        
        // Test main entry points
        assert!(package_content.contains("\"main\": \"pkg/ladder_rs_wasm.js\""), "Must have correct main entry point");
        assert!(package_content.contains("\"types\": \"pkg/ladder_rs_wasm.d.ts\""), "Must have correct types entry point");
    }

    #[test]
    fn test_publishing_configuration() {
        let package_content = fs::read_to_string("package.json")
            .expect("Should be able to read package.json");
        
        // Test publishing configuration
        assert!(package_content.contains("\"publishConfig\""), "Must have publish configuration");
        assert!(package_content.contains("\"access\": \"public\""), "Must be configured for public publishing");
        
        // Test repository and metadata
        assert!(package_content.contains("\"bugs\""), "Must have bug reporting URL");
        assert!(package_content.contains("\"homepage\""), "Must have homepage URL");
        assert!(package_content.contains("\"ladder\""), "Must include ladder keyword");
        assert!(package_content.contains("\"ranking\""), "Must include ranking keyword");
    }

    #[test]
    fn test_analysis_documentation() {
        let analysis_path = Path::new("PACKAGE_STRUCTURE_ANALYSIS.md");
        assert!(analysis_path.exists(), "Package structure analysis documentation must exist");
        
        let content = fs::read_to_string(analysis_path)
            .expect("Should be able to read analysis documentation");
        
        assert!(content.contains("Task 1.1.1"), "Must document Task 1.1.1 completion");
        assert!(content.contains("Bundle Size Optimization"), "Must document bundle size considerations");
        assert!(content.contains("200KB target"), "Must document size targets");
        assert!(content.contains("Multi-Target Build System"), "Must document multi-target support");
    }
}

#[cfg(test)]
mod build_system_validation_tests {
    use super::*;

    #[test]
    fn test_build_script_syntax() {
        let build_script_path = Path::new("build.sh");
        let content = fs::read_to_string(build_script_path)
            .expect("Should be able to read build.sh");
        
        // Test bash syntax validity (basic checks)
        assert!(content.starts_with("#!/bin/bash"), "Must have proper bash shebang");
        assert!(content.contains("set -e"), "Must exit on error");
        
        // Count function definitions and calls for basic syntax validation
        let function_defs = content.matches("() {").count();
        assert!(function_defs >= 3, "Must have at least 3 function definitions");
        
        // Test color codes are properly defined
        assert!(content.contains("RED="), "Must define color codes");
        assert!(content.contains("GREEN="), "Must define color codes");
        assert!(content.contains("BLUE="), "Must define color codes");
        assert!(content.contains("NC="), "Must define color reset");
    }

    #[test]
    fn test_wasm_pack_configuration_validity() {
        let wasm_pack_path = Path::new(".wasm-pack.json");
        let content = fs::read_to_string(wasm_pack_path)
            .expect("Should be able to read .wasm-pack.json");
        
        // Basic JSON validation
        let open_braces = content.chars().filter(|&c| c == '{').count();
        let close_braces = content.chars().filter(|&c| c == '}').count();
        assert_eq!(open_braces, close_braces, ".wasm-pack.json must have balanced braces");
        
        // Required fields validation
        assert!(content.contains("out-dir"), "Must specify output directory");
        assert!(content.contains("target"), "Must specify target");
    }

    #[test]
    fn test_cargo_profiles_syntax() {
        let cargo_content = fs::read_to_string("Cargo.toml")
            .expect("Should be able to read Cargo.toml");
        
        // Test profile syntax
        assert!(cargo_content.contains("[profile."), "Must have profile sections");
        assert!(cargo_content.contains("opt-level"), "Must specify optimization level");
        assert!(cargo_content.contains("lto"), "Must specify LTO setting");
        
        // Test TOML syntax validity (basic)
        let profile_count = cargo_content.matches("[profile.").count();
        assert!(profile_count >= 3, "Must have at least 3 profiles defined");
    }
}

#[cfg(test)]
mod integration_readiness_tests {
    use super::*;

    #[test]
    fn test_task_1_1_1_completion_criteria() {
        // Test all Task 1.1.1 completion criteria are met
        
        // 1. Package structure is properly organized
        assert!(Path::new("Cargo.toml").exists(), "Cargo.toml must exist");
        assert!(Path::new("package.json").exists(), "package.json must exist");
        assert!(Path::new(".wasm-pack.json").exists(), ".wasm-pack.json must exist");
        assert!(Path::new("build.sh").exists(), "Enhanced build.sh must exist");
        
        // 2. Build system is enhanced
        let build_content = fs::read_to_string("build.sh").expect("Should read build.sh");
        assert!(build_content.contains("check_bundle_size"), "Build system must include size checking");
        assert!(build_content.contains("--all-targets"), "Build system must support all targets");
        
        // 3. Size optimization is configured
        let cargo_content = fs::read_to_string("Cargo.toml").expect("Should read Cargo.toml");
        assert!(cargo_content.contains("profile.wasm-size"), "Must have size optimization profile");
        assert!(cargo_content.contains("lto = \"fat\""), "Must use fat LTO");
        
        // 4. Multi-target support is implemented
        let package_content = fs::read_to_string("package.json").expect("Should read package.json");
        assert!(package_content.contains("\"exports\""), "Must have multi-target exports");
        assert!(package_content.contains("build:all"), "Must have all-targets build script");
        
        // 5. Development workflow is enhanced
        assert!(package_content.contains("validate"), "Must have validation script");
        assert!(package_content.contains("size-report"), "Must have size reporting");
        
        // 6. Documentation is complete
        assert!(Path::new("PACKAGE_STRUCTURE_ANALYSIS.md").exists(), "Analysis documentation must exist");
    }

    #[test] 
    fn test_ready_for_task_1_1_2() {
        // Verify this package structure is ready for Task 1.1.2 (wasm-pack configuration)
        
        let cargo_content = fs::read_to_string("Cargo.toml").expect("Should read Cargo.toml");
        
        // Must have proper wasm-pack metadata sections
        assert!(cargo_content.contains("[package.metadata.wasm-pack"), "Must have wasm-pack metadata");
        
        // Must have multiple profiles ready for Task 1.1.2 enhancement
        assert!(cargo_content.contains("profile.dev"), "Must have dev profile for Task 1.1.2");
        assert!(cargo_content.contains("profile.release"), "Must have release profile for Task 1.1.2");
        assert!(cargo_content.contains("profile.profiling"), "Must have profiling profile for Task 1.1.2");
        
        // Build system must be ready for wasm-pack configuration enhancement
        let build_content = fs::read_to_string("build.sh").expect("Should read build.sh");
        assert!(build_content.contains("wasm-pack build"), "Build system must use wasm-pack");
        assert!(build_content.contains("--target"), "Build system must support target configuration");
    }
}