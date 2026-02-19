//! %factory.withProperty(instance, name, value) function implementation
//!
//! Sets a property on an existing FHIR type instance (immutable, returns new instance).
//! Syntax: %factory.withProperty(instance, name, value)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct FactoryWithPropertyFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryWithPropertyFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "withProperty".to_string(),
                description: "Sets a property on an existing FHIR instance (returns new copy)"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "instance".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "The instance to modify".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "name".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Property name".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "value".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Property value".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 3,
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

/// Convert a FhirPathValue to a serde_json::Value for property assignment
fn to_json_value(value: &FhirPathValue) -> serde_json::Value {
    match value {
        FhirPathValue::String(s, _, _) => serde_json::Value::String(s.clone()),
        FhirPathValue::Integer(i, _, _) => serde_json::json!(*i),
        FhirPathValue::Decimal(d, _, _) => d
            .to_string()
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::String(d.to_string())),
        FhirPathValue::Boolean(b, _, _) => serde_json::Value::Bool(*b),
        FhirPathValue::Date(d, _, _) => serde_json::Value::String(d.to_string()),
        FhirPathValue::DateTime(dt, _, _) => serde_json::Value::String(dt.to_string()),
        FhirPathValue::Time(t, _, _) => serde_json::Value::String(t.to_string()),
        FhirPathValue::Resource(json, _, _) => json.as_ref().clone(),
        FhirPathValue::Quantity {
            value: v,
            unit,
            code,
            system,
            ..
        } => {
            let mut q = serde_json::Map::new();
            if let Ok(f) = v.to_string().parse::<f64>()
                && let Some(n) = serde_json::Number::from_f64(f)
            {
                q.insert("value".to_string(), serde_json::Value::Number(n));
            }
            if let Some(u) = unit {
                q.insert("unit".to_string(), serde_json::Value::String(u.clone()));
            }
            if let Some(c) = code {
                q.insert("code".to_string(), serde_json::Value::String(c.clone()));
            }
            if let Some(s) = system {
                q.insert("system".to_string(), serde_json::Value::String(s.clone()));
            }
            serde_json::Value::Object(q)
        }
        _ => serde_json::Value::String(value.to_string().unwrap_or_default()),
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for FactoryWithPropertyFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "withProperty function can only be called on %factory variable".to_string(),
            ));
        }

        // First arg: instance
        let instance = args.first().and_then(|a| a.first()).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "withProperty requires an instance argument".to_string(),
            )
        })?;

        // Second arg: property name
        let prop_name = match args.get(1).and_then(|a| a.first()) {
            Some(FhirPathValue::String(s, _, _)) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "withProperty requires a string name argument".to_string(),
                ));
            }
        };

        // Third arg: value (can be single or collection)
        let value_args = args.get(2).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "withProperty requires a value argument".to_string(),
            )
        })?;

        match instance {
            FhirPathValue::Resource(json, type_info, _) => {
                let mut new_json = json.as_ref().clone();

                if let Some(obj) = new_json.as_object_mut() {
                    // If collection has multiple values, store as array
                    if value_args.len() > 1 {
                        let arr: Vec<serde_json::Value> =
                            value_args.iter().map(to_json_value).collect();
                        obj.insert(prop_name, serde_json::Value::Array(arr));
                    } else if let Some(val) = value_args.first() {
                        obj.insert(prop_name, to_json_value(val));
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
                "withProperty instance must be a FHIR complex type".to_string(),
            )),
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
