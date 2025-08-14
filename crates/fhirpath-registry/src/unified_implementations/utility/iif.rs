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

//! Unified iif() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified iif() function implementation
/// 
/// Conditional function that returns one of two values based on a boolean condition.
/// Syntax: iif(condition, trueValue, falseValue)
pub struct UnifiedIifFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedIifFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "iif",
            vec![
                ParameterInfo::required("condition", TypeInfo::Boolean),
                ParameterInfo::required("trueValue", TypeInfo::Any),
                ParameterInfo::required("falseValue", TypeInfo::Any),
            ],
            TypeInfo::Any,
        );
        
        let metadata = MetadataBuilder::new("iif", FunctionCategory::Utilities)
            .display_name("Conditional")
            .description("Returns trueValue if condition is true, falseValue otherwise")
            .example("iif(Patient.active, 'Active', 'Inactive')")
            .example("iif(value > 10, 'High', 'Low')")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::Any, TypePattern::Any, TypePattern::Any])
            .output_type(TypePattern::Any)
            .supports_collections(true)
            .pure(true)
            .complexity(PerformanceComplexity::Constant)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("iif(${1:condition}, ${2:trueValue}, ${3:falseValue})")
            .completion_visibility(CompletionVisibility::Always)
            .keywords(vec!["conditional", "if", "ternary", "branch"])
            .usage_pattern(
                "Conditional value selection",
                "iif(Patient.active, 'Active', 'Inactive')",
                "Selecting values based on boolean conditions"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedIifFunction {
    fn name(&self) -> &str {
        "iif"
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
        // Validate arguments - 2 or 3 arguments allowed
        if args.len() < 2 || args.len() > 3 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 2,
                max: Some(3),
                actual: args.len(),
            });
        }

        // iif() is a method function that requires a single input item per FHIRPath spec
        // For collections with multiple items, it should return empty
        match &context.input {
            FhirPathValue::Collection(items) if items.len() > 1 => {
                return Ok(FhirPathValue::Empty);
            }
            _ => {}
        }
        
        let condition = &args[0];
        let true_value = &args[1];
        let false_value = if args.len() > 2 { 
            Some(&args[2]) 
        } else { 
            None 
        };
        
        // Evaluate condition as boolean - per FHIRPath spec, condition must be boolean
        let is_true = match condition {
            FhirPathValue::Boolean(b) => *b,
            FhirPathValue::Empty => false, // Empty is treated as false
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    false
                } else if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::Boolean(b)) => *b,
                        Some(FhirPathValue::Empty) => false,
                        Some(_) => {
                            // Non-boolean value - return empty per FHIRPath spec
                            return Ok(FhirPathValue::Empty);
                        }
                        None => false,
                    }
                } else {
                    // Collections with multiple items - invalid condition
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => {
                // Non-boolean condition - return empty per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };
        
        // Return the appropriate value
        let result = if is_true {
            true_value.clone()
        } else {
            match false_value {
                Some(val) => val.clone(),
                None => FhirPathValue::Empty, // 2-argument form returns empty for false
            }
        };
        
        // Return the result - FHIRPath functions return single values as collections
        Ok(match result {
            FhirPathValue::Empty => FhirPathValue::Empty,
            FhirPathValue::Collection(_) => result, // Already a collection
            single => FhirPathValue::collection(vec![single]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_iif_function() {
        let func = UnifiedIifFunction::new();
        
        // Test true condition
        let args = vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::String("true_value".into()),
            FhirPathValue::String("false_value".into()),
        ];
        let context = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::String("true_value".into()));
        
        // Test false condition
        let args = vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::String("true_value".into()),
            FhirPathValue::String("false_value".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::String("false_value".into()));
        
        // Test empty condition (should be false)
        let args = vec![
            FhirPathValue::Empty,
            FhirPathValue::String("true_value".into()),
            FhirPathValue::String("false_value".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::String("false_value".into()));
    }
}