//! Ln function implementation
//!
//! The ln function returns the natural logarithm of the input.
//! Syntax: number.ln()

use rust_decimal::prelude::*;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Ln function evaluator
pub struct LnFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl LnFunctionEvaluator {
    /// Create a new ln function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "ln".to_string(),
                description: "Returns the natural logarithm of the input".to_string(),
                signature: FunctionSignature {
                    input_type: "Number".to_string(),
                    parameters: vec![],
                    return_type: "Decimal".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
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
impl FunctionEvaluator for LnFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "ln function takes no arguments".to_string(),
            ));
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "ln function can only be called on a single numeric value".to_string(),
            ));
        }

        let input_float = match &input[0] {
            FhirPathValue::Integer(i, _, _) => {
                if *i <= 0 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "ln function requires a positive input value".to_string(),
                    ));
                }
                *i as f64
            }
            FhirPathValue::Decimal(d, _, _) => {
                if *d <= Decimal::ZERO {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "ln function requires a positive input value".to_string(),
                    ));
                }
                d.to_f64().ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "Decimal value cannot be converted to f64 for ln calculation".to_string(),
                    )
                })?
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "ln function can only be called on numeric values (Integer or Decimal)"
                        .to_string(),
                ));
            }
        };

        let result_float = input_float.ln();

        // Check for invalid results
        if !result_float.is_finite() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "ln result is infinite or not a number".to_string(),
            ));
        }

        let result_decimal = Decimal::from_f64(result_float).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "ln result cannot be represented as Decimal".to_string(),
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
