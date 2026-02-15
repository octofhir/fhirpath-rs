//! elementDefinition function implementation
//!
//! The elementDefinition function returns the ElementDefinition for the current element
//! from the FHIR model provider.
//! Syntax: element.elementDefinition()

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// ElementDefinition function evaluator
pub struct ElementDefinitionFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ElementDefinitionFunctionEvaluator {
    /// Create a new elementDefinition function evaluator
    pub fn create() -> Arc<dyn ProviderPureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "elementDefinition".to_string(),
                description:
                    "Returns the ElementDefinition for the current element from the FHIR model"
                        .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: true,
            },
        })
    }
}

#[async_trait::async_trait]
impl ProviderPureFunctionEvaluator for ElementDefinitionFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "elementDefinition function takes no arguments".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        let item = &input[0];
        let type_info = item.type_info();
        let model_provider = context.model_provider();

        // Get the type name from the element
        let type_name = type_info.name.as_deref().unwrap_or(&type_info.type_name);

        // Use model provider to get element info
        if let Ok(elements) = model_provider.get_elements(type_name).await
            && !elements.is_empty()
        {
            // Build a Resource value representing the element definition
            let mut result_obj = serde_json::Map::new();
            result_obj.insert(
                "path".to_string(),
                serde_json::Value::String(type_name.to_string()),
            );

            // Include element info from the first element
            if let Some(first) = elements.first() {
                result_obj.insert(
                    "type".to_string(),
                    serde_json::json!([{"code": first.element_type}]),
                );
                if let Some(ref doc) = first.documentation {
                    result_obj.insert("short".to_string(), serde_json::Value::String(doc.clone()));
                }
            }

            let type_info = crate::core::model_provider::TypeInfo {
                type_name: "ElementDefinition".to_string(),
                name: Some("ElementDefinition".to_string()),
                is_empty: Some(false),
                namespace: Some("FHIR".to_string()),
                singleton: Some(true),
            };

            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::Resource(
                    Arc::new(serde_json::Value::Object(result_obj)),
                    type_info,
                    None,
                )),
            });
        }

        Ok(EvaluationResult {
            value: Collection::empty(),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
