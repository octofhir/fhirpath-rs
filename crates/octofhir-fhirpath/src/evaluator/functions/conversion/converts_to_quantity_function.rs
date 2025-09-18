//! ConvertsToQuantity function implementation
//!
//! This function tests if a value can be converted to a Quantity.

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use std::sync::Arc;

/// ConvertsToQuantity function evaluator
pub struct ConvertsToQuantityFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToQuantityFunctionEvaluator {
    /// Create a new convertsToQuantity function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
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
impl FunctionEvaluator for ConvertsToQuantityFunctionEvaluator {
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
                "convertsToQuantity function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            let can_convert = match &value {
                FhirPathValue::String(s, _, _) => {
                    // Test if string can be parsed as Quantity
                    // A quantity string format is typically: "value unit" or just "value"
                    if let Ok(_) = s.trim().parse::<f64>() {
                        true // Can be converted as a unitless quantity
                    } else {
                        // Check for "value unit" format
                        let parts: Vec<&str> = s.trim().split_whitespace().collect();
                        if parts.len() == 2 {
                            if let Ok(_) = parts[0].parse::<f64>() {
                                true // First part is a valid number
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                }
                FhirPathValue::Integer(_, _, _) => true,  // Numbers can be converted to quantities
                FhirPathValue::Decimal(_, _, _) => true,  // Decimals can be converted to quantities
                FhirPathValue::Quantity { .. } => true, // Already a Quantity
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