//! Length function implementation
//!
//! The length function returns the length of a string.
//! Syntax: string.length()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Length function evaluator
pub struct LengthFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl LengthFunctionEvaluator {
    /// Create a new length function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "length".to_string(),
                description: "Returns the length of a string".to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![],
                    return_type: "Integer".to_string(),
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
impl FunctionEvaluator for LengthFunctionEvaluator {
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
                "length function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            match &value {
                FhirPathValue::String(s, _, _) => {
                    results.push(FhirPathValue::integer(s.chars().count() as i64));
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        "length function can only be called on strings".to_string(),
                    ));
                }
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
