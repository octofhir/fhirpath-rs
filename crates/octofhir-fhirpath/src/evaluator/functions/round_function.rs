//! Round function implementation
//!
//! The round function rounds the input to the nearest integer.
//! Syntax: number.round() or number.round(precision)

use rust_decimal::prelude::*;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Round function evaluator
pub struct RoundFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl RoundFunctionEvaluator {
    /// Create a new round function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "round".to_string(),
                description:
                    "Rounds the input to the nearest integer or to the specified precision"
                        .to_string(),
                signature: FunctionSignature {
                    input_type: "Number".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "precision".to_string(),
                        parameter_type: vec!["Integer".to_string()],
                        optional: true,
                        is_expression: true,
                        description: "Number of decimal places to round to (default: 0)"
                            .to_string(),
                        default_value: Some("0".to_string()),
                    }],
                    return_type: "Decimal".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(1),
                },
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
impl FunctionEvaluator for RoundFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() > 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "round function takes at most one argument (precision)".to_string(),
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
                "round function can only be called on a single numeric value".to_string(),
            ));
        }

        // Get the input value as decimal
        let input_decimal = match &input[0] {
            FhirPathValue::Integer(i, _, _) => Decimal::from(*i),
            FhirPathValue::Decimal(d, _, _) => *d,
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "round function can only be called on numeric values (Integer or Decimal)"
                        .to_string(),
                ));
            }
        };

        // Get precision (default to 0)
        let precision = if args.is_empty() {
            0u32
        } else {
            let precision_result = evaluator.evaluate(&args[0], context).await?;
            let precision_values: Vec<FhirPathValue> =
                precision_result.value.iter().cloned().collect();

            if precision_values.len() != 1 {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "round function precision argument must evaluate to a single value".to_string(),
                ));
            }

            match &precision_values[0] {
                FhirPathValue::Integer(i, _, _) => {
                    if *i < 0 {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0051,
                            "round function precision must be non-negative".to_string(),
                        ));
                    }
                    if *i > 28 {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0051,
                            "round function precision cannot exceed 28 decimal places".to_string(),
                        ));
                    }
                    *i as u32
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0057,
                        "round function precision argument must be an integer".to_string(),
                    ));
                }
            }
        };

        // Round to specified precision
        let result_decimal = input_decimal.round_dp(precision);

        // For precision 0, return as integer if possible
        let result = if precision == 0 && result_decimal.fract() == Decimal::ZERO {
            if let Some(int_value) = result_decimal.to_i64() {
                FhirPathValue::integer(int_value)
            } else {
                FhirPathValue::decimal(result_decimal)
            }
        } else {
            FhirPathValue::decimal(result_decimal)
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![result]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
