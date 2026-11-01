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
    FunctionParameter, FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};

pub struct FactoryWithPropertyFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryWithPropertyFunctionEvaluator {
    pub fn create() -> Arc<dyn ProviderPureFunctionEvaluator> {
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
                requires_model: true,
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
        FhirPathValue::Resource(json, _, _) => json.to_json(),
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

/// The JSON key of a choice element (`value`) holding `value`: `valueQuantity`, `valueCode`.
fn choice_key(name: &str, value: &FhirPathValue) -> String {
    let type_name = match value {
        FhirPathValue::Resource(_, type_info, _) | FhirPathValue::String(_, type_info, _)
            if type_info.namespace.as_deref() == Some("FHIR") =>
        {
            type_info
                .name
                .clone()
                .unwrap_or_else(|| type_info.type_name.clone())
        }
        other => other.type_name().to_string(),
    };
    let mut chars = type_name.chars();
    match chars.next() {
        Some(first) => format!("{name}{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => name.to_string(),
    }
}

#[async_trait::async_trait]
impl ProviderPureFunctionEvaluator for FactoryWithPropertyFunctionEvaluator {
    /// A copy of `instance` with `name` set to `value`, replacing any existing value (an
    /// empty value removes it). With a model, `name` must be an element of the instance's
    /// type, a repeating element takes an array, and a choice element (`value`) takes its
    /// typed name (`valueQuantity`).
    async fn evaluate(
        &self,
        input: Collection,
        args: Vec<Collection>,
        context: &crate::evaluator::EvaluationContext,
    ) -> Result<EvaluationResult> {
        let error = |message: &str| {
            FhirPathError::evaluation_error(crate::core::error_code::FP0056, message.to_string())
        };
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "withProperty function can only be called on %factory variable".to_string(),
            ));
        }
        let instance = args
            .first()
            .and_then(|a| a.first())
            .ok_or_else(|| error("withProperty requires an instance argument"))?;
        let name = match args.get(1).and_then(|a| a.first()) {
            Some(FhirPathValue::String(s, _, _)) => s.clone(),
            _ => return Err(error("withProperty requires a string name argument")),
        };
        let values = args
            .get(2)
            .ok_or_else(|| error("withProperty requires a value argument"))?;
        let FhirPathValue::Resource(json, type_info, _) = instance else {
            return Err(error("withProperty instance must be a FHIR complex type"));
        };

        let model = context.model_provider();
        let element_type = model
            .get_element_type(type_info, &name)
            .await
            .ok()
            .flatten();
        let element_names = model.get_element_names(type_info);
        let is_choice = element_type.is_none()
            && element_names.iter().any(|n| {
                n.len() > name.len()
                    && n.starts_with(name.as_str())
                    && n[name.len()..].starts_with(|c: char| c.is_ascii_uppercase())
            });
        if element_type.is_none() && !is_choice && !element_names.is_empty() {
            return Err(error(&format!(
                "'{name}' is not an element of {}",
                type_info.name.as_deref().unwrap_or(&type_info.type_name)
            )));
        }
        let repeats = element_type.as_ref().and_then(|t| t.singleton) == Some(false);

        let mut new_json = json.to_json();
        let Some(object) = new_json.as_object_mut() else {
            return Err(error("withProperty instance must be a FHIR complex type"));
        };
        // The new value replaces the old one, including a choice's other typed forms.
        object.remove(&name);
        if is_choice {
            object.retain(|key, _| {
                !(key.len() > name.len()
                    && key.starts_with(name.as_str())
                    && key[name.len()..].starts_with(|c: char| c.is_ascii_uppercase()))
            });
        }
        match values.len() {
            0 => {}
            1 if is_choice => {
                object.insert(choice_key(&name, &values[0]), to_json_value(&values[0]));
            }
            1 if !repeats => {
                object.insert(name, to_json_value(&values[0]));
            }
            _ if repeats || element_names.is_empty() => {
                object.insert(
                    name,
                    serde_json::Value::Array(values.iter().map(to_json_value).collect()),
                );
            }
            _ => return Err(error(&format!("'{name}' takes a single value"))),
        }

        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource_wrapped(new_json, type_info.clone())),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
