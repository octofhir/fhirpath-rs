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

//! Unified power() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};

/// Unified power() function implementation
///
/// Raises a number to the exponent power
pub struct UnifiedPowerFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedPowerFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("power", FunctionCategory::MathNumbers)
            .display_name("Power")
            .description("Raises a number to the exponent power")
            .example("2.power(3)")
            .example("2.5.power(2)")
            .output_type(TypePattern::OneOf(vec![
                TypeInfo::Integer,
                TypeInfo::Decimal,
            ]))
            .execution_mode(ExecutionMode::Sync)
            .pure(true) // Pure function - same input always produces same output
            .lsp_snippet("power(${1:exponent})")
            .keywords(vec!["power", "exponent", "math", "raise", "pow"])
            .usage_pattern(
                "Raise number to power",
                "base.power(exponent)",
                "Mathematical operations and calculations"
            )
            .build();

        Self { metadata }
    }

    /// Perform power calculation with proper type handling
    fn calculate_power(base: &FhirPathValue, exponent: &FhirPathValue) -> FunctionResult<FhirPathValue> {
        match (base, exponent) {
            // Integer base, Integer exponent -> Integer result (if positive exponent)
            (FhirPathValue::Integer(base_i), FhirPathValue::Integer(exp_i)) => {
                if *exp_i < 0 {
                    // Negative exponent produces decimal result
                    let base_f = *base_i as f64;
                    let exp_f = *exp_i as f64;
                    let result_f = base_f.powf(exp_f);
                    
                    if result_f.is_finite() && result_f > 0.0 {
                        if let Some(decimal) = Decimal::from_f64(result_f) {
                            return Ok(FhirPathValue::Decimal(decimal));
                        }
                    }
                    return Ok(FhirPathValue::Empty);
                } else {
                    // Non-negative integer exponent
                    let base_i = *base_i;
                    let exp_i = *exp_i;
                    
                    if exp_i == 0 {
                        return Ok(FhirPathValue::Integer(1));
                    }
                    
                    // Use checked_pow to prevent overflow
                    if let Some(result) = base_i.checked_pow(exp_i as u32) {
                        Ok(FhirPathValue::Integer(result))
                    } else {
                        // Overflow, convert to decimal
                        let base_f = base_i as f64;
                        let exp_f = exp_i as f64;
                        let result_f = base_f.powf(exp_f);
                        
                        if result_f.is_finite() {
                            if let Some(decimal) = Decimal::from_f64(result_f) {
                                return Ok(FhirPathValue::Decimal(decimal));
                            }
                        }
                        Ok(FhirPathValue::Empty)
                    }
                }
            },
            
            // Any decimal involved -> Decimal result
            (FhirPathValue::Integer(base_i), FhirPathValue::Decimal(exp_d)) |
            (FhirPathValue::Decimal(exp_d), FhirPathValue::Integer(base_i)) => {
                let base_f = *base_i as f64;
                let exp_f = exp_d.to_f64().unwrap_or(0.0);
                
                // Check for invalid operations (like negative base with fractional exponent)
                if base_f < 0.0 && exp_f.fract() != 0.0 {
                    return Ok(FhirPathValue::Empty);
                }
                
                let result_f = base_f.powf(exp_f);
                
                if result_f.is_finite() && result_f >= 0.0 {
                    if let Some(decimal) = Decimal::from_f64(result_f) {
                        return Ok(FhirPathValue::Decimal(decimal));
                    }
                }
                Ok(FhirPathValue::Empty)
            },
            
            (FhirPathValue::Decimal(base_d), FhirPathValue::Decimal(exp_d)) => {
                let base_f = base_d.to_f64().unwrap_or(0.0);
                let exp_f = exp_d.to_f64().unwrap_or(0.0);
                
                // Check for invalid operations
                if base_f < 0.0 && exp_f.fract() != 0.0 {
                    return Ok(FhirPathValue::Empty);
                }
                
                let result_f = base_f.powf(exp_f);
                
                if result_f.is_finite() && result_f >= 0.0 {
                    if let Some(decimal) = Decimal::from_f64(result_f) {
                        return Ok(FhirPathValue::Decimal(decimal));
                    }
                }
                Ok(FhirPathValue::Empty)
            },
            
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedPowerFunction {
    fn name(&self) -> &str {
        "power"
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
        // Validate exactly one argument - the exponent
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }

        let exponent = &args[0];

        // Get the input collection from context (the base)
        let input = &context.input;

        // Check if exponent is empty
        if matches!(exponent, FhirPathValue::Empty) {
            return Ok(FhirPathValue::Empty);
        }

        match input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "power() can only be applied to single items".to_string(),
                    });
                }

                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }

                let base = items.first().unwrap();
                Self::calculate_power(base, exponent)
            }
            _ => {
                // Single item case
                Self::calculate_power(input, exponent)
            }
        }
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
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_unified_power_function() {
        let power_func = UnifiedPowerFunction::new();

        // Test case from specification: 2.power(3) = 8
        let context = EvaluationContext::new(FhirPathValue::Integer(2));
        let result = power_func.evaluate_sync(&[FhirPathValue::Integer(3)], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(8));

        // Test case from specification: 2.5.power(2) = 6.25
        let context = EvaluationContext::new(FhirPathValue::Decimal(dec!(2.5)));
        let result = power_func.evaluate_sync(&[FhirPathValue::Integer(2)], &context).unwrap();
        assert_eq!(result, FhirPathValue::Decimal(dec!(6.25)));

        // Test case from specification: (-1).power(0.5) = empty
        let context = EvaluationContext::new(FhirPathValue::Integer(-1));
        let result = power_func.evaluate_sync(&[FhirPathValue::Decimal(dec!(0.5))], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test zero exponent
        let context = EvaluationContext::new(FhirPathValue::Integer(5));
        let result = power_func.evaluate_sync(&[FhirPathValue::Integer(0)], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        // Test negative exponent
        let context = EvaluationContext::new(FhirPathValue::Integer(2));
        let result = power_func.evaluate_sync(&[FhirPathValue::Integer(-2)], &context).unwrap();
        match result {
            FhirPathValue::Decimal(d) => {
                assert_eq!(d, dec!(0.25));
            },
            _ => panic!("Expected Decimal result for negative exponent"),
        }

        // Test empty base
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = power_func.evaluate_sync(&[FhirPathValue::Integer(2)], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test empty exponent
        let context = EvaluationContext::new(FhirPathValue::Integer(2));
        let result = power_func.evaluate_sync(&[FhirPathValue::Empty], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test invalid argument count
        let context = EvaluationContext::new(FhirPathValue::Integer(2));
        let result = power_func.evaluate_sync(&[], &context);
        assert!(result.is_err());

        let result = power_func.evaluate_sync(&[FhirPathValue::Integer(2), FhirPathValue::Integer(3)], &context);
        assert!(result.is_err());

        // Test metadata
        assert_eq!(power_func.name(), "power");
        assert_eq!(power_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(power_func.metadata().basic.display_name, "Power");
        assert!(power_func.metadata().basic.is_pure);
    }
}