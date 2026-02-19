//! %server.update(resource) function — update a resource on the server
//!
//! Syntax: %server.update(resource)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};
use crate::evaluator::server_variable::is_server_variable;
use crate::evaluator::{EvaluationContext, EvaluationResult};

pub struct ServerUpdateFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ServerUpdateFunctionEvaluator {
    pub fn create() -> Arc<dyn ProviderPureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "serverUpdate".to_string(),
                description: "Update an existing resource on the FHIR server".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "resource".to_string(),
                        parameter_type: vec!["Any".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The resource to update (must have an id)".to_string(),
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
                deterministic: false,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl ProviderPureFunctionEvaluator for ServerUpdateFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        args: Vec<Collection>,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_server_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "serverUpdate function can only be called on %server variable".to_string(),
            ));
        }

        let resource_json = match args.first().and_then(|a| a.first()) {
            Some(FhirPathValue::Resource(json, _, _)) => json.as_ref().clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "serverUpdate requires a resource argument".to_string(),
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

        match server_provider.update(&resource_json).await {
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
