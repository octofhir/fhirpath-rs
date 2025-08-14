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

//! Unified toBoolean() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified toBoolean() function implementation
/// 
/// Converts values to booleans
pub struct UnifiedToBooleanFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedToBooleanFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("toBoolean", crate::function::FunctionCategory::TypeConversion)
            .display_name("To Boolean")
            .description("Converts a value to a boolean")
            .example("'true'.toBoolean()")
            .example("1.toBoolean()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("toBoolean()")
            .keywords(vec!["toBoolean", "boolean", "convert", "cast", "bool"])
            .usage_pattern(
                "Convert value to boolean",
                "value.toBoolean()",
                "Type conversion and logical operations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedToBooleanFunction {
    fn name(&self) -> &str {
        "toBoolean"
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
        
        let boolean_value = match &context.input {
            FhirPathValue::Boolean(b) => *b,
            FhirPathValue::Integer(i) => *i != 0,
            FhirPathValue::Decimal(d) => !d.is_zero(),
            FhirPathValue::String(s) => {
                let trimmed = s.trim().to_lowercase();
                match trimmed.as_str() {
                    // Per FHIRPath spec: valid true values
                    "true" | "t" | "yes" | "y" | "1" | "1.0" => true,
                    // Per FHIRPath spec: valid false values  
                    "false" | "f" | "no" | "n" | "0" | "0.0" => false,
                    // Per FHIRPath spec: return empty on conversion failure
                    _ => return Ok(FhirPathValue::Empty),
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Ok(FhirPathValue::Empty), // Per FHIRPath spec: return empty for unsupported types
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(boolean_value)]))
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
    async fn test_unified_toBoolean_function() {
        let to_boolean_func = UnifiedToBooleanFunction::new();
        
        // Test string to boolean
        let context = EvaluationContext::new(FhirPathValue::String("true".into()));
        let result = to_boolean_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test integer to boolean
        let context = EvaluationContext::new(FhirPathValue::Integer(0));
        let result = to_boolean_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(false)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(to_boolean_func.name(), "toBoolean");
        assert_eq!(to_boolean_func.execution_mode(), ExecutionMode::Sync);
    }
}