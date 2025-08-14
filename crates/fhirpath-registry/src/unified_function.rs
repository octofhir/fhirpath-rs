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

//! Unified function trait and execution mode support

use crate::function::{EvaluationContext, FunctionResult};
use crate::enhanced_metadata::EnhancedFunctionMetadata;
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;

/// Execution mode for function dispatch optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ExecutionMode {
    /// Pure synchronous function (no async operations)
    /// These functions can be called in sync contexts without overhead
    Sync,
    
    /// Requires async execution (network calls, model provider operations)
    /// These functions must be called in async contexts
    Async,
    
    /// Sync preferred with async fallback available
    /// These functions have both sync and async implementations
    SyncFirst,
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionMode::Sync => write!(f, "Sync"),
            ExecutionMode::Async => write!(f, "Async"),
            ExecutionMode::SyncFirst => write!(f, "SyncFirst"),
        }
    }
}

/// Unified trait for implementing FHIRPath functions with optimal sync/async dispatch
#[async_trait]
pub trait UnifiedFhirPathFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;
    
    /// Get enhanced metadata for this function
    fn metadata(&self) -> &EnhancedFunctionMetadata;
    
    /// Get the execution mode for optimal dispatch
    fn execution_mode(&self) -> ExecutionMode;
    
    /// Synchronous evaluation (must be implemented for Sync and SyncFirst modes)
    /// 
    /// For pure sync functions, this provides optimal performance without async overhead.
    /// For SyncFirst functions, this is the preferred execution path.
    fn evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        Err(crate::function::FunctionError::ExecutionModeNotSupported {
            function: self.name().to_string(),
            requested_mode: "sync".to_string(),
        })
    }
    
    /// Asynchronous evaluation (must be implemented for Async and SyncFirst modes)
    /// 
    /// For pure async functions, this is the only execution path.
    /// For SyncFirst functions, this provides a fallback when sync execution fails.
    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Default implementation tries sync first for SyncFirst and Sync modes
        match self.execution_mode() {
            ExecutionMode::Sync | ExecutionMode::SyncFirst => {
                // Try sync execution first, fallback to error if not implemented
                self.evaluate_sync(args, context)
            }
            _ => Err(crate::function::FunctionError::ExecutionModeNotSupported {
                function: self.name().to_string(),
                requested_mode: "async".to_string(),
            }),
        }
    }
    
    /// Validate function arguments (common for both sync and async)
    /// 
    /// This method can be overridden for custom validation logic.
    /// The default implementation uses the function signature for validation.
    fn validate_args(&self, args: &[FhirPathValue]) -> FunctionResult<()> {
        let signature = &self.metadata().signature;
        let arg_count = args.len();

        // Check arity
        if arg_count < signature.min_arity {
            return Err(crate::function::FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: signature.min_arity,
                max: signature.max_arity,
                actual: arg_count,
            });
        }

        if let Some(max) = signature.max_arity {
            if arg_count > max {
                return Err(crate::function::FunctionError::InvalidArity {
                    name: self.name().to_string(),
                    min: signature.min_arity,
                    max: signature.max_arity,
                    actual: arg_count,
                });
            }
        }

        // Type validation would go here if more sophisticated checking is needed
        // For now, we rely on runtime type checking within the function implementation
        
        Ok(())
    }
    
    /// Check if this function is pure (deterministic with no side effects)
    /// 
    /// Pure functions can be safely cached and optimized.
    fn is_pure(&self) -> bool {
        self.metadata().performance.is_pure
    }
    
    /// Get function documentation
    fn documentation(&self) -> &str {
        &self.metadata().basic.description
    }
    
    /// Check if this function supports lambda expressions
    /// 
    /// Functions that support lambda expressions can receive unevaluated expressions
    /// as arguments, allowing for advanced evaluation patterns like sort criteria
    /// and filtering predicates.
    fn supports_lambda_expressions(&self) -> bool {
        false // Default implementation - override in functions that support lambda
    }
    
    /// Evaluate function with lambda expression arguments
    /// 
    /// This method is called for functions that support lambda expressions.
    /// It receives unevaluated expression arguments and a lambda evaluation context.
    async fn evaluate_lambda(
        &self,
        _args: &[crate::expression_argument::ExpressionArgument],
        _context: &crate::lambda_function::LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue> {
        Err(crate::function::FunctionError::ExecutionModeNotSupported {
            function: self.name().to_string(),
            requested_mode: "lambda".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata_builder::MetadataBuilder;
    use crate::function::{FunctionCategory, EvaluationContext};
    use octofhir_fhirpath_model::FhirPathValue;
    
    // Test implementation of a sync function
    struct TestSyncFunction {
        metadata: EnhancedFunctionMetadata,
    }
    
    impl TestSyncFunction {
        fn new() -> Self {
            let metadata = MetadataBuilder::new("testSync", FunctionCategory::Utilities)
                .description("A test sync function")
                .pure(true)
                .build();
            
            Self { metadata }
        }
    }
    
    #[async_trait]
    impl UnifiedFhirPathFunction for TestSyncFunction {
        fn name(&self) -> &str {
            "testSync"
        }
        
        fn metadata(&self) -> &EnhancedFunctionMetadata {
            &self.metadata
        }
        
        fn execution_mode(&self) -> ExecutionMode {
            ExecutionMode::Sync
        }
        
        fn evaluate_sync(
            &self,
            _args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> FunctionResult<FhirPathValue> {
            Ok(FhirPathValue::String("sync_result".into()))
        }
    }
    
    #[tokio::test]
    async fn test_sync_function() {
        let func = TestSyncFunction::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        // Test sync evaluation
        let result = func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("sync_result".into()));
        
        // Test async evaluation (should work via default implementation)
        let result = func.evaluate_async(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("sync_result".into()));
        
        // Test metadata
        assert_eq!(func.name(), "testSync");
        assert_eq!(func.execution_mode(), ExecutionMode::Sync);
        assert!(func.is_pure());
        assert_eq!(func.documentation(), "A test sync function");
    }
    
    #[test]
    fn test_execution_mode_display() {
        assert_eq!(ExecutionMode::Sync.to_string(), "Sync");
        assert_eq!(ExecutionMode::Async.to_string(), "Async");
        assert_eq!(ExecutionMode::SyncFirst.to_string(), "SyncFirst");
    }
    
    #[test]
    fn test_execution_mode_serialization() {
        // Test that execution modes can be serialized/deserialized
        let mode = ExecutionMode::Sync;
        let serialized = serde_json::to_string(&mode).unwrap();
        let deserialized: ExecutionMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mode, deserialized);
    }
}