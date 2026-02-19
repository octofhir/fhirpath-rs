//! %server.capabilities(mode) function — get server capabilities
//!
//! Syntax: %server.capabilities(mode)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};
use crate::evaluator::server_variable::is_server_variable;
use crate::evaluator::{EvaluationContext, EvaluationResult};

pub struct ServerCapabilitiesFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ServerCapabilitiesFunctionEvaluator {
    pub fn create() -> Arc<dyn ProviderPureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "capabilities".to_string(),
                description: "Retrieve server capabilities".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "mode".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: true,
                        is_expression: false,
                        description: "The mode to fetch capabilities for".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 0,
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
impl ProviderPureFunctionEvaluator for ServerCapabilitiesFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        args: Vec<Collection>,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_server_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "capabilities function can only be called on %server variable".to_string(),
            ));
        }

        let mode = args.first().and_then(|a| a.first()).and_then(|v| match v {
            FhirPathValue::String(s, _, _) => Some(s.as_str()),
            _ => None,
        });

        let server_provider =
            crate::evaluator::server_variable::extract_server_provider(&input[0], context)
                .ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "No server provider configured".to_string(),
                    )
                })?;

        // Need to work around borrow issues with the mode reference
        let mode_owned = mode.map(|s| s.to_string());
        match server_provider.capabilities(mode_owned.as_deref()).await {
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
