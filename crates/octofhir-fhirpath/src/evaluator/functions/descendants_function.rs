//! descendants function implementation
//!
//! Retrieves all descendant elements of the current context

use std::sync::{Arc, LazyLock};

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Static TypeInfo for Element type, reused across all descendant elements
/// to avoid allocating a new TypeInfo for each descendant.
static ELEMENT_TYPE_INFO: LazyLock<Arc<TypeInfo>> = LazyLock::new(|| {
    Arc::new(TypeInfo {
        type_name: "Element".to_string(),
        singleton: Some(true),
        namespace: Some("FHIR".to_string()),
        name: Some("Element".to_string()),
        is_empty: Some(false),
    })
});

/// Static TypeInfo for Reference type, reused for reference elements.
static REFERENCE_TYPE_INFO: LazyLock<Arc<TypeInfo>> = LazyLock::new(|| {
    Arc::new(TypeInfo {
        type_name: "Reference".to_string(),
        singleton: Some(true),
        namespace: Some("FHIR".to_string()),
        name: Some("Reference".to_string()),
        is_empty: Some(false),
    })
});

pub struct DescendantsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DescendantsFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "descendants".to_string(),
                description: "Returns all descendant elements of the current context".to_string(),
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
                category: FunctionCategory::TreeNavigation,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    fn get_descendants(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Resource(json, _type_info, _primitive) => {
                // Estimate capacity based on JSON structure to reduce reallocations
                let estimated_size = Self::estimate_descendant_count(json);
                let mut descendants = Vec::with_capacity(estimated_size);

                // Use iterative approach with explicit stack to avoid recursion overhead
                self.collect_json_descendants_iterative(json, &mut descendants);
                descendants
            }
            _ => {
                // For primitive types, no descendants
                Vec::new()
            }
        }
    }

    /// Estimate the number of descendants for capacity pre-allocation.
    /// This is a heuristic based on typical FHIR resource structure.
    fn estimate_descendant_count(json: &crate::core::node::FhirNode) -> usize {
        use crate::core::node::FhirNode;
        match json {
            FhirNode::Object(map) => {
                // Estimate ~3 descendants per top-level field on average
                map.len() * 3
            }
            FhirNode::Array(arr) => arr.len() * 2,
            _ => 0,
        }
    }

    /// Iterative implementation using explicit stack to avoid recursion overhead.
    fn collect_json_descendants_iterative(
        &self,
        root: &crate::core::node::FhirNode,
        descendants: &mut Vec<FhirPathValue>,
    ) {
        use crate::core::node::FhirNode;
        // Use a stack of references to avoid cloning during traversal
        let mut stack: Vec<&FhirNode> = vec![root];

        while let Some(json) = stack.pop() {
            match json {
                FhirNode::Object(_) => {
                    for (_, value) in json.entries() {
                        match value {
                            FhirNode::Array(_) => {
                                stack.push(value);
                            }
                            FhirNode::Object(_) => {
                                if let Ok(fhir_value) = self.json_to_fhirpath_value(value) {
                                    descendants.push(fhir_value);
                                }
                                stack.push(value);
                            }
                            _ => {
                                if let Ok(fhir_value) = self.json_to_fhirpath_value(value) {
                                    descendants.push(fhir_value);
                                }
                            }
                        }
                    }
                }
                FhirNode::Array(arr) => {
                    for item in arr.iter() {
                        match item {
                            FhirNode::Array(_) => {
                                stack.push(item);
                            }
                            FhirNode::Object(_) => {
                                if let Ok(fhir_value) = self.json_to_fhirpath_value(item) {
                                    descendants.push(fhir_value);
                                }
                                stack.push(item);
                            }
                            _ => {
                                if let Ok(fhir_value) = self.json_to_fhirpath_value(item) {
                                    descendants.push(fhir_value);
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Primitive values don't have descendants
                }
            }
        }
    }

    fn json_to_fhirpath_value(&self, json: &crate::core::node::FhirNode) -> Result<FhirPathValue> {
        use crate::core::node::FhirNode;
        match json {
            FhirNode::Str(s) => Ok(FhirPathValue::string(s.to_string())),
            FhirNode::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(FhirPathValue::integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(FhirPathValue::decimal(
                        rust_decimal::Decimal::from_f64_retain(f).unwrap_or_default(),
                    ))
                } else {
                    Ok(FhirPathValue::string(n.to_string()))
                }
            }
            FhirNode::Bool(b) => Ok(FhirPathValue::boolean(*b)),
            FhirNode::Object(_) => {
                let type_info = if let Some(resource_type) =
                    json.get("resourceType").and_then(|value| value.as_str())
                {
                    Arc::new(TypeInfo {
                        type_name: resource_type.to_string(),
                        singleton: Some(true),
                        namespace: Some("FHIR".to_string()),
                        name: Some(resource_type.to_string()),
                        is_empty: Some(false),
                    })
                } else if json.get("reference").is_some() {
                    REFERENCE_TYPE_INFO.clone()
                } else {
                    ELEMENT_TYPE_INFO.clone()
                };

                Ok(FhirPathValue::Resource(json.clone(), type_info, None))
            }
            FhirNode::Array(_) => {
                // Arrays are handled recursively
                Ok(FhirPathValue::string(json.to_string()))
            }
            FhirNode::Null => Ok(FhirPathValue::string("".to_string())),
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for DescendantsFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::FP0053,
                "descendants function takes no arguments".to_string(),
            ));
        }

        let mut all_descendants = Vec::new();

        for item in input {
            let descendants = self.get_descendants(&item);
            all_descendants.extend(descendants);
        }

        Ok(EvaluationResult {
            value: Collection::from(all_descendants),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
