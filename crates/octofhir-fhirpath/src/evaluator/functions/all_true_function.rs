//! AllTrue function implementation
//!
//! The allTrue function takes a collection of Boolean values and returns true if all the items are true.
//! If any items are false, the result is false. If the input is empty, the result is true.
//! Syntax: collection.allTrue()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// AllTrue function evaluator
pub struct AllTrueFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AllTrueFunctionEvaluator {
    /// Create a new allTrue function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
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
impl FunctionEvaluator for AllTrueFunctionEvaluator {
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
                    // Non-boolean values result in false
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::single(FhirPathValue::boolean(false)),
                    });
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
