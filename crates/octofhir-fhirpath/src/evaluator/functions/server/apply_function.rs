//! %server.apply(resource, subject, parameters) function
//!
//! Syntax: %server.apply(resource, subject, parameters)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};
use crate::evaluator::server_variable::is_server_variable;
use crate::evaluator::{EvaluationContext, EvaluationResult};

pub struct ServerApplyFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ServerApplyFunctionEvaluator {
    pub fn create() -> Arc<dyn ProviderPureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "apply".to_string(),
                description: "Execute the $apply operation on PlanDefinition/ActivityDefinition"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "resource".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "PlanDefinition or ActivityDefinition".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "subject".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Subject reference (resource or type/id string)"
                                .to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "parameters".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Additional parameters".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 3,
                    max_params: Some(3),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: false,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl ProviderPureFunctionEvaluator for ServerApplyFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        args: Vec<Collection>,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_server_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "apply function can only be called on %server variable".to_string(),
            ));
        }

        let resource = match args.first().and_then(|a| a.first()) {
            Some(FhirPathValue::Resource(json, _, _)) => json.as_ref().clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "apply requires a resource argument".to_string(),
                ));
            }
        };

        let subject = match args.get(1).and_then(|a| a.first()) {
            Some(FhirPathValue::String(s, _, _)) => s.clone(),
            Some(FhirPathValue::Resource(json, _, _)) => {
                // Extract type/id from resource
                let rt = json
                    .get("resourceType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let id = json.get("id").and_then(|v| v.as_str()).unwrap_or("");
                format!("{rt}/{id}")
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "apply requires a subject argument".to_string(),
                ));
            }
        };

        let parameters = match args.get(2).and_then(|a| a.first()) {
            Some(FhirPathValue::Resource(json, _, _)) => json.as_ref().clone(),
            Some(FhirPathValue::String(s, _, _)) => serde_json::Value::String(s.clone()),
            _ => serde_json::Value::Null,
        };

        let server_provider =
            crate::evaluator::server_variable::extract_server_provider(&input[0], context)
                .ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "No server provider configured".to_string(),
                    )
                })?;

        match server_provider
            .apply(&resource, &subject, &parameters)
            .await
        {
            Ok(Some(json)) => Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::resource(json)),
            }),
            _ => Ok(EvaluationResult {
                value: Collection::empty(),
            }),
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
