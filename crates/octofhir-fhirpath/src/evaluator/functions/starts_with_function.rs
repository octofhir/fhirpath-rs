//! StartsWith function implementation
//!
//! The startsWith function returns true if the string starts with the given prefix.
//! Syntax: string.startsWith(prefix)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// StartsWith function evaluator
pub struct StartsWithFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl StartsWithFunctionEvaluator {
    /// Create a new startsWith function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "startsWith".to_string(),
                description: "Returns true if the string starts with the given prefix".to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "prefix".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The prefix to check for".to_string(),
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
impl FunctionEvaluator for StartsWithFunctionEvaluator {
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
                format!("startsWith function expects 1 argument, got {}", args.len()),
            ));
        }

        // Evaluate the prefix argument
        let prefix_result = evaluator.evaluate(&args[0], context).await?;
        let prefix = prefix_result
            .value
            .first()
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "startsWith function requires a string argument".to_string(),
                )
            })?;

        let mut results = Vec::new();

        for value in input {
            if let FhirPathValue::String(s, _, _) = &value {
                let starts_with_result = s.starts_with(&prefix);
                results.push(FhirPathValue::boolean(starts_with_result));
            } else {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "startsWith function can only be applied to strings, got {}",
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
