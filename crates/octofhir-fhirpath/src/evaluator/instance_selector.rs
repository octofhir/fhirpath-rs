//! Instance selector evaluation (FHIRPath §1.5.2.3, STU): `[type] { element: value, ... }`.
//!
//! The selector builds one object from the single input item: each element value is
//! evaluated with that item as focus, elements whose value is empty are left out, and an
//! element takes an array when the model says it repeats (or when it gets several values on
//! an anonymous object). Without a type name the type comes from the element the selector
//! is assigned to (`Identifier { type: { coding: { code: 'MR' } } }`), else the object is
//! anonymous. A primitive type takes its value from the `value` element (`code { value:
//! 'final' }`).

use std::sync::Arc;

use async_recursion::async_recursion;
use serde_json::{Map, Value as JsonValue};

use crate::ast::{ExpressionNode, InstanceSelectorNode};
use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};

use super::{EvaluationContext, EvaluationResult, Evaluator};

fn error(message: impl Into<String>) -> FhirPathError {
    FhirPathError::evaluation_error(crate::core::error_code::FP0055, message.into())
}

/// `FHIR.Coding` → (`Some("FHIR")`, `Coding`).
fn split_type(name: &str) -> (Option<&str>, &str) {
    match name.split_once('.') {
        Some((namespace, name)) => (Some(namespace), name),
        None => (None, name),
    }
}

/// The JSON key of a choice element (`value`) holding `value` (`valueQuantity`).
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

impl Evaluator {
    /// Evaluate an instance selector; `context_type` is the type of the element it is
    /// assigned to, for a selector without a type name.
    #[async_recursion]
    pub(super) async fn evaluate_instance_selector(
        &self,
        node: &InstanceSelectorNode,
        context_type: Option<TypeInfo>,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        let input = context.input_collection();
        let focus = match input.len() {
            0 => {
                return Ok(EvaluationResult {
                    value: Collection::empty(),
                });
            }
            1 => input[0].clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0054,
                    "An instance selector needs a single input item",
                ));
            }
        };
        let item_context = context.create_child_context(Collection::single(focus.clone()));
        item_context.set_this(focus);

        // The type to create: named, taken from the context, or anonymous.
        let type_info = match &node.type_name {
            Some(qualified) => {
                let (namespace, name) = split_type(qualified);
                if namespace == Some("System") {
                    Some(TypeInfo::system_type(name.to_string(), true))
                } else {
                    let found = self
                        .model_provider()
                        .get_type(name)
                        .await
                        .map_err(|e| error(format!("Model provider error: {e}")))?;
                    Some(found.ok_or_else(|| error(format!("Unknown type '{qualified}'")))?)
                }
            }
            None => context_type,
        };

        let is_primitive = type_info.as_ref().is_some_and(|t| {
            t.namespace.as_deref() == Some("System")
                || t.name
                    .as_deref()
                    .unwrap_or(&t.type_name)
                    .starts_with(|c: char| c.is_ascii_lowercase())
        });
        if is_primitive {
            return self
                .primitive_instance(node, type_info.unwrap(), &item_context)
                .await;
        }

        let mut object = Map::new();
        let type_name = type_info
            .as_ref()
            .map(|t| t.name.clone().unwrap_or_else(|| t.type_name.clone()));
        // Only resources carry `resourceType`.
        if let Some(type_name) = &type_name
            && type_name != "Resource"
            && self
                .model_provider()
                .is_type_derived_from(type_name, "Resource")
        {
            object.insert(
                "resourceType".to_string(),
                JsonValue::String(type_name.clone()),
            );
        }
        let element_names = type_info
            .as_ref()
            .map(|parent| self.model_provider().get_element_names(parent))
            .unwrap_or_default();

        for (name, expression) in &node.elements {
            let element_type = match &type_info {
                Some(parent) => self
                    .model_provider()
                    .get_element_type(parent, name)
                    .await
                    .map_err(|e| error(format!("Model provider error: {e}")))?,
                None => None,
            };
            // A choice element (`value[x]`) is named without its type: `value: 5 'mg'`. The
            // model lists it next to its typed forms (`valueQuantity`, ...).
            let is_typed_form = |n: &String| {
                n.len() > name.len()
                    && n.starts_with(name.as_str())
                    && n[name.len()..].starts_with(|c: char| c.is_ascii_uppercase())
            };
            let is_choice = element_type.is_none()
                && (element_names.iter().any(|n| n == &format!("{name}[x]"))
                    || element_names.iter().any(is_typed_form));
            // A model that knows the type's elements rejects a name it does not have.
            if element_type.is_none() && !is_choice && !element_names.is_empty() {
                return Err(error(format!(
                    "'{name}' is not an element of {}",
                    type_name.as_deref().unwrap_or("the type")
                )));
            }

            let values = match expression {
                ExpressionNode::InstanceSelector(inner) if inner.type_name.is_none() => {
                    self.evaluate_instance_selector(inner, element_type.clone(), &item_context)
                        .await?
                        .value
                }
                _ => self.evaluate_node(expression, &item_context).await?.value,
            };
            if values.is_empty() {
                continue;
            }

            let repeats = element_type.as_ref().and_then(|t| t.singleton) == Some(false);

            if is_choice {
                if values.len() > 1 {
                    return Err(error(format!("'{name}' takes a single value")));
                }
                let key = choice_key(name, &values[0]);
                if !element_names.is_empty() && !element_names.contains(&key) {
                    return Err(error(format!(
                        "'{name}' cannot hold a {} ('{key}' is not an element of {})",
                        values[0].type_name(),
                        type_name.as_deref().unwrap_or("the type")
                    )));
                }
                object.insert(key, values[0].to_json_value());
            } else if repeats || (values.len() > 1 && type_info.is_none()) {
                object.insert(
                    name.clone(),
                    JsonValue::Array(values.iter().map(FhirPathValue::to_json_value).collect()),
                );
            } else if values.len() > 1 {
                return Err(error(format!("'{name}' takes a single value")));
            } else {
                object.insert(name.clone(), values[0].to_json_value());
            }
        }

        let json = JsonValue::Object(object);
        let value = match type_info {
            Some(type_info) if type_info.type_name == "Quantity" => {
                let quantity = crate::core::value::utils::json_to_fhirpath_value(json);
                FhirPathValue::wrap_value(quantity, Arc::new(type_info), None)
            }
            Some(mut type_info) => {
                type_info.singleton = Some(true);
                FhirPathValue::Resource(
                    crate::core::node::FhirNode::from_json(&json),
                    Arc::new(type_info),
                    None,
                )
            }
            None => FhirPathValue::Resource(
                crate::core::node::FhirNode::from_json(&json),
                Arc::new(TypeInfo::system_type("Tuple".to_string(), true)),
                None,
            ),
        };
        Ok(EvaluationResult {
            value: Collection::single(value),
        })
    }

    /// A primitive created as `code { value: 'final' }`: the `value` element, typed.
    async fn primitive_instance(
        &self,
        node: &InstanceSelectorNode,
        type_info: TypeInfo,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        let mut value = None;
        for (name, expression) in &node.elements {
            if name != "value" {
                return Err(error(format!(
                    "A primitive instance selector only sets 'value', not '{name}'"
                )));
            }
            value = Some(self.evaluate_node(expression, context).await?.value);
        }
        let Some(values) = value.filter(|values| !values.is_empty()) else {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        };
        if values.len() > 1 {
            return Err(error("A primitive takes a single value"));
        }
        let typed = FhirPathValue::wrap_value(values[0].clone(), Arc::new(type_info), None);
        Ok(EvaluationResult {
            value: Collection::single(typed),
        })
    }
}
