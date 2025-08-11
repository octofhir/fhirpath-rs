// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


//! Type analysis for FHIRPath expressions
//!
//! This module provides comprehensive type inference capabilities, determining
//! the return types of expressions and validating type safety.

use crate::ast::{ExpressionNode, LiteralValue, BinaryOperator, UnaryOperator};
use crate::model::provider::{ModelProvider, TypeReflectionInfo};
use crate::analyzer::{AnalysisContext, AnalysisError};
use std::sync::Arc;
use std::collections::HashMap;

/// Result of type analysis
#[derive(Debug, Clone)]
pub struct TypeAnalysisResult {
    /// The inferred return type
    pub return_type: Option<TypeReflectionInfo>,
    /// Whether the result is a collection
    pub is_collection: bool,
    /// All types referenced during analysis
    pub referenced_types: Vec<String>,
    /// Confidence level of the type inference (0.0 to 1.0)
    pub confidence: f32,
}

impl Default for TypeAnalysisResult {
    fn default() -> Self {
        Self {
            return_type: None,
            is_collection: false,
            referenced_types: Vec::new(),
            confidence: 0.0,
        }
    }
}

/// Type analyzer for FHIRPath expressions
pub struct TypeAnalyzer<P: ModelProvider> {
    provider: Arc<P>,
    type_cache: tokio::sync::RwLock<HashMap<String, TypeReflectionInfo>>,
}

impl<P: ModelProvider> TypeAnalyzer<P> {
    /// Create a new type analyzer
    pub fn new(provider: Arc<P>) -> Self {
        Self {
            provider,
            type_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Analyze the type of an expression
    pub async fn analyze_expression(
        &self,
        expression: &ExpressionNode,
        context_type: Option<&str>,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let context = AnalysisContext::new(context_type.map(String::from));
        self.analyze_expression_with_context(expression, &context).await
    }

    /// Analyze expression with full context
    pub fn analyze_expression_with_context<'a>(
        &'a self,
        expression: &'a ExpressionNode,
        context: &'a AnalysisContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<TypeAnalysisResult, AnalysisError>> + Send + 'a>> {
        Box::pin(async move {
        if context.depth > 50 {
            return Err(AnalysisError::MaxDepthExceeded { max_depth: 50 });
        }

        let child_context = context.child();
        
        match expression {
            ExpressionNode::Literal(literal) => {
                self.analyze_literal(literal, &child_context).await
            }
            ExpressionNode::Identifier(name) => {
                self.analyze_identifier(name, &child_context).await
            }
            ExpressionNode::Path { base, path } => {
                self.analyze_path(base, path, &child_context).await
            }
            ExpressionNode::BinaryOp(data) => {
                self.analyze_binary_op(&data.op, &data.left, &data.right, &child_context).await
            }
            ExpressionNode::UnaryOp { op, operand } => {
                self.analyze_unary_op(op, operand, &child_context).await
            }
            ExpressionNode::FunctionCall(data) => {
                self.analyze_function_call(&data.name, &data.args, &child_context).await
            }
            ExpressionNode::MethodCall(data) => {
                self.analyze_method_call(&data.base, &data.method, &data.args, &child_context).await
            }
            ExpressionNode::Index { base, index } => {
                self.analyze_index(base, index, &child_context).await
            }
            ExpressionNode::Filter { base, condition } => {
                self.analyze_filter(base, condition, &child_context).await
            }
            ExpressionNode::Union { left, right } => {
                self.analyze_union(left, right, &child_context).await
            }
            ExpressionNode::TypeCheck { expression, type_name } => {
                self.analyze_type_check(expression, type_name, &child_context).await
            }
            ExpressionNode::TypeCast { expression, type_name } => {
                self.analyze_type_cast(expression, type_name, &child_context).await
            }
            ExpressionNode::Lambda(data) => {
                self.analyze_lambda(&data.params, &data.body, &child_context).await
            }
            ExpressionNode::Conditional(data) => {
                self.analyze_conditional(&data.condition, &data.then_expr, data.else_expr.as_deref(), &child_context).await
            }
            ExpressionNode::Variable(name) => {
                self.analyze_variable(name, &child_context).await
            }
        }
        })
    }

    /// Analyze literal values
    async fn analyze_literal(
        &self,
        literal: &LiteralValue,
        _context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let (type_name, is_collection) = match literal {
            LiteralValue::Boolean(_) => ("Boolean", false),
            LiteralValue::Integer(_) => ("Integer", false),
            LiteralValue::Decimal(_) => ("Decimal", false),
            LiteralValue::String(_) => ("String", false),
            LiteralValue::Date(_) => ("Date", false),
            LiteralValue::DateTime(_) => ("DateTime", false),
            LiteralValue::Time(_) => ("Time", false),
            LiteralValue::Quantity { .. } => ("Quantity", false),
            LiteralValue::Null => return Ok(TypeAnalysisResult::default()),
        };

        let type_info = self.get_or_cache_type(type_name).await;
        
        Ok(TypeAnalysisResult {
            return_type: type_info,
            is_collection,
            referenced_types: vec![type_name.to_string()],
            confidence: 1.0,
        })
    }

    /// Analyze identifier references
    async fn analyze_identifier(
        &self,
        name: &str,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        // Check if it's a context variable
        if let Some(var_type) = context.variables.get(name) {
            return Ok(TypeAnalysisResult {
                return_type: Some(var_type.clone()),
                is_collection: false,
                referenced_types: vec![var_type.name().to_string()],
                confidence: 1.0,
            });
        }

        // Check if it's the root context type
        if let Some(root_type) = &context.root_type {
            if name == "$this" || name == root_type {
                let type_info = self.get_or_cache_type(root_type).await;
                return Ok(TypeAnalysisResult {
                    return_type: type_info.clone(),
                    is_collection: false,
                    referenced_types: vec![root_type.clone()],
                    confidence: 1.0,
                });
            }
        }

        // Unknown identifier - low confidence
        Ok(TypeAnalysisResult {
            return_type: None,
            is_collection: false,
            referenced_types: vec![],
            confidence: 0.1,
        })
    }

    /// Analyze path navigation (base.property)
    async fn analyze_path(
        &self,
        base: &ExpressionNode,
        path: &str,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        // Analyze the base expression first
        let base_result = self.analyze_expression_with_context(base, context).await?;
        
        if let Some(base_type) = &base_result.return_type {
            // Get property type from the base type
            if let Some(property_type) = self.provider.get_property_type(&base_type.name(), path).await {
                let mut referenced_types = base_result.referenced_types;
                referenced_types.push(property_type.name().to_string());
                
                return Ok(TypeAnalysisResult {
                    return_type: Some(property_type.clone()),
                    is_collection: property_type.is_collection() || base_result.is_collection,
                    referenced_types,
                    confidence: base_result.confidence * 0.9, // Slightly lower confidence due to navigation
                });
            }
        }

        // Fallback for unknown property
        Ok(TypeAnalysisResult {
            return_type: None,
            is_collection: base_result.is_collection,
            referenced_types: base_result.referenced_types,
            confidence: 0.2,
        })
    }

    /// Analyze binary operations
    async fn analyze_binary_op(
        &self,
        op: &BinaryOperator,
        left: &ExpressionNode,
        right: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let left_result = self.analyze_expression_with_context(left, context).await?;
        let right_result = self.analyze_expression_with_context(right, context).await?;

        let mut referenced_types = left_result.referenced_types.clone();
        referenced_types.extend(right_result.referenced_types.clone());

        let (result_type, is_collection, confidence) = match op {
            // Comparison operators always return Boolean
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual => {
                let bool_type = self.get_or_cache_type("Boolean").await;
                (bool_type, false, (left_result.confidence + right_result.confidence) * 0.5)
            }

            // Logical operators return Boolean
            BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor 
            | BinaryOperator::Equivalent | BinaryOperator::NotEquivalent | BinaryOperator::Implies => {
                let bool_type = self.get_or_cache_type("Boolean").await;
                (bool_type, false, (left_result.confidence + right_result.confidence) * 0.5)
            }

            // Arithmetic operators - need to determine result type
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide 
            | BinaryOperator::IntegerDivide | BinaryOperator::Modulo => {
                self.infer_arithmetic_result_type(&left_result, &right_result).await
            }

            // String concatenation
            BinaryOperator::Concatenate => {
                let string_type = self.get_or_cache_type("String").await;
                (string_type, false, (left_result.confidence + right_result.confidence) * 0.5)
            }

            // Collection operations
            BinaryOperator::Union => {
                // Union preserves the common type if possible, otherwise Any
                let result_type = if left_result.return_type == right_result.return_type {
                    left_result.return_type.or(right_result.return_type)
                } else {
                    None // Mixed types - could infer common base type
                };
                (result_type, true, (left_result.confidence + right_result.confidence) * 0.4)
            }

            // Type membership and string operations
            BinaryOperator::In | BinaryOperator::Contains => {
                let bool_type = self.get_or_cache_type("Boolean").await;
                (bool_type, false, (left_result.confidence + right_result.confidence) * 0.5)
            }

            // Type checking
            BinaryOperator::Is => {
                let bool_type = self.get_or_cache_type("Boolean").await;
                (bool_type, false, (left_result.confidence + right_result.confidence) * 0.5)
            }
        };

        referenced_types.dedup();

        Ok(TypeAnalysisResult {
            return_type: result_type,
            is_collection,
            referenced_types,
            confidence,
        })
    }

    /// Analyze unary operations
    async fn analyze_unary_op(
        &self,
        op: &UnaryOperator,
        operand: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let operand_result = self.analyze_expression_with_context(operand, context).await?;

        let result_type = match op {
            UnaryOperator::Not => self.get_or_cache_type("Boolean").await,
            UnaryOperator::Plus | UnaryOperator::Minus => {
                // Preserve numeric type
                operand_result.return_type.clone()
            }
        };

        Ok(TypeAnalysisResult {
            return_type: result_type,
            is_collection: operand_result.is_collection,
            referenced_types: operand_result.referenced_types,
            confidence: operand_result.confidence * 0.9,
        })
    }

    /// Analyze function calls
    async fn analyze_function_call(
        &self,
        name: &str,
        args: &[ExpressionNode],
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        // Analyze all arguments
        let mut referenced_types = Vec::new();
        let mut confidence: f32 = 1.0;
        
        for arg in args {
            let arg_result = self.analyze_expression_with_context(arg, context).await?;
            referenced_types.extend(arg_result.referenced_types);
            confidence = confidence.min(arg_result.confidence);
        }

        // Infer return type based on function name
        let (return_type, is_collection) = self.infer_function_return_type(name, args).await;

        Ok(TypeAnalysisResult {
            return_type,
            is_collection,
            referenced_types,
            confidence: confidence * 0.8, // Function calls have slightly lower confidence
        })
    }

    /// Analyze method calls
    async fn analyze_method_call(
        &self,
        base: &ExpressionNode,
        method: &str,
        args: &[ExpressionNode],
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let base_result = self.analyze_expression_with_context(base, context).await?;
        
        // Analyze arguments
        let mut referenced_types = base_result.referenced_types.clone();
        let mut confidence = base_result.confidence;
        
        for arg in args {
            let arg_result = self.analyze_expression_with_context(arg, context).await?;
            referenced_types.extend(arg_result.referenced_types);
            confidence = confidence.min(arg_result.confidence);
        }

        // Infer method return type
        let (return_type, is_collection) = self.infer_method_return_type(
            method,
            &base_result,
            args,
        ).await;

        Ok(TypeAnalysisResult {
            return_type,
            is_collection,
            referenced_types,
            confidence: confidence * 0.8,
        })
    }

    /// Analyze index operations
    async fn analyze_index(
        &self,
        base: &ExpressionNode,
        _index: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let base_result = self.analyze_expression_with_context(base, context).await?;
        
        // Index operations typically convert collections to singletons
        Ok(TypeAnalysisResult {
            return_type: base_result.return_type,
            is_collection: false, // Indexing returns a single item
            referenced_types: base_result.referenced_types,
            confidence: base_result.confidence * 0.9,
        })
    }

    /// Analyze filter operations
    async fn analyze_filter(
        &self,
        base: &ExpressionNode,
        _condition: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let base_result = self.analyze_expression_with_context(base, context).await?;
        
        // Filter preserves the base type but ensures it's a collection
        Ok(TypeAnalysisResult {
            return_type: base_result.return_type,
            is_collection: true, // Filter always returns a collection
            referenced_types: base_result.referenced_types,
            confidence: base_result.confidence * 0.8,
        })
    }

    /// Analyze union operations
    async fn analyze_union(
        &self,
        left: &ExpressionNode,
        right: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let left_result = self.analyze_expression_with_context(left, context).await?;
        let right_result = self.analyze_expression_with_context(right, context).await?;

        let mut referenced_types = left_result.referenced_types.clone();
        referenced_types.extend(right_result.referenced_types.clone());
        referenced_types.dedup();

        // Union result type is the common type or None if different
        let return_type = if left_result.return_type == right_result.return_type {
            left_result.return_type.or(right_result.return_type)
        } else {
            // Could try to find common base type here
            None
        };

        Ok(TypeAnalysisResult {
            return_type,
            is_collection: true, // Union always returns a collection
            referenced_types,
            confidence: (left_result.confidence + right_result.confidence) * 0.4,
        })
    }

    /// Analyze type check operations (is Type)
    async fn analyze_type_check(
        &self,
        expression: &ExpressionNode,
        type_name: &str,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let expr_result = self.analyze_expression_with_context(expression, context).await?;
        let mut referenced_types = expr_result.referenced_types;
        referenced_types.push(type_name.to_string());

        let bool_type = self.get_or_cache_type("Boolean").await;

        Ok(TypeAnalysisResult {
            return_type: bool_type,
            is_collection: false,
            referenced_types,
            confidence: expr_result.confidence,
        })
    }

    /// Analyze type cast operations (as Type)
    async fn analyze_type_cast(
        &self,
        expression: &ExpressionNode,
        type_name: &str,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let expr_result = self.analyze_expression_with_context(expression, context).await?;
        let mut referenced_types = expr_result.referenced_types;
        referenced_types.push(type_name.to_string());

        let cast_type = self.get_or_cache_type(type_name).await;

        Ok(TypeAnalysisResult {
            return_type: cast_type,
            is_collection: expr_result.is_collection,
            referenced_types,
            confidence: expr_result.confidence * 0.7, // Type casts have lower confidence
        })
    }

    /// Analyze lambda expressions
    async fn analyze_lambda(
        &self,
        _params: &[String],
        body: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        // For lambda analysis, we need to analyze the body
        // Parameters would need special handling in the context
        self.analyze_expression_with_context(body, context).await
    }

    /// Analyze conditional expressions
    async fn analyze_conditional(
        &self,
        _condition: &ExpressionNode,
        then_expr: &ExpressionNode,
        else_expr: Option<&ExpressionNode>,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        let then_result = self.analyze_expression_with_context(then_expr, context).await?;
        
        if let Some(else_expr) = else_expr {
            let else_result = self.analyze_expression_with_context(else_expr, context).await?;
            
            let mut referenced_types = then_result.referenced_types;
            referenced_types.extend(else_result.referenced_types);
            referenced_types.dedup();

            // Result type is the common type of then and else branches
            let return_type = if then_result.return_type == else_result.return_type {
                then_result.return_type.or(else_result.return_type)
            } else {
                None // Different types - would need common base type inference
            };

            Ok(TypeAnalysisResult {
                return_type,
                is_collection: then_result.is_collection || else_result.is_collection,
                referenced_types,
                confidence: (then_result.confidence + else_result.confidence) * 0.5,
            })
        } else {
            // No else clause - result might be empty
            Ok(TypeAnalysisResult {
                return_type: then_result.return_type,
                is_collection: then_result.is_collection,
                referenced_types: then_result.referenced_types,
                confidence: then_result.confidence * 0.8,
            })
        }
    }

    /// Analyze variable references
    async fn analyze_variable(
        &self,
        name: &str,
        context: &AnalysisContext,
    ) -> Result<TypeAnalysisResult, AnalysisError> {
        // Check context variables first
        if let Some(var_type) = context.variables.get(name) {
            return Ok(TypeAnalysisResult {
                return_type: Some(var_type.clone()),
                is_collection: var_type.is_collection(),
                referenced_types: vec![var_type.name().to_string()],
                confidence: 1.0,
            });
        }

        // Handle special system variables
        let (type_name, is_collection) = match name {
            "$this" => {
                if let Some(root_type) = &context.root_type {
                    (root_type.as_str(), false)
                } else {
                    return Ok(TypeAnalysisResult::default());
                }
            }
            "$index" => ("Integer", false),
            "$total" => ("Integer", false),
            _ => return Ok(TypeAnalysisResult::default()), // Unknown variable
        };

        let type_info = self.get_or_cache_type(type_name).await;

        Ok(TypeAnalysisResult {
            return_type: type_info,
            is_collection,
            referenced_types: vec![type_name.to_string()],
            confidence: 0.9,
        })
    }

    /// Get or cache a type reflection
    async fn get_or_cache_type(&self, type_name: &str) -> Option<TypeReflectionInfo> {
        // Check cache first
        {
            let cache = self.type_cache.read().await;
            if let Some(cached_type) = cache.get(type_name) {
                return Some(cached_type.clone());
            }
        }

        // Get from provider
        if let Some(type_info) = self.provider.get_type_reflection(type_name).await {
            // Cache the result
            {
                let mut cache = self.type_cache.write().await;
                cache.insert(type_name.to_string(), type_info.clone());
            }
            Some(type_info)
        } else {
            None
        }
    }

    /// Infer arithmetic result type
    async fn infer_arithmetic_result_type(
        &self,
        left_result: &TypeAnalysisResult,
        right_result: &TypeAnalysisResult,
    ) -> (Option<TypeReflectionInfo>, bool, f32) {
        // Simplified arithmetic type inference
        let confidence = (left_result.confidence + right_result.confidence) * 0.5;
        
        match (&left_result.return_type, &right_result.return_type) {
            (Some(left_type), Some(right_type)) => {
                let result_type_name = match (&left_type.name()[..], &right_type.name()[..]) {
                    ("Integer", "Integer") => "Integer",
                    ("Decimal", _) | (_, "Decimal") => "Decimal",
                    ("Quantity", "Quantity") => "Quantity",
                    _ => "Decimal", // Default to Decimal for mixed numeric operations
                };
                
                let result_type = self.get_or_cache_type(result_type_name).await;
                (result_type, false, confidence)
            }
            _ => (None, false, confidence * 0.5),
        }
    }

    /// Infer function return type based on function name
    async fn infer_function_return_type(
        &self,
        name: &str,
        _args: &[ExpressionNode],
    ) -> (Option<TypeReflectionInfo>, bool) {
        // Basic function type inference
        match name {
            // Boolean functions
            "empty" | "exists" | "all" | "allTrue" | "anyTrue" | "allFalse" | "anyFalse" => {
                (self.get_or_cache_type("Boolean").await, false)
            }
            // Numeric functions
            "count" | "length" => {
                (self.get_or_cache_type("Integer").await, false)
            }
            // String functions
            "toString" | "substring" | "upper" | "lower" => {
                (self.get_or_cache_type("String").await, false)
            }
            // Collection functions that preserve type
            "first" | "last" | "tail" => {
                // Would need argument analysis to determine exact type
                (None, false)
            }
            // Collection functions that return collections
            "where" | "select" | "distinct" => {
                (None, true)
            }
            _ => (None, false), // Unknown function
        }
    }

    /// Infer method return type
    async fn infer_method_return_type(
        &self,
        method: &str,
        base_result: &TypeAnalysisResult,
        _args: &[ExpressionNode],
    ) -> (Option<TypeReflectionInfo>, bool) {
        match method {
            // Collection methods that preserve base type
            "where" | "select" => (base_result.return_type.clone(), true),
            "first" | "last" | "single" => (base_result.return_type.clone(), false),
            // Type conversion methods
            "toString" => (self.get_or_cache_type("String").await, false),
            "toInteger" => (self.get_or_cache_type("Integer").await, false),
            "toDecimal" => (self.get_or_cache_type("Decimal").await, false),
            // Boolean methods
            "empty" | "exists" => (self.get_or_cache_type("Boolean").await, false),
            // Numeric methods
            "count" | "length" => (self.get_or_cache_type("Integer").await, false),
            _ => (base_result.return_type.clone(), base_result.is_collection),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::mock_provider::MockModelProvider;
    use crate::ast::ExpressionNode;

    #[tokio::test]
    async fn test_literal_analysis() {
        let provider = Arc::new(MockModelProvider::empty());
        let analyzer = TypeAnalyzer::new(provider);

        let expr = ExpressionNode::literal(LiteralValue::Integer(42));
        let result = analyzer.analyze_expression(&expr, None).await.unwrap();

        assert!(result.return_type.is_some());
        assert!(!result.is_collection);
        assert_eq!(result.confidence, 1.0);
        assert!(result.referenced_types.contains(&"Integer".to_string()));
    }

    #[tokio::test]
    async fn test_path_analysis() {
        let provider = Arc::new(MockModelProvider::empty());
        let analyzer = TypeAnalyzer::new(provider);

        let base = ExpressionNode::identifier("Patient");
        let expr = ExpressionNode::path(base, "name");
        
        let result = analyzer.analyze_expression(&expr, Some("Patient")).await.unwrap();

        // With MockModelProvider, this will have low confidence
        assert!(result.confidence < 1.0);
    }

    #[tokio::test]
    async fn test_binary_op_analysis() {
        let provider = Arc::new(MockModelProvider::empty());
        let analyzer = TypeAnalyzer::new(provider);

        let left = ExpressionNode::literal(LiteralValue::Integer(1));
        let right = ExpressionNode::literal(LiteralValue::Integer(2));
        let expr = ExpressionNode::binary_op(BinaryOperator::Add, left, right);
        
        let result = analyzer.analyze_expression(&expr, None).await.unwrap();

        assert!(result.return_type.is_some());
        assert!(!result.is_collection);
    }
}