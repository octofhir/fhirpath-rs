//! Is function implementation
//!
//! The is function checks if a value is of a specific type.
//! Syntax: value.is(TypeName)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Is function evaluator
pub struct IsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl IsFunctionEvaluator {
    /// Create a new is function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
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
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Check if a value type inherits from or is compatible with the target type
    fn is_type_compatible(&self, value: &FhirPathValue, target_type: &str) -> bool {
        let type_info = value.type_info();
        let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);

        // Direct type match
        if actual_type == target_type {
            return true;
        }

        // Check inheritance relationships according to FHIR type hierarchy
        match target_type {
            "string" | "String" | "System.String" => {
                matches!(actual_type, "string" | "code" | "id" | "uri" | "url" | "canonical" | "oid" | "uuid" | "markdown" | "base64Binary" | "xhtml")
            }
            "code" | "Code" => {
                matches!(actual_type, "code")
            }
            "id" | "Id" => {
                matches!(actual_type, "id")
            }
            "uri" | "Uri" => {
                matches!(actual_type, "uri" | "url" | "canonical")
            }
            "url" | "Url" => {
                matches!(actual_type, "url")
            }
            "canonical" | "Canonical" => {
                matches!(actual_type, "canonical")
            }
            "Boolean" | "boolean" | "System.Boolean" => {
                matches!(value, FhirPathValue::Boolean(_, _, _))
            }
            "Integer" | "integer" | "System.Integer" => {
                matches!(value, FhirPathValue::Integer(_, _, _))
            }
            "Decimal" | "decimal" | "System.Decimal" => {
                matches!(value, FhirPathValue::Decimal(_, _, _))
            }
            "Date" | "date" | "System.Date" => {
                matches!(value, FhirPathValue::Date(_, _, _))
            }
            "DateTime" | "dateTime" | "System.DateTime" => {
                matches!(value, FhirPathValue::DateTime(_, _, _))
            }
            "Time" | "time" | "System.Time" => {
                matches!(value, FhirPathValue::Time(_, _, _))
            }
            "Quantity" | "quantity" | "System.Quantity" => {
                // Check for direct Quantity type
                if matches!(value, FhirPathValue::Quantity { .. }) {
                    return true;
                }
                // Check for FHIR types that inherit from Quantity: Age, Count, Distance, Duration, Money
                matches!(actual_type, "Quantity" | "Age" | "Count" | "Distance" | "Duration" | "Money")
            }
            _ => {
                // For other types, check exact match
                actual_type == target_type
            }
        }
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for IsFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "is function requires exactly one argument".to_string(),
            ));
        }

        // Extract the type name from the AST node (can be identifier or string literal)
        let type_name = match &args[0] {
            ExpressionNode::Literal(literal_node) => {
                match &literal_node.value {
                    crate::ast::literal::LiteralValue::String(s) => s.clone(),
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0055,
                            "Type argument must be a string literal".to_string(),
                        ));
                    }
                }
            }
            ExpressionNode::Identifier(identifier_node) => {
                // Accept identifier as type name (e.g., String, Integer)
                identifier_node.name.clone()
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "Type argument must be a type name or string literal".to_string(),
                ));
            }
        };

        let mut results = Vec::new();

        for value in input {
            let is_type = self.is_type_compatible(&value, &type_name);
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