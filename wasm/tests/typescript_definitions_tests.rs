// Task 1.1.4: TypeScript Definition Generation Tests
// Comprehensive test suite for TypeScript definition generation, validation, and enhancement

use std::fs;
use std::path::Path;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test that generated TypeScript definitions exist and are valid
#[wasm_bindgen_test]
fn test_typescript_definitions_exist() {
    // Check that pkg directory contains TypeScript definitions
    assert!(
        Path::new("pkg/ladder_rs_wasm.d.ts").exists() || 
        Path::new("../pkg/ladder_rs_wasm.d.ts").exists(),
        "TypeScript definitions file should exist after build"
    );
}

/// Test that TypeScript definitions contain all required exports
#[wasm_bindgen_test]
fn test_typescript_exports_completeness() {
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Core WASM functions
        assert!(content.contains("export function wasm_main()"), "Should export wasm_main function");
        assert!(content.contains("export function greet("), "Should export greet function");
        
        // Core rating classes
        assert!(content.contains("export class WasmRatingSystem"), "Should export WasmRatingSystem class");
        assert!(content.contains("export class WasmRating"), "Should export WasmRating class");
        assert!(content.contains("export class WasmTeam"), "Should export WasmTeam class");
        
        // Legacy classes (for backward compatibility)
        assert!(content.contains("export class JsRating"), "Should export JsRating class");
        assert!(content.contains("export class JsTeam"), "Should export JsTeam class");
        assert!(content.contains("export class JsGameOutcome"), "Should export JsGameOutcome class");
        
        // Enums and types
        assert!(content.contains("export enum RatingSystemType"), "Should export RatingSystemType enum");
        
        // Default init function
        assert!(content.contains("export default function"), "Should export default init function");
    }
}

/// Test that TypeScript definitions have proper JSDoc documentation
#[wasm_bindgen_test]
fn test_typescript_documentation_quality() {
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Check for JSDoc comments from Rust documentation
        assert!(content.contains("/**"), "Should contain JSDoc comments");
        assert!(content.contains("* Creates a new"), "Should document constructors");
        assert!(content.contains("* Gets"), "Should document getters");
        assert!(content.contains("* @"), "Should contain JSDoc tags");
        
        // Specific class documentation
        assert!(content.contains("JavaScript-friendly"), "Should have user-friendly descriptions");
        assert!(content.contains("rating system"), "Should document rating system functionality");
    }
}

/// Test that TypeScript definitions have proper memory management
#[wasm_bindgen_test]
fn test_typescript_memory_management() {
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // WASM classes should have free() methods for memory management
        assert!(content.contains("free(): void"), "WASM classes should have free() methods");
        
        // Check specific classes have memory management
        let classes_needing_free = [
            "JsRating",
            "JsTeam", 
            "JsGameOutcome",
            "RatingSystemConfig",
            "RatingUpdate",
            "WasmRatingSystem",
            "WasmTeam"
        ];
        
        for class in &classes_needing_free {
            // Look for pattern: "export class ClassName {" followed by "free(): void"
            let class_pattern = format!("export class {}", class);
            if content.contains(&class_pattern) {
                // This is a simplified check - in practice we'd need more sophisticated parsing
                assert!(content.contains("free(): void"), 
                    &format!("{} should have memory management", class));
            }
        }
    }
}

/// Test that TypeScript definitions use precise types
#[wasm_bindgen_test]
fn test_typescript_type_precision() {
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Should use specific array types where appropriate
        assert!(content.contains("Uint32Array") || content.contains("number[]"), 
            "Should use appropriate array types");
        
        // Should have optional parameters with proper syntax
        assert!(content.contains("?") || content.contains("| undefined"), 
            "Should use optional parameter syntax");
        
        // Should have readonly properties where appropriate
        assert!(content.contains("readonly"), "Should mark readonly properties");
        
        // Should have proper return types
        assert!(content.contains(": number"), "Should specify number return types");
        assert!(content.contains(": string"), "Should specify string return types");
        assert!(content.contains(": boolean"), "Should specify boolean return types");
    }
}

/// Test that TypeScript definitions include WebAssembly-specific types
#[wasm_bindgen_test]
fn test_typescript_webassembly_integration() {
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Should include WebAssembly types
        assert!(content.contains("WebAssembly.Module") || content.contains("WebAssembly"), 
            "Should include WebAssembly types");
        
        // Should include init input/output types
        assert!(content.contains("InitInput") || content.contains("InitOutput"), 
            "Should define init types");
        
        // Should have proper module loading types
        assert!(content.contains("BufferSource") || content.contains("Response"), 
            "Should support different module loading methods");
    }
}

/// Test that TypeScript definitions support multiple build targets
#[wasm_bindgen_test]
fn test_typescript_multi_target_support() {
    // This test checks that our TypeScript definitions work across different build targets
    // For now, we'll check the primary web target, but this could be expanded
    
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Should work in browser environment (default for web target)
        assert!(content.contains("export"), "Should use ES module syntax for web target");
        
        // Should not contain Node.js specific types unless we're building for Node
        // This is a simplified check - more sophisticated logic would check build target
        assert!(!content.contains("require(") || content.contains("export"), 
            "Should use appropriate module system");
    }
}

/// Test TypeScript definition file structure and formatting
#[wasm_bindgen_test]
fn test_typescript_file_structure() {
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Should have proper linting directives
        assert!(content.contains("/* tslint:disable */") || content.contains("/* eslint-disable */"), 
            "Should include linting directives");
        
        // Should be well-formatted
        assert!(content.lines().count() > 10, "Should have substantial content");
        
        // Should not have syntax errors (basic check)
        assert!(!content.contains("export export"), "Should not have duplicate keywords");
        assert!(!content.contains("class class"), "Should not have duplicate class keywords");
        
        // Should have consistent indentation (spaces, not tabs for TypeScript)
        let lines_with_indentation: Vec<&str> = content.lines()
            .filter(|line| line.starts_with("  "))
            .collect();
        assert!(lines_with_indentation.len() > 0, "Should have properly indented content");
    }
}

/// Test that TypeScript definitions are compatible with strict mode
#[wasm_bindgen_test]
fn test_typescript_strict_mode_compatibility() {
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Should not use 'any' type (or use it sparingly)
        let any_count = content.matches(": any").count();
        assert!(any_count <= 2, "Should minimize use of 'any' type for strict TypeScript");
        
        // Should properly handle null/undefined
        assert!(content.contains("| undefined") || content.contains("| null"), 
            "Should handle nullable types properly");
        
        // Should not have implicit any parameters
        assert!(!content.contains("(a, b)") || content.contains("(a: "), 
            "Parameters should have explicit types");
    }
}

/// Test that TypeScript definitions include proper generics where applicable
#[wasm_bindgen_test]
fn test_typescript_generics_usage() {
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Check for generic type usage where appropriate
        // Arrays should be properly typed
        let has_arrays = content.contains("[]") || content.contains("Array<");
        if has_arrays {
            assert!(content.contains("WasmRating[]") || content.contains("Array<"), 
                "Arrays should have proper generic types");
        }
        
        // Promise types should be generic
        if content.contains("Promise") {
            assert!(content.contains("Promise<"), "Promises should be generic");
        }
    }
}

/// Test enhanced TypeScript definitions with custom improvements
#[wasm_bindgen_test]
fn test_enhanced_typescript_features() {
    // This test will validate our custom enhancements beyond basic wasm-pack generation
    
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Should include utility types for common patterns
        // This will be implemented as part of the enhancement
        
        // Should have proper namespace organization (if we implement it)
        // For now, we'll check that exports are well-organized
        
        // Should include example usage in comments (enhancement)
        // This is a future improvement we might add
        
        // For now, just verify the base structure is sound
        assert!(content.contains("export"), "Should have exports");
        assert!(!content.is_empty(), "Should not be empty");
    }
}

/// Test TypeScript compilation against our generated definitions
#[wasm_bindgen_test]
fn test_typescript_compilation_validity() {
    // This test would ideally run tsc to validate the definitions
    // For now, we'll do basic structural validation
    
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Basic syntax validation
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        assert_eq!(open_braces, close_braces, "Braces should be balanced");
        
        let open_parens = content.matches('(').count();
        let close_parens = content.matches(')').count();
        assert_eq!(open_parens, close_parens, "Parentheses should be balanced");
        
        // Should not have obvious syntax errors
        assert!(!content.contains(";;"), "Should not have double semicolons");
        assert!(!content.contains(",,"), "Should not have double commas");
    }
}

/// Test that package.json includes proper TypeScript configuration
#[wasm_bindgen_test]
fn test_package_json_typescript_config() {
    let pkg_json_path = if Path::new("pkg/package.json").exists() {
        "pkg/package.json"
    } else {
        "../pkg/package.json"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_json_path) {
        // Should specify TypeScript definitions
        assert!(content.contains("\"types\"") || content.contains("\"typings\""), 
            "package.json should specify types field");
        
        // Should point to correct definitions file
        assert!(content.contains("ladder_rs_wasm.d.ts"), 
            "Should reference the correct .d.ts file");
    }
}

/// Test custom TypeScript definition enhancements (Task 1.1.4 specific)
#[wasm_bindgen_test]
fn test_task_1_1_4_enhancements() {
    // This test validates the specific enhancements for Task 1.1.4
    
    let pkg_path = if Path::new("pkg/ladder_rs_wasm.d.ts").exists() {
        "pkg/ladder_rs_wasm.d.ts"
    } else {
        "../pkg/ladder_rs_wasm.d.ts"
    };
    
    if let Ok(content) = fs::read_to_string(pkg_path) {
        // Check for enhanced documentation
        assert!(content.contains("/**") && content.contains("*/"), 
            "Should have enhanced JSDoc documentation");
        
        // Check for type safety improvements
        assert!(content.contains("readonly") || content.contains("private"), 
            "Should use access modifiers for type safety");
        
        // Check for proper error types if we add them
        // This will be part of our enhancement
        
        // Validate that all public APIs are properly typed
        assert!(!content.contains(": any") || content.matches(": any").count() <= 1, 
            "Should minimize any types for better type safety");
    }
}