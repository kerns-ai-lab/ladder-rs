use std::fs;
use std::path::Path;

#[test]
fn test_bundle_size_script_exists() {
    let script_path = Path::new("scripts/bundle_size_check.sh");
    assert!(script_path.exists(), "bundle_size_check.sh must exist");
    let content = fs::read_to_string(script_path).expect("read script");
    // Governance model (ADR-0001 amendment): soft target warns, hard cap fails.
    // Both thresholds must be declared for the script to be considered configured.
    assert!(
        content.contains("SOFT_TARGET="),
        "script must declare SOFT_TARGET (advisory warning threshold)"
    );
    assert!(
        content.contains("HARD_CAP="),
        "script must declare HARD_CAP (CI-failing panic threshold)"
    );
}

#[test]
fn test_ci_check_invokes_bundle_size_check() {
    let ci_path = Path::new("scripts/ci-check.sh");
    let content = fs::read_to_string(ci_path).expect("read ci-check.sh");
    assert!(
        content.contains("bundle_size_check.sh"),
        "CI script must call bundle_size_check.sh"
    );
}
