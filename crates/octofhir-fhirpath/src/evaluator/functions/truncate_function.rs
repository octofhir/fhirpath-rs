//! Truncate function implementation
//!
//! The truncate function returns the integer part of the input (truncated towards zero).
//! Syntax: number.truncate()

use rust_decimal::prelude::*;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionEvaluator, PureFunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy,
};use crate::evaluator::EvaluationResult;

/// Truncate function evaluator
pub struct TruncateFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TruncateFunctionEvaluator {
    /// Create a new truncate function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "truncate".to_string(),
                description: "Returns the integer part of the input (truncated towards zero)"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Number".to_string(),
                    parameters: vec![],
                    return_type: "Integer".to_string(),
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
impl PureFunctionEvaluator for TruncateFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "truncate function takes no arguments".to_string(),
            ));
        }

        // Handle empty input (empty propagation)
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "truncate function can only be called on a single numeric value".to_string(),
            ));
        }

        let result =
            match &input[0] {
                FhirPathValue::Integer(i, _, _) => {
                    // Integer already truncated
                    FhirPathValue::integer(*i)
                }
                FhirPathValue::Decimal(d, _, _) => {
                    let truncated_value = d.trunc();
                    // Convert to integer if it fits
                    if let Some(int_value) = truncated_value.to_i64() {
                        FhirPathValue::integer(int_value)
                    } else {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0058,
                            "truncate result is too large to represent as integer".to_string(),
                        ));
                    }
                }
                _ => return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "truncate function can only be called on numeric values (Integer or Decimal)"
                        .to_string(),
                )),
            };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![result]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}