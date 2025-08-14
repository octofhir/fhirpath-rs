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

//! Unified exp() function implementation

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

/// Unified exp() function implementation
/// 
/// Returns e raised to the power of the input
pub struct UnifiedExpFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedExpFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::math_function("exp")
            .display_name("Exponential")
            .description("Returns e raised to the power of the input")
            .example("(1).exp()")
            .example("(0).exp()")
            .output_type(TypePattern::Exact(TypeInfo::Decimal))
            .lsp_snippet("exp()")
            .keywords(vec!["exp", "exponential", "e", "math"])
            .usage_pattern(
                "Calculate exponential",
                "value.exp()",
                "Mathematical calculations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedExpFunction {
    fn name(&self) -> &str {
        "exp"
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
                let f_result = (*i as f64).exp();
                Decimal::from_f64(f_result).unwrap_or(Decimal::ZERO)
            },
            FhirPathValue::Decimal(d) => {
                let f_val = d.to_f64().unwrap_or(0.0);
                let f_result = f_val.exp();
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
    async fn test_unified_exp_function() {
        let exp_func = UnifiedExpFunction::new();
        
        // Test exp(0) = 1
        let context = EvaluationContext::new(FhirPathValue::Integer(0));
        let result = exp_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::Decimal(d)) = items.get(0) {
                // Check that exp(0) is approximately 1
                assert!((d.to_f64().unwrap() - 1.0).abs() < 0.000001);
            } else {
                panic!("Expected decimal result");
            }
        } else {
            panic!("Expected collection result");
        }
        
        // Test exp(1) ≈ e
        let context = EvaluationContext::new(FhirPathValue::Integer(1));
        let result = exp_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::Decimal(d)) = items.get(0) {
                // Check that exp(1) is approximately e (2.718281828)
                assert!((d.to_f64().unwrap() - std::f64::consts::E).abs() < 0.000001);
            } else {
                panic!("Expected decimal result");
            }
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(exp_func.name(), "exp");
        assert_eq!(exp_func.execution_mode(), ExecutionMode::Sync);
    }
}