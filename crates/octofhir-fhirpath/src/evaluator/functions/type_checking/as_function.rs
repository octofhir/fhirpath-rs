//! As function implementation
//!
//! The as function attempts to cast a value to a specific type.
//! Syntax: value.as(TypeName)

use async_trait::async_trait;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// As function evaluator for type casting
pub struct AsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AsFunctionEvaluator {
    /// Create a new as function evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self::new())
    }

    /// Check if a type name is valid using ModelProvider
    async fn is_valid_type_name(&self, type_name: &str, context: &EvaluationContext) -> bool {
        let model_provider = context.model_provider();

        // Check if it's a primitive type
        if let Ok(primitive_types) = model_provider.get_primitive_types().await {
            if primitive_types.contains(&type_name.to_string()) {
                return true;
            }
        }

        // Check if it's a complex type
        if let Ok(complex_types) = model_provider.get_complex_types().await {
            if complex_types.contains(&type_name.to_string()) {
                return true;
            }
        }

        false
    }

    /// Perform strict type casting using ModelProvider
    fn strict_type_cast(&self, value: &FhirPathValue, target_type: &str, _context: &EvaluationContext) -> Option<FhirPathValue> {
        // Perform actual type conversions for as() function
        match target_type {
            "string" | "String" | "System.String" => {
                // Convert various types to string
                match value {
                    FhirPathValue::String(_, _, _) => Some(value.clone()),
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
                    FhirPathValue::Integer(i, _, _) => Some(FhirPathValue::decimal(rust_decimal::Decimal::from(*i))),
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
                    FhirPathValue::String(s, _, _) => {
                        match s.to_lowercase().as_str() {
                            "true" => Some(FhirPathValue::boolean(true)),
                            "false" => Some(FhirPathValue::boolean(false)),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => {
                // For other types, only allow exact type matches (strict casting)
                let type_info = value.type_info();
                let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                if actual_type == target_type {
                    Some(value.clone())
                } else {
                    None
                }
            }
        }
    }
}

#[async_trait]
impl FunctionEvaluator for AsFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
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

        // Extract the type name from the AST node (can be identifier or string literal)
        let type_name = match &args[0] {
            ExpressionNode::Literal(literal_node) => {
                match &literal_node.value {
                    crate::ast::literal::LiteralValue::String(s) => s.clone(),
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0055,
                            "as function type argument must be a string literal or identifier",
                        ));
                    }
                }
            }
            ExpressionNode::Identifier(identifier_node) => {
                // Accept identifier as type name (e.g., String, code, Patient)
                identifier_node.name.clone()
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "as function type argument must be a type identifier or string literal",
                ));
            }
        };

        // Validate the type name - reject invalid types like "string1"
        if !self.is_valid_type_name(&type_name, context).await {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                format!("Invalid type name: {}", type_name),
            ));
        }

        // Process input collection
        let mut casted_items = Vec::new();

        for item in input {
            // Attempt strict type casting using ModelProvider
            if let Some(casted_value) = self.strict_type_cast(&item, &type_name, context) {
                casted_items.push(casted_value);
            }
            // If casting fails, the item is not included in the result (empty collection)
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
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        category: FunctionCategory::Utility,
        requires_terminology: false,
        requires_model: true, // Requires ModelProvider for strict schema-based type casting
    }
}