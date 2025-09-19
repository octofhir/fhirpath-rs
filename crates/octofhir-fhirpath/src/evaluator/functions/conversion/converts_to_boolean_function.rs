//! ConvertsToBoolean function implementation
//!
//! The convertsToBoolean function tests whether a value can be converted to a boolean.
//! Syntax: value.convertsToBoolean()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// ConvertsToBoolean function evaluator
pub struct ConvertsToBooleanFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToBooleanFunctionEvaluator {
    /// Create a new convertsToBoolean function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "convertsToBoolean".to_string(),
                description: "Tests whether a value can be converted to a boolean".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Conversion,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Check if a value can be converted to boolean
    fn can_convert_to_boolean(&self, value: &FhirPathValue) -> bool {
        match value {
            // Already boolean
            FhirPathValue::Boolean(_, _, _) => true,
            // String can be converted if it's "true" or "false" (case insensitive)
            FhirPathValue::String(s, _, _) => {
                let lowercase = s.to_lowercase();
                lowercase == "true" || lowercase == "false"
            }
            // Integer can be converted (0=false, 1=true, others false)
            FhirPathValue::Integer(i, _, _) => *i == 0 || *i == 1,
            // Decimal can be converted (0.0=false, 1.0=true, others false)
            FhirPathValue::Decimal(d, _, _) => {
                use rust_decimal::Decimal;
                *d == Decimal::ZERO || *d == Decimal::ONE
            }
            // Other types cannot be converted
            _ => false,
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ConvertsToBooleanFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "convertsToBoolean function takes no arguments".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // For single input, return single boolean result
        if input.len() == 1 {
            let can_convert = self.can_convert_to_boolean(&input[0]);
            return Ok(EvaluationResult {
                value: crate::core::Collection::from(vec![FhirPathValue::boolean(can_convert)]),
            });
        }

        // For multiple inputs, check if all can be converted
        let all_convertible = input.iter().all(|v| self.can_convert_to_boolean(v));
        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::boolean(all_convertible)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
