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

//! Unified toInteger() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::ToPrimitive;

/// Unified toInteger() function implementation
/// 
/// Converts values to integers
pub struct UnifiedToIntegerFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedToIntegerFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("toInteger", crate::function::FunctionCategory::TypeConversion)
            .display_name("To Integer")
            .description("Converts a value to an integer")
            .example("'42'.toInteger()")
            .example("3.14.toInteger()")
            .output_type(TypePattern::Exact(TypeInfo::Integer))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("toInteger()")
            .keywords(vec!["toInteger", "integer", "convert", "cast", "int"])
            .usage_pattern(
                "Convert value to integer",
                "value.toInteger()",
                "Type conversion and numeric calculations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedToIntegerFunction {
    fn name(&self) -> &str {
        "toInteger"
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
        
        let integer_value = match &context.input {
            FhirPathValue::Integer(i) => *i,
            FhirPathValue::Decimal(d) => d.to_i64().unwrap_or_else(|| {
                // If conversion fails, try truncating
                d.trunc().to_i64().unwrap_or(0)
            }),
            FhirPathValue::String(s) => {
                match s.trim().parse::<i64>() {
                    Ok(value) => value,
                    Err(_) => return Ok(FhirPathValue::Empty), // Per FHIRPath spec: return empty on conversion failure
                }
            },
            FhirPathValue::Boolean(b) => if *b { 1 } else { 0 },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Ok(FhirPathValue::Empty), // Per FHIRPath spec: return empty for unsupported types
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(integer_value)]))
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
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;

    #[tokio::test]
    async fn test_unified_toInteger_function() {
        let to_integer_func = UnifiedToIntegerFunction::new();
        
        // Test string to integer
        let context = EvaluationContext::new(FhirPathValue::String("42".into()));
        let result = to_integer_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(42)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test decimal to integer
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_f64(3.14).unwrap()));
        let result = to_integer_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(3)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test boolean to integer
        let context = EvaluationContext::new(FhirPathValue::Boolean(true));
        let result = to_integer_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(1)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(to_integer_func.name(), "toInteger");
        assert_eq!(to_integer_func.execution_mode(), ExecutionMode::Sync);
    }
}