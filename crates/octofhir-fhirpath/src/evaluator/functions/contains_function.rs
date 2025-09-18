//! Contains function implementation
//!
//! The contains function returns true if the string contains the given substring.
//! Syntax: string.contains(substring)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Contains function evaluator
pub struct ContainsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ContainsFunctionEvaluator {
    /// Create a new contains function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "contains".to_string(),
                description: "Returns true if the string contains the given substring".to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "substring".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The substring to search for".to_string(),
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
impl FunctionEvaluator for ContainsFunctionEvaluator {
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
                format!("contains function expects 1 argument, got {}", args.len()),
            ));
        }

        // Evaluate the substring argument
        let substring_result = evaluator.evaluate(&args[0], context).await?;
        let substring = substring_result
            .value
            .first()
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "contains function requires a string argument".to_string(),
                )
            })?;

        let mut results = Vec::new();

        for value in input {
            if let FhirPathValue::String(s, _, _) = &value {
                let contains_result = s.contains(&substring);
                results.push(FhirPathValue::boolean(contains_result));
            } else {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "contains function can only be applied to strings, got {}",
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
