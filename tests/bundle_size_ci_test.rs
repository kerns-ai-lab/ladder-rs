use std::fs;
use std::path::Path;

#[test]
fn test_bundle_size_script_exists() {
    let script_path = Path::new("scripts/bundle_size_check.sh");
    assert!(script_path.exists(), "bundle_size_check.sh must exist");
    let content = fs::read_to_string(script_path).expect("read script");
    assert!(content.contains("MAX_BUNDLE_SIZE=204800"), "script must set size limit");
}

#[test]
fn test_ci_check_invokes_bundle_size_check() {
    let ci_path = Path::new("scripts/ci-check.sh");
    let content = fs::read_to_string(ci_path).expect("read ci-check.sh");
    assert!(content.contains("bundle_size_check.sh"), "CI script must call bundle_size_check.sh");
}
