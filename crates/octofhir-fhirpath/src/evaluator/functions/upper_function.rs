//! Upper function implementation
//!
//! The upper function converts a string to uppercase.
//! Syntax: string.upper()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Upper function evaluator
pub struct UpperFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl UpperFunctionEvaluator {
    /// Create a new upper function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "upper".to_string(),
                description: "Converts a string to uppercase".to_string(),
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
impl FunctionEvaluator for UpperFunctionEvaluator {
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
                "upper function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            if let FhirPathValue::String(s, type_info, primitive) = &value {
                let upper_string = s.to_uppercase();
                results.push(FhirPathValue::String(
                    upper_string,
                    type_info.clone(),
                    primitive.clone(),
                ));
            } else {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "upper function can only be applied to strings, got {}",
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
