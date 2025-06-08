/// Comprehensive tests for Phase 1 error handling
/// Tests the error types and Result handling that form the error management foundation
use ladder_rs::error::{Error, Result};
use std::error::Error as StdError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_input_error() {
        let error = Error::InvalidInput("test message".to_string());

        // Test error message
        assert_eq!(error.to_string(), "Invalid input: test message");

        // Test debug formatting
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidInput"));
        assert!(debug_str.contains("test message"));
    }

    #[test]
    fn test_calculation_error() {
        let error = Error::CalculationError("division by zero".to_string());

        assert_eq!(error.to_string(), "Calculation error: division by zero");

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("CalculationError"));
        assert!(debug_str.contains("division by zero"));
    }

    #[test]
    fn test_numerical_error() {
        let error = Error::NumericalError("precision loss detected".to_string());

        assert_eq!(
            error.to_string(),
            "Numerical precision error: precision loss detected"
        );

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NumericalError"));
        assert!(debug_str.contains("precision loss detected"));
    }

    #[test]
    fn test_convergence_failure_error() {
        let error = Error::ConvergenceFailure(
            "algorithm failed to converge after 1000 iterations".to_string(),
        );

        assert_eq!(
            error.to_string(),
            "Failed to converge: algorithm failed to converge after 1000 iterations"
        );

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ConvergenceFailure"));
        assert!(debug_str.contains("algorithm failed to converge"));
    }

    #[test]
    fn test_invalid_configuration_error() {
        let error = Error::InvalidConfiguration("beta parameter must be positive".to_string());

        assert_eq!(
            error.to_string(),
            "Invalid configuration: beta parameter must be positive"
        );

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidConfiguration"));
        assert!(debug_str.contains("beta parameter must be positive"));
    }

    #[test]
    fn test_invalid_outcome_error() {
        let error =
            Error::InvalidOutcome("ranks vector length does not match team count".to_string());

        assert_eq!(
            error.to_string(),
            "Invalid outcome: ranks vector length does not match team count"
        );

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidOutcome"));
        assert!(debug_str.contains("ranks vector length"));
    }

    #[test]
    fn test_other_error() {
        let error = Error::Other("unexpected error occurred".to_string());

        assert_eq!(error.to_string(), "Other error: unexpected error occurred");

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Other"));
        assert!(debug_str.contains("unexpected error occurred"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = Error::InvalidInput("test".to_string());

        // Test that Error implements std::error::Error
        let _: &dyn StdError = &error;

        // Test source method (should be None for our custom errors)
        assert!(error.source().is_none());
    }

    #[test]
    fn test_result_type_usage() {
        // Test successful result
        let success: Result<i32> = Ok(42);
        assert!(success.is_ok());
        assert_eq!(success.as_ref().unwrap(), &42);

        // Test error result
        let failure: Result<i32> = Err(Error::InvalidInput("bad input".to_string()));
        assert!(failure.is_err());

        match failure {
            Ok(_) => panic!("Expected error"),
            Err(Error::InvalidInput(msg)) => assert_eq!(msg, "bad input"),
            Err(_) => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_result_chaining() {
        fn operation_that_fails() -> Result<i32> {
            Err(Error::CalculationError("test error".to_string()))
        }

        fn operation_that_succeeds() -> Result<i32> {
            Ok(100)
        }

        // Test chaining with map
        let result = operation_that_succeeds().map(|x| x * 2);
        assert_eq!(result.unwrap(), 200);

        // Test chaining with map (improved from and_then)
        let result = operation_that_succeeds().map(|x| x + 50);
        assert_eq!(result.unwrap(), 150);

        // Test error propagation
        let result = operation_that_fails().map(|x| x * 2);
        assert!(result.is_err());

        if let Err(Error::CalculationError(msg)) = result {
            assert_eq!(msg, "test error");
        } else {
            panic!("Expected CalculationError");
        }
    }

    #[test]
    fn test_error_comparison() {
        let error1 = Error::InvalidInput("same message".to_string());
        let error2 = Error::InvalidInput("same message".to_string());
        let error3 = Error::InvalidInput("different message".to_string());
        let error4 = Error::CalculationError("same message".to_string());

        // Test equality (Note: thiserror doesn't automatically derive PartialEq,
        // so we test by string representation)
        assert_eq!(error1.to_string(), error2.to_string());
        assert_ne!(error1.to_string(), error3.to_string());
        assert_ne!(error1.to_string(), error4.to_string());
    }

    #[test]
    fn test_error_messages_formatting() {
        // Test that error messages are properly formatted and informative
        let errors = vec![
            Error::InvalidInput("parameter x must be between 0 and 1".to_string()),
            Error::CalculationError("overflow in multiplication".to_string()),
            Error::NumericalError("insufficient precision for accurate result".to_string()),
            Error::ConvergenceFailure("iteration limit reached without convergence".to_string()),
            Error::InvalidConfiguration("gamma parameter cannot be negative".to_string()),
            Error::InvalidOutcome("draw probability must be between 0 and 1".to_string()),
            Error::Other("unexpected I/O error".to_string()),
        ];

        for error in errors {
            let msg = error.to_string();
            // Ensure messages are not empty and contain meaningful information
            assert!(!msg.is_empty());
            assert!(msg.len() > 10); // Reasonable minimum length for meaningful messages

            // Ensure each error type has its proper prefix
            match error {
                Error::InvalidInput(_) => assert!(msg.starts_with("Invalid input:")),
                Error::CalculationError(_) => assert!(msg.starts_with("Calculation error:")),
                Error::NumericalError(_) => assert!(msg.starts_with("Numerical precision error:")),
                Error::ConvergenceFailure(_) => assert!(msg.starts_with("Failed to converge:")),
                Error::InvalidConfiguration(_) => {
                    assert!(msg.starts_with("Invalid configuration:"))
                }
                Error::InvalidOutcome(_) => assert!(msg.starts_with("Invalid outcome:")),
                Error::Other(_) => assert!(msg.starts_with("Other error:")),
            }
        }
    }

    #[test]
    fn test_empty_error_messages() {
        // Test behavior with empty error messages
        let errors = vec![
            Error::InvalidInput(String::new()),
            Error::CalculationError(String::new()),
            Error::NumericalError(String::new()),
            Error::ConvergenceFailure(String::new()),
            Error::InvalidConfiguration(String::new()),
            Error::InvalidOutcome(String::new()),
            Error::Other(String::new()),
        ];

        for error in errors {
            let msg = error.to_string();
            // Even with empty messages, should still have the error type prefix
            assert!(msg.contains(":"));
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn test_long_error_messages() {
        // Test with very long error messages
        let long_message = "a".repeat(1000);
        let error = Error::InvalidInput(long_message.clone());
        let formatted = error.to_string();

        assert!(formatted.contains(&long_message));
        assert!(formatted.starts_with("Invalid input:"));
    }

    #[test]
    fn test_special_characters_in_error_messages() {
        // Test error messages with special characters
        let special_chars = "Testing with special chars: αβγ, 中文, 🦀, \n\t\"'\\";
        let error = Error::Other(special_chars.to_string());
        let formatted = error.to_string();

        assert!(formatted.contains(special_chars));
    }

    #[test]
    fn test_functional_error_handling_patterns() {
        // Test common functional programming patterns with Result

        fn divide(a: f64, b: f64) -> Result<f64> {
            if b == 0.0 {
                Err(Error::CalculationError("division by zero".to_string()))
            } else {
                Ok(a / b)
            }
        }

        fn square_root(x: f64) -> Result<f64> {
            if x < 0.0 {
                Err(Error::CalculationError(
                    "square root of negative number".to_string(),
                ))
            } else {
                Ok(x.sqrt())
            }
        }

        // Test successful computation chain
        let result = divide(16.0, 4.0).and_then(square_root).map(|x| x * 2.0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4.0);

        // Test error propagation in chain
        let result = divide(16.0, 0.0).and_then(square_root).map(|x| x * 2.0);

        assert!(result.is_err());
        if let Err(Error::CalculationError(msg)) = result {
            assert_eq!(msg, "division by zero");
        } else {
            panic!("Expected CalculationError with division by zero message");
        }

        // Test error in second operation
        let result = divide(16.0, -4.0).and_then(square_root).map(|x| x * 2.0);

        assert!(result.is_err());
        if let Err(Error::CalculationError(msg)) = result {
            assert_eq!(msg, "square root of negative number");
        } else {
            panic!("Expected CalculationError with square root message");
        }
    }
}
