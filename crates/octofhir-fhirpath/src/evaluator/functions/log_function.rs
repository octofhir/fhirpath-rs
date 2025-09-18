//! Log function implementation
//!
//! The log function returns the logarithm of the input to the specified base.
//! Syntax: number.log(base)

use rust_decimal::prelude::*;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Log function evaluator
pub struct LogFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl LogFunctionEvaluator {
    /// Create a new log function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "log".to_string(),
                description: "Returns the logarithm of the input to the specified base".to_string(),
                signature: FunctionSignature {
                    input_type: "Number".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "base".to_string(),
                        parameter_type: vec!["Number".to_string()],
                        optional: false,
                        is_expression: true,
                        description: "The base for the logarithm".to_string(),
                        default_value: None,
                    }],
                    return_type: "Decimal".to_string(),
                    polymorphic: false,
                    min_params: 1,
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
impl FunctionEvaluator for LogFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "log function requires exactly one argument (base)".to_string(),
            ));
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "log function can only be called on a single numeric value".to_string(),
            ));
        }

        // Get the input value
        let input_float = match &input[0] {
            FhirPathValue::Integer(i, _, _) => {
                if *i <= 0 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "log function requires a positive input value".to_string(),
                    ));
                }
                *i as f64
            }
            FhirPathValue::Decimal(d, _, _) => {
                if *d <= Decimal::ZERO {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "log function requires a positive input value".to_string(),
                    ));
                }
                d.to_f64().ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "Decimal value cannot be converted to f64 for log calculation".to_string(),
                    )
                })?
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "log function can only be called on numeric values (Integer or Decimal)"
                        .to_string(),
                ));
            }
        };

        // Evaluate base argument
        let base_result = evaluator.evaluate(&args[0], context).await?;
        let base_values: Vec<FhirPathValue> = base_result.value.iter().cloned().collect();

        if base_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "log function base argument must evaluate to a single value".to_string(),
            ));
        }

        let base_float = match &base_values[0] {
            FhirPathValue::Integer(i, _, _) => {
                if *i <= 0 || *i == 1 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "log function requires a positive base not equal to 1".to_string(),
                    ));
                }
                *i as f64
            }
            FhirPathValue::Decimal(d, _, _) => {
                if *d <= Decimal::ZERO || *d == Decimal::ONE {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "log function requires a positive base not equal to 1".to_string(),
                    ));
                }
                d.to_f64().ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "Decimal base cannot be converted to f64 for log calculation".to_string(),
                    )
                })?
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "log function base argument must be a numeric value".to_string(),
                ));
            }
        };

        let result_float = input_float.log(base_float);

        // Check for invalid results
        if !result_float.is_finite() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "log result is infinite or not a number".to_string(),
            ));
        }

        let result_decimal = Decimal::from_f64(result_float).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "log result cannot be represented as Decimal".to_string(),
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
