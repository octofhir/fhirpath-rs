//! Simplified FHIRPath Operation Traits
//!
//! This module provides clean, simple traits for FHIRPath operations that replace
//! the over-engineered previous system. Operations are split into sync and async
//! based on their actual needs, not artificial complexity.
//!
//! # Design Philosophy
//!
//! - **Sync operations** (80%): String manipulation, math, collections, type conversion
//! - **Async operations** (20%): ModelProvider access, system calls, lambda evaluation
//! - **Minimal metadata**: Only function signature, no performance metrics or LSP features
//! - **Idiomatic Rust**: Clean, readable code that junior to senior developers can understand

use crate::signature::FunctionSignature;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Evaluation context for operations
///
/// Contains the current input value and access to the ModelProvider and registry.
/// Simplified from the complex previous EvaluationContext.
pub struct EvaluationContext {
    /// The current input value being operated on
    pub input: FhirPathValue,
    /// Root input value (for resolve() function and context variables) - shared for memory efficiency
    pub root: std::sync::Arc<FhirPathValue>,
    /// Reference to the model provider for type information
    pub model_provider: std::sync::Arc<dyn octofhir_fhirpath_model::ModelProvider>,
    /// Environment variables for evaluation
    pub variables: rustc_hash::FxHashMap<String, FhirPathValue>,
}

impl EvaluationContext {
    /// Create a new evaluation context
    pub fn new(
        input: FhirPathValue,
        root: std::sync::Arc<FhirPathValue>,
        model_provider: std::sync::Arc<dyn octofhir_fhirpath_model::ModelProvider>,
    ) -> Self {
        Self {
            input,
            root,
            model_provider,
            variables: rustc_hash::FxHashMap::default(),
        }
    }

    /// Create a new context with the given input (preserving model provider, root, and variables)
    pub fn with_input(&self, input: FhirPathValue) -> Self {
        Self {
            input,
            root: self.root.clone(),
            model_provider: self.model_provider.clone(),
            variables: self.variables.clone(),
        }
    }

    /// Add a variable to the context
    pub fn with_variable(mut self, name: String, value: FhirPathValue) -> Self {
        self.variables.insert(name, value);
        self
    }
}

/// Trait for synchronous FHIRPath operations
///
/// Use this trait for operations that don't require I/O, network calls, or async evaluation:
/// - String manipulation (length, contains, upper, etc.)
/// - Mathematical operations (abs, round, sqrt, etc.)  
/// - Collection operations (count, first, distinct, etc.)
/// - Type conversion (toString, toInteger, etc.)
/// - Data extraction operations
///
/// # Example
/// ```rust
/// use octofhir_fhirpath_registry::traits::{SyncOperation, EvaluationContext};
/// use octofhir_fhirpath_registry::signature::{FunctionSignature, ValueType};
/// use octofhir_fhirpath_model::FhirPathValue;
/// use octofhir_fhirpath_core::{Result, FhirPathError};
///
/// pub struct LengthFunction;
///
/// impl SyncOperation for LengthFunction {
///     fn name(&self) -> &'static str {
///         "length"
///     }
///     
///     fn signature(&self) -> &FunctionSignature {
///         static SIGNATURE: FunctionSignature = FunctionSignature {
///             name: "length",
///             parameters: vec![],
///             return_type: ValueType::Integer,
///             variadic: false,
///         };
///         &SIGNATURE
///     }
///     
///     fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
///         // Direct synchronous implementation
///         let input_string = context.input.as_string()
///             .ok_or_else(|| FhirPathError::type_error("length() can only be called on strings"))?;
///         Ok(FhirPathValue::Integer(input_string.len() as i64))
///     }
/// }
/// ```
pub trait SyncOperation: Send + Sync {
    /// Operation name (e.g., "length", "count", "upper")
    fn name(&self) -> &'static str;

    /// Function signature with parameter and return type information
    ///
    /// This replaces the complex metadata system with simple, essential information.
    fn signature(&self) -> &FunctionSignature;

    /// Execute the operation synchronously
    ///
    /// # Arguments
    /// * `args` - Function arguments (empty for operations that work on context input)
    /// * `context` - Evaluation context containing input value and environment
    ///
    /// # Returns
    /// The result of the operation as a FhirPathValue
    ///
    /// # Errors
    /// Returns FhirPathError for invalid arguments, type mismatches, or evaluation errors
    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext)
    -> Result<FhirPathValue>;

    /// Validate arguments before execution (optional override)
    ///
    /// Default implementation checks argument count against signature.
    /// Override for custom validation logic.
    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        let signature = self.signature();
        let expected_count = signature.parameters.len();

        if signature.variadic {
            if args.len() < expected_count {
                return Err(FhirPathError::InvalidArgumentCount {
                    function_name: signature.name.to_string(),
                    expected: expected_count,
                    actual: args.len(),
                });
            }
        } else if args.len() != expected_count {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: signature.name.to_string(),
                expected: expected_count,
                actual: args.len(),
            });
        }

        Ok(())
    }
}

/// Trait for asynchronous FHIRPath operations  
///
/// Use this trait for operations that require I/O, network calls, or async evaluation:
/// - FHIR operations (resolve, children, descendants, conforms_to)
/// - Type operations (as, is, of_type, type - need ModelProvider)
/// - System operations (now, today - system calls)
/// - Lambda operations (where, select, all, any - expression evaluation)
///
/// # Example
/// ```rust
/// use octofhir_fhirpath_registry::traits::{AsyncOperation, EvaluationContext};
/// use octofhir_fhirpath_registry::signature::{FunctionSignature, ValueType};
/// use octofhir_fhirpath_model::FhirPathValue;
/// use octofhir_fhirpath_core::{Result, FhirPathError};
/// use async_trait::async_trait;
///
/// fn extract_reference(value: &FhirPathValue) -> Result<String> {
///     // Mock implementation
///     Ok("Patient/123".to_string())
/// }
///
/// pub struct ResolveFunction;
///
/// #[async_trait]
/// impl AsyncOperation for ResolveFunction {
///     fn name(&self) -> &'static str {
///         "resolve"
///     }
///     
///     fn signature(&self) -> &FunctionSignature {
///         static SIGNATURE: FunctionSignature = FunctionSignature {
///             name: "resolve",
///             parameters: vec![],
///             return_type: ValueType::Any,
///             variadic: false,
///         };
///         &SIGNATURE
///     }
///     
///     async fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
///         // Async implementation with ModelProvider access
///         let reference = extract_reference(&context.input)?;
///         // Note: resolve_reference requires ResolutionContext parameter
///         Ok(FhirPathValue::String(format!("Resolved: {}", reference).into()))
///     }
/// }
/// ```
#[async_trait]
pub trait AsyncOperation: Send + Sync {
    /// Operation name (e.g., "resolve", "now", "is")
    fn name(&self) -> &'static str;

    /// Function signature with parameter and return type information
    ///
    /// This replaces the complex metadata system with simple, essential information.
    fn signature(&self) -> &FunctionSignature;

    /// Execute the operation asynchronously
    ///
    /// # Arguments
    /// * `args` - Function arguments (empty for operations that work on context input)
    /// * `context` - Evaluation context containing input value and ModelProvider access
    ///
    /// # Returns
    /// The result of the operation as a FhirPathValue
    ///
    /// # Errors
    /// Returns FhirPathError for invalid arguments, type mismatches, or evaluation errors
    async fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue>;

    /// Validate arguments before execution (optional override)
    ///
    /// Default implementation checks argument count against signature.
    /// Override for custom validation logic.
    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        let signature = self.signature();
        let expected_count = signature.parameters.len();

        if signature.variadic {
            if args.len() < expected_count {
                return Err(FhirPathError::InvalidArgumentCount {
                    function_name: signature.name.to_string(),
                    expected: expected_count,
                    actual: args.len(),
                });
            }
        } else if args.len() != expected_count {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: signature.name.to_string(),
                expected: expected_count,
                actual: args.len(),
            });
        }

        Ok(())
    }
}

/// Helper trait for operations that need special argument validation
///
/// Some operations have complex argument requirements that can't be expressed
/// in the simple FunctionSignature. This trait provides custom validation.
pub trait CustomValidation {
    /// Custom argument validation logic
    fn validate_custom(&self, args: &[FhirPathValue]) -> Result<()>;
}

/// Convenience functions for common validation patterns
pub mod validation {
    use super::*;

    /// Validate that no arguments were provided
    pub fn validate_no_args(args: &[FhirPathValue], function_name: &str) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: function_name.to_string(),
                expected: 0,
                actual: args.len(),
            });
        }
        Ok(())
    }

    /// Validate that exactly one string argument was provided
    pub fn validate_single_string_arg(
        args: &[FhirPathValue],
        function_name: &str,
    ) -> Result<String> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: function_name.to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        args[0]
            .as_string()
            .map(|s| s.to_string())
            .ok_or_else(|| FhirPathError::TypeError {
                message: format!("{function_name}() argument must be a string"),
            })
    }

    /// Validate that the context input is a string
    pub fn validate_string_input(
        context: &EvaluationContext,
        function_name: &str,
    ) -> Result<String> {
        context
            .input
            .as_string()
            .map(|s| s.to_string())
            .ok_or_else(|| FhirPathError::TypeError {
                message: format!("{function_name}() can only be called on string values"),
            })
    }

    /// Validate that the context input is a numeric value (Integer or Decimal)
    pub fn validate_numeric_input(context: &EvaluationContext, function_name: &str) -> Result<()> {
        match &context.input {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => Ok(()),
            _ => Err(FhirPathError::TypeError {
                message: format!("{function_name}() can only be called on numeric values"),
            }),
        }
    }

    /// Validate that the context input is a collection or single value (for collection operations)
    pub fn get_collection_items(input: &FhirPathValue) -> Vec<&FhirPathValue> {
        match input {
            FhirPathValue::Collection(items) => items.iter().collect(),
            FhirPathValue::Empty => vec![],
            single => vec![single],
        }
    }

    /// Validate argument count
    pub fn validate_arg_count(actual: usize, expected: usize, function_name: &str) -> Result<()> {
        if actual != expected {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: function_name.to_string(),
                expected,
                actual,
            });
        }
        Ok(())
    }

    /// Extract string argument at specific index
    pub fn extract_string_arg(
        args: &[FhirPathValue],
        index: usize,
        function_name: &str,
        param_name: &str,
    ) -> Result<String> {
        if index >= args.len() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: function_name.to_string(),
                expected: index + 1,
                actual: args.len(),
            });
        }

        args[index]
            .as_string()
            .map(|s| s.to_string())
            .ok_or_else(|| FhirPathError::TypeError {
                message: format!("{function_name}() {param_name} parameter must be a string"),
            })
    }

    /// Extract integer argument at specific index
    pub fn extract_integer_arg(
        args: &[FhirPathValue],
        index: usize,
        function_name: &str,
        param_name: &str,
    ) -> Result<i64> {
        if index >= args.len() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: function_name.to_string(),
                expected: index + 1,
                actual: args.len(),
            });
        }

        args[index]
            .as_integer()
            .ok_or_else(|| FhirPathError::TypeError {
                message: format!("{function_name}() {param_name} parameter must be an integer"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::{ParameterType, ValueType};

    // Test sync operation implementation
    struct TestSyncOperation;

    impl SyncOperation for TestSyncOperation {
        fn name(&self) -> &'static str {
            "testSync"
        }

        fn signature(&self) -> &FunctionSignature {
            use std::sync::LazyLock;
            static SIGNATURE: LazyLock<FunctionSignature> = LazyLock::new(|| FunctionSignature {
                name: "testSync",
                parameters: vec![ParameterType::String],
                return_type: ValueType::String,
                variadic: false,
            });
            &SIGNATURE
        }

        fn execute(
            &self,
            args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> Result<FhirPathValue> {
            if let Some(arg) = args.first() {
                if let Some(s) = arg.as_string() {
                    return Ok(FhirPathValue::String(format!("processed: {s}").into()));
                }
            }
            Err(FhirPathError::TypeError {
                message: "Expected string argument".to_string(),
            })
        }
    }

    // Test async operation implementation
    struct TestAsyncOperation;

    #[async_trait]
    impl AsyncOperation for TestAsyncOperation {
        fn name(&self) -> &'static str {
            "testAsync"
        }

        fn signature(&self) -> &FunctionSignature {
            static SIGNATURE: FunctionSignature = FunctionSignature {
                name: "testAsync",
                parameters: vec![],
                return_type: ValueType::String,
                variadic: false,
            };
            &SIGNATURE
        }

        async fn execute(
            &self,
            _args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> Result<FhirPathValue> {
            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            Ok(FhirPathValue::String("async result".into()))
        }
    }

    #[test]
    fn test_sync_operation() {
        use octofhir_fhirpath_model::MockModelProvider;

        let op = TestSyncOperation;
        assert_eq!(op.name(), "testSync");

        let args = vec![FhirPathValue::String("test".into())];
        let model_provider = std::sync::Arc::new(MockModelProvider::new());
        let context = EvaluationContext::new(
            FhirPathValue::Empty,
            std::sync::Arc::new(FhirPathValue::Empty),
            model_provider,
        );

        let result = op.execute(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::String("processed: test".into()));
    }

    #[tokio::test]
    async fn test_async_operation() {
        use octofhir_fhirpath_model::MockModelProvider;

        let op = TestAsyncOperation;
        assert_eq!(op.name(), "testAsync");

        let args = vec![];
        let model_provider = std::sync::Arc::new(MockModelProvider::new());
        let context = EvaluationContext::new(
            FhirPathValue::Empty,
            std::sync::Arc::new(FhirPathValue::Empty),
            model_provider,
        );

        let result = op.execute(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("async result".into()));
    }

    #[test]
    fn test_argument_validation() {
        let op = TestSyncOperation;

        // Valid arguments
        let valid_args = vec![FhirPathValue::String("test".into())];
        assert!(op.validate_args(&valid_args).is_ok());

        // Invalid argument count
        let invalid_args = vec![];
        assert!(op.validate_args(&invalid_args).is_err());

        let too_many_args = vec![
            FhirPathValue::String("test1".into()),
            FhirPathValue::String("test2".into()),
        ];
        assert!(op.validate_args(&too_many_args).is_err());
    }

    #[test]
    fn test_validation_helpers() {
        // Test no args validation
        assert!(validation::validate_no_args(&[], "test").is_ok());
        assert!(
            validation::validate_no_args(&[FhirPathValue::String("arg".into())], "test").is_err()
        );

        // Test single string arg validation
        let string_arg = vec![FhirPathValue::String("test".into())];
        assert_eq!(
            validation::validate_single_string_arg(&string_arg, "test").unwrap(),
            "test"
        );

        let non_string_arg = vec![FhirPathValue::Integer(42)];
        assert!(validation::validate_single_string_arg(&non_string_arg, "test").is_err());
    }
}
