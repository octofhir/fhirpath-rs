//! ConvertsToDecimal function implementation
//!
//! The convertsToDecimal function tests whether a value can be converted to a decimal.
//! Syntax: value.convertsToDecimal()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// ConvertsToDecimal function evaluator
pub struct ConvertsToDecimalFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToDecimalFunctionEvaluator {
    /// Create a new convertsToDecimal function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "convertsToDecimal".to_string(),
                description: "Tests whether a value can be converted to a decimal".to_string(),
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

    /// Check if a value can be converted to decimal
    fn can_convert_to_decimal(&self, value: &FhirPathValue) -> bool {
        match value {
            // Already decimal
            FhirPathValue::Decimal(_, _, _) => true,
            // Integer can be converted to decimal
            FhirPathValue::Integer(_, _, _) => true,
            // String can be converted if it represents a valid decimal
            FhirPathValue::String(s, _, _) => {
                use rust_decimal::Decimal;
                s.parse::<Decimal>().is_ok()
            }
            // Boolean can be converted to decimal (false=0.0, true=1.0)
            FhirPathValue::Boolean(_, _, _) => true,
            // Other types cannot be converted
            _ => false,
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ConvertsToDecimalFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "convertsToDecimal function takes no arguments".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // For single input, return single boolean result
        if input.len() == 1 {
            let can_convert = self.can_convert_to_decimal(&input[0]);
            return Ok(EvaluationResult {
                value: crate::core::Collection::from(vec![FhirPathValue::boolean(can_convert)]),
            });
        }

        // For multiple inputs, check if all can be converted
        let all_convertible = input.iter().all(|v| self.can_convert_to_decimal(v));
        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::boolean(all_convertible)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
