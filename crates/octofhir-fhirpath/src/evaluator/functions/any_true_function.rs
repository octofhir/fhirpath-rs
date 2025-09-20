//! AnyTrue function implementation
//!
//! The anyTrue function takes a collection of Boolean values and returns true if any of the items are true.
//! If all the items are false, or if the input is empty, the result is false.
//! Syntax: collection.anyTrue()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// AnyTrue function evaluator
pub struct AnyTrueFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AnyTrueFunctionEvaluator {
    /// Create a new anyTrue function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "anyTrue".to_string(),
                description: "Takes a collection of Boolean values and returns true if any of the items are true. If all the items are false, or if the input is empty, the result is false.".to_string(),
                signature: FunctionSignature {
                    input_type: "Boolean".to_string(),
                    parameters: vec![],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
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
impl PureFunctionEvaluator for AnyTrueFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "anyTrue function takes no arguments".to_string(),
            ));
        }

        // Empty input returns false per spec
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::single(FhirPathValue::boolean(false)),
            });
        }

        // Check if any item in the collection is true
        for item in &input {
            match item {
                FhirPathValue::Boolean(true, _, _) => {
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::single(FhirPathValue::boolean(true)),
                    });
                }
                FhirPathValue::Boolean(false, _, _) => {
                    // Continue checking other items
                }
                _ => {
                    // Skip non-boolean values
                }
            }
        }

        // All items are false or non-boolean
        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(false)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
