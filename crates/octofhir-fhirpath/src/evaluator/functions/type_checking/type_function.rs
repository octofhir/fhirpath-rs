//! Type function implementation
//!
//! The type function returns type information about a value.
//! Syntax: value.type()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use serde_json::json;

/// Type function evaluator
pub struct TypeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TypeFunctionEvaluator {
    /// Create a new type function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "type".to_string(),
                description: "Returns type information about the input value".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "TypeInfo".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
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

    fn get_type_info(&self, value: &FhirPathValue) -> (String, String) {
        match value {
            FhirPathValue::Boolean(_, type_info, _) => {
                // Check if this is a FHIR primitive or System primitive
                if type_info.namespace.as_deref() == Some("FHIR") {
                    ("FHIR".to_string(), "boolean".to_string())
                } else {
                    ("System".to_string(), "Boolean".to_string())
                }
            }
            FhirPathValue::Integer(_, type_info, _) => {
                if type_info.namespace.as_deref() == Some("FHIR") {
                    ("FHIR".to_string(), "integer".to_string())
                } else {
                    ("System".to_string(), "Integer".to_string())
                }
            }
            FhirPathValue::Decimal(_, type_info, _) => {
                if type_info.namespace.as_deref() == Some("FHIR") {
                    ("FHIR".to_string(), "decimal".to_string())
                } else {
                    ("System".to_string(), "Decimal".to_string())
                }
            }
            FhirPathValue::String(_, type_info, _) => {
                if type_info.namespace.as_deref() == Some("FHIR") {
                    // Use the actual FHIR type name (could be string, uri, uuid, etc.)
                    ("FHIR".to_string(), type_info.type_name.to_lowercase())
                } else {
                    ("System".to_string(), "String".to_string())
                }
            }
            FhirPathValue::Date(_, type_info, _) => {
                if type_info.namespace.as_deref() == Some("FHIR") {
                    ("FHIR".to_string(), "date".to_string())
                } else {
                    ("System".to_string(), "Date".to_string())
                }
            }
            FhirPathValue::DateTime(_, type_info, _) => {
                if type_info.namespace.as_deref() == Some("FHIR") {
                    ("FHIR".to_string(), "dateTime".to_string())
                } else {
                    ("System".to_string(), "DateTime".to_string())
                }
            }
            FhirPathValue::Time(_, type_info, _) => {
                if type_info.namespace.as_deref() == Some("FHIR") {
                    ("FHIR".to_string(), "time".to_string())
                } else {
                    ("System".to_string(), "Time".to_string())
                }
            }
            FhirPathValue::Quantity { .. } => ("System".to_string(), "Quantity".to_string()),
            FhirPathValue::Resource(_, type_info, _) => {
                ("FHIR".to_string(), type_info.type_name.clone())
            }
            FhirPathValue::Collection(_) => ("System".to_string(), "Collection".to_string()),
            FhirPathValue::Empty => ("System".to_string(), "Empty".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for TypeFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "type function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            let (namespace, name) = self.get_type_info(&value);

            // Create a JSON object representing the type information
            let type_info = json!({
                "namespace": namespace,
                "name": name
            });

            // Convert to FhirPathValue::Resource to represent the type object
            let default_type_info = crate::core::model_provider::TypeInfo {
                type_name: "TypeInfo".to_string(),
                singleton: Some(true),
                namespace: Some("System".to_string()),
                name: Some("TypeInfo".to_string()),
                is_empty: Some(false),
            };

            results.push(FhirPathValue::Resource(
                std::sync::Arc::new(type_info),
                default_type_info,
                None, // No primitive element
            ));
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
