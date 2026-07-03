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
    fn extract_children(&self, json: &crate::core::node::FhirNode) -> Vec<FhirPathValue> {
        use crate::core::node::FhirNode;
        let mut children = Vec::new();

        match json {
            FhirNode::Object(_) => {
                // For objects, add all property values as children
                for (key, value) in json.entries() {
                    // Skip meta fields that aren't typically considered children in FHIRPath
                    if key.starts_with('_') || key == "resourceType" {
                        continue;
                    }

                    children.extend(convert_json_to_fhir_path_values(value));
                }
            }
            FhirNode::Array(arr) => {
                // For arrays, add all elements as children
                for value in arr.iter() {
                    children.extend(convert_json_to_fhir_path_values(value));
                }
            }
            _ => {
                // Primitive values have no children
            }
        }

        children
    }
}

fn convert_json_to_fhir_path_values(json: &crate::core::node::FhirNode) -> Vec<FhirPathValue> {
    use crate::core::node::FhirNode;

    match json {
        FhirNode::Str(s) => vec![FhirPathValue::string(s.to_string())],
        FhirNode::Number(n) => {
            if let Some(i) = n.as_i64() {
                vec![FhirPathValue::integer(i)]
            } else if let Some(f) = n.as_f64() {
                rust_decimal::Decimal::try_from(f)
                    .map(FhirPathValue::decimal)
                    .into_iter()
                    .collect()
            } else {
                vec![]
            }
        }
        FhirNode::Bool(b) => vec![FhirPathValue::boolean(*b)],
        FhirNode::Array(arr) => arr
            .iter()
            .flat_map(convert_json_to_fhir_path_values)
            .collect(),
        FhirNode::Object(_) => vec![FhirPathValue::resource_from_node(json.clone())],
        FhirNode::Null => vec![],
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ChildrenFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
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
