//! %server.at(url) function — get a server reference pointing at a particular endpoint
//!
//! Syntax: %server.at(url)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use crate::evaluator::server_variable::is_server_variable;

pub struct ServerAtFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ServerAtFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "at".to_string(),
                description: "Get a server reference pointing at a particular FHIR endpoint"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "url".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "URL of the FHIR server".to_string(),
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
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ServerAtFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        let url = match args.first().and_then(|a| a.first()) {
            Some(FhirPathValue::String(s, _, _)) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "at function requires a string url argument".to_string(),
                ));
            }
        };

        // Check if called on %server or %terminologies
        let is_server = input.len() == 1 && is_server_variable(&input[0]);
        let is_terminologies = input.len() == 1
            && crate::evaluator::terminologies_variable::is_terminologies_variable(&input[0]);

        if is_server {
            // Return a new server variable with custom URL
            let mut server_object = serde_json::Map::new();
            server_object.insert(
                "resourceType".to_string(),
                serde_json::Value::String("ServerVariable".to_string()),
            );
            server_object.insert(
                "_serverProvider".to_string(),
                serde_json::Value::String("internal".to_string()),
            );
            server_object.insert("_baseUrl".to_string(), serde_json::Value::String(url));

            Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::resource(serde_json::Value::Object(
                    server_object,
                ))),
            })
        } else if is_terminologies {
            // Delegate to the existing terminologies at behavior
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
        } else {
            Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "at function can only be called on %server or %terminologies variable".to_string(),
            ))
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
