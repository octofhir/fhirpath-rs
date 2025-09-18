//! Trim function implementation
//!
//! The trim function removes whitespace from the beginning and end of strings.
//! Syntax: string.trim()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Trim function evaluator
pub struct TrimFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TrimFunctionEvaluator {
    /// Create a new trim function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "trim".to_string(),
                description: "Removes whitespace from the beginning and end of a string"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![],
                    return_type: "String".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::StringManipulation,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for TrimFunctionEvaluator {
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
                "trim function takes no arguments".to_string(),
            ));
        }

        // Handle empty input (empty propagation)
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "trim function can only be called on a single string value".to_string(),
            ));
        }

        // Get the input string
        let input_str = match &input[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "trim function can only be called on string values".to_string(),
                ));
            }
        };

        // Trim whitespace
        let trimmed = input_str.trim().to_string();

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::string(trimmed)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
