#[cfg(test)]
mod script_improvements_tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_bundle_size_check_validates_empty_size() {
        let script_path = Path::new("scripts/bundle_size_check.sh");
        assert!(script_path.exists(), "bundle_size_check.sh must exist");

        let content =
            fs::read_to_string(script_path).expect("Should be able to read bundle_size_check.sh");

        // Verify the script checks for empty size variable
        assert!(
            content.contains("if [ -z \"$size\" ]")
                || content.contains("if [[ -z \"$size\" ]]")
                || content.contains("if [ -z $size ]")
                || content.contains("if [[ -z $size ]]"),
            "Script should validate that size variable is not empty"
        );

        // Verify error handling for empty size
        assert!(
            content.contains("Failed to determine file size")
                || content.contains("Failed to get file size")
                || content.contains("Could not determine file size"),
            "Script should have clear error message for size determination failure"
        );
    }

    #[test]
    fn test_install_hooks_creates_directory() {
        let script_path = Path::new("scripts/install-hooks.sh");
        assert!(script_path.exists(), "install-hooks.sh must exist");

        let content =
            fs::read_to_string(script_path).expect("Should be able to read install-hooks.sh");

        // Verify the script creates the hooks directory if it doesn't exist
        assert!(
            content.contains("mkdir -p \"$HOOK_DIR\"")
                || content.contains("mkdir -p $HOOK_DIR")
                || content.contains("mkdir -p \"${HOOK_DIR}\"")
                || content.contains("mkdir -p ${HOOK_DIR}"),
            "Script should ensure hooks directory exists with mkdir -p"
        );
    }

    #[test]
    fn test_scripts_have_proper_error_handling() {
        // Test that scripts use set -e for error handling
        let scripts = vec!["scripts/bundle_size_check.sh", "scripts/install-hooks.sh"];

        for script_path in scripts {
            let path = Path::new(script_path);
            assert!(path.exists(), "{} must exist", script_path);

            let content = fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("Should be able to read {}", script_path));

            assert!(
                content.contains("set -e"),
                "{} should use 'set -e' for proper error handling",
                script_path
            );
        }
    }
}
