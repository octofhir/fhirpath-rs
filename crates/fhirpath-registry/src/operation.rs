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

use crate::metadata::{OperationMetadata, OperationType};
use crate::operations::EvaluationContext;
use crate::signature::{FunctionSignature, OperatorSignature};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

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

    /// Operation signature for type checking and validation
    ///
    /// This provides rich signature information that includes parameter types,
    /// return types, and validation rules. Default implementation extracts
    /// signature from metadata for backward compatibility.
    fn signature(&self) -> OperationSignature {
        OperationSignature::from_metadata(self.metadata())
    }

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
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
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
    /// It checks argument count, types, and constraints based on the operation's signature.
    /// Default implementation uses the signature for validation.
    ///
    /// # Arguments
    /// * `args` - Arguments to validate
    ///
    /// # Returns
    /// Ok(()) if arguments are valid, Err otherwise
    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        self.signature().validate_args(args)
    }

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

    /// Downcast support for specialized operation types
    ///
    /// This allows the engine to downcast operations to specific types
    /// (e.g., LambdaFunction) when needed.
    fn as_any(&self) -> &dyn std::any::Any;
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

/// Unified operation signature that combines function and operator signatures
#[derive(Debug, Clone)]
pub enum OperationSignature {
    /// Function signature with parameters and return type
    Function(FunctionSignature),
    /// Operator signature with operand and result types
    Operator(OperatorSignature),
}

impl OperationSignature {
    /// Create signature from operation metadata
    pub fn from_metadata(metadata: &OperationMetadata) -> Self {
        use crate::signature::ParameterInfo;

        match metadata.basic.operation_type {
            OperationType::Function => {
                let parameters: Vec<ParameterInfo> = metadata
                    .types
                    .parameters
                    .iter()
                    .map(|param| ParameterInfo {
                        name: param.name.clone(),
                        param_type: type_constraint_to_type_info(&param.constraint),
                        optional: param.optional,
                    })
                    .collect();

                let mut signature = FunctionSignature::new(
                    &metadata.basic.name,
                    parameters,
                    type_constraint_to_type_info(&metadata.types.return_type),
                );

                if metadata.types.variadic {
                    signature.max_arity = None;
                }

                OperationSignature::Function(signature)
            }
            OperationType::BinaryOperator { .. } => {
                let left_type = metadata
                    .types
                    .parameters
                    .first()
                    .map(|p| type_constraint_to_type_info(&p.constraint))
                    .unwrap_or(TypeInfo::Any);
                let right_type = metadata
                    .types
                    .parameters
                    .get(1)
                    .map(|p| type_constraint_to_type_info(&p.constraint))
                    .unwrap_or(TypeInfo::Any);
                let result_type = type_constraint_to_type_info(&metadata.types.return_type);

                OperationSignature::Operator(OperatorSignature::binary(
                    &metadata.basic.name,
                    left_type,
                    right_type,
                    result_type,
                ))
            }
            OperationType::UnaryOperator => {
                let operand_type = metadata
                    .types
                    .parameters
                    .first()
                    .map(|p| type_constraint_to_type_info(&p.constraint))
                    .unwrap_or(TypeInfo::Any);
                let result_type = type_constraint_to_type_info(&metadata.types.return_type);

                OperationSignature::Operator(OperatorSignature::unary(
                    &metadata.basic.name,
                    operand_type,
                    result_type,
                ))
            }
        }
    }

    /// Validate arguments against this signature
    pub fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        match self {
            OperationSignature::Function(sig) => validate_function_args(sig, args),
            OperationSignature::Operator(sig) => validate_operator_args(sig, args),
        }
    }
}

/// Convert type constraint to TypeInfo
fn type_constraint_to_type_info(constraint: &crate::metadata::TypeConstraint) -> TypeInfo {
    use crate::metadata::TypeConstraint;

    match constraint {
        TypeConstraint::Any => TypeInfo::Any,
        TypeConstraint::Specific(fhir_type) => fhir_type_to_type_info(fhir_type),
        TypeConstraint::OneOf(types) => {
            // For now, use the first type or Any if empty
            types
                .first()
                .map(fhir_type_to_type_info)
                .unwrap_or(TypeInfo::Any)
        }
        TypeConstraint::Collection(_) => TypeInfo::Collection(Box::new(TypeInfo::Any)),
        TypeConstraint::Numeric => TypeInfo::Integer, // Default to integer for numeric
        TypeConstraint::Comparable => TypeInfo::Any,
    }
}

/// Convert FhirPathType to TypeInfo
fn fhir_type_to_type_info(fhir_type: &crate::metadata::FhirPathType) -> TypeInfo {
    use crate::metadata::FhirPathType;

    match fhir_type {
        FhirPathType::Empty => TypeInfo::Any, // No Empty in TypeInfo, use Any
        FhirPathType::Boolean => TypeInfo::Boolean,
        FhirPathType::Integer => TypeInfo::Integer,
        FhirPathType::Decimal => TypeInfo::Decimal,
        FhirPathType::String => TypeInfo::String,
        FhirPathType::Date => TypeInfo::Date,
        FhirPathType::DateTime => TypeInfo::DateTime,
        FhirPathType::Time => TypeInfo::Time,
        FhirPathType::Quantity => TypeInfo::Quantity,
        FhirPathType::Resource => TypeInfo::Resource("".to_string()), // Generic resource
        FhirPathType::Collection => TypeInfo::Collection(Box::new(TypeInfo::Any)),
        FhirPathType::Any => TypeInfo::Any,
    }
}

/// Validate function arguments
fn validate_function_args(signature: &FunctionSignature, args: &[FhirPathValue]) -> Result<()> {
    if args.len() < signature.min_arity {
        return Err(FhirPathError::InvalidArgumentCount {
            function_name: signature.name.clone(),
            expected: signature.min_arity,
            actual: args.len(),
        });
    }

    if let Some(max) = signature.max_arity {
        if args.len() > max {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: signature.name.clone(),
                expected: max,
                actual: args.len(),
            });
        }
    }

    // Check parameter types (simplified for now)
    for (i, arg) in args.iter().enumerate() {
        if let Some(param) = signature.parameters.get(i) {
            let arg_type = value_to_type_info(arg);
            if param.param_type != TypeInfo::Any && !type_matches(&arg_type, &param.param_type) {
                return Err(FhirPathError::TypeError {
                    message: format!(
                        "Argument {} of function '{}' expected type {:?}, got {:?}",
                        i + 1,
                        signature.name,
                        param.param_type,
                        arg_type
                    ),
                });
            }
        }
    }

    Ok(())
}

/// Validate operator arguments
fn validate_operator_args(signature: &OperatorSignature, args: &[FhirPathValue]) -> Result<()> {
    match signature.right_type {
        Some(_) => {
            // Binary operator
            if args.len() != 2 {
                return Err(FhirPathError::InvalidArgumentCount {
                    function_name: signature.symbol.clone(),
                    expected: 2,
                    actual: args.len(),
                });
            }
        }
        None => {
            // Unary operator
            if args.len() != 1 {
                return Err(FhirPathError::InvalidArgumentCount {
                    function_name: signature.symbol.clone(),
                    expected: 1,
                    actual: args.len(),
                });
            }
        }
    }

    Ok(())
}

/// Convert FhirPathValue to TypeInfo
fn value_to_type_info(value: &FhirPathValue) -> TypeInfo {
    match value {
        FhirPathValue::Empty => TypeInfo::Any, // No Empty in TypeInfo
        FhirPathValue::Boolean(_) => TypeInfo::Boolean,
        FhirPathValue::Integer(_) => TypeInfo::Integer,
        FhirPathValue::Decimal(_) => TypeInfo::Decimal,
        FhirPathValue::String(_) => TypeInfo::String,
        FhirPathValue::Date(_) => TypeInfo::Date,
        FhirPathValue::DateTime(_) => TypeInfo::DateTime,
        FhirPathValue::Time(_) => TypeInfo::Time,
        FhirPathValue::Quantity(_) => TypeInfo::Quantity,
        FhirPathValue::Resource(_) => TypeInfo::Resource("".to_string()),
        FhirPathValue::Collection(_) => TypeInfo::Collection(Box::new(TypeInfo::Any)),
        FhirPathValue::JsonValue(_) => TypeInfo::Any, // JSON values can be any type
        FhirPathValue::TypeInfoObject { .. } => TypeInfo::Any, // Type info objects
    }
}

/// Check if two types match (simplified matching)
fn type_matches(actual: &TypeInfo, expected: &TypeInfo) -> bool {
    match (actual, expected) {
        (_, TypeInfo::Any) => true,
        (TypeInfo::Any, _) => true,
        (a, b) => a == b,
    }
}

impl From<crate::metadata::PerformanceComplexity> for OperationComplexity {
    fn from(complexity: crate::metadata::PerformanceComplexity) -> Self {
        match complexity {
            crate::metadata::PerformanceComplexity::Constant => OperationComplexity::Constant,
            crate::metadata::PerformanceComplexity::Linear => OperationComplexity::Linear,
            crate::metadata::PerformanceComplexity::Logarithmic => OperationComplexity::Logarithmic,
            crate::metadata::PerformanceComplexity::Linearithmic => OperationComplexity::Linear,
            crate::metadata::PerformanceComplexity::Quadratic => OperationComplexity::Quadratic,
            crate::metadata::PerformanceComplexity::Exponential => OperationComplexity::Expensive,
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

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

/// Macro to add as_any implementation to existing FhirPathOperation impls
#[macro_export]
macro_rules! impl_as_any {
    ($type:ty) => {
        impl $type {
            pub fn as_any_operation(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{
        BasicOperationInfo, FunctionMetadata, OperationSpecificMetadata, PerformanceMetadata,
        TypeConstraints,
    };

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
            static METADATA: once_cell::sync::Lazy<OperationMetadata> =
                once_cell::sync::Lazy::new(|| OperationMetadata {
                    basic: BasicOperationInfo {
                        name: "test".to_string(),
                        operation_type: OperationType::Function,
                        description: "Test operation".to_string(),
                        examples: vec!["test()".to_string()],
                    },
                    types: TypeConstraints::default(),
                    performance: PerformanceMetadata {
                        complexity: crate::metadata::PerformanceComplexity::Constant,
                        supports_sync: false,
                        avg_time_ns: 100,
                        memory_usage: 64,
                    },
                    specific: OperationSpecificMetadata::Function(FunctionMetadata::default()),
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
                return Err(FhirPathError::EvaluationError {
                    message: "test operation takes no arguments".to_string(),
                });
            }
            Ok(())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
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
        let context = {
            use crate::FhirPathRegistry;
            use octofhir_fhirpath_model::MockModelProvider;
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Empty, registry, model_provider)
        };
        let result = op.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("test".into()));
    }

    #[test]
    fn test_complexity_conversion() {
        use crate::metadata::PerformanceComplexity;

        assert_eq!(
            OperationComplexity::from(PerformanceComplexity::Constant),
            OperationComplexity::Constant
        );
        assert_eq!(
            OperationComplexity::from(PerformanceComplexity::Linear),
            OperationComplexity::Linear
        );
        assert_eq!(
            OperationComplexity::from(PerformanceComplexity::Logarithmic),
            OperationComplexity::Logarithmic
        );
        assert_eq!(
            OperationComplexity::from(PerformanceComplexity::Quadratic),
            OperationComplexity::Quadratic
        );
        assert_eq!(
            OperationComplexity::from(PerformanceComplexity::Exponential),
            OperationComplexity::Expensive
        );
    }
}
