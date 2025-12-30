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
static ELEMENT_TYPE_INFO: LazyLock<TypeInfo> = LazyLock::new(|| TypeInfo {
    type_name: "Element".to_string(),
    singleton: Some(true),
    namespace: Some("FHIR".to_string()),
    name: Some("Element".to_string()),
    is_empty: Some(false),
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
    fn estimate_descendant_count(json: &serde_json::Value) -> usize {
        match json {
            serde_json::Value::Object(map) => {
                // Estimate ~3 descendants per top-level field on average
                map.len() * 3
            }
            serde_json::Value::Array(arr) => arr.len() * 2,
            _ => 0,
        }
    }

    /// Iterative implementation using explicit stack to avoid recursion overhead.
    fn collect_json_descendants_iterative(
        &self,
        root: &serde_json::Value,
        descendants: &mut Vec<FhirPathValue>,
    ) {
        // Use a stack of references to avoid cloning during traversal
        let mut stack: Vec<&serde_json::Value> = vec![root];

        while let Some(json) = stack.pop() {
            match json {
                serde_json::Value::Object(map) => {
                    for (_, value) in map {
                        // Add the direct child
                        if let Ok(fhir_value) = self.json_to_fhirpath_value(value) {
                            descendants.push(fhir_value);
                        }
                        // Push to stack for iterative processing (reverse order for DFS)
                        stack.push(value);
                    }
                }
                serde_json::Value::Array(arr) => {
                    for item in arr {
                        // Add each array item
                        if let Ok(fhir_value) = self.json_to_fhirpath_value(item) {
                            descendants.push(fhir_value);
                        }
                        // Push to stack for iterative processing
                        stack.push(item);
                    }
                }
                _ => {
                    // Primitive values don't have descendants
                }
            }
        }
    }

    fn json_to_fhirpath_value(&self, json: &serde_json::Value) -> Result<FhirPathValue> {
        match json {
            serde_json::Value::String(s) => Ok(FhirPathValue::string(s.clone())),
            serde_json::Value::Number(n) => {
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
            serde_json::Value::Bool(b) => Ok(FhirPathValue::boolean(*b)),
            serde_json::Value::Object(_) => {
                // For complex objects, wrap as Resource using static TypeInfo
                // This avoids allocating a new TypeInfo for each descendant element
                Ok(FhirPathValue::Resource(
                    Arc::new(json.clone()),
                    ELEMENT_TYPE_INFO.clone(),
                    None,
                ))
            }
            serde_json::Value::Array(_) => {
                // Arrays are handled recursively
                Ok(FhirPathValue::string(json.to_string()))
            }
            serde_json::Value::Null => Ok(FhirPathValue::string("".to_string())),
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for DescendantsFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
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
