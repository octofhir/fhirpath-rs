//! descendants function implementation
//!
//! Retrieves all descendant elements of the current context

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

pub struct DescendantsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DescendantsFunctionEvaluator {
    pub fn create() -> Arc<dyn FunctionEvaluator> {
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
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::TreeNavigation,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    fn get_descendants(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        let mut descendants = Vec::new();

        match value {
            FhirPathValue::Resource(json, type_info, primitive) => {
                // For JSON objects, recursively collect all nested values
                self.collect_json_descendants(json, &mut descendants);
            }
            _ => {
                // For primitive types, no descendants
            }
        }

        descendants
    }

    fn collect_json_descendants(&self, json: &serde_json::Value, descendants: &mut Vec<FhirPathValue>) {
        match json {
            serde_json::Value::Object(map) => {
                for (_, value) in map {
                    // Add the direct child
                    if let Ok(fhir_value) = self.json_to_fhirpath_value(value) {
                        descendants.push(fhir_value);
                    }
                    // Recursively add descendants of this child
                    self.collect_json_descendants(value, descendants);
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    // Add each array item
                    if let Ok(fhir_value) = self.json_to_fhirpath_value(item) {
                        descendants.push(fhir_value);
                    }
                    // Recursively add descendants of this item
                    self.collect_json_descendants(item, descendants);
                }
            }
            _ => {
                // Primitive values don't have descendants
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
                    Ok(FhirPathValue::decimal(rust_decimal::Decimal::from_f64_retain(f).unwrap_or_default()))
                } else {
                    Ok(FhirPathValue::string(n.to_string()))
                }
            }
            serde_json::Value::Bool(b) => Ok(FhirPathValue::boolean(*b)),
            serde_json::Value::Object(_) => {
                // For complex objects, wrap as Resource
                let type_info = crate::core::model_provider::TypeInfo {
                    type_name: "Element".to_string(),
                    singleton: Some(true),
                    namespace: Some("FHIR".to_string()),
                    name: Some("Element".to_string()),
                    is_empty: Some(false),
                };
                Ok(FhirPathValue::Resource(json.clone().into(), type_info, None))
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
impl FunctionEvaluator for DescendantsFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
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