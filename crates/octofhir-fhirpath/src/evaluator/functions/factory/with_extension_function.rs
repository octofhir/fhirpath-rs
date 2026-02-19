//! %factory.withExtension(instance, url, value) function implementation
//!
//! Adds an extension to an existing FHIR type instance (immutable, returns new instance).
//! Syntax: %factory.withExtension(instance, url, value)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

use super::extension_function::add_extension_value;

pub struct FactoryWithExtensionFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryWithExtensionFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "withExtension".to_string(),
                description: "Adds an extension to an existing FHIR instance (returns new copy)"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "instance".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "The instance to add the extension to".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "url".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Extension URL".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "value".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Extension value".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 2,
                    max_params: Some(3),
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
impl PureFunctionEvaluator for FactoryWithExtensionFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "withExtension function can only be called on %factory variable".to_string(),
            ));
        }

        // First arg: instance to modify
        let instance = args.first().and_then(|a| a.first()).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "withExtension requires an instance argument".to_string(),
            )
        })?;

        // Second arg: url
        let url = match args.get(1).and_then(|a| a.first()) {
            Some(FhirPathValue::String(s, _, _)) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "withExtension requires a string url argument".to_string(),
                ));
            }
        };

        // Clone the instance and add the extension
        match instance {
            FhirPathValue::Resource(json, type_info, _) => {
                let mut new_json = json.as_ref().clone();

                // Build the extension object
                let mut ext = serde_json::Map::new();
                ext.insert("url".to_string(), serde_json::Value::String(url));

                // Optional value argument
                if let Some(value_arg) = args.get(2).and_then(|a| a.first()) {
                    add_extension_value(&mut ext, value_arg);
                }

                // Add to extension array
                if let Some(obj) = new_json.as_object_mut() {
                    let extensions = obj
                        .entry("extension")
                        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
                    if let Some(arr) = extensions.as_array_mut() {
                        arr.push(serde_json::Value::Object(ext));
                    }
                }

                Ok(EvaluationResult {
                    value: Collection::single(FhirPathValue::resource_wrapped(
                        new_json,
                        type_info.clone(),
                    )),
                })
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "withExtension instance must be a FHIR complex type".to_string(),
            )),
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
