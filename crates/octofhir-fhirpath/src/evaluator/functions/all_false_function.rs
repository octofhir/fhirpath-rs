//! AllFalse function implementation
//!
//! The allFalse function takes a collection of Boolean values and returns true if all the items are false.
//! If any items are true, the result is false. If the input is empty, the result is true.
//! Syntax: collection.allFalse()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// AllFalse function evaluator
pub struct AllFalseFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AllFalseFunctionEvaluator {
    /// Create a new allFalse function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "allFalse".to_string(),
                description: "Takes a collection of Boolean values and returns true if all the items are false. If any items are true, the result is false. If the input is empty, the result is true.".to_string(),
                signature: FunctionSignature {
                    input_type: "Boolean".to_string(),
                    parameters: vec![],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::None,
                empty_propagation: EmptyPropagation::NoPropagation, // Returns true for empty collections
                deterministic: true,
                category: FunctionCategory::Existence,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for AllFalseFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "allFalse function takes no arguments".to_string(),
            ));
        }

        // If the input is empty, the result is true
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // Check all items: return false if any item is true
        for item in &input {
            match item {
                FhirPathValue::Boolean(true, _, _) => {
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::single(FhirPathValue::boolean(false)),
                    });
                }
                FhirPathValue::Boolean(false, _, _) => {
                    // Continue checking other items
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0001,
                        format!(
                            "allFalse function can only be applied to Boolean values, found: {}",
                            item.type_name()
                        ),
                    ));
                }
            }
        }

        // All items are boolean false
        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(true)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
