//! Lower function implementation
//!
//! The lower function converts a string to lowercase.
//! Syntax: string.lower()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Lower function evaluator
pub struct LowerFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl LowerFunctionEvaluator {
    /// Create a new lower function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "lower".to_string(),
                description: "Converts a string to lowercase".to_string(),
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
impl FunctionEvaluator for LowerFunctionEvaluator {
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
                "lower function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            if let FhirPathValue::String(s, type_info, primitive) = &value {
                let lower_string = s.to_lowercase();
                results.push(FhirPathValue::String(
                    lower_string,
                    type_info.clone(),
                    primitive.clone(),
                ));
            } else {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "lower function can only be applied to strings, got {}",
                        value.type_name()
                    ),
                ));
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
