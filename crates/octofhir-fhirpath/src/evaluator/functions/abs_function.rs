//! Abs function implementation
//!
//! The abs function returns the absolute value of a numeric value.
//! Syntax: number.abs()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Abs function evaluator
pub struct AbsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AbsFunctionEvaluator {
    /// Create a new abs function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "abs".to_string(),
                description: "Returns the absolute value of a numeric value".to_string(),
                signature: FunctionSignature {
                    input_type: "Number | Quantity".to_string(),
                    parameters: vec![],
                    return_type: "Number | Quantity".to_string(),
                    polymorphic: true,
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
impl FunctionEvaluator for AbsFunctionEvaluator {
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
                "abs function takes no arguments".to_string(),
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
                "abs function can only be called on a single numeric value".to_string(),
            ));
        }

        let result = match &input[0] {
            FhirPathValue::Integer(i, _, _) => FhirPathValue::integer(i.abs()),
            FhirPathValue::Decimal(d, _, _) => FhirPathValue::decimal(d.abs()),
            FhirPathValue::Quantity { value, unit, .. } => {
                // Support absolute value for quantities
                FhirPathValue::quantity(value.abs(), unit.clone())
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "abs function can only be called on numeric values (Integer, Decimal, or Quantity)"
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
