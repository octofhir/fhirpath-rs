//! Power function implementation
//!
//! The power function returns the input raised to the specified power.
//! Syntax: number.power(exponent)

use rust_decimal::prelude::*;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Power function evaluator
pub struct PowerFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl PowerFunctionEvaluator {
    /// Create a new power function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "power".to_string(),
                description: "Returns the input raised to the specified power".to_string(),
                signature: FunctionSignature {
                    input_type: "Number".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "exponent".to_string(),
                        parameter_type: vec!["Number".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The exponent to raise the input to".to_string(),
                        default_value: None,
                    }],
                    return_type: "Decimal".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Math,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for PowerFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "power function requires exactly one argument (exponent)".to_string(),
            ));
        }

        // Handle empty input - propagate empty collections
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "power function can only be called on a single numeric value".to_string(),
            ));
        }

        // Get the base value
        let base_float = match &input[0] {
            FhirPathValue::Integer(i, _, _) => *i as f64,
            FhirPathValue::Decimal(d, _, _) => d.to_f64().ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    "Decimal value cannot be converted to f64 for power calculation".to_string(),
                )
            })?,
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "power function can only be called on numeric values (Integer or Decimal)"
                        .to_string(),
                ));
            }
        };

        // Get the pre-evaluated exponent argument
        // Handle empty exponent parameter - propagate empty collections
        if args[0].is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if args[0].len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "power function exponent argument must be a single value".to_string(),
            ));
        }

        let exponent_float = match &args[0][0] {
            FhirPathValue::Integer(i, _, _) => *i as f64,
            FhirPathValue::Decimal(d, _, _) => d.to_f64().ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    "Decimal exponent cannot be converted to f64 for power calculation".to_string(),
                )
            })?,
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "power function exponent argument must be a numeric value".to_string(),
                ));
            }
        };

        // Handle special cases
        if base_float == 0.0 && exponent_float < 0.0 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "power function: 0 raised to a negative power is undefined".to_string(),
            ));
        }

        if base_float < 0.0 && exponent_float.fract() != 0.0 {
            // Per FHIRPath spec, negative base with non-integer exponent returns empty collection
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let result_float = base_float.powf(exponent_float);

        // Check for invalid results
        if !result_float.is_finite() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "power result is infinite or not a number".to_string(),
            ));
        }

        let result_decimal = Decimal::from_f64(result_float).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "power result cannot be represented as Decimal".to_string(),
            )
        })?;

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::decimal(result_decimal)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
