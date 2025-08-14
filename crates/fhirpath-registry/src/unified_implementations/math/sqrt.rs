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

//! Unified sqrt() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

/// Unified sqrt() function implementation
/// 
/// Returns the square root of a number
pub struct UnifiedSqrtFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedSqrtFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::math_function("sqrt")
            .display_name("Square Root")
            .description("Returns the square root of a number")
            .example("(16).sqrt()")
            .example("(2.25).sqrt()")
            .output_type(TypePattern::Exact(TypeInfo::Decimal))
            .lsp_snippet("sqrt()")
            .keywords(vec!["sqrt", "square", "root", "math"])
            .usage_pattern(
                "Calculate square root",
                "value.sqrt()",
                "Mathematical calculations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedSqrtFunction {
    fn name(&self) -> &str {
        "sqrt"
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
            FhirPathValue::Integer(i) => {
                if *i < 0 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Cannot calculate square root of negative number".to_string(),
                    });
                }
                // Convert to f64 for sqrt calculation, then back to Decimal
                let f_result = (*i as f64).sqrt();
                Decimal::from_f64(f_result).unwrap_or(Decimal::ZERO)
            },
            FhirPathValue::Decimal(d) => {
                if *d < Decimal::ZERO {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Cannot calculate square root of negative number".to_string(),
                    });
                }
                // Convert to f64 for sqrt calculation, then back to Decimal
                let f_val = d.to_f64().unwrap_or(0.0);
                let f_result = f_val.sqrt();
                Decimal::from_f64(f_result).unwrap_or(Decimal::ZERO)
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected numeric value, got {}", context.input.type_name()),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(result)]))
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
    async fn test_unified_sqrt_function() {
        let sqrt_func = UnifiedSqrtFunction::new();
        
        // Test perfect square
        let context = EvaluationContext::new(FhirPathValue::Integer(16));
        let result = sqrt_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Decimal(Decimal::from(4))));
        } else {
            panic!("Expected collection result");
        }
        
        // Test decimal square
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_f64(2.25).unwrap()));
        let result = sqrt_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Decimal(Decimal::from_f64(1.5).unwrap())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test negative number error
        let context = EvaluationContext::new(FhirPathValue::Integer(-4));
        let result = sqrt_func.evaluate_sync(&[], &context);
        assert!(result.is_err());
        
        // Test metadata
        assert_eq!(sqrt_func.name(), "sqrt");
        assert_eq!(sqrt_func.execution_mode(), ExecutionMode::Sync);
    }
}