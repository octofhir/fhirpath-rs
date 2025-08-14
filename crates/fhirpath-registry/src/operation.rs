// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Unified operation trait for all FHIRPath callable operations
//!
//! This module defines the unified trait that combines functions and operators
//! into a single interface with async-first design and optional sync optimization.

use crate::function::EvaluationContext;
use crate::metadata::{OperationMetadata, OperationType};
use async_trait::async_trait;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// Unified trait for all FHIRPath callable operations
///
/// This trait provides a single interface for both functions and operators,
/// with async-first design and optional sync optimization for performance-critical paths.
#[async_trait]
pub trait FhirPathOperation: Send + Sync {
    /// Operation identifier (function name or operator symbol)
    ///
    /// For functions: "count", "first", "where", etc.
    /// For operators: "+", "=", "and", etc.
    fn identifier(&self) -> &str;
    
    /// Operation type (Function, BinaryOperator, UnaryOperator)
    fn operation_type(&self) -> OperationType;
    
    /// Enhanced metadata for the operation
    ///
    /// This includes type constraints, performance characteristics,
    /// LSP support information, and operation-specific metadata.
    fn metadata(&self) -> &OperationMetadata;
    
    /// Async evaluation - primary interface (non-blocking)
    ///
    /// This is the main evaluation method that all operations must implement.
    /// It provides non-blocking evaluation suitable for all operation types.
    ///
    /// # Arguments
    /// * `args` - Arguments to the operation (empty for nullary operations)
    /// * `context` - Evaluation context containing the current focus and environment
    ///
    /// # Returns
    /// The result of the operation as a FhirPathValue
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue>;
    
    /// Optional sync evaluation for performance-critical paths
    ///
    /// Returns None if sync evaluation is not supported by this operation.
    /// This allows performance-critical operations to provide a fast synchronous
    /// path while still supporting the async interface.
    ///
    /// # Arguments
    /// * `args` - Arguments to the operation
    /// * `context` - Evaluation context
    ///
    /// # Returns
    /// Some(Result) if sync evaluation is supported, None otherwise
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        None // Default: async-only
    }
    
    /// Check if operation supports sync evaluation
    ///
    /// This is a fast check to determine if `try_evaluate_sync` might return Some.
    /// Used for optimization decisions in the evaluator.
    fn supports_sync(&self) -> bool {
        false
    }
    
    /// Validate arguments before evaluation
    ///
    /// This method performs argument validation without side effects.
    /// It should check argument count, types, and constraints based on
    /// the operation's metadata.
    ///
    /// # Arguments
    /// * `args` - Arguments to validate
    ///
    /// # Returns
    /// Ok(()) if arguments are valid, Err otherwise
    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()>;

    /// Get expected argument count range
    ///
    /// Returns (min_args, max_args) where max_args is None for variadic functions.
    /// Default implementation extracts this from metadata.
    fn arg_count_range(&self) -> (usize, Option<usize>) {
        let metadata = self.metadata();
        let min_args = metadata.types.parameters.len();
        let max_args = if metadata.types.variadic {
            None
        } else {
            Some(min_args)
        };
        (min_args, max_args)
    }

    /// Check if operation is pure (no side effects)
    ///
    /// Pure operations can be safely cached and optimized.
    /// Default implementation returns true for most operations.
    fn is_pure(&self) -> bool {
        match self.operation_type() {
            OperationType::Function => {
                // Most functions are pure, except debug/trace functions
                !matches!(self.identifier(), "trace" | "defineVariable")
            }
            OperationType::BinaryOperator { .. } | OperationType::UnaryOperator => {
                // All operators are pure
                true
            }
        }
    }

    /// Get operation complexity hint
    ///
    /// This provides a hint about the computational complexity of the operation
    /// for optimization and scheduling decisions.
    fn complexity_hint(&self) -> OperationComplexity {
        self.metadata().performance.complexity.clone().into()
    }
}

/// Operation complexity classification for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationComplexity {
    /// Constant time operations (O(1))
    Constant,
    /// Linear time operations (O(n))
    Linear,
    /// Logarithmic operations (O(log n))
    Logarithmic,
    /// Quadratic operations (O(n²))
    Quadratic,
    /// Expensive operations (network, I/O, complex computation)
    Expensive,
}

impl From<crate::enhanced_metadata::PerformanceComplexity> for OperationComplexity {
    fn from(complexity: crate::enhanced_metadata::PerformanceComplexity) -> Self {
        match complexity {
            crate::enhanced_metadata::PerformanceComplexity::Constant => Self::Constant,
            crate::enhanced_metadata::PerformanceComplexity::Linear => Self::Linear,
            crate::enhanced_metadata::PerformanceComplexity::Logarithmic => Self::Logarithmic,
            crate::enhanced_metadata::PerformanceComplexity::Linearithmic => Self::Linear,
            crate::enhanced_metadata::PerformanceComplexity::Quadratic => Self::Quadratic,
            crate::enhanced_metadata::PerformanceComplexity::Custom(_) => Self::Expensive,
        }
    }
}

/// Trait for operations that support compilation optimization
///
/// Operations implementing this trait can be compiled to more efficient forms
/// when the arguments are known at compile time.
pub trait CompilableOperation: FhirPathOperation {
    /// Attempt to compile the operation with known arguments
    ///
    /// Returns a compiled version of the operation if possible,
    /// or None if compilation is not beneficial for these arguments.
    fn try_compile(&self, args: &[FhirPathValue]) -> Option<CompiledOperation>;
}

/// A compiled operation optimized for specific arguments
pub struct CompiledOperation {
    /// The compiled evaluation function
    pub evaluate: Box<dyn Fn(&EvaluationContext) -> Result<FhirPathValue> + Send + Sync>,
    /// Whether this compiled operation is pure
    pub is_pure: bool,
}

/// Helper trait for operations that work with collections
pub trait CollectionOperation: FhirPathOperation {
    /// Evaluate the operation on a collection
    ///
    /// This is a convenience method for operations that primarily work with collections.
    /// The default implementation delegates to the main evaluate method.
    async fn evaluate_collection(
        &self,
        collection: &[FhirPathValue],
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Create a temporary context with the collection as focus
        let collection_value = FhirPathValue::collection(collection.to_vec());
        let collection_context = context.with_focus(collection_value);
        self.evaluate(args, &collection_context).await
    }
}

/// Helper trait for operations that work with single values
pub trait ScalarOperation: FhirPathOperation {
    /// Evaluate the operation on a single value
    ///
    /// This is a convenience method for operations that work on scalar values.
    /// The default implementation delegates to the main evaluate method.
    async fn evaluate_scalar(
        &self,
        value: &FhirPathValue,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        let scalar_context = context.with_focus(value.clone());
        self.evaluate(args, &scalar_context).await
    }
}

/// Macro to implement common operation patterns
#[macro_export]
macro_rules! impl_operation_basics {
    ($type:ty, $identifier:expr, $operation_type:expr) => {
        impl $type {
            /// Create a new instance of this operation
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for $type {
            fn default() -> Self {
                Self::new()
            }
        }

        impl FhirPathOperation for $type {
            fn identifier(&self) -> &str {
                $identifier
            }

            fn operation_type(&self) -> OperationType {
                $operation_type
            }
        }
    };
}

/// Macro to implement sync operation support
#[macro_export]
macro_rules! impl_sync_operation {
    ($type:ty, $sync_fn:ident) => {
        impl FhirPathOperation for $type {
            fn supports_sync(&self) -> bool {
                true
            }

            fn try_evaluate_sync(
                &self,
                args: &[FhirPathValue],
                context: &EvaluationContext,
            ) -> Option<Result<FhirPathValue>> {
                Some(self.$sync_fn(args, context))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{BasicOperationInfo, PerformanceMetadata, LspMetadata, OperationSpecificMetadata, FunctionMetadata, TypeConstraints};

    // Test operation implementation
    struct TestOperation;

    #[async_trait]
    impl FhirPathOperation for TestOperation {
        fn identifier(&self) -> &str {
            "test"
        }

        fn operation_type(&self) -> OperationType {
            OperationType::Function
        }

        fn metadata(&self) -> &OperationMetadata {
            static METADATA: once_cell::sync::Lazy<OperationMetadata> = once_cell::sync::Lazy::new(|| {
                OperationMetadata {
                    basic: BasicOperationInfo {
                        name: "test".to_string(),
                        operation_type: OperationType::Function,
                        description: "Test operation".to_string(),
                        examples: vec!["test()".to_string()],
                    },
                    types: TypeConstraints::default(),
                    performance: PerformanceMetadata {
                        complexity: crate::enhanced_metadata::PerformanceComplexity::Constant,
                        supports_sync: false,
                        avg_time_ns: 100,
                        memory_usage: 64,
                    },
                    lsp: LspMetadata::default(),
                    specific: OperationSpecificMetadata::Function(FunctionMetadata::default()),
                }
            });
            &METADATA
        }

        async fn evaluate(
            &self,
            _args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> Result<FhirPathValue> {
            Ok(FhirPathValue::String("test".into()))
        }

        fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
            if !args.is_empty() {
                return Err(FhirPathError::InvalidArguments(
                    "test operation takes no arguments".to_string()
                ));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_operation_interface() {
        let op = TestOperation;
        
        assert_eq!(op.identifier(), "test");
        assert_eq!(op.operation_type(), OperationType::Function);
        assert!(!op.supports_sync());
        assert!(op.is_pure());
        assert_eq!(op.arg_count_range(), (0, Some(0)));
        assert_eq!(op.complexity_hint(), OperationComplexity::Constant);
        
        // Test argument validation
        assert!(op.validate_args(&[]).is_ok());
        assert!(op.validate_args(&[FhirPathValue::Integer(1)]).is_err());
        
        // Test evaluation
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = op.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("test".into()));
    }

    #[test]
    fn test_complexity_conversion() {
        use crate::enhanced_metadata::PerformanceComplexity;
        
        assert_eq!(OperationComplexity::from(PerformanceComplexity::Constant), OperationComplexity::Constant);
        assert_eq!(OperationComplexity::from(PerformanceComplexity::Linear), OperationComplexity::Linear);
        assert_eq!(OperationComplexity::from(PerformanceComplexity::Logarithmic), OperationComplexity::Logarithmic);
        assert_eq!(OperationComplexity::from(PerformanceComplexity::Quadratic), OperationComplexity::Quadratic);
        assert_eq!(OperationComplexity::from(PerformanceComplexity::Exponential), OperationComplexity::Expensive);
    }
}