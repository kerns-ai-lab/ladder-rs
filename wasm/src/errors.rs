//! Error handling framework for WASM bindings
//!
//! This module provides comprehensive error handling that converts Rust errors
//! to JavaScript-friendly errors while maintaining small bundle sizes.

use std::fmt;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use js_sys;

/// Error codes for different error types
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Invalid input parameters
    InvalidInput = 1000,
    /// Mathematical calculation error
    CalculationError = 2000,
    /// Numerical precision issues
    NumericalError = 2001,
    /// Algorithm failed to converge
    ConvergenceFailure = 2002,
    /// Invalid configuration
    InvalidConfiguration = 3000,
    /// Invalid game outcome
    InvalidOutcome = 4000,
    /// Serialization/deserialization error
    SerializationError = 5000,
    /// Unknown or unexpected error
    UnknownError = 9999,
}

/// WASM-friendly error type that can be passed to JavaScript
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmError {
    code: ErrorCode,
    message: String,
    #[wasm_bindgen(skip)]
    pub details: Option<String>,
    #[wasm_bindgen(skip)]
    pub context: Option<ErrorContext>,
}

/// Additional error context for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// The operation that failed
    pub operation: String,
    /// Additional context-specific data
    pub data: Option<serde_json::Value>,
}

#[wasm_bindgen]
impl WasmError {
    /// Create a new WasmError
    #[wasm_bindgen(constructor)]
    pub fn new(code: ErrorCode, message: String) -> WasmError {
        WasmError {
            code,
            message,
            details: None,
            context: None,
        }
    }

    /// Get the error code
    #[wasm_bindgen(getter)]
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    /// Get the error message
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Get the full error details as a JSON string
    #[wasm_bindgen(getter)]
    pub fn details_json(&self) -> Option<String> {
        self.details.clone()
    }

    /// Convert to a JavaScript Error object
    pub fn to_js_error(&self) -> JsValue {
        let error = js_sys::Error::new(&self.message);
        
        // Add custom properties
        js_sys::Reflect::set(
            &error,
            &JsValue::from_str("code"),
            &JsValue::from_f64(self.code as i32 as f64),
        ).ok();

        if let Some(details) = &self.details {
            js_sys::Reflect::set(
                &error,
                &JsValue::from_str("details"),
                &JsValue::from_str(details),
            ).ok();
        }

        error.into()
    }
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code as i32, self.message)
    }
}

impl std::error::Error for WasmError {}

/// Builder for creating detailed errors
pub struct WasmErrorBuilder {
    code: ErrorCode,
    message: String,
    details: Option<String>,
    context: Option<ErrorContext>,
}

impl WasmErrorBuilder {
    /// Create a new error builder
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        WasmErrorBuilder {
            code,
            message: message.into(),
            details: None,
            context: None,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add context to the error
    pub fn with_context(mut self, operation: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        self.context = Some(ErrorContext {
            operation: operation.into(),
            data,
        });
        self
    }

    /// Build the WasmError
    pub fn build(self) -> WasmError {
        let mut error = WasmError {
            code: self.code,
            message: self.message,
            details: self.details,
            context: self.context,
        };

        // Generate full details JSON if context is present
        if let Some(context) = &error.context {
            let details = serde_json::json!({
                "code": error.code as i32,
                "message": &error.message,
                "context": context,
            });
            error.details = Some(details.to_string());
        }

        error
    }
}

/// Trait for converting from ladder_rs errors to WASM errors
pub trait ToWasmError {
    fn to_wasm_error(&self) -> WasmError;
}

impl ToWasmError for ladder_rs::error::Error {
    fn to_wasm_error(&self) -> WasmError {
        use ladder_rs::error::Error;
        
        match self {
            Error::InvalidInput(msg) => WasmErrorBuilder::new(
                ErrorCode::InvalidInput,
                format!("Invalid input: {}", msg)
            ).build(),
            
            Error::CalculationError(msg) => WasmErrorBuilder::new(
                ErrorCode::CalculationError,
                format!("Calculation error: {}", msg)
            ).build(),
            
            Error::NumericalError(msg) => WasmErrorBuilder::new(
                ErrorCode::NumericalError,
                format!("Numerical precision error: {}", msg)
            ).build(),
            
            Error::ConvergenceFailure(msg) => WasmErrorBuilder::new(
                ErrorCode::ConvergenceFailure,
                format!("Algorithm failed to converge: {}", msg)
            ).build(),
            
            Error::InvalidConfiguration(msg) => WasmErrorBuilder::new(
                ErrorCode::InvalidConfiguration,
                format!("Invalid configuration: {}", msg)
            ).build(),
            
            Error::InvalidOutcome(msg) => WasmErrorBuilder::new(
                ErrorCode::InvalidOutcome,
                format!("Invalid game outcome: {}", msg)
            ).build(),
            
            Error::Other(msg) => WasmErrorBuilder::new(
                ErrorCode::UnknownError,
                format!("Unexpected error: {}", msg)
            ).build(),
        }
    }
}

/// Convert a Result<T, ladder_rs::error::Error> to Result<T, JsValue>
pub fn to_js_result<T>(result: Result<T, ladder_rs::error::Error>) -> Result<T, JsValue> {
    result.map_err(|e| e.to_wasm_error().to_js_error())
}