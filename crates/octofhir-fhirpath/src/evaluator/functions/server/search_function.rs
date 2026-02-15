//! %server.search(doPost, parameters) function — search for resources
//!
//! Syntax: %server.search(doPost, parameters)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};
use crate::evaluator::server_variable::is_server_variable;
use crate::evaluator::{EvaluationContext, EvaluationResult};

pub struct ServerSearchFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ServerSearchFunctionEvaluator {
    pub fn create() -> Arc<dyn ProviderPureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "serverSearch".to_string(),
                description: "Search for resources on the FHIR server".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "doPost".to_string(),
                            parameter_type: vec!["Boolean".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Use POST-based search if true".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "parameters".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Parameters resource or URL parameter string".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 2,
                    max_params: Some(2),
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
impl ProviderPureFunctionEvaluator for ServerSearchFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_server_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "serverSearch function can only be called on %server variable".to_string(),
            ));
        }

        let do_post = match args.first().and_then(|a| a.first()) {
            Some(FhirPathValue::Boolean(b, _, _)) => *b,
            _ => false,
        };

        let parameters = match args.get(1).and_then(|a| a.first()) {
            Some(FhirPathValue::Resource(json, _, _)) => json.as_ref().clone(),
            Some(FhirPathValue::String(s, _, _)) => serde_json::Value::String(s.clone()),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "serverSearch requires parameters argument".to_string(),
                ));
            }
        };

        let server_provider =
            crate::evaluator::server_variable::extract_server_provider(&input[0], context)
                .ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "No server provider configured".to_string(),
                    )
                })?;

        match server_provider.search(do_post, &parameters).await {
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
