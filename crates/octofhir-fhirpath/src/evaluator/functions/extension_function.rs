//! Extension function implementation
//!
//! The extension function filters extensions by URL.
//! Syntax: element.extension(url)

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Extension function evaluator for filtering extensions by URL
pub struct ExtensionFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl Default for ExtensionFunctionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionFunctionEvaluator {
    /// Create a new extension function evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self::new())
    }
}

#[async_trait]
impl PureFunctionEvaluator for ExtensionFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        // Check argument count
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "extension function requires exactly one argument (URL)",
            ));
        }

        // Get the URL argument (pre-evaluated)
        let url_values = &args[0];

        if url_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "extension function URL argument must evaluate to a single value",
            ));
        }

        let target_url = match &url_values[0] {
            FhirPathValue::String(url, _, _) => url.as_str(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "extension function URL argument must be a string",
                ));
            }
        };

        // Process input collection to find matching extensions
        let mut matching_extensions = Vec::new();

        for item in input {
            // Handle complex types/resources with direct extension arrays
            if let FhirPathValue::Resource(json, _, _) = &item {
                if let Some(extensions) = json.get("extension").and_then(|e| e.as_array()) {
                    for extension in extensions {
                        if let Some(url_str) = extension.get("url").and_then(|u| u.as_str())
                            && url_str == target_url
                        {
                            let type_info = TypeInfo {
                                type_name: "Extension".to_string(),
                                name: Some("Extension".to_string()),
                                is_empty: Some(false),
                                namespace: Some("FHIR".to_string()),
                                singleton: Some(true),
                            };
                            let extension_value = FhirPathValue::Resource(
                                Arc::new(extension.clone()),
                                type_info,
                                None,
                            );
                            matching_extensions.push(extension_value);
                        }
                    }
                }
            } else {
                // Handle primitives with extensions attached via wrapped primitive element
                if let Some(pe) = item.wrapped_primitive_element() {
                    for ext in &pe.extensions {
                        if ext.url == target_url {
                            // Build minimal JSON with url to represent the extension
                            let type_info = TypeInfo {
                                type_name: "Extension".to_string(),
                                name: Some("Extension".to_string()),
                                is_empty: Some(false),
                                namespace: Some("FHIR".to_string()),
                                singleton: Some(true),
                            };
                            let json = serde_json::json!({ "url": ext.url });
                            let extension_value =
                                FhirPathValue::Resource(Arc::new(json), type_info, None);
                            matching_extensions.push(extension_value);
                        }
                    }
                }
            }
        }

        Ok(EvaluationResult {
            value: Collection::from_values(matching_extensions),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// Create metadata for the extension function
fn create_metadata() -> FunctionMetadata {
    FunctionMetadata {
        name: "extension".to_string(),
        description: "Filter extensions by URL".to_string(),
        signature: FunctionSignature {
            input_type: "Any".to_string(),
            parameters: vec![FunctionParameter {
                name: "url".to_string(),
                parameter_type: vec!["String".to_string()],
                optional: false,
                is_expression: false,
                description: "URL of the extension to filter by".to_string(),
                default_value: None,
            }],
            return_type: "Collection".to_string(),
            polymorphic: true,
            min_params: 1,
            max_params: Some(1),
        },
        argument_evaluation: ArgumentEvaluationStrategy::Current,
        null_propagation: NullPropagationStrategy::Focus,
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        category: FunctionCategory::FilteringProjection,
        requires_terminology: false,
        requires_model: false,
    }
}
