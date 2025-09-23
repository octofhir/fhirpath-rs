//! ConvertsToQuantity function implementation
//!
//! This function tests if a value can be converted to a Quantity.

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use std::sync::Arc;

/// ConvertsToQuantity function evaluator
pub struct ConvertsToQuantityFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToQuantityFunctionEvaluator {
    /// Create a new convertsToQuantity function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "convertsToQuantity".to_string(),
                description: "Tests if the input can be converted to a Quantity".to_string(),
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
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ConvertsToQuantityFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "convertsToQuantity function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            let can_convert = match &value {
                FhirPathValue::String(s, _, _) => {
                    // Use strict FHIRPath-aware string→Quantity parser
                    crate::evaluator::quantity_utils::parse_string_to_quantity_value(s)
                        .is_some()
                }
                FhirPathValue::Integer(_, _, _) => true, // Numbers can be converted to quantities
                FhirPathValue::Decimal(_, _, _) => true, // Decimals can be converted to quantities
                FhirPathValue::Quantity { .. } => true,  // Already a Quantity
                FhirPathValue::Boolean(_, _, _) => true,  // Booleans are considered convertible per spec
                _ => false, // Other types cannot be converted to Quantity
            };

            results.push(FhirPathValue::boolean(can_convert));
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
