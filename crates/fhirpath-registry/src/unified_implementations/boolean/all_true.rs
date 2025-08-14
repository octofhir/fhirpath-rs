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

//! Unified allTrue() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified allTrue() function implementation
/// 
/// Returns true if all items in the collection are true
pub struct UnifiedAllTrueFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedAllTrueFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("allTrue", FunctionCategory::BooleanLogic)
            .display_name("All True")
            .description("Returns true if all items in the collection are true")
            .example("Patient.name.given.allTrue()")
            .example("Bundle.entry.where(active).allTrue()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("allTrue()")
            .keywords(vec!["allTrue", "all", "boolean", "logic", "collection"])
            .usage_pattern(
                "Check if all values are true",
                "collection.allTrue()",
                "Boolean logic and validation"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedAllTrueFunction {
    fn name(&self) -> &str {
        "allTrue"
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
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    // Empty collection is vacuously true per FHIRPath spec
                    true
                } else {
                    // All items must be boolean true
                    items.iter().all(|item| matches!(item, FhirPathValue::Boolean(true)))
                }
            },
            FhirPathValue::Empty => {
                // Empty input is vacuously true
                true
            },
            FhirPathValue::Boolean(b) => {
                // Single boolean value
                *b
            },
            _ => {
                // Non-boolean single value is false
                false
            }
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(result)]))
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
    async fn test_unified_allTrue_function() {
        let all_true_func = UnifiedAllTrueFunction::new();
        
        // Test empty collection (vacuously true)
        let context = EvaluationContext::new(FhirPathValue::collection(vec![]));
        let result = all_true_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test all true values
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(true),
        ]));
        let result = all_true_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test mixed values (should be false)
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(true),
        ]));
        let result = all_true_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(false)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test single boolean value
        let context = EvaluationContext::new(FhirPathValue::Boolean(true));
        let result = all_true_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(all_true_func.name(), "allTrue");
        assert_eq!(all_true_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(all_true_func.metadata().basic.display_name, "All True");
    }
}