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

        // Direct name match
        if actual_name == target_name {
            return true;
        }

        // If target has no namespace, check both FHIR and System namespaces
        if target_namespace.is_empty() {
            if actual_name == target_name {
                return true;
            }
            // Also check case variations
            if actual_name.to_lowercase() == target_name.to_lowercase() {
                return true;
            }
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

        // Specific namespace/name matches and primitive type matches
        match target_type {
            "Boolean" | "boolean" | "System.Boolean" | "FHIR.boolean" => {
                matches!(value, FhirPathValue::Boolean(_, _, _))
            }
            "Integer" | "integer" | "System.Integer" | "FHIR.integer" => {
                matches!(value, FhirPathValue::Integer(_, _, _))
            }
            "Decimal" | "decimal" | "System.Decimal" | "FHIR.decimal" => {
                matches!(value, FhirPathValue::Decimal(_, _, _))
            }
            "String" | "string" | "System.String" | "FHIR.string" => {
                matches!(value, FhirPathValue::String(_, _, _))
            }
            "Date" | "date" | "System.Date" | "FHIR.date" => {
                matches!(value, FhirPathValue::Date(_, _, _))
            }
            "DateTime" | "dateTime" | "System.DateTime" | "FHIR.dateTime" => {
                matches!(value, FhirPathValue::DateTime(_, _, _))
            }
            "Time" | "time" | "System.Time" | "FHIR.time" => {
                matches!(value, FhirPathValue::Time(_, _, _))
            }
            "Quantity" | "quantity" | "System.Quantity" => {
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
