//! SQL-on-FHIR functions: `getResourceKey()` and `getReferenceKey([type])`.
//!
//! These are not FHIRPath functions: the SQL on FHIR ViewDefinition specification lists them
//! as required additional functions. They are compiled in with the `sql-on-fhir` feature and
//! registered by [`FunctionRegistryBuilder::with_sql_on_fhir_functions`].
//!
//! - `getResourceKey()` keys each input resource on its `id`.
//! - `getReferenceKey([type])` keys each input Reference on the id of the resource it points
//!   to: `Type/id` (relative, or with a leading `/`) on `id`, and a fragment `#id` on the local
//!   id of the root resource's `contained[]` entry, so it matches `getResourceKey()` on that
//!   contained resource. Absolute URLs and `urn:` references have no key. With a type
//!   argument (a type name, `getReferenceKey(Patient)`), references to other types have no
//!   key.
//!
//! [`FunctionRegistryBuilder::with_sql_on_fhir_functions`]:
//!     crate::evaluator::function_registry::FunctionRegistryBuilder::with_sql_on_fhir_functions

use std::sync::{Arc, LazyLock};

use crate::ast::ExpressionNode;
use crate::core::node::FhirNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

fn metadata(name: &str, description: &str, parameters: Vec<FunctionParameter>) -> FunctionMetadata {
    let max = parameters.len();
    FunctionMetadata {
        name: name.to_string(),
        description: description.to_string(),
        signature: FunctionSignature {
            input_type: "Any".to_string(),
            parameters,
            return_type: "String".to_string(),
            polymorphic: true,
            min_params: 0,
            max_params: Some(max),
        },
        argument_evaluation: ArgumentEvaluationStrategy::Current,
        null_propagation: NullPropagationStrategy::None,
        empty_propagation: EmptyPropagation::NoPropagation,
        deterministic: true,
        category: FunctionCategory::Utility,
        requires_terminology: false,
        requires_model: false,
    }
}

/// The JSON node of a resource or complex value.
fn node_of(value: &FhirPathValue) -> Option<&FhirNode> {
    match value {
        FhirPathValue::Resource(node, _, _) => Some(node),
        _ => None,
    }
}

/// The resource type of the root resource's `contained[]` entry with this local id.
fn contained_type<'a>(root: Option<&'a FhirNode>, local_id: &str) -> Option<&'a str> {
    root?
        .get("contained")?
        .as_array()?
        .iter()
        .find(|c| c.get("id").and_then(FhirNode::as_str) == Some(local_id))?
        .get("resourceType")?
        .as_str()
}

/// The key of the resource a Reference points to (see the module docs).
pub(crate) fn reference_key(
    reference: &FhirNode,
    root: Option<&FhirNode>,
    want_type: Option<&str>,
) -> Option<String> {
    let target = reference.get("reference")?.as_str()?;
    if let Some(local) = target.strip_prefix('#') {
        if local.is_empty() {
            return None;
        }
        let actual = contained_type(root, local)
            .or_else(|| reference.get("type").and_then(FhirNode::as_str));
        return match (want_type, actual) {
            (Some(want), Some(actual)) if want != actual => None,
            (Some(_), None) => None,
            _ => Some(local.to_string()),
        };
    }
    if target.starts_with("urn:") || target.contains("://") {
        return None;
    }
    let mut parts = target.trim_start_matches('/').split('/');
    let resource_type = parts.next()?;
    let id = parts.next().filter(|id| !id.is_empty())?;
    match want_type {
        Some(want) if want != resource_type => None,
        _ => Some(id.to_string()),
    }
}

/// The type name given as a type specifier argument: `Patient` or `FHIR.Patient`.
fn type_argument(node: &ExpressionNode) -> Result<String> {
    let invalid = || {
        FhirPathError::evaluation_error(
            crate::core::error_code::FP0053,
            "getReferenceKey() type argument must be a type name",
        )
    };
    match node {
        ExpressionNode::Identifier(identifier) => Ok(identifier.name.clone()),
        ExpressionNode::TypeInfo(type_info) => Ok(type_info.name.clone()),
        ExpressionNode::PropertyAccess(access) => match access.object.as_ref() {
            ExpressionNode::Identifier(_) => Ok(access.property.clone()),
            _ => Err(invalid()),
        },
        _ => Err(invalid()),
    }
}

/// `getResourceKey()` function evaluator.
pub struct GetResourceKeyFunctionEvaluator;

impl GetResourceKeyFunctionEvaluator {
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for GetResourceKeyFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        _context: &EvaluationContext,
        _args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        let keys: Vec<FhirPathValue> = input
            .iter()
            .filter_map(|value| node_of(value)?.get("id")?.as_str())
            .map(FhirPathValue::string)
            .collect();
        Ok(EvaluationResult {
            value: Collection::from(keys),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        static METADATA: LazyLock<FunctionMetadata> = LazyLock::new(|| {
            metadata(
                "getResourceKey",
                "SQL on FHIR: the key of each input resource (its id)",
                vec![],
            )
        });
        &METADATA
    }
}

/// `getReferenceKey([type])` function evaluator.
pub struct GetReferenceKeyFunctionEvaluator;

impl GetReferenceKeyFunctionEvaluator {
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for GetReferenceKeyFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        let want_type = args.first().map(type_argument).transpose()?;
        let root = context.root_resource_value();
        let root_node = root.as_ref().and_then(node_of);
        let keys: Vec<FhirPathValue> = input
            .iter()
            .filter_map(|value| reference_key(node_of(value)?, root_node, want_type.as_deref()))
            .map(FhirPathValue::string)
            .collect();
        Ok(EvaluationResult {
            value: Collection::from(keys),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        static METADATA: LazyLock<FunctionMetadata> = LazyLock::new(|| {
            metadata(
                "getReferenceKey",
                "SQL on FHIR: the key of the resource each input Reference points to",
                vec![FunctionParameter {
                    name: "type".to_string(),
                    parameter_type: vec!["TypeSpecifier".to_string()],
                    optional: true,
                    is_expression: true,
                    description: "Only references to this resource type have a key".to_string(),
                    default_value: None,
                }],
            )
        });
        &METADATA
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn key(
        reference: serde_json::Value,
        root: serde_json::Value,
        want: Option<&str>,
    ) -> Option<String> {
        let root = FhirNode::from_json(&root);
        reference_key(&FhirNode::from_json(&reference), Some(&root), want)
    }

    #[test]
    fn reference_keys() {
        let root = json!({"resourceType": "Observation",
            "contained": [{"resourceType": "Patient", "id": "p1"}]});
        assert_eq!(
            key(json!({"reference": "Patient/123"}), root.clone(), None).as_deref(),
            Some("123")
        );
        assert_eq!(
            key(
                json!({"reference": "/Patient/123"}),
                root.clone(),
                Some("Patient")
            )
            .as_deref(),
            Some("123")
        );
        assert_eq!(
            key(
                json!({"reference": "Patient/123/_history/2"}),
                root.clone(),
                None
            )
            .as_deref(),
            Some("123")
        );
        assert_eq!(
            key(
                json!({"reference": "Patient/123"}),
                root.clone(),
                Some("Group")
            ),
            None
        );
        assert_eq!(
            key(json!({"reference": "#p1"}), root.clone(), Some("Patient")).as_deref(),
            Some("p1")
        );
        assert_eq!(
            key(json!({"reference": "#p1"}), root.clone(), Some("Group")),
            None
        );
        assert_eq!(
            key(
                json!({"reference": "http://x/fhir/Patient/1"}),
                root.clone(),
                None
            ),
            None
        );
        assert_eq!(
            key(json!({"reference": "urn:uuid:1"}), root.clone(), None),
            None
        );
        assert_eq!(key(json!({"display": "no reference"}), root, None), None);
    }
}
