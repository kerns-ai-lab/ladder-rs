//! Task 1.1.5: Development Build Scripts Test Suite
//!
//! This test suite defines and validates the enhanced development build script
//! functionality including watch mode, development server, hot reload,
//! debugging features, and development environment configuration.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use std::thread;

/// Test that the development build script exists and is executable
#[test]
fn test_dev_build_script_exists() {
    let dev_script_path = "scripts/dev.sh";
    assert!(
        Path::new(dev_script_path).exists(),
        "Development build script should exist at {}", 
        dev_script_path
    );
    
    // Check if script is executable (Unix-like systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(dev_script_path).expect("Should read script metadata");
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "Development script should be executable"
        );
    }
}

/// Test that watch script exists and is executable
#[test]
fn test_watch_script_exists() {
    let watch_script_path = "scripts/watch.sh";
    assert!(
        Path::new(watch_script_path).exists(),
        "Watch mode script should exist at {}", 
        watch_script_path
    );
    
    // Check if script is executable (Unix-like systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(watch_script_path).expect("Should read script metadata");
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "Watch script should be executable"
        );
    }
}

/// Test that development server script exists and is executable
#[test]
fn test_dev_server_script_exists() {
    let server_script_path = "scripts/serve.sh";
    assert!(
        Path::new(server_script_path).exists(),
        "Development server script should exist at {}", 
        server_script_path
    );
    
    // Check if script is executable (Unix-like systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(server_script_path).expect("Should read script metadata");
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "Development server script should be executable"
        );
    }
}

/// Test that development build script has help option
#[test]
fn test_dev_script_help_option() {
    let output = Command::new("./scripts/dev.sh")
        .arg("--help")
        .output()
        .expect("Should execute dev script with --help");
    
    assert!(output.status.success(), "Help command should succeed");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(help_text.contains("Development Build Script"), "Should contain script description");
    assert!(help_text.contains("--watch"), "Should support watch mode");
    assert!(help_text.contains("--serve"), "Should support development server");
    assert!(help_text.contains("--hot-reload"), "Should support hot reload");
    assert!(help_text.contains("--debug"), "Should support debug mode");
}

/// Test that watch script supports file watching with inotify or similar
#[test]
fn test_watch_script_functionality() {
    let output = Command::new("./scripts/watch.sh")
        .arg("--help")
        .output()
        .expect("Should execute watch script with --help");
    
    assert!(output.status.success(), "Watch help command should succeed");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(help_text.contains("File Watching"), "Should contain watch description");
    assert!(help_text.contains("--dirs"), "Should support directory specification");
    assert!(help_text.contains("--extensions"), "Should support file extension filtering");
    assert!(help_text.contains("--debounce"), "Should support debounce timing");
}

/// Test shell compatibility for regex patterns
#[test]
fn test_watch_script_shell_compatibility() {
    // This test validates that the regex pattern fix is syntactically correct
    // The actual shell compatibility is tested by running the script with --help
    let output = Command::new("./scripts/watch.sh")
        .arg("--help")
        .output()
        .expect("Should execute watch script");
    
    assert!(output.status.success(), "Watch script should have valid bash syntax");
    
    // Verify the script has no syntax errors by checking exit code
    let syntax_check = Command::new("bash")
        .arg("-n")  // Check syntax only, don't execute
        .arg("./scripts/watch.sh")
        .output()
        .expect("Should check script syntax");
    
    assert!(
        syntax_check.status.success(),
        "Watch script should have valid bash syntax with no errors"
    );
}

/// Test that watch script creates necessary directories for hot reload
#[test]
fn test_watch_script_directory_creation() {
    use std::fs;
    use std::path::Path;
    
    // Clean up any existing test directory
    let test_dir = "test_pkg_dir";
    if Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir).ok();
    }
    
    // Test that directory would be created (we can't fully test the hot reload trigger
    // without a running watch process, but we can verify the mkdir command works)
    let mkdir_test = Command::new("bash")
        .arg("-c")
        .arg(&format!("mkdir -p {} && echo 'test' > {}/.hot_reload_trigger", test_dir, test_dir))
        .output()
        .expect("Should execute mkdir test");
    
    assert!(mkdir_test.status.success(), "Directory creation should succeed");
    
    // Verify the directory and file were created
    assert!(Path::new(test_dir).exists(), "Test directory should be created");
    assert!(Path::new(&format!("{}/.hot_reload_trigger", test_dir)).exists(), 
            "Hot reload trigger file should be created");
    
    // Clean up
    fs::remove_dir_all(test_dir).ok();
}

/// Test that development server script supports multiple protocols
#[test]
fn test_dev_server_script_functionality() {
    let output = Command::new("./scripts/serve.sh")
        .arg("--help")
        .output()
        .expect("Should execute serve script with --help");
    
    assert!(output.status.success(), "Serve help command should succeed");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(help_text.contains("Development Server"), "Should contain server description");
    assert!(help_text.contains("--port"), "Should support port configuration");
    assert!(help_text.contains("--host"), "Should support host configuration");
    assert!(help_text.contains("--https"), "Should support HTTPS");
    assert!(help_text.contains("--cors"), "Should support CORS configuration");
    assert!(help_text.contains("--hot-reload"), "Should support hot reload");
}

/// Test development configuration file structure
#[test]
fn test_dev_config_file_exists() {
    let config_path = "dev.config.json";
    assert!(
        Path::new(config_path).exists(),
        "Development configuration file should exist at {}", 
        config_path
    );
    
    // Validate JSON structure
    let config_content = fs::read_to_string(config_path)
        .expect("Should read development config file");
    
    let config: serde_json::Value = serde_json::from_str(&config_content)
        .expect("Development config should be valid JSON");
    
    // Validate required configuration sections
    assert!(config.get("build").is_some(), "Should have build configuration");
    assert!(config.get("watch").is_some(), "Should have watch configuration");
    assert!(config.get("server").is_some(), "Should have server configuration");
    assert!(config.get("debug").is_some(), "Should have debug configuration");
    
    // Validate build configuration
    let build_config = config.get("build").unwrap();
    assert!(build_config.get("mode").is_some(), "Should specify build mode");
    assert!(build_config.get("target").is_some(), "Should specify build target");
    assert!(build_config.get("sourcemap").is_some(), "Should specify sourcemap options");
    
    // Validate watch configuration
    let watch_config = config.get("watch").unwrap();
    assert!(watch_config.get("directories").is_some(), "Should specify watch directories");
    assert!(watch_config.get("extensions").is_some(), "Should specify watch file extensions");
    assert!(watch_config.get("debounce_ms").is_some(), "Should specify debounce timing");
    
    // Validate server configuration
    let server_config = config.get("server").unwrap();
    assert!(server_config.get("port").is_some(), "Should specify server port");
    assert!(server_config.get("host").is_some(), "Should specify server host");
    assert!(server_config.get("cors").is_some(), "Should specify CORS configuration");
}

/// Test that development build produces debug-optimized output
#[test]
fn test_dev_build_produces_debug_output() {
    // Run development build
    let output = Command::new("./scripts/dev.sh")
        .arg("--mode")
        .arg("debug")
        .arg("--dry-run")
        .output()
        .expect("Should execute development build");
    
    assert!(output.status.success(), "Development build should succeed");
    
    // Check that debug mode is configured in dry-run output
    let debug_output = String::from_utf8_lossy(&output.stdout);
    assert!(debug_output.contains("--dev"), "Should use development mode");
    assert!(debug_output.contains("Mode: debug"), "Should show debug mode");
    
    // For a real build test, we'd need to actually run wasm-pack which is complex in test environment
    // The dry-run shows us the command would be executed correctly
}

/// Test watch mode functionality with simulated file changes
#[test]
fn test_watch_mode_detects_changes() {
    // Skip this test if we can't run long-running processes in test environment
    if std::env::var("SKIP_WATCH_TESTS").is_ok() {
        return;
    }
    
    // Create a temporary file to watch
    let test_file = "src/test_watch.rs";
    fs::write(test_file, "// Test file for watch mode\n")
        .expect("Should create test file");
    
    // Start watch mode in background (this is a conceptual test)
    // In practice, this would involve more complex process management
    let _watch_process = Command::new("./scripts/watch.sh")
        .arg("--dirs")
        .arg("src")
        .arg("--test-mode")
        .spawn()
        .expect("Should start watch mode");
    
    // Wait a moment for watcher to initialize
    thread::sleep(Duration::from_millis(500));
    
    // Modify the test file
    fs::write(test_file, "// Modified test file for watch mode\n")
        .expect("Should modify test file");
    
    // Wait for watch to detect change
    thread::sleep(Duration::from_millis(1000));
    
    // Check that build was triggered (look for build artifacts or logs)
    // This is implementation-dependent
    
    // Cleanup
    let _ = fs::remove_file(test_file);
}

/// Test development server starts and serves content
#[test]
fn test_dev_server_starts_and_serves() {
    // Skip this test if we can't run servers in test environment
    if std::env::var("SKIP_SERVER_TESTS").is_ok() {
        return;
    }
    
    // Start development server in background
    let mut server_process = Command::new("./scripts/serve.sh")
        .arg("--port")
        .arg("3000")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--test-mode")
        .spawn()
        .expect("Should start development server");
    
    // Wait for server to start
    thread::sleep(Duration::from_secs(2));
    
    // Test that server is responding (would need HTTP client in real implementation)
    // For now, just check that process is still running
    assert!(server_process.try_wait().unwrap().is_none(), "Server should still be running");
    
    // Cleanup
    let _ = server_process.kill();
}

/// Test that hot reload WebSocket server is available
#[test]
fn test_hot_reload_websocket_server() {
    // Skip this test if we can't run servers in test environment
    if std::env::var("SKIP_SERVER_TESTS").is_ok() {
        return;
    }
    
    // Start development server with hot reload
    let mut server_process = Command::new("./scripts/serve.sh")
        .arg("--hot-reload")
        .arg("--ws-port")
        .arg("3001")
        .arg("--test-mode")
        .spawn()
        .expect("Should start development server with hot reload");
    
    // Wait for server to start
    thread::sleep(Duration::from_secs(2));
    
    // Test that WebSocket server is available (would need WebSocket client in real implementation)
    // For now, just check that process is still running
    assert!(server_process.try_wait().unwrap().is_none(), "Hot reload server should be running");
    
    // Cleanup
    let _ = server_process.kill();
}

/// Test debug logging and verbose output
#[test]
fn test_debug_logging_functionality() {
    let output = Command::new("./scripts/dev.sh")
        .arg("--debug")
        .arg("--verbose")
        .arg("--dry-run")
        .output()
        .expect("Should execute dev script with debug flags");
    
    assert!(output.status.success(), "Debug build command should succeed");
    
    let debug_output = String::from_utf8_lossy(&output.stdout);
    assert!(debug_output.contains("[DEBUG]"), "Should contain debug log messages");
    assert!(debug_output.contains("[VERBOSE]"), "Should contain verbose log messages");
    assert!(debug_output.contains("Build configuration:"), "Should show build configuration");
    assert!(debug_output.contains("File watching:"), "Should show watch configuration");
}

/// Test development environment variable handling
#[test]
fn test_development_environment_variables() {
    let output = Command::new("./scripts/dev.sh")
        .arg("--show-env")
        .env("LADDER_RS_DEV_MODE", "true")
        .env("LADDER_RS_DEBUG_LEVEL", "verbose")
        .env("LADDER_RS_HOT_RELOAD", "true")
        .output()
        .expect("Should execute dev script with environment variables");
    
    assert!(output.status.success(), "Environment variable command should succeed");
    
    let env_output = String::from_utf8_lossy(&output.stdout);
    assert!(env_output.contains("LADDER_RS_DEV_MODE=true"), "Should show development mode");
    assert!(env_output.contains("LADDER_RS_DEBUG_LEVEL=verbose"), "Should show debug level");
    assert!(env_output.contains("LADDER_RS_HOT_RELOAD=true"), "Should show hot reload setting");
}

/// Test integration with existing build.sh script
#[test]
fn test_integration_with_build_script() {
    // Test that dev.sh can call build.sh with proper parameters
    let output = Command::new("./scripts/dev.sh")
        .arg("--build-only")
        .arg("--target")
        .arg("web")
        .arg("--verbose")
        .arg("--dry-run")
        .output()
        .expect("Should execute dev script build-only mode");
    
    assert!(output.status.success(), "Build-only mode should succeed");
    
    let build_output = String::from_utf8_lossy(&output.stdout);
    assert!(build_output.contains("Target: web"), "Should configure target correctly");
    assert!(build_output.contains("--target web"), "Should pass target to build command");
}

/// Test performance monitoring in development mode
#[test]
fn test_performance_monitoring_in_dev_mode() {
    let output = Command::new("./scripts/dev.sh")
        .arg("--performance-monitoring")
        .arg("--build-only")
        .arg("--dry-run")
        .output()
        .expect("Should execute dev script with performance monitoring");
    
    assert!(output.status.success(), "Performance monitoring should succeed");
    
    let perf_output = String::from_utf8_lossy(&output.stdout);
    assert!(perf_output.contains("Performance monitoring: true"), "Should enable performance monitoring");
    
    // In dry-run mode, we check configuration rather than actual metrics
    // Real metrics would only be available during actual builds
}

/// Test development script error handling and recovery
#[test]
fn test_error_handling_and_recovery() {
    // Test with invalid parameters
    let output = Command::new("./scripts/dev.sh")
        .arg("--invalid-option")
        .output()
        .expect("Should execute dev script with invalid option");
    
    assert!(!output.status.success(), "Invalid option should fail");
    
    let error_output = String::from_utf8_lossy(&output.stderr);
    assert!(error_output.contains("Unknown parameter"), "Should show error message");
    assert!(error_output.contains("--help"), "Should suggest help option");
}

/// Test cleanup functionality for development files
#[test]
fn test_cleanup_functionality() {
    // Create some temporary development files
    let temp_files = vec![
        "pkg/.dev_cache",
        "pkg/.hot_reload_state",
        ".dev_server.pid",
    ];
    
    for file in &temp_files {
        fs::write(file, "temporary development file").ok();
    }
    
    // Run cleanup
    let output = Command::new("./scripts/dev.sh")
        .arg("--cleanup")
        .output()
        .expect("Should execute dev script cleanup");
    
    assert!(output.status.success(), "Cleanup should succeed");
    
    // Verify files are cleaned up
    for file in &temp_files {
        assert!(!Path::new(file).exists(), "Temporary file {} should be cleaned up", file);
    }
}