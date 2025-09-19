//! Split function implementation
//!
//! The split function splits a string into a collection of strings based on a separator.
//! Syntax: string.split(separator)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Split function evaluator
pub struct SplitFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SplitFunctionEvaluator {
    /// Create a new split function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "split".to_string(),
                description: "Splits a string into a collection of strings based on a separator"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "separator".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "String separator to split on".to_string(),
                        default_value: None,
                    }],
                    return_type: "Collection".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
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
impl PureFunctionEvaluator for SplitFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "split function requires exactly one argument (separator)".to_string(),
            ));
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "split function can only be called on a single string value".to_string(),
            ));
        }

        // Get the input string
        let input_str = match &input[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "split function can only be called on string values".to_string(),
                ));
            }
        };

        // Get the pre-evaluated separator argument
        if args[0].len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "split function separator argument must be a single value".to_string(),
            ));
        }

        let separator_str = match &args[0][0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "split function separator argument must be a string".to_string(),
                ));
            }
        };

        // Split the string
        let parts: Vec<FhirPathValue> = if separator_str.is_empty() {
            // If separator is empty, split into individual characters
            input_str
                .chars()
                .map(|c| FhirPathValue::string(c.to_string()))
                .collect()
        } else {
            // Split by separator
            input_str
                .split(&separator_str)
                .map(|s| FhirPathValue::string(s.to_string()))
                .collect()
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(parts),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
