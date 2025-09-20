//! AST enhancement utilities for FHIRPath expressions
//!
//! This module provides shared functionality for enhancing AST nodes with type information
//! using ModelProvider and FunctionRegistry. Used by both server endpoints and CLI commands.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::future::Future;
use std::pin::Pin;

/// AST node format with type information, compatible with FHIRPath Lab tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    #[serde(rename = "ExpressionType")]
    pub expression_type: String,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Arguments", skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<AstNode>>,

    #[serde(rename = "ReturnType", skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,

    #[serde(rename = "Position", skip_serializing_if = "Option::is_none")]
    pub position: Option<usize>,

    #[serde(rename = "Length", skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
}

/// Extract resource type from a FHIR resource JSON
pub fn extract_resource_type(resource: &JsonValue) -> Option<String> {
    resource
        .get("resourceType")
        .and_then(|rt| rt.as_str())
        .map(|s| s.to_string())
}

/// Add type information to AST using ModelProvider and FunctionRegistry
pub fn add_type_information<'a>(
    mut ast_node: AstNode,
    original_ast: &'a octofhir_fhirpath::ast::ExpressionNode,
    model_provider: &'a dyn octofhir_fhirpath::ModelProvider,
    function_registry: Option<&'a octofhir_fhirpath::FunctionRegistry>,
    base_type: Option<&'a str>,
) -> Pin<Box<dyn Future<Output = Result<AstNode, Box<dyn std::error::Error>>> + Send + 'a>> {
    Box::pin(async move {
        use octofhir_fhirpath::ast::*;

        match original_ast {
            ExpressionNode::PropertyAccess(node) => {
                // First enhance the object recursively
                if let Some(ref mut args) = ast_node.arguments {
                    if let Some(object_arg) = args.get_mut(0) {
                        let enhanced_object = add_type_information(
                            object_arg.clone(),
                            &node.object,
                            model_provider,
                            function_registry,
                            base_type,
                        )
                        .await?;

                        // Get the object's return type to use as the base for property lookup
                        let object_type = enhanced_object
                            .return_type
                            .as_ref()
                            .map(|t| t.trim_end_matches("[]")) // Remove array notation if present
                            .unwrap_or(base_type.unwrap_or("Patient")); // Default to Patient if unknown

                        // Now infer the property type using ModelProvider
                        let property_type = infer_property_access_type_async(
                            object_type,
                            &node.property,
                            model_provider,
                        )
                        .await?;

                        // Update the object and set return type
                        args[0] = enhanced_object;
                        ast_node.return_type = property_type;
                    }
                }
            }
            ExpressionNode::FunctionCall(node) => {
                // Enhance function call arguments recursively
                if let Some(ref mut args) = ast_node.arguments {
                    for (i, arg_ast) in node.arguments.iter().enumerate() {
                        if let Some(ast_arg) = args.get_mut(i) {
                            let enhanced_arg = add_type_information(
                                ast_arg.clone(),
                                arg_ast,
                                model_provider,
                                function_registry,
                                base_type,
                            )
                            .await?;
                            args[i] = enhanced_arg;
                        }
                    }
                }

                // Get return type from function registry if available
                if let Some(registry) = function_registry {
                    // FunctionRegistry is currently a placeholder
                    // FunctionRegistry is currently a placeholder
                    // Type inference would be implemented here when registry is complete
                    let _ = registry; // Suppress unused variable warning
                }
            }
            ExpressionNode::MethodCall(node) => {
                // Enhance method call object and arguments recursively
                if let Some(ref mut args) = ast_node.arguments {
                    if let Some(object_arg) = args.get_mut(0) {
                        let enhanced_object = add_type_information(
                            object_arg.clone(),
                            &node.object,
                            model_provider,
                            function_registry,
                            base_type,
                        )
                        .await?;
                        args[0] = enhanced_object;
                    }

                    // Enhance method arguments
                    for (i, method_arg) in node.arguments.iter().enumerate() {
                        if let Some(ast_arg) = args.get_mut(i + 1) {
                            // +1 because object is first
                            let enhanced_arg = add_type_information(
                                ast_arg.clone(),
                                method_arg,
                                model_provider,
                                function_registry,
                                base_type,
                            )
                            .await?;
                            args[i + 1] = enhanced_arg;
                        }
                    }
                }

                // Get return type from function registry if available with enhanced type inference
                if let Some(registry) = function_registry {
                    // FunctionRegistry is currently a placeholder
                    // Enhanced method call type inference would be implemented here
                    let _ = registry; // Suppress unused variable warning
                }
            }
            ExpressionNode::BinaryOperation(node) => {
                // Enhance binary operation operands
                if let Some(ref mut args) = ast_node.arguments {
                    if args.len() >= 2 {
                        let enhanced_left = add_type_information(
                            args[0].clone(),
                            &node.left,
                            model_provider,
                            function_registry,
                            base_type,
                        )
                        .await?;
                        let enhanced_right = add_type_information(
                            args[1].clone(),
                            &node.right,
                            model_provider,
                            function_registry,
                            base_type,
                        )
                        .await?;
                        args[0] = enhanced_left;
                        args[1] = enhanced_right;
                    }
                }

                // Set return type based on operator
                use octofhir_fhirpath::ast::BinaryOperator;
                ast_node.return_type = match node.operator {
                    BinaryOperator::Union => {
                        // Union returns collection of left type
                        if let Some(ref args) = ast_node.arguments {
                            if let Some(left_arg) = args.first() {
                                left_arg
                                    .return_type
                                    .clone()
                                    .map(|t| format!("{}[]", t.trim_end_matches("[]")))
                            } else {
                                Some("Collection".to_string())
                            }
                        } else {
                            Some("Collection".to_string())
                        }
                    }
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::LessThan
                    | BinaryOperator::GreaterThanOrEqual
                    | BinaryOperator::LessThanOrEqual => Some("boolean".to_string()),
                    BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor => {
                        Some("boolean".to_string())
                    }
                    BinaryOperator::Add
                    | BinaryOperator::Subtract
                    | BinaryOperator::Multiply
                    | BinaryOperator::Divide => {
                        // For arithmetic operations, return type depends on operands
                        // For now, return generic number type
                        Some("decimal".to_string())
                    }
                    BinaryOperator::Concatenate => Some("string".to_string()),
                    _ => None, // For other operators, let type inference handle it
                };
            }
            ExpressionNode::Identifier(_) => {
                // Identifier (like $this) returns the base type
                ast_node.return_type = base_type.map(|t| t.to_string());
            }
            ExpressionNode::Literal(node) => {
                // Set return type based on literal value type
                use octofhir_fhirpath::ast::LiteralValue;
                ast_node.return_type = match &node.value {
                    LiteralValue::String(_) => Some("string".to_string()),
                    LiteralValue::Integer(_) => Some("integer".to_string()),
                    LiteralValue::Decimal(_) => Some("decimal".to_string()),
                    LiteralValue::Boolean(_) => Some("boolean".to_string()),
                    LiteralValue::Date(_) => Some("date".to_string()),
                    LiteralValue::DateTime(_) => Some("dateTime".to_string()),
                    LiteralValue::Time(_) => Some("time".to_string()),
                    LiteralValue::Quantity { .. } => Some("Quantity".to_string()),
                };
            }
            ExpressionNode::Variable(_) => {
                // Variable reference - type depends on what was assigned to the variable
                // For now, we'll leave it untyped as variables can hold any type
                ast_node.return_type = None;
            }
            ExpressionNode::IndexAccess(node) => {
                // Index access returns single item from collection
                if let Some(ref mut args) = ast_node.arguments {
                    if let Some(object_arg) = args.get_mut(0) {
                        let enhanced_object = add_type_information(
                            object_arg.clone(),
                            &node.object,
                            model_provider,
                            function_registry,
                            base_type,
                        )
                        .await?;
                        // Index access removes array notation - get type before moving
                        let return_type = enhanced_object
                            .return_type
                            .as_ref()
                            .map(|t| t.trim_end_matches("[]").to_string());

                        args[0] = enhanced_object;
                        ast_node.return_type = return_type;
                    }
                }
            }
            ExpressionNode::UnaryOperation(node) => {
                // Enhance operand and set return type based on operator
                if let Some(ref mut args) = ast_node.arguments {
                    if let Some(operand_arg) = args.get_mut(0) {
                        let enhanced_operand = add_type_information(
                            operand_arg.clone(),
                            &node.operand,
                            model_provider,
                            function_registry,
                            base_type,
                        )
                        .await?;
                        // Set return type based on unary operator - get type before moving
                        use octofhir_fhirpath::ast::UnaryOperator;
                        let operand_return_type = enhanced_operand.return_type.clone();

                        args[0] = enhanced_operand;

                        ast_node.return_type = match node.operator {
                            UnaryOperator::Not => Some("boolean".to_string()),
                            UnaryOperator::Negate | UnaryOperator::Positive => {
                                // Arithmetic unary operators preserve numeric type
                                operand_return_type
                            }
                        };
                    }
                }
            }
            ExpressionNode::Collection(node) => {
                // Collection of items - enhance all children
                if let Some(ref mut args) = ast_node.arguments {
                    for (i, item_ast) in node.elements.iter().enumerate() {
                        if let Some(ast_arg) = args.get_mut(i) {
                            let enhanced_item = add_type_information(
                                ast_arg.clone(),
                                item_ast,
                                model_provider,
                                function_registry,
                                base_type,
                            )
                            .await?;
                            args[i] = enhanced_item;
                        }
                    }
                }
                // Collection return type is array of first item's type or generic collection
                if let Some(ref args) = ast_node.arguments {
                    if let Some(first_item) = args.first() {
                        ast_node.return_type = first_item
                            .return_type
                            .clone()
                            .map(|t| format!("{}[]", t.trim_end_matches("[]")));
                    } else {
                        ast_node.return_type = Some("Collection".to_string());
                    }
                } else {
                    ast_node.return_type = Some("Collection".to_string());
                }
            }
            ExpressionNode::Parenthesized(expr) => {
                // Parenthesized expressions have same type as inner expression
                if let Some(ref mut args) = ast_node.arguments {
                    if let Some(inner_arg) = args.get_mut(0) {
                        let enhanced_inner = add_type_information(
                            inner_arg.clone(),
                            expr,
                            model_provider,
                            function_registry,
                            base_type,
                        )
                        .await?;
                        args[0] = enhanced_inner.clone();
                        ast_node.return_type = enhanced_inner.return_type;
                    }
                }
            }
            _ => {
                // For remaining node types (TypeCast, Filter, Union, TypeCheck, Path, Lambda, etc.)
                // recursively enhance children if they exist
                if let Some(ref mut args) = ast_node.arguments {
                    for _arg in args.iter() {
                        // Note: We'd need access to the original AST children to properly enhance these
                        // For now, we'll leave them as-is since we don't have a direct mapping
                    }
                }
                // These complex node types will need specialized handling if needed
                ast_node.return_type = None;
            }
        }

        Ok(ast_node)
    })
}

/// Helper function to infer FHIR type for property access using ModelProvider
async fn infer_property_access_type_async(
    _object_type: &str,
    _property_name: &str,
    _model_provider: &dyn octofhir_fhirpath::ModelProvider,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Use ModelProvider navigation to get correct type information (same as metadata_navigator)
    // navigate_typed_path method no longer available in new ModelProvider API
    // Return None for now - type inference not supported with simplified provider
    Ok(None)
}

/// Convert Rust AST to FHIRPath Lab format with type information
pub fn convert_ast_to_lab_format(
    ast: &octofhir_fhirpath::ast::ExpressionNode,
    function_registry: Option<&octofhir_fhirpath::FunctionRegistry>,
    model_provider: Option<&dyn octofhir_fhirpath::ModelProvider>,
) -> AstNode {
    use octofhir_fhirpath::ast::*;

    match ast {
        ExpressionNode::Identifier(node) => {
            // For simple identifiers, use AxisExpression with "builtin.that"
            AstNode {
                expression_type: "AxisExpression".to_string(),
                name: "builtin.that".to_string(),
                arguments: None,
                return_type: None,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::PropertyAccess(node) => {
            let object_arg =
                convert_ast_to_lab_format(&node.object, function_registry, model_provider);

            AstNode {
                expression_type: "ChildExpression".to_string(),
                name: node.property.clone(),
                arguments: Some(vec![object_arg]),
                return_type: None, // Will be filled by enhancement
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::FunctionCall(node) => {
            let mut args = vec![];
            for arg in &node.arguments {
                args.push(convert_ast_to_lab_format(
                    arg,
                    function_registry,
                    model_provider,
                ));
            }

            // Get return type from function registry if available
            let return_type = if let Some(_registry) = function_registry {
                // Query the function registry for the return type
                // FunctionRegistry is currently a placeholder
                if false {
                    // let Some(function_info) = registry.get_function_metadata(&node.name) {
                    // Convert the function's return type to FHIRPath Lab format
                    // Some(function_info.signature.returns.display_name())
                    None
                } else {
                    None
                }
            } else {
                None
            };

            AstNode {
                expression_type: "FunctionCallExpression".to_string(),
                name: node.name.clone(),
                arguments: if args.is_empty() { None } else { Some(args) },
                return_type,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::Literal(node) => {
            use octofhir_fhirpath::ast::LiteralValue;
            let (name, return_type) = match &node.value {
                LiteralValue::String(s) => (format!("\"{s}\""), "string"),
                LiteralValue::Integer(i) => (i.to_string(), "integer"),
                LiteralValue::Decimal(d) => (d.to_string(), "decimal"),
                LiteralValue::Boolean(b) => (b.to_string(), "boolean"),
                LiteralValue::Date(d) => (format!("@{d}"), "date"),
                LiteralValue::DateTime(dt) => (format!("@{dt}"), "dateTime"),
                LiteralValue::Time(t) => (format!("@{t}"), "time"),
                LiteralValue::Quantity { value, unit } => {
                    let unit_str = unit.as_ref().map(|u| u.as_str()).unwrap_or("");
                    (format!("{value} '{unit_str}'"), "Quantity")
                }
            };

            AstNode {
                expression_type: "ConstantExpression".to_string(),
                name,
                arguments: None,
                return_type: Some(return_type.to_string()),
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::BinaryOperation(node) => {
            let left_arg = convert_ast_to_lab_format(&node.left, function_registry, model_provider);
            let right_arg =
                convert_ast_to_lab_format(&node.right, function_registry, model_provider);

            AstNode {
                expression_type: "BinaryExpression".to_string(),
                name: format!("{:?}", node.operator).to_lowercase(),
                arguments: Some(vec![left_arg, right_arg]),
                return_type: None, // Will be filled by enhancement
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::Variable(node) => AstNode {
            expression_type: "VariableRefExpression".to_string(),
            name: format!("%{}", node.name),
            arguments: None,
            return_type: None, // Variables can hold any type
            position: node.location.as_ref().map(|l| l.offset),
            length: node.location.as_ref().map(|l| l.length),
        },

        ExpressionNode::MethodCall(node) => {
            // Convert object argument first
            let object_arg =
                convert_ast_to_lab_format(&node.object, function_registry, model_provider);

            // Convert method arguments
            let mut args = vec![object_arg];
            for arg in &node.arguments {
                args.push(convert_ast_to_lab_format(
                    arg,
                    function_registry,
                    model_provider,
                ));
            }

            // Get return type from function registry if available
            let return_type = if let Some(_registry) = function_registry {
                // FunctionRegistry is currently a placeholder
                if false {
                    // let Some(function_info) = registry.get_function_metadata(&node.method) {
                    // Some(function_info.signature.returns.display_name())
                    None
                } else {
                    None
                }
            } else {
                None
            };

            AstNode {
                expression_type: "FunctionCallExpression".to_string(),
                name: node.method.clone(),
                arguments: Some(args),
                return_type,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::IndexAccess(node) => {
            let object_arg =
                convert_ast_to_lab_format(&node.object, function_registry, model_provider);
            let index_arg =
                convert_ast_to_lab_format(&node.index, function_registry, model_provider);

            AstNode {
                expression_type: "IndexerExpression".to_string(),
                name: "[]".to_string(),
                arguments: Some(vec![object_arg, index_arg]),
                return_type: None, // Will be filled by enhancement
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::UnaryOperation(node) => {
            let operand_arg =
                convert_ast_to_lab_format(&node.operand, function_registry, model_provider);

            let op_name = match node.operator {
                octofhir_fhirpath::ast::UnaryOperator::Not => "not",
                octofhir_fhirpath::ast::UnaryOperator::Negate => "-",
                octofhir_fhirpath::ast::UnaryOperator::Positive => "+",
            };

            AstNode {
                expression_type: "UnaryExpression".to_string(),
                name: op_name.to_string(),
                arguments: Some(vec![operand_arg]),
                return_type: None, // Will be filled by enhancement
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::Collection(node) => {
            let mut args = vec![];
            for item in &node.elements {
                args.push(convert_ast_to_lab_format(
                    item,
                    function_registry,
                    model_provider,
                ));
            }

            AstNode {
                expression_type: "CollectionExpression".to_string(),
                name: "{}".to_string(),
                arguments: if args.is_empty() { None } else { Some(args) },
                return_type: None, // Will be filled by enhancement
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::Parenthesized(expr) => {
            // For parenthesized expressions, just convert the inner expression
            convert_ast_to_lab_format(expr, function_registry, model_provider)
        }

        _ => {
            // For unsupported expression types, create a generic node
            let expression_type = match ast {
                ExpressionNode::TypeCast(_) => "TypeCastExpression",
                ExpressionNode::Filter(_) => "FilterExpression",
                ExpressionNode::Union(_) => "UnionExpression",
                ExpressionNode::TypeCheck(_) => "TypeCheckExpression",
                ExpressionNode::Path(_) => "PathExpression",
                ExpressionNode::Lambda(_) => "LambdaExpression",
                _ => "UnsupportedExpression",
            };

            AstNode {
                expression_type: expression_type.to_string(),
                name: "unsupported".to_string(),
                arguments: None,
                return_type: None,
                position: None,
                length: None,
            }
        }
    }
}
