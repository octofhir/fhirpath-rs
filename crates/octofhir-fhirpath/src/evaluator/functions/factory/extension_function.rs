//! %factory.Extension(url, value) function implementation
//!
//! Creates a FHIR Extension instance.
//! Syntax: %factory.Extension(url, value)

use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct FactoryExtensionFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryExtensionFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "Extension".to_string(),
                description: "Creates a FHIR Extension instance with url and value".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
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
                    return_type: "Extension".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(2),
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
impl PureFunctionEvaluator for FactoryExtensionFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "Extension function can only be called on %factory variable".to_string(),
            ));
        }

        let url = match args.first().and_then(|a| a.first()) {
            Some(FhirPathValue::String(s, _, _)) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "Extension function requires a string url argument".to_string(),
                ));
            }
        };

        let mut ext = serde_json::Map::new();
        ext.insert("url".to_string(), serde_json::Value::String(url));

        // If value argument is provided, add it based on type
        if let Some(value_arg) = args.get(1).and_then(|a| a.first()) {
            add_extension_value(&mut ext, value_arg);
        }

        let type_info = Arc::new(TypeInfo::new_complex("Extension"));
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource_wrapped(
                serde_json::Value::Object(ext),
                type_info,
            )),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// Add an extension value to an Extension JSON object with the correct value[x] key
pub(crate) fn add_extension_value(
    ext: &mut serde_json::Map<String, serde_json::Value>,
    value: &FhirPathValue,
) {
    match value {
        FhirPathValue::String(s, _, _) => {
            ext.insert(
                "valueString".to_string(),
                serde_json::Value::String(s.clone()),
            );
        }
        FhirPathValue::Integer(i, _, _) => {
            ext.insert("valueInteger".to_string(), serde_json::json!(*i));
        }
        FhirPathValue::Decimal(d, _, _) => {
            ext.insert(
                "valueDecimal".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(d.to_string().parse::<f64>().unwrap_or(0.0))
                        .unwrap_or(serde_json::Number::from(0)),
                ),
            );
        }
        FhirPathValue::Boolean(b, _, _) => {
            ext.insert("valueBoolean".to_string(), serde_json::Value::Bool(*b));
        }
        FhirPathValue::Date(d, _, _) => {
            ext.insert(
                "valueDate".to_string(),
                serde_json::Value::String(d.to_string()),
            );
        }
        FhirPathValue::DateTime(dt, _, _) => {
            ext.insert(
                "valueDateTime".to_string(),
                serde_json::Value::String(dt.to_string()),
            );
        }
        FhirPathValue::Time(t, _, _) => {
            ext.insert(
                "valueTime".to_string(),
                serde_json::Value::String(t.to_string()),
            );
        }
        FhirPathValue::Resource(json, ti, _) => {
            // Determine the value[x] key from the type info
            let type_name = ti.name.as_deref().unwrap_or(&ti.type_name);
            let key = format!("value{type_name}");
            ext.insert(key, json.as_ref().clone());
        }
        FhirPathValue::Quantity {
            value: v,
            unit,
            code,
            system,
            ..
        } => {
            let mut q = serde_json::Map::new();
            q.insert(
                "value".to_string(),
                serde_json::json!(v.to_string().parse::<f64>().unwrap_or(0.0)),
            );
            if let Some(u) = unit {
                q.insert("unit".to_string(), serde_json::Value::String(u.clone()));
            }
            if let Some(c) = code {
                q.insert("code".to_string(), serde_json::Value::String(c.clone()));
            }
            if let Some(s) = system {
                q.insert("system".to_string(), serde_json::Value::String(s.clone()));
            }
            ext.insert("valueQuantity".to_string(), serde_json::Value::Object(q));
        }
        _ => {
            // For other types, try to convert to string
            ext.insert(
                "valueString".to_string(),
                serde_json::Value::String(value.to_string().unwrap_or_default()),
            );
        }
    }
}
