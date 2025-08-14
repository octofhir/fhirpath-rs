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

//! Unified round() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
    signature::{FunctionSignature, ParameterInfo},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::ToPrimitive;

/// Unified round() function implementation
/// 
/// Returns the rounded value of a decimal to the nearest integer or to specified precision
pub struct UnifiedRoundFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedRoundFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "round",
            vec![ParameterInfo::optional("precision", TypeInfo::Integer)],
            TypeInfo::Decimal, // Can return Integer or Decimal depending on precision
        );
        
        let metadata = MetadataBuilder::math_function("round")
            .display_name("Round")
            .description("Returns the value rounded to the nearest integer or to specified precision")
            .example("(3.6).round()")
            .example("(-1.4).round()")
            .example("(1.23456).round(2)")
            .signature(signature)
            .output_type(TypePattern::Exact(TypeInfo::Integer))
            .lsp_snippet("round()")
            .keywords(vec!["round", "nearest", "math"])
            .usage_pattern(
                "Round to nearest integer",
                "value.round()",
                "Mathematical rounding operations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedRoundFunction {
    fn name(&self) -> &str {
        "round"
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
        // Validate arguments - 0 or 1 argument allowed (optional precision)
        if args.len() > 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        // Get optional precision argument
        let precision = if args.is_empty() {
            0 // Default precision - round to integer
        } else {
            match &args[0] {
                FhirPathValue::Integer(p) => {
                    if *p < 0 {
                        return Err(FunctionError::EvaluationError {
                            name: self.name().to_string(),
                            message: "Precision must be non-negative".to_string(),
                        });
                    }
                    *p as u32
                },
                _ => return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "Precision argument must be an integer".to_string(),
                }),
            }
        };
        
        let result = match &context.input {
            FhirPathValue::Integer(i) => {
                if precision == 0 {
                    FhirPathValue::Integer(*i)
                } else {
                    // Integer remains unchanged for positive precision
                    FhirPathValue::Decimal(rust_decimal::Decimal::from(*i))
                }
            },
            FhirPathValue::Decimal(d) => {
                if precision == 0 {
                    // Round to nearest integer
                    FhirPathValue::Integer(d.round().to_i64().unwrap_or(0))
                } else {
                    // Round to specified decimal places
                    let scale = 10_i64.pow(precision);
                    let scaled = (*d * rust_decimal::Decimal::from(scale)).round();
                    let rounded = scaled / rust_decimal::Decimal::from(scale);
                    FhirPathValue::Decimal(rounded)
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected numeric value, got {}", context.input.type_name()),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![result]))
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
    async fn test_unified_round_function() {
        let round_func = UnifiedRoundFunction::new();
        
        // Test rounding up
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_f64(3.6).unwrap()));
        let result = round_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(4)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test rounding down
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_f64(3.4).unwrap()));
        let result = round_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(3)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(round_func.name(), "round");
        assert_eq!(round_func.execution_mode(), ExecutionMode::Sync);
    }
}