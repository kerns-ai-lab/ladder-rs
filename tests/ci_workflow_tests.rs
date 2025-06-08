use std::fs;
use std::path::Path;

#[test]
fn test_ci_workflow_runs_ci_check() {
    let workflow = fs::read_to_string(".github/workflows/ci.yml").expect("read ci workflow");
    assert!(workflow.contains("./scripts/ci-check.sh"), "CI workflow must run ci-check.sh");
}

#[test]
fn test_ci_workflow_installs_wasm_pack() {
    let workflow = fs::read_to_string(".github/workflows/ci.yml").expect("read ci workflow");
    assert!(workflow.contains("wasm-pack"), "CI workflow must install wasm-pack");
}

#[test]
fn test_recommendation_workflow_runs_ci_check() {
    let workflow = fs::read_to_string(".github/workflows/ci-recommendation.yml").expect("read ci recommendation");
    assert!(workflow.contains("./scripts/ci-check.sh"), "Recommendation workflow must run ci-check.sh");
}

#[test]
fn test_pre_push_hook_exists() {
    assert!(Path::new("githooks/pre-push").exists(), "pre-push hook must exist");
}
