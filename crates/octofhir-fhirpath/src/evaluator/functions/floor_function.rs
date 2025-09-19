//! Floor function implementation
//!
//! The floor function returns the largest integer less than or equal to the input.
//! Syntax: number.floor()

use rust_decimal::prelude::*;
use std::sync::Arc;

use crate::Collection;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Floor function evaluator
pub struct FloorFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FloorFunctionEvaluator {
    /// Create a new floor function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "floor".to_string(),
                description: "Returns the largest integer less than or equal to the input"
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
impl PureFunctionEvaluator for FloorFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "floor function takes no arguments".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "floor function can only be called on a single numeric value".to_string(),
            ));
        }

        let result = match &input[0] {
            FhirPathValue::Integer(i, _, _) => {
                // Integer already at floor
                FhirPathValue::integer(*i)
            }
            FhirPathValue::Decimal(d, _, _) => {
                let floor_value = d.floor();
                // Convert to integer if it fits
                if let Some(int_value) = floor_value.to_i64() {
                    FhirPathValue::integer(int_value)
                } else {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "floor result is too large to represent as integer".to_string(),
                    ));
                }
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "floor function can only be called on numeric values (Integer or Decimal)"
                        .to_string(),
                ));
            }
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![result]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
