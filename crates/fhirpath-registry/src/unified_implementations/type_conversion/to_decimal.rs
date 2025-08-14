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

//! Unified toDecimal() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::Decimal;

/// Unified toDecimal() function implementation
/// 
/// Converts values to decimals
pub struct UnifiedToDecimalFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedToDecimalFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("toDecimal", crate::function::FunctionCategory::TypeConversion)
            .display_name("To Decimal")
            .description("Converts a value to a decimal")
            .example("'3.14'.toDecimal()")
            .example("42.toDecimal()")
            .output_type(TypePattern::Exact(TypeInfo::Decimal))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("toDecimal()")
            .keywords(vec!["toDecimal", "decimal", "convert", "cast", "float"])
            .usage_pattern(
                "Convert value to decimal",
                "value.toDecimal()",
                "Type conversion and numeric calculations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedToDecimalFunction {
    fn name(&self) -> &str {
        "toDecimal"
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
        
        let decimal_value = match &context.input {
            FhirPathValue::Decimal(d) => *d,
            FhirPathValue::Integer(i) => Decimal::from(*i),
            FhirPathValue::String(s) => {
                match s.trim().parse::<Decimal>() {
                    Ok(value) => value,
                    Err(_) => return Ok(FhirPathValue::Empty), // Per FHIRPath spec: return empty on conversion failure
                }
            },
            FhirPathValue::Boolean(b) => Decimal::from(if *b { 1 } else { 0 }),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Ok(FhirPathValue::Empty), // Per FHIRPath spec: return empty for unsupported types
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(decimal_value)]))
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
    use rust_decimal::prelude::FromPrimitive;

    #[tokio::test]
    async fn test_unified_toDecimal_function() {
        let to_decimal_func = UnifiedToDecimalFunction::new();
        
        // Test string to decimal
        let context = EvaluationContext::new(FhirPathValue::String("3.14".into()));
        let result = to_decimal_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::Decimal(d)) = items.get(0) {
                assert_eq!(*d, Decimal::from_f64(3.14).unwrap());
            } else {
                panic!("Expected decimal value");
            }
        } else {
            panic!("Expected collection result");
        }
        
        // Test integer to decimal
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = to_decimal_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::Decimal(d)) = items.get(0) {
                assert_eq!(*d, Decimal::from(42));
            } else {
                panic!("Expected decimal value");
            }
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(to_decimal_func.name(), "toDecimal");
        assert_eq!(to_decimal_func.execution_mode(), ExecutionMode::Sync);
    }
}