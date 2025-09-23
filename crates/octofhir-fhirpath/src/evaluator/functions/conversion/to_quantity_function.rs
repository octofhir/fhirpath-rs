//! ToQuantity function implementation
//!
//! The toQuantity function converts a value to a quantity.
//! Syntax: value.toQuantity()

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use rust_decimal::Decimal;
use std::sync::Arc;

pub struct ToQuantityFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToQuantityFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toQuantity".to_string(),
                description: "Converts a value to a quantity".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Quantity".to_string(),
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
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ToQuantityFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "toQuantity function takes no arguments".to_string(),
            ));
        }

        if input.len() != 1 {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let result = match &input[0] {
            FhirPathValue::Quantity { .. } => {
                // Already a quantity, return as-is
                Some(input[0].clone())
            }
            FhirPathValue::Integer(i, _, _) => {
                // Convert integer to dimensionless quantity with unit '1'
                Some(FhirPathValue::quantity(Decimal::from(*i), Some("1".to_string())))
            }
            FhirPathValue::Decimal(d, _, _) => {
                // Convert decimal to dimensionless quantity with unit '1'
                Some(FhirPathValue::quantity(*d, Some("1".to_string())))
            }
            FhirPathValue::String(s, _, _) => {
                // Use strict FHIRPath-aware parser for string quantities
                crate::evaluator::quantity_utils::parse_string_to_quantity_value(s)
            }
            _ => {
                // Other types cannot be converted to quantity
                None
            }
        };

        Ok(EvaluationResult {
            value: match result {
                Some(quantity_value) => crate::core::Collection::from(vec![quantity_value]),
                None => crate::core::Collection::empty(),
            },
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
