//! Test helpers for FHIRPath evaluation.
//!
//! This module provides convenience functions and utilities for testing
//! FHIRPath expressions. It is only available when running tests.
//!
//! # Example
//!
//! ```rust,ignore
//! use octofhir_fhirpath::testing::*;
//!
//! #[tokio::test]
//! async fn test_addition() {
//!     let engine = test_engine().await;
//!     let context = test_context();
//!
//!     let result = engine.evaluate("1 + 2", &context).await
//!         .expect_ok("addition should succeed");
//!
//!     assert_eq!(result.value.first_integer(), Some(3));
//! }
//! ```

use std::fmt::Debug;
use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;

use crate::core::value::utils::json_to_fhirpath_value;
use crate::core::{Collection, FhirPathValue};
use crate::evaluator::{EvaluationContext, FhirPathEngine, FunctionRegistry};

/// Creates a FhirPathEngine with EmptyModelProvider for testing.
///
/// This is equivalent to `create_engine_with_empty_provider()` but with
/// a more explicit name for test contexts.
pub async fn test_engine() -> FhirPathEngine {
    let registry = Arc::new(crate::create_function_registry());
    let model_provider = Arc::new(EmptyModelProvider);
    FhirPathEngine::new(registry, model_provider)
        .await
        .expect("Failed to create test engine")
}

/// Creates an empty EvaluationContext for testing.
///
/// Returns a context with an empty collection and EmptyModelProvider.
pub fn test_context() -> EvaluationContext {
    let model_provider = Arc::new(EmptyModelProvider);
    EvaluationContext::new(Collection::empty(), model_provider, None, None, None)
}

/// Creates an EvaluationContext with a JSON resource as input.
///
/// # Arguments
/// * `json` - A JSON string representing a FHIR resource
///
/// # Panics
/// Panics if the JSON cannot be parsed.
pub fn test_context_with_json(json: &str) -> EvaluationContext {
    let value: serde_json::Value =
        serde_json::from_str(json).expect("Failed to parse JSON for test context");
    let fhir_value = json_to_fhirpath_value(value);
    let model_provider = Arc::new(EmptyModelProvider);
    EvaluationContext::new(
        Collection::single(fhir_value),
        model_provider,
        None,
        None,
        None,
    )
}

/// Extension trait for Result that provides better error messages in tests.
pub trait TestResultExt<T> {
    /// Unwraps the result, providing a descriptive error message on failure.
    ///
    /// # Arguments
    /// * `context` - A description of what operation was being performed
    ///
    /// # Panics
    /// Panics with a descriptive message if the result is an error.
    fn expect_ok(self, context: &str) -> T;
}

impl<T, E: Debug> TestResultExt<T> for std::result::Result<T, E> {
    fn expect_ok(self, context: &str) -> T {
        self.unwrap_or_else(|e| panic!("{}: {:?}", context, e))
    }
}

/// Extension trait for Option that provides better error messages in tests.
pub trait TestOptionExt<T> {
    /// Unwraps the option, providing a descriptive error message if None.
    ///
    /// # Arguments
    /// * `context` - A description of what was expected
    ///
    /// # Panics
    /// Panics with a descriptive message if the option is None.
    fn expect_some(self, context: &str) -> T;
}

impl<T> TestOptionExt<T> for Option<T> {
    fn expect_some(self, context: &str) -> T {
        self.unwrap_or_else(|| panic!("{}: expected Some, got None", context))
    }
}

/// Extension trait for Collection that provides convenient test assertions.
pub trait CollectionTestExt {
    /// Returns the first value as an integer, or None.
    fn first_integer(&self) -> Option<i64>;

    /// Returns the first value as a string, or None.
    fn first_string(&self) -> Option<&str>;

    /// Returns the first value as a boolean, or None.
    fn first_boolean(&self) -> Option<bool>;

    /// Asserts the collection has exactly one value.
    fn assert_single(&self) -> &FhirPathValue;

    /// Asserts the collection is empty.
    fn assert_empty(&self);

    /// Asserts the collection has the expected number of values.
    fn assert_count(&self, expected: usize);
}

impl CollectionTestExt for Collection {
    fn first_integer(&self) -> Option<i64> {
        self.first().and_then(|v| v.as_integer())
    }

    fn first_string(&self) -> Option<&str> {
        self.first().and_then(|v| v.as_string())
    }

    fn first_boolean(&self) -> Option<bool> {
        self.first().and_then(|v| v.as_boolean())
    }

    fn assert_single(&self) -> &FhirPathValue {
        assert_eq!(
            self.len(),
            1,
            "Expected single value, got {} values",
            self.len()
        );
        self.first().unwrap()
    }

    fn assert_empty(&self) {
        assert!(
            self.is_empty(),
            "Expected empty collection, got {} values",
            self.len()
        );
    }

    fn assert_count(&self, expected: usize) {
        assert_eq!(
            self.len(),
            expected,
            "Expected {} values, got {}",
            expected,
            self.len()
        );
    }
}

/// Creates a FunctionRegistry for testing.
///
/// This is a convenience wrapper around `create_function_registry()`.
pub fn test_registry() -> FunctionRegistry {
    crate::create_function_registry()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expect_ok_success() {
        let result: std::result::Result<i32, &str> = Ok(42);
        assert_eq!(result.expect_ok("test operation"), 42);
    }

    #[test]
    #[should_panic(expected = "test operation")]
    fn test_expect_ok_failure() {
        let result: std::result::Result<i32, &str> = Err("error");
        result.expect_ok("test operation");
    }

    #[test]
    fn test_expect_some_success() {
        let option: Option<i32> = Some(42);
        assert_eq!(option.expect_some("test value"), 42);
    }

    #[test]
    #[should_panic(expected = "test value")]
    fn test_expect_some_failure() {
        let option: Option<i32> = None;
        option.expect_some("test value");
    }
}
