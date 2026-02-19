//! at function implementation for %terminologies
//!
//! The at function creates a terminology service connection to a specific server URL.
//! Syntax: %terminologies.at(url)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// At function evaluator for %terminologies.at(url)
pub struct AtFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AtFunctionEvaluator {
    /// Create a new at function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "at".to_string(),
                description: "Creates a terminology service connection to a specific server URL"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "url".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The URL of the terminology server".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Terminology,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for AtFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "at function requires exactly one argument (url)".to_string(),
            ));
        }

        let url = match args[0].first() {
            Some(FhirPathValue::String(s, _, _)) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "at function argument must be a string URL".to_string(),
                ));
            }
        };

        // Verify input is a terminologies variable
        let is_terminologies = input.len() == 1
            && crate::evaluator::terminologies_variable::is_terminologies_variable(&input[0]);

        if !is_terminologies {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "at function can only be called on %terminologies variable".to_string(),
            ));
        }

        // Create a new terminologies-like variable with the custom URL stored
        let mut terminologies_object = serde_json::Map::new();
        terminologies_object.insert(
            "resourceType".to_string(),
            serde_json::Value::String("TerminologiesVariable".to_string()),
        );
        terminologies_object.insert(
            "_terminologyProvider".to_string(),
            serde_json::Value::String("internal".to_string()),
        );
        terminologies_object.insert("_serverUrl".to_string(), serde_json::Value::String(url));

        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource(serde_json::Value::Object(
                terminologies_object,
            ))),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
