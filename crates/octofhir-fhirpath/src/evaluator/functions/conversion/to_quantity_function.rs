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
                // Convert integer to unitless quantity
                Some(FhirPathValue::quantity(Decimal::from(*i), None))
            }
            FhirPathValue::Decimal(d, _, _) => {
                // Convert decimal to unitless quantity
                Some(FhirPathValue::quantity(*d, None))
            }
            FhirPathValue::String(s, _, _) => {
                // Try to parse string as quantity
                let trimmed = s.trim();

                // First try parsing as a simple number (unitless quantity)
                if let Ok(value) = trimmed.parse::<f64>() {
                    Some(FhirPathValue::quantity(
                        Decimal::from_f64_retain(value).unwrap_or_default(),
                        None,
                    ))
                } else {
                    // Try parsing as "value unit" format
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() == 2 {
                        if let Ok(value) = parts[0].parse::<f64>() {
                            let unit = parts[1].to_string();
                            Some(FhirPathValue::quantity(
                                Decimal::from_f64_retain(value).unwrap_or_default(),
                                Some(unit),
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
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
