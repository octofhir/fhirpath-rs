//! Children function implementation
//!
//! The children function returns the direct child elements of a resource or element.
//! Unlike descendants(), this only returns immediate children, not all nested descendants.
//! Syntax: collection.children()

use std::sync::Arc;

use crate::core::{Collection, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Children function evaluator
pub struct ChildrenFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ChildrenFunctionEvaluator {
    /// Create a new children function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "children".to_string(),
                description: "Returns the direct child elements of a resource or element"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: false, // Returns unordered collection
                category: FunctionCategory::TreeNavigation,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Extract direct children from a JSON value
    fn extract_children(&self, json: &serde_json::Value) -> Vec<FhirPathValue> {
        let mut children = Vec::new();

        match json {
            serde_json::Value::Object(map) => {
                // For objects, add all property values as children
                for (key, value) in map {
                    // Skip meta fields that aren't typically considered children in FHIRPath
                    if key.starts_with('_') || key == "resourceType" {
                        continue;
                    }

                    children.extend(self.convert_json_to_fhir_path_values(value));
                }
            }
            serde_json::Value::Array(arr) => {
                // For arrays, add all elements as children
                for value in arr {
                    children.extend(self.convert_json_to_fhir_path_values(value));
                }
            }
            _ => {
                // Primitive values have no children
            }
        }

        children
    }

    /// Convert JSON value to FhirPathValue(s)
    #[allow(clippy::only_used_in_recursion)]
    fn convert_json_to_fhir_path_values(&self, json: &serde_json::Value) -> Vec<FhirPathValue> {
        match json {
            serde_json::Value::String(s) => vec![FhirPathValue::string(s.clone())],
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    vec![FhirPathValue::integer(i)]
                } else if let Some(f) = n.as_f64() {
                    if let Ok(decimal) = rust_decimal::Decimal::try_from(f) {
                        vec![FhirPathValue::decimal(decimal)]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            serde_json::Value::Bool(b) => vec![FhirPathValue::boolean(*b)],
            serde_json::Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    result.extend(self.convert_json_to_fhir_path_values(item));
                }
                result
            }
            serde_json::Value::Object(_) => {
                // Objects become Resource FhirPathValues
                vec![FhirPathValue::resource(json.clone())]
            }
            serde_json::Value::Null => vec![],
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ChildrenFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "children function expects no arguments".to_string(),
            ));
        }

        let mut all_children = Vec::new();

        for item in input {
            match item {
                FhirPathValue::Resource(json, _type_info, _primitive) => {
                    all_children.extend(self.extract_children(&json));
                }
                FhirPathValue::Collection(collection) => {
                    // For collections, get children of all items in the collection
                    for collection_item in collection.iter() {
                        if let FhirPathValue::Resource(json, _, _) = collection_item {
                            all_children.extend(self.extract_children(json));
                        }
                    }
                }
                _ => {
                    // Primitive types have no children
                }
            }
        }

        Ok(EvaluationResult {
            value: Collection::from(all_children),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
