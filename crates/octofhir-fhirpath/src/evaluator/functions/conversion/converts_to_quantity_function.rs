//! ConvertsToQuantity function implementation
//!
//! This function tests if a value can be converted to a Quantity.

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
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
                    parameters: vec![FunctionParameter {
                        name: "unit".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: true,
                        is_expression: false,
                        description:
                            "UCUM code or calendar duration keyword the value must convert to"
                                .to_string(),
                        default_value: None,
                    }],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(1),
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
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        let unit = super::to_quantity_function::unit_argument(&args, "convertsToQuantity")?;
        let value = match input.len() {
            0 => None,
            1 => Some(FhirPathValue::boolean(
                super::to_quantity_function::to_quantity(&input[0], unit.as_deref()).is_some(),
            )),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0053,
                    "convertsToQuantity requires a single input item".to_string(),
                ));
            }
        };
        Ok(EvaluationResult {
            value: value
                .map(Collection::single)
                .unwrap_or_else(Collection::empty),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
