use thiserror::Error;

/// Result type for the ladder-rs library.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the ladder-rs library.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid input data provided.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Mathematical error occurred during calculations.
    #[error("Calculation error: {0}")]
    CalculationError(String),

    /// Numerical precision issues.
    #[error("Numerical precision error: {0}")]
    NumericalError(String),

    /// Numerical issues during computation.
    #[error("Numerical issue: {0}")]
    NumericalIssue(String),

    /// Convergence failure in iterative algorithms.
    #[error("Failed to converge: {0}")]
    ConvergenceFailure(String),

    /// Invalid configuration parameters.
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Error related to game outcomes.
    #[error("Invalid outcome: {0}")]
    InvalidOutcome(String),

    /// Other types of errors.
    #[error("Other error: {0}")]
    Other(String),
}