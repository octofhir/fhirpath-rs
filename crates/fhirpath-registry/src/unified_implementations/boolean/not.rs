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

//! Unified not() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified not() function implementation
/// 
/// Logical negation function
pub struct UnifiedNotFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedNotFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("not", FunctionCategory::BooleanLogic)
            .display_name("Not")
            .description("Returns true if the input evaluates to false, and false if it evaluates to true")
            .example("Patient.active.not()")
            .example("Bundle.entry.empty().not()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("not()")
            .keywords(vec!["not", "negation", "boolean", "logic", "invert"])
            .usage_pattern(
                "Logical negation",
                "value.not()",
                "Boolean logic and conditional expressions"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedNotFunction {
    fn name(&self) -> &str {
        "not"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }
    
    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(0),
                actual: args.len(),
            });
        }
        
        let result = match &context.input {
            FhirPathValue::Boolean(b) => {
                // Direct boolean negation
                FhirPathValue::Boolean(!b)
            },
            FhirPathValue::Integer(i) => {
                // Per FHIRPath spec: 0 is false, non-zero is true
                let bool_val = *i != 0;
                FhirPathValue::Boolean(!bool_val)
            },
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    // Empty collection is false, not() becomes true
                    FhirPathValue::Boolean(true)
                } else if items.len() == 1 {
                    // Single item collection - apply not() to the item
                    match items.get(0) {
                        Some(FhirPathValue::Boolean(b)) => FhirPathValue::Boolean(!b),
                        Some(FhirPathValue::Integer(i)) => {
                            let bool_val = *i != 0;
                            FhirPathValue::Boolean(!bool_val)
                        },
                        _ => {
                            // Non-boolean/integer item returns empty per FHIRPath spec
                            return Ok(FhirPathValue::Empty);
                        }
                    }
                } else {
                    // Multiple items - return empty per FHIRPath spec for not()
                    return Ok(FhirPathValue::Empty);
                }
            },
            FhirPathValue::Empty => {
                // Empty is false, not() becomes true
                FhirPathValue::Boolean(true)
            },
            _ => {
                // Other types return empty per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };
        
        Ok(result)
    }
    
    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.evaluate_sync(args, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;

    #[tokio::test]
    async fn test_unified_not_function() {
        let not_func = UnifiedNotFunction::new();
        
        // Test boolean true
        let context = EvaluationContext::new(FhirPathValue::Boolean(true));
        let result = not_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
        
        // Test boolean false
        let context = EvaluationContext::new(FhirPathValue::Boolean(false));
        let result = not_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        
        // Test integer 0 (false)
        let context = EvaluationContext::new(FhirPathValue::Integer(0));
        let result = not_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        
        // Test integer non-zero (true)
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = not_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
        
        // Test empty collection (false, not becomes true)
        let context = EvaluationContext::new(FhirPathValue::collection(vec![]));
        let result = not_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        
        // Test single item collection with boolean
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true)
        ]));
        let result = not_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
        
        // Test multiple items collection (should return empty)
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(false),
        ]));
        let result = not_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test empty input
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = not_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        
        // Test string input (should return empty)
        let context = EvaluationContext::new(FhirPathValue::String("test".into()));
        let result = not_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test metadata
        assert_eq!(not_func.name(), "not");
        assert_eq!(not_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(not_func.metadata().basic.display_name, "Not");
    }
}