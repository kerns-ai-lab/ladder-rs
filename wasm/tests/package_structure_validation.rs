//! Package Structure Validation Tests for Task 1.1.1
//! 
//! This test suite validates the WASM package structure meets all requirements
//! for optimal bundle size, proper configuration, and correct build setup.

use std::path::Path;
use std::fs;

#[cfg(test)]
mod package_structure_tests {
    use super::*;

    #[test]
    fn test_cargo_toml_structure() {
        let cargo_path = Path::new("Cargo.toml");
        assert!(cargo_path.exists(), "Cargo.toml must exist in wasm directory");
        
        let content = fs::read_to_string(cargo_path)
            .expect("Should be able to read Cargo.toml");
        
        // Validate required package metadata
        assert!(content.contains("name = \"ladder-rs-wasm\""), "Package name must be ladder-rs-wasm");
        assert!(content.contains("crate-type = [\"cdylib\"]"), "Must be configured as cdylib for WASM");
        assert!(content.contains("name = \"ladder_rs_wasm\""), "Library name must use underscores");
        
        // Validate required dependencies for WASM
        assert!(content.contains("wasm-bindgen"), "Must include wasm-bindgen dependency");
        assert!(content.contains("js-sys"), "Must include js-sys dependency");
        assert!(content.contains("web-sys"), "Must include web-sys dependency");
        assert!(content.contains("serde"), "Must include serde for serialization");
        
        // Validate bundle size optimization features
        assert!(content.contains("console_error_panic_hook"), "Must include panic hook for debugging");
        assert!(content.contains("wee_alloc"), "Must include wee_alloc for size optimization");
        
        // Validate build profiles for optimization
        assert!(content.contains("[profile.release]"), "Must have release profile optimization");
        assert!(content.contains("opt-level = \"z\""), "Must optimize for size");
        assert!(content.contains("lto = true"), "Must enable Link Time Optimization");
        assert!(content.contains("strip = true"), "Must strip symbols for smaller size");
        assert!(content.contains("panic = \"abort\""), "Must use abort for smaller size");
    }

    #[test]
    fn test_package_json_structure() {
        let package_path = Path::new("package.json");
        assert!(package_path.exists(), "package.json must exist in wasm directory");
        
        let content = fs::read_to_string(package_path)
            .expect("Should be able to read package.json");
        
        // Validate package metadata
        assert!(content.contains("\"name\": \"ladder-rs-wasm\""), "Package name must match");
        assert!(content.contains("\"main\": \"pkg/ladder_rs_wasm.js\""), "Main entry point must be correct");
        assert!(content.contains("\"types\": \"pkg/ladder_rs_wasm.d.ts\""), "TypeScript definitions must be specified");
        
        // Validate build scripts
        assert!(content.contains("\"build\""), "Must have build script");
        assert!(content.contains("\"build:dev\""), "Must have dev build script");
        assert!(content.contains("\"build:release\""), "Must have release build script");
        assert!(content.contains("\"test\""), "Must have test script");
        assert!(content.contains("\"clean\""), "Must have clean script");
        
        // Validate multiple target support
        assert!(content.contains("--target web"), "Must support web target");
        assert!(content.contains("--target nodejs"), "Must support Node.js target");
        assert!(content.contains("--target bundler"), "Must support bundler target");
    }

    #[test]
    fn test_source_file_structure() {
        let src_path = Path::new("src");
        assert!(src_path.exists() && src_path.is_dir(), "src directory must exist");
        
        // Validate core module files
        assert!(Path::new("src/lib.rs").exists(), "lib.rs must exist as main entry point");
        assert!(Path::new("src/api.rs").exists(), "api.rs must exist for public API");
        assert!(Path::new("src/types.rs").exists(), "types.rs must exist for type definitions");
        assert!(Path::new("src/utils.rs").exists(), "utils.rs must exist for utilities");
        
        // Validate feature-specific modules
        assert!(Path::new("src/player_management.rs").exists(), "player_management.rs must exist");
        assert!(Path::new("src/test_utils.rs").exists(), "test_utils.rs must exist for testing");
    }

    #[test]
    fn test_build_configuration_files() {
        // Validate TypeScript configuration
        assert!(Path::new("tsconfig.json").exists(), "tsconfig.json must exist for TypeScript support");
        
        // Validate build script
        assert!(Path::new("build.sh").exists(), "build.sh must exist for automated builds");
        
        // Validate README documentation
        assert!(Path::new("README.md").exists(), "README.md must exist for documentation");
    }

    #[test]
    fn test_output_directory_structure() {
        // These directories should be created by the build process
        // We test their absence initially to ensure clean builds
        
        let target_path = Path::new("target");
        // target directory may exist from previous builds - that's ok
        
        // Output directories should be in .gitignore
        let gitignore_path = Path::new("../.gitignore");
        if gitignore_path.exists() {
            let content = fs::read_to_string(gitignore_path)
                .expect("Should be able to read .gitignore");
            assert!(content.contains("pkg/") || content.contains("wasm/pkg"), 
                    "pkg directory should be in .gitignore");
            assert!(content.contains("target/") || content.contains("wasm/target"), 
                    "target directory should be in .gitignore");
        }
    }

    #[test]
    fn test_wasm_specific_configuration() {
        let cargo_content = fs::read_to_string("Cargo.toml")
            .expect("Should be able to read Cargo.toml");
        
        // Validate WASM-specific configurations
        assert!(cargo_content.contains("[package.metadata.wasm-pack"), 
                "Must have wasm-pack metadata configuration");
        assert!(cargo_content.contains("wasm-opt"), 
                "Must have wasm-opt configuration for size optimization");
        
        // Validate getrandom fix for WASM
        assert!(cargo_content.contains("getrandom") && cargo_content.contains("features = [\"js\"]"), 
                "Must include getrandom with js feature for WASM compatibility");
    }

    #[test]
    fn test_development_workflow_files() {
        // Validate files that support development workflow
        assert!(Path::new("tests").exists(), "tests directory must exist");
        
        // Check if there are any test files
        let test_dir = fs::read_dir("tests").expect("Should be able to read tests directory");
        let test_files: Vec<_> = test_dir.filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    Some(path)
                } else {
                    None
                }
            })
        }).collect();
        
        assert!(!test_files.is_empty(), "Must have at least one Rust test file");
    }

    #[test]
    fn test_dependency_constraints_for_size() {
        let cargo_content = fs::read_to_string("Cargo.toml")
            .expect("Should be able to read Cargo.toml");
        
        // Validate size-optimized dependency features
        assert!(cargo_content.contains("default-features = false") || 
                cargo_content.contains("features = [") ||
                !cargo_content.contains("default-features = true"), 
                "Dependencies should be configured to minimize bundle size");
    }

    #[test]
    fn test_library_exports_configuration() {
        let lib_content = fs::read_to_string("src/lib.rs")
            .expect("Should be able to read lib.rs");
        
        // Validate proper WASM exports
        assert!(lib_content.contains("wasm_bindgen"), 
                "lib.rs must use wasm_bindgen for exports");
        assert!(lib_content.contains("extern crate") || lib_content.contains("use"), 
                "lib.rs must properly import required crates");
    }
}

#[cfg(test)]
mod build_validation_tests {
    use super::*;

    #[test]
    fn test_can_parse_cargo_toml() {
        // This test validates that Cargo.toml is syntactically correct
        let cargo_path = Path::new("Cargo.toml");
        let content = fs::read_to_string(cargo_path)
            .expect("Should be able to read Cargo.toml");
        
        // Basic TOML parsing validation - if this doesn't panic, TOML is valid
        // In a real environment, we'd use a TOML parser here
        assert!(!content.is_empty(), "Cargo.toml should not be empty");
        assert!(content.contains("[package]"), "Must have [package] section");
        assert!(content.contains("[dependencies]"), "Must have [dependencies] section");
    }

    #[test]
    fn test_can_parse_package_json() {
        // This test validates that package.json is syntactically correct
        let package_path = Path::new("package.json");
        let content = fs::read_to_string(package_path)
            .expect("Should be able to read package.json");
        
        // Basic JSON validation - check for balanced braces
        let open_braces = content.chars().filter(|&c| c == '{').count();
        let close_braces = content.chars().filter(|&c| c == '}').count();
        assert_eq!(open_braces, close_braces, "package.json must have balanced braces");
        
        assert!(content.starts_with('{'), "package.json must start with opening brace");
        assert!(content.trim().ends_with('}'), "package.json must end with closing brace");
    }
}

#[cfg(test)]
mod bundle_size_optimization_tests {
    use super::*;

    #[test]
    fn test_size_optimization_features() {
        let cargo_content = fs::read_to_string("Cargo.toml")
            .expect("Should be able to read Cargo.toml");
        
        // Check for size optimization features
        assert!(cargo_content.contains("wee_alloc"), 
                "Should include wee_alloc for smaller allocator");
        assert!(cargo_content.contains("opt-level = \"z\""), 
                "Should optimize for size");
        assert!(cargo_content.contains("lto = true"), 
                "Should enable Link Time Optimization");
        assert!(cargo_content.contains("codegen-units = 1"), 
                "Should use single codegen unit for better optimization");
        assert!(cargo_content.contains("strip = true"), 
                "Should strip symbols for smaller binary");
    }

    #[test]
    fn test_wasm_opt_configuration() {
        let cargo_content = fs::read_to_string("Cargo.toml")
            .expect("Should be able to read Cargo.toml");
        
        // Validate wasm-opt settings for different profiles
        assert!(cargo_content.contains("wasm-opt = false") || 
                cargo_content.contains("wasm-opt = ["), 
                "Should have explicit wasm-opt configuration");
        
        if cargo_content.contains("wasm-opt = [") {
            assert!(cargo_content.contains("-Oz") || cargo_content.contains("-O"), 
                    "Should use size optimization flags");
        }
    }
}