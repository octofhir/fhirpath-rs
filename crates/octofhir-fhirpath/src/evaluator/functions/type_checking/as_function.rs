//! As function implementation
//!
//! The as function attempts to cast a value to a specific type.
//! Syntax: value.as(TypeName)

use async_trait::async_trait;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// As function evaluator for type casting
pub struct AsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl Default for AsFunctionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl AsFunctionEvaluator {
    /// Create a new as function evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self::new())
    }

    /// Check if a type name is valid using ModelProvider
    async fn is_valid_type_name(&self, type_name: &str, context: &EvaluationContext) -> bool {
        crate::core::model_provider::utils::type_exists(
            context.model_provider().as_ref(),
            type_name,
        )
        .await
    }

    /// Perform strict type casting using ModelProvider
    fn strict_type_cast(
        &self,
        value: &FhirPathValue,
        target_type: &str,
        _context: &EvaluationContext,
    ) -> Option<FhirPathValue> {
        // Perform actual type conversions for as() function
        match target_type {
            "string" | "String" | "System.String" => {
                // Only allow strict string type conversion
                match value {
                    FhirPathValue::String(_, type_info, _) => {
                        // For FHIR primitives, only allow actual FHIR.string to be treated as string
                        if type_info.namespace.as_deref() == Some("FHIR") {
                            let actual = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                            if actual.eq_ignore_ascii_case("string") {
                                Some(value.clone())
                            } else {
                                None // Cannot cast FHIR.code/FHIR.uri/etc to string via as(string)
                            }
                        } else {
                            // System.String is allowed
                            Some(value.clone())
                        }
                    }
                    FhirPathValue::Integer(i, _, _) => Some(FhirPathValue::string(i.to_string())),
                    FhirPathValue::Decimal(d, _, _) => Some(FhirPathValue::string(d.to_string())),
                    FhirPathValue::Boolean(b, _, _) => Some(FhirPathValue::string(b.to_string())),
                    _ => None, // Cannot convert other types to string
                }
            }
            "integer" | "Integer" | "System.Integer" => {
                // Convert to integer
                match value {
                    FhirPathValue::Integer(_, _, _) => Some(value.clone()),
                    FhirPathValue::String(s, _, _) => {
                        if let Ok(i) = s.parse::<i64>() {
                            Some(FhirPathValue::integer(i))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            "decimal" | "Decimal" | "System.Decimal" => {
                // Convert to decimal
                match value {
                    FhirPathValue::Decimal(_, _, _) => Some(value.clone()),
                    FhirPathValue::Integer(i, _, _) => {
                        Some(FhirPathValue::decimal(rust_decimal::Decimal::from(*i)))
                    }
                    FhirPathValue::String(s, _, _) => {
                        if let Ok(d) = s.parse::<rust_decimal::Decimal>() {
                            Some(FhirPathValue::decimal(d))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            "boolean" | "Boolean" | "System.Boolean" => {
                // Convert to boolean
                match value {
                    FhirPathValue::Boolean(_, _, _) => Some(value.clone()),
                    FhirPathValue::String(s, _, _) => match s.to_lowercase().as_str() {
                        "true" => Some(FhirPathValue::boolean(true)),
                        "false" => Some(FhirPathValue::boolean(false)),
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => {
                // For complex/resource types: allow exact match or inheritance per ModelProvider
                let type_info = value.type_info();
                let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                // Strip namespace from target type for comparison
                let base_target = if let Some(dot_pos) = target_type.rfind('.') {
                    &target_type[dot_pos + 1..]
                } else {
                    target_type
                };

                // Polymorphic elements (e.g. Bundle.entry.resource) give resources a
                // generic static TypeInfo ("Resource"); the concrete type lives in the
                // JSON `resourceType` field, so check that too.
                if let FhirPathValue::Resource(node, _, _) = value
                    && let Some(resource_type) = node
                        .get("resourceType")
                        .and_then(crate::core::node::FhirNode::as_str)
                    && (resource_type == target_type
                        || resource_type == base_target
                        || _context
                            .model_provider()
                            .is_type_derived_from(resource_type, base_target))
                {
                    return Some(value.clone());
                }

                if actual_type == target_type || actual_type == base_target {
                    Some(value.clone())
                } else {
                    // Use model provider inheritance checks when possible
                    let provider = _context.model_provider();
                    if provider.is_type_derived_from(actual_type, base_target)
                        || provider.is_type_derived_from(actual_type, target_type)
                    {
                        Some(value.clone())
                    } else {
                        None
                    }
                }
            }
        }
    }
}

#[async_trait]
impl LazyFunctionEvaluator for AsFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        // Check argument count
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "as function requires exactly one argument",
            ));
        }

        // Extract type identifier from AST node directly instead of evaluating
        let type_name = match &args[0] {
            ExpressionNode::Identifier(identifier_node) => identifier_node.name.clone(),
            ExpressionNode::PropertyAccess(property_access_node) => {
                // Handle property access like FHIR.boolean, System.Boolean
                if let ExpressionNode::Identifier(base_identifier) =
                    property_access_node.object.as_ref()
                {
                    format!("{}.{}", base_identifier.name, property_access_node.property)
                } else {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        format!(
                            "Type argument must be an identifier or property access, got {:?}",
                            args[0]
                        ),
                    ));
                }
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "Type argument must be an identifier or property access, got {:?}",
                        args[0]
                    ),
                ));
            }
        };

        // Validate the type name - reject invalid types like "string1"
        if !self.is_valid_type_name(&type_name, context).await {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                format!("Invalid type name: {type_name}"),
            ));
        }

        // FHIR's own invariants (notably `dom-3`, `dom-4`) apply `as()` to
        // multi-item collections like `descendants().as(canonical)`. Apply
        // the cast per-item, equivalent to `ofType()` on a collection: items
        // that cast successfully are kept, others are dropped.
        let mut casted_items = Vec::with_capacity(input.len());
        for item in input {
            if let Some(casted_value) = self.strict_type_cast(&item, &type_name, context) {
                casted_items.push(casted_value);
            }
        }

        Ok(EvaluationResult {
            value: Collection::from_values(casted_items),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// Create metadata for the as function
fn create_metadata() -> FunctionMetadata {
    FunctionMetadata {
        name: "as".to_string(),
        description: "Cast value to specified type".to_string(),
        signature: FunctionSignature {
            input_type: "Any".to_string(),
            parameters: vec![FunctionParameter {
                name: "type".to_string(),
                parameter_type: vec!["String".to_string()],
                optional: false,
                is_expression: false,
                description: "Type identifier or string literal to cast to".to_string(),
                default_value: None,
            }],
            return_type: "Any".to_string(),
            polymorphic: true,
            min_params: 1,
            max_params: Some(1),
        },
        argument_evaluation: ArgumentEvaluationStrategy::Current,
        null_propagation: NullPropagationStrategy::Focus,
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        category: FunctionCategory::Utility,
        requires_terminology: false,
        requires_model: true, // Requires ModelProvider for strict schema-based type casting
    }
}
