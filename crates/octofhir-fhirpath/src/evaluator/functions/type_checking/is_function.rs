//! Is function implementation
//!
//! The is function checks if a value is of a specific type.
//! Syntax: value.is(TypeName)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Is function evaluator
pub struct IsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl IsFunctionEvaluator {
    /// Create a new is function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "is".to_string(),
                description: "Tests if the input is of a specified type".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "type".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The type to check against".to_string(),
                        default_value: None,
                    }],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
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

    /// Check if a value type inherits from or is compatible with the target type
    fn is_type_compatible(
        &self,
        value: &FhirPathValue,
        target_type: &str,
        model_provider: &dyn crate::core::ModelProvider,
    ) -> bool {
        // Get the actual type namespace and name using the same logic as type function
        let (actual_namespace, actual_name) = self.get_actual_type_info(value);
        let full_actual_type = format!("{actual_namespace}.{actual_name}");

        // Parse target type
        let (target_namespace, target_name) = if target_type.contains('.') {
            let parts: Vec<&str> = target_type.split('.').collect();
            if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("System".to_string(), target_type.to_string())
            }
        } else {
            // For unqualified types, try both FHIR and System namespaces
            ("".to_string(), target_type.to_string())
        };

        // Direct full type match
        if full_actual_type == target_type {
            return true;
        }

        // Direct name match only when target has no namespace
        if target_namespace.is_empty() && actual_name == target_name {
            return true;
        }

        // If target has no namespace, we already handled exact name match above.
        // Do NOT perform namespace-insensitive matching between 'Boolean' and 'boolean' or
        // 'System.Patient' vs 'FHIR.Patient'. Namespaces must match when provided.

        // If target has an explicit namespace that differs from actual, it's not compatible
        if !target_namespace.is_empty() && target_namespace != actual_namespace {
            return false;
        }

        // First check inheritance using model provider for all types
        let result1 = model_provider.is_type_derived_from(&actual_name, &target_name);
        let result2 = model_provider.is_type_derived_from(&actual_name, target_type);
        let result3 = model_provider.is_type_derived_from(&full_actual_type, target_type);

        if result1 || result2 || result3 {
            return true;
        }

        // Workaround: Check FHIR R5 quantity type hierarchy since FhirSchemaModelProvider doesn't use TYPE_MAPPING for inheritance
        if target_type == "Quantity" || target_name == "Quantity" {
            match actual_name.as_str() {
                "Age" | "Count" | "Distance" | "Duration" | "Money" | "SimpleQuantity" => {
                    return true;
                }
                _ => {}
            }
        }

        // Specific namespace/name matches and primitive type matches (namespace-aware)
        match target_type {
            // System primitives (uppercase)
            "Boolean" | "System.Boolean" => {
                matches!(
                    value,
                    FhirPathValue::Boolean(_, type_info, _) if type_info.namespace.as_deref() != Some("FHIR")
                )
            }
            "Integer" | "System.Integer" => {
                matches!(
                    value,
                    FhirPathValue::Integer(_, type_info, _) if type_info.namespace.as_deref() != Some("FHIR")
                )
            }
            "Decimal" | "System.Decimal" => {
                matches!(
                    value,
                    FhirPathValue::Decimal(_, type_info, _) if type_info.namespace.as_deref() != Some("FHIR")
                )
            }
            "String" | "System.String" => {
                // System.String: any non-FHIR (System) string
                matches!(
                    value,
                    FhirPathValue::String(_, type_info, _) if type_info.namespace.as_deref() != Some("FHIR")
                )
            }
            "Date" | "System.Date" => {
                matches!(
                    value,
                    FhirPathValue::Date(_, type_info, _) if type_info.namespace.as_deref() != Some("FHIR")
                )
            }
            "DateTime" | "System.DateTime" => {
                matches!(
                    value,
                    FhirPathValue::DateTime(_, type_info, _) if type_info.namespace.as_deref() != Some("FHIR")
                )
            }
            "Time" | "System.Time" => {
                matches!(
                    value,
                    FhirPathValue::Time(_, type_info, _) if type_info.namespace.as_deref() != Some("FHIR")
                )
            }

            // FHIR primitives (lowercase)
            "boolean" | "FHIR.boolean" => {
                matches!(
                    value,
                    FhirPathValue::Boolean(_, type_info, _) if type_info.namespace.as_deref() == Some("FHIR")
                )
            }
            "integer" | "FHIR.integer" => {
                matches!(
                    value,
                    FhirPathValue::Integer(_, type_info, _) if type_info.namespace.as_deref() == Some("FHIR")
                )
            }
            "decimal" | "FHIR.decimal" => {
                matches!(
                    value,
                    FhirPathValue::Decimal(_, type_info, _) if type_info.namespace.as_deref() == Some("FHIR")
                )
            }
            "string" | "FHIR.string" => {
                match value {
                    FhirPathValue::String(_, type_info, _)
                        if type_info.namespace.as_deref() == Some("FHIR") =>
                    {
                        let actual = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                        matches!(
                            actual,
                            // string itself
                            "string" | "FHIR.string"
                            // string-derived FHIR primitives
                            | "code" | "id" | "markdown"
                            | "uri" | "url" | "canonical" | "oid" | "uuid"
                        )
                    }
                    _ => false,
                }
            }
            "date" | "FHIR.date" => {
                matches!(
                    value,
                    FhirPathValue::Date(_, type_info, _) if type_info.namespace.as_deref() == Some("FHIR")
                )
            }
            "dateTime" | "FHIR.dateTime" => {
                matches!(
                    value,
                    FhirPathValue::DateTime(_, type_info, _) if type_info.namespace.as_deref() == Some("FHIR")
                )
            }
            "time" | "FHIR.time" => {
                matches!(
                    value,
                    FhirPathValue::Time(_, type_info, _) if type_info.namespace.as_deref() == Some("FHIR")
                )
            }

            // FHIR.uri is a supertype for uri-like primitives
            "uri" | "FHIR.uri" => match value {
                FhirPathValue::String(_, type_info, _)
                    if type_info.namespace.as_deref() == Some("FHIR") =>
                {
                    let actual = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                    matches!(actual, "uri" | "url" | "canonical" | "uuid" | "oid")
                }
                _ => false,
            },

            // Quantity (modeled as System type in our implementation)
            "Quantity" | "System.Quantity" => {
                matches!(value, FhirPathValue::Quantity { .. })
            }
            _ => false,
        }
    }

    /// Get actual type info using the same logic as the type function
    fn get_actual_type_info(&self, value: &FhirPathValue) -> (String, String) {
        match value {
            FhirPathValue::Boolean(_, type_info, _) => {
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
                    // Use the specific FHIR primitive name when available (e.g., code, uri)
                    let actual = type_info
                        .name
                        .as_deref()
                        .unwrap_or(&type_info.type_name)
                        .to_string();
                    ("FHIR".to_string(), actual)
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
                let name = type_info
                    .name
                    .as_deref()
                    .unwrap_or(&type_info.type_name)
                    .to_string();
                ("FHIR".to_string(), name)
            }
            FhirPathValue::Collection(_) => ("System".to_string(), "Collection".to_string()),
            FhirPathValue::Empty => ("System".to_string(), "Empty".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for IsFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "is function requires exactly one argument".to_string(),
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

        let mut results = Vec::new();

        for value in input {
            let is_type = self.is_type_compatible(&value, &type_name, &**context.model_provider());
            results.push(FhirPathValue::boolean(is_type));
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
