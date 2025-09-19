//! AllTrue function implementation
//!
//! The allTrue function takes a collection of Boolean values and returns true if all the items are true.
//! If any items are false, the result is false. If the input is empty, the result is true.
//! Syntax: collection.allTrue()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};use crate::evaluator::EvaluationResult;

/// AllTrue function evaluator
pub struct AllTrueFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AllTrueFunctionEvaluator {
    /// Create a new allTrue function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "allTrue".to_string(),
                description: "Takes a collection of Boolean values and returns true if all the items are true. If any items are false, the result is false. If the input is empty, the result is true.".to_string(),
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
impl PureFunctionEvaluator for AllTrueFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "allTrue function takes no arguments".to_string(),
            ));
        }

        // If the input is empty, the result is true
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // Check all items: return false if any item is non-boolean or false
        // Return true only if ALL items are boolean true
        for item in &input {
            match item {
                FhirPathValue::Boolean(false, _, _) => {
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::single(FhirPathValue::boolean(false)),
                    });
                }
                FhirPathValue::Boolean(true, _, _) => {
                    // Continue checking other items
                }
                _ => {
                    // Non-boolean values should cause an execution error
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0001,
                        format!("allTrue function can only be applied to Boolean values, found: {}", item.type_name()),
                    ));
                }
            }
        }

        // All items are boolean true
        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(true)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
