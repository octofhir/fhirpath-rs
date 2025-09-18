//! ToBoolean function implementation
//!
//! The toBoolean function converts a value to a boolean.
//! Syntax: value.toBoolean()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// ToBoolean function evaluator
pub struct ToBooleanFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToBooleanFunctionEvaluator {
    /// Create a new toBoolean function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toBoolean".to_string(),
                description: "Converts a value to a boolean".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
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
impl FunctionEvaluator for ToBooleanFunctionEvaluator {
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
                "toBoolean function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            let boolean_result = match &value {
                FhirPathValue::Boolean(b, _, _) => *b,
                FhirPathValue::Integer(i, _, _) => *i != 0,
                FhirPathValue::Decimal(d, _, _) => !d.is_zero(),
                FhirPathValue::String(s, _, _) => match s.trim().to_lowercase().as_str() {
                    "true" | "t" | "yes" | "y" | "1" => true,
                    "false" | "f" | "no" | "n" | "0" => false,
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0055,
                            format!("Cannot convert '{}' to boolean", s),
                        ));
                    }
                },
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        format!("Cannot convert {} to boolean", value.type_name()),
                    ));
                }
            };

            results.push(FhirPathValue::boolean(boolean_result));
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
