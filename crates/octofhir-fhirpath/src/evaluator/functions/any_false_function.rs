//! AnyFalse function implementation
//!
//! The anyFalse function takes a collection of Boolean values and returns true if any of the items are false.
//! If all the items are true, or if the input is empty, the result is false.
//! Syntax: collection.anyFalse()

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// AnyFalse function evaluator
pub struct AnyFalseFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AnyFalseFunctionEvaluator {
    /// Create a new anyFalse function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "anyFalse".to_string(),
                description: "Takes a collection of Boolean values and returns true if any of the items are false. If all the items are true, or if the input is empty, the result is false.".to_string(),
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
                empty_propagation: EmptyPropagation::NoPropagation, // Returns false for empty collections
                deterministic: true,
                category: FunctionCategory::Existence,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for AnyFalseFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "anyFalse function takes no arguments".to_string(),
            ));
        }

        // Empty input returns false per spec
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::single(FhirPathValue::boolean(false)),
            });
        }

        // Check if any item in the collection is false
        for item in &input {
            match item {
                FhirPathValue::Boolean(false, _, _) => {
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::single(FhirPathValue::boolean(true)),
                    });
                }
                FhirPathValue::Boolean(true, _, _) => {
                    // Continue checking other items
                }
                _ => {
                    // Skip non-boolean values
                }
            }
        }

        // All items are true or non-boolean
        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(false)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
