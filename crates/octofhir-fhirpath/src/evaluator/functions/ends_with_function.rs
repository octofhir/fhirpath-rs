//! EndsWith function implementation
//!
//! The endsWith function returns true if the string ends with the given suffix.
//! Syntax: string.endsWith(suffix)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// EndsWith function evaluator
pub struct EndsWithFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl EndsWithFunctionEvaluator {
    /// Create a new endsWith function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "endsWith".to_string(),
                description: "Returns true if the string ends with the given suffix".to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "suffix".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The suffix to check for".to_string(),
                        default_value: None,
                    }],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
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
impl FunctionEvaluator for EndsWithFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("endsWith function expects 1 argument, got {}", args.len()),
            ));
        }

        // Evaluate the suffix argument
        let suffix_result = evaluator.evaluate(&args[0], context).await?;
        let suffix = suffix_result
            .value
            .first()
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "endsWith function requires a string argument".to_string(),
                )
            })?;

        let mut results = Vec::new();

        for value in input {
            if let FhirPathValue::String(s, _, _) = &value {
                let ends_with_result = s.ends_with(&suffix);
                results.push(FhirPathValue::boolean(ends_with_result));
            } else {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "endsWith function can only be applied to strings, got {}",
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
