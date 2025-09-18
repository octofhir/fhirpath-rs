//! ofType function implementation
//!
//! The ofType function returns a collection that contains all items in the input
//! collection that are of the given type or a subclass thereof.
//! Per FHIRPath spec: "ofType(type : type specifier) : collection"

use async_trait::async_trait;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// ofType function evaluator for filtering collections by type
pub struct OfTypeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl OfTypeFunctionEvaluator {
    /// Create a new ofType function evaluator
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
}

#[async_trait]
impl FunctionEvaluator for OfTypeFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        // Spec: ofType(type) filters the input collection to only items of the specified type

        // Check argument count
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0062,
                "ofType function requires exactly one argument",
            ));
        }

        // Extract the type name from the AST node (can be identifier or string literal)
        let type_name = match &args[0] {
            ExpressionNode::Literal(literal_node) => {
                match &literal_node.value {
                    crate::ast::literal::LiteralValue::String(s) => s.clone(),
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0062,
                            "ofType function type argument must be a string literal or identifier",
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
                    crate::core::error_code::FP0062,
                    "ofType function type argument must be a type identifier or string literal",
                ));
            }
        };

        // Validate the type name - reject invalid types like "string1"
        if !self.is_valid_type_name(&type_name, context).await {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0062,
                format!("Invalid type name: {}", type_name),
            ));
        }

        // Filter the input collection based on type matching
        // For basic primitive types, also allow conversion (similar to as() function)
        let mut filtered_items = Vec::new();

        for item in input {
            // For primitive types, prioritize conversion logic over ModelProvider
            // This handles cases like ofType(string) with integer values
            let converted_value = match type_name.as_str() {
                "string" | "String" | "System.String" => {
                    match &item {
                        FhirPathValue::String(_, _, _) => Some(item.clone()),
                        FhirPathValue::Integer(i, _, _) => Some(FhirPathValue::string(i.to_string())),
                        FhirPathValue::Decimal(d, _, _) => Some(FhirPathValue::string(d.to_string())),
                        FhirPathValue::Boolean(b, _, _) => Some(FhirPathValue::string(b.to_string())),
                        _ => None,
                    }
                }
                "integer" | "Integer" | "System.Integer" => {
                    match &item {
                        FhirPathValue::Integer(_, _, _) => Some(item.clone()),
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
                    match &item {
                        FhirPathValue::Decimal(_, _, _) => Some(item.clone()),
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
                    match &item {
                        FhirPathValue::Boolean(_, _, _) => Some(item.clone()),
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
                    // For complex types, fall back to ModelProvider check
                    let item_type_info = item.type_info();
                    if context
                        .model_provider()
                        .of_type(&item_type_info, &type_name)
                        .is_some()
                    {
                        Some(item.clone())
                    } else {
                        None
                    }
                }
            };

            if let Some(converted) = converted_value {
                filtered_items.push(converted);
            }
        }

        Ok(EvaluationResult {
            value: Collection::from_values(filtered_items),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// Create metadata for the ofType function
fn create_metadata() -> FunctionMetadata {
    FunctionMetadata {
        name: "ofType".to_string(),
        description: "Filter collection to items of specified type or subclass thereof".to_string(),
        signature: FunctionSignature {
            input_type: "Collection".to_string(),
            parameters: vec![FunctionParameter {
                name: "type".to_string(),
                parameter_type: vec!["String".to_string()],
                optional: false,
                is_expression: false,
                description: "Type identifier or string literal to filter by".to_string(),
                default_value: None,
            }],
            return_type: "Collection".to_string(),
            polymorphic: true,
            min_params: 1,
            max_params: Some(1),
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        category: FunctionCategory::FilteringProjection,
        requires_terminology: false,
        requires_model: true, // Requires ModelProvider for strict schema-based type checking
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;
    use crate::evaluator::AsyncNodeEvaluator;
    use crate::parser::ExpressionNode;

    async fn create_test_evaluator() -> AsyncNodeEvaluator<'static> {
        // This is a stub for tests - in real usage this would be properly constructed
        unsafe { std::mem::zeroed() }
    }

    #[tokio::test]
    async fn test_of_type_metadata() {
        let evaluator = OfTypeFunctionEvaluator::new();
        let metadata = evaluator.metadata();

        assert_eq!(metadata.name, "ofType");
        assert_eq!(metadata.signature.parameters.len(), 1);
        assert_eq!(metadata.signature.parameters[0].name, "type");
    }
}
