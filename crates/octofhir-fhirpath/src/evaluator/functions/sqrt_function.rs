//! Sqrt function implementation
//!
//! The sqrt function returns the square root of the input.
//! Syntax: number.sqrt()

use rust_decimal::prelude::*;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionEvaluator, PureFunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy,
};use crate::evaluator::EvaluationResult;

/// Sqrt function evaluator
pub struct SqrtFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SqrtFunctionEvaluator {
    /// Create a new sqrt function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "sqrt".to_string(),
                description: "Returns the square root of the input".to_string(),
                signature: FunctionSignature {
                    input_type: "Number".to_string(),
                    parameters: vec![],
                    return_type: "Decimal".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
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
impl PureFunctionEvaluator for SqrtFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "sqrt function takes no arguments".to_string(),
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
                "sqrt function can only be called on a single numeric value".to_string(),
            ));
        }

        let input_float = match &input[0] {
            FhirPathValue::Integer(i, _, _) => {
                if *i < 0 {
                    // Per FHIRPath spec, sqrt of negative number returns empty collection
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::empty(),
                    });
                }
                *i as f64
            }
            FhirPathValue::Decimal(d, _, _) => {
                if *d < Decimal::ZERO {
                    // Per FHIRPath spec, sqrt of negative number returns empty collection
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::empty(),
                    });
                }
                d.to_f64().ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "Decimal value cannot be converted to f64 for sqrt calculation".to_string(),
                    )
                })?
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "sqrt function can only be called on numeric values (Integer or Decimal)"
                        .to_string(),
                ));
            }
        };

        let result_float = input_float.sqrt();

        // Check for invalid results
        if !result_float.is_finite() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "sqrt result is infinite or not a number".to_string(),
            ));
        }

        let result_decimal = Decimal::from_f64(result_float).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "sqrt result cannot be represented as Decimal".to_string(),
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