//! Round function implementation
//!
//! The round function rounds a Decimal or Quantity (Integer and Long are converted
//! to Decimal) to the nearest whole number, halves away from zero.
//! Syntax: number.round() or number.round(precision)

use rust_decimal::prelude::*;
use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Round function evaluator
pub struct RoundFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl RoundFunctionEvaluator {
    /// Create a new round function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "round".to_string(),
                description:
                    "Rounds the input to the nearest integer or to the specified precision"
                        .to_string(),
                signature: FunctionSignature {
                    input_type: "Decimal | Quantity".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "precision".to_string(),
                        parameter_type: vec!["Integer".to_string()],
                        optional: true,
                        is_expression: false,
                        description: "Number of decimal places to round to (default: 0)"
                            .to_string(),
                        default_value: Some("0".to_string()),
                    }],
                    return_type: "Decimal".to_string(),
                    polymorphic: true,
                    min_params: 0,
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
impl PureFunctionEvaluator for RoundFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
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

        // Integer and Long are converted to Decimal; a Quantity keeps its unit.
        let input_decimal = match &input[0] {
            FhirPathValue::Integer(i, _, _) => Decimal::from(*i),
            FhirPathValue::Decimal(d, _, _) => *d,
            FhirPathValue::Quantity { value, .. } => *value,
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "round function can only be called on Integer, Decimal or Quantity values"
                        .to_string(),
                ));
            }
        };

        // Get precision (default to 0)
        let precision = if args.is_empty() {
            0u32
        } else {
            if args[0].len() != 1 {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "round function precision argument must be a single value".to_string(),
                ));
            }

            match &args[0][0] {
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

        // Traditional rounding: halves go away from zero (2.5 -> 3, -0.5 -> -1).
        let rounded =
            input_decimal.round_dp_with_strategy(precision, RoundingStrategy::MidpointAwayFromZero);

        let result = match &input[0] {
            FhirPathValue::Quantity { .. } => {
                let mut quantity = input[0].clone();
                if let FhirPathValue::Quantity { value, .. } = &mut quantity {
                    *value = rounded;
                }
                quantity
            }
            _ => FhirPathValue::decimal(rounded),
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![result]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
