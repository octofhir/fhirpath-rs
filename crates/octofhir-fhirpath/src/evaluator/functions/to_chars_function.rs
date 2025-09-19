//! toChars function implementation
//!
//! The toChars function converts a string into a collection containing each
//! character of the original string as a single-character string value.

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionEvaluator, PureFunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy,
};use crate::evaluator::EvaluationResult;

/// toChars function evaluator
pub struct ToCharsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToCharsFunctionEvaluator {
    /// Create a new toChars function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toChars".to_string(),
                description: "Converts a string into a collection of single-character strings"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![],
                    return_type: "Collection".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
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
impl PureFunctionEvaluator for ToCharsFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("toChars function expects no arguments, got {}", _args.len()),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "toChars function can only be invoked on a single string value".to_string(),
            ));
        }

        let value = input.into_iter().next().unwrap();
        let string_value = match value {
            FhirPathValue::String(s, _, _) => s,
            other => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "toChars function can only be applied to string values, got {}",
                        other.type_name()
                    ),
                ));
            }
        };

        let characters: Vec<FhirPathValue> = string_value
            .chars()
            .map(|ch| FhirPathValue::string(ch.to_string()))
            .collect();

        Ok(EvaluationResult {
            value: Collection::from(characters),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}