/// Tests for Phase 1 project structure and module organization
/// Validates that the library is properly organized and exports are correct

use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating, Outcome},
    error::{Error, Result},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_module_exports() {
        // Test that all core traits are properly exported and accessible
        
        // GameOutcome should be available
        let outcome = GameOutcome::new(vec![1, 2]);
        assert_eq!(outcome.ranks(), &[1, 2]);
        
        // All traits should be in scope and usable
        // This is validated by the mock implementations in core_traits_comprehensive_tests.rs
        // but we test basic trait object creation here
        
        // Test that we can create trait objects (ensuring traits are object-safe where appropriate)
        let outcome_trait: &dyn Outcome = &GameOutcome::new(vec![1]);
        assert!(outcome_trait.is_valid_for_team_count(1));
    }

    #[test]
    fn test_error_module_exports() {
        // Test that error types are properly exported
        let error = Error::InvalidInput("test".to_string());
        assert_eq!(error.to_string(), "Invalid input: test");
        
        // Test Result type alias
        let success: Result<i32> = Ok(42);
        let failure: Result<i32> = Err(Error::Other("fail".to_string()));
        
        assert!(success.is_ok());
        assert!(failure.is_err());
    }

    #[test]
    fn test_module_visibility() {
        // Test that the public API is accessible and private details are hidden
        
        // These should be accessible (public API)
        let _outcome = GameOutcome::new(vec![1, 2, 3]);
        let _win_outcome = GameOutcome::win(0, 3);
        let _draw_outcome = GameOutcome::draw(2);
        let _error = Error::InvalidInput("test".to_string());
        
        // Test that we can use the re-exported types
        let outcome: ladder_rs::core::GameOutcome = GameOutcome::new(vec![1]);
        assert!(outcome.is_valid_for_team_count(1));
        
        let error: ladder_rs::error::Error = Error::Other("test".to_string());
        assert_eq!(error.to_string(), "Other error: test");
    }

    #[test]
    fn test_trait_coherence() {
        // Test that traits work together coherently
        
        // GameOutcome implements Outcome
        let outcome = GameOutcome::new(vec![1, 2]);
        let outcome_ref: &dyn Outcome = &outcome;
        assert!(outcome_ref.is_valid_for_team_count(2));
        
        // Test Clone and Debug are properly implemented
        let cloned = outcome.clone();
        assert_eq!(outcome.ranks(), cloned.ranks());
        
        let _debug_str = format!("{:?}", outcome);
    }

    #[test]
    fn test_library_structure_completeness() {
        // Test that all expected modules are present and accessible
        
        // Core module should provide all foundational traits
        fn _check_rating_trait<T: Rating>(_: T) {}
        fn _check_team_rating_trait<T: TeamRating>(_: T) {}
        fn _check_rating_system_trait<T: RatingSystem>(_: T) {}
        fn _check_outcome_trait<T: Outcome>(_: T) {}
        
        // Error module should provide error handling
        fn _check_error_type(_: Error) {}
        fn _check_result_type<T>(_: Result<T>) {}
        
        // GameOutcome should satisfy Outcome trait
        _check_outcome_trait(GameOutcome::new(vec![1]));
    }

    #[test]
    fn test_no_unused_imports() {
        // This test ensures our imports are actually used
        // If this compiles without warnings, our imports are correct
        
        use ladder_rs::core::*;
        use ladder_rs::error::*;
        
        let outcome = GameOutcome::new(vec![1]);
        let _: &dyn Outcome = &outcome;
        
        let error = Error::InvalidInput("test".to_string());
        let result: Result<i32> = Err(error);
        assert!(result.is_err());
    }

    #[test]
    fn test_consistent_naming_conventions() {
        // Test that naming follows Rust conventions
        
        // Types should be PascalCase
        let _game_outcome = GameOutcome::new(vec![1]);
        let _error = Error::Other("test".to_string());
        
        // Methods should be snake_case
        let outcome = GameOutcome::new(vec![1, 2]);
        assert!(outcome.is_valid_for_team_count(2));
        assert_eq!(outcome.ranks().len(), 2);
        
        let win_outcome = GameOutcome::win(0, 2);
        assert_eq!(win_outcome.ranks()[0], 1);
        
        let draw_outcome = GameOutcome::draw(3);
        assert_eq!(draw_outcome.ranks().len(), 3);
    }

    #[test]
    fn test_documentation_requirements() {
        // Test that public API elements have proper documentation
        // This is enforced by the compiler when missing_docs is enabled
        // Here we just test that the API is usable as documented
        
        // GameOutcome::new should create from ranks vector
        let outcome = GameOutcome::new(vec![1, 2, 3]);
        assert_eq!(outcome.ranks(), &[1, 2, 3]);
        
        // GameOutcome::win should create win scenario
        let win = GameOutcome::win(1, 3);
        assert_eq!(win.ranks(), &[2, 1, 2]);
        
        // GameOutcome::draw should create draw scenario
        let draw = GameOutcome::draw(2);
        assert_eq!(draw.ranks(), &[1, 1]);
        
        // Error messages should be descriptive
        let error = Error::InvalidInput("parameter x must be positive".to_string());
        assert!(error.to_string().contains("Invalid input"));
        assert!(error.to_string().contains("parameter x must be positive"));
    }

    #[test]
    fn test_forward_compatibility() {
        // Test that the current API design allows for future extensions
        
        // GameOutcome should be extensible for complex scenarios
        let complex_ranks = vec![1, 1, 3, 3, 3, 6, 7, 7];
        let outcome = GameOutcome::new(complex_ranks.clone());
        assert_eq!(outcome.ranks(), &complex_ranks);
        assert!(outcome.is_valid_for_team_count(8));
        
        // Error types should be comprehensive enough for future use cases
        let errors = vec![
            Error::InvalidInput("future parameter validation".to_string()),
            Error::CalculationError("future algorithm issue".to_string()),
            Error::NumericalError("future precision problem".to_string()),
            Error::ConvergenceFailure("future convergence issue".to_string()),
            Error::InvalidConfiguration("future config problem".to_string()),
            Error::InvalidOutcome("future outcome validation".to_string()),
            Error::Other("future unforeseen error".to_string()),
        ];
        
        for error in errors {
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn test_ergonomic_api_design() {
        // Test that the API is ergonomic and easy to use
        
        // Creating outcomes should be straightforward
        let simple_win = GameOutcome::win(0, 2);
        let simple_draw = GameOutcome::draw(2);
        let custom_outcome = GameOutcome::new(vec![1, 2, 2, 4]);
        
        assert_ne!(simple_win.ranks(), simple_draw.ranks());
        assert_eq!(custom_outcome.ranks().len(), 4);
        
        // Error handling should be idiomatic
        let result: Result<f64> = Ok(3.14);
        let processed = result.map(|x| x * 2.0).unwrap();
        assert_eq!(processed, 6.28);
        
        let error_result: Result<f64> = Err(Error::InvalidInput("bad input".to_string()));
        assert!(error_result.is_err());
    }

    #[test]
    fn test_trait_bounds_and_generics() {
        // Test that trait bounds work correctly for generic programming
        
        fn process_outcome<T: Outcome>(outcome: &T, team_count: usize) -> bool {
            outcome.is_valid_for_team_count(team_count)
        }
        
        let outcome = GameOutcome::new(vec![1, 2, 3]);
        assert!(process_outcome(&outcome, 3));
        assert!(!process_outcome(&outcome, 2));
        
        // Test that Clone and Debug work in generic contexts
        fn clone_and_debug<T: Clone + std::fmt::Debug>(item: T) -> String {
            let cloned = item.clone();
            format!("{:?}", cloned)
        }
        
        let outcome = GameOutcome::new(vec![1, 2]);
        let debug_string = clone_and_debug(outcome);
        assert!(debug_string.contains("GameOutcome"));
    }
}