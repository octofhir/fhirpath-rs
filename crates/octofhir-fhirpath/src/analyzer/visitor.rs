//! Expression visitor pattern for AST traversal
//!
//! This module provides a visitor pattern for traversing FHIRPath expression ASTs
//! in a type-safe and extensible way.

use crate::ast::expression::*;
use crate::core::Result;

/// Trait for visiting expression nodes in the AST
pub trait ExpressionVisitor {
    /// The type returned by each visit method
    type Output;

    /// Visit any expression node (dispatches to specific methods)
    fn visit_expression(&mut self, expr: &ExpressionNode) -> Self::Output {
        match expr {
            ExpressionNode::Literal(node) => self.visit_literal(node),
            ExpressionNode::Identifier(node) => self.visit_identifier(node),
            ExpressionNode::FunctionCall(node) => self.visit_function_call(node),
            ExpressionNode::MethodCall(node) => self.visit_method_call(node),
            ExpressionNode::PropertyAccess(node) => self.visit_property_access(node),
            ExpressionNode::IndexAccess(node) => self.visit_index_access(node),
            ExpressionNode::BinaryOperation(node) => self.visit_binary_operation(node),
            ExpressionNode::UnaryOperation(node) => self.visit_unary_operation(node),
            ExpressionNode::Lambda(node) => self.visit_lambda(node),
            ExpressionNode::Collection(node) => self.visit_collection(node),
            ExpressionNode::Parenthesized(expr) => self.visit_parenthesized(expr),
            ExpressionNode::TypeCast(node) => self.visit_type_cast(node),
            ExpressionNode::Filter(node) => self.visit_filter(node),
            ExpressionNode::Union(node) => self.visit_union(node),
            ExpressionNode::TypeCheck(node) => self.visit_type_check(node),
            ExpressionNode::Variable(node) => self.visit_variable(node),
            ExpressionNode::Path(node) => self.visit_path(node),
            ExpressionNode::TypeInfo(node) => self.visit_type_info(node),
        }
    }

    /// Visit a literal value
    fn visit_literal(&mut self, literal: &LiteralNode) -> Self::Output;

    /// Visit an identifier
    fn visit_identifier(&mut self, identifier: &IdentifierNode) -> Self::Output;

    /// Visit a function call
    fn visit_function_call(&mut self, call: &FunctionCallNode) -> Self::Output;

    /// Visit a method call
    fn visit_method_call(&mut self, call: &MethodCallNode) -> Self::Output;

    /// Visit a property access
    fn visit_property_access(&mut self, access: &PropertyAccessNode) -> Self::Output;

    /// Visit an index access
    fn visit_index_access(&mut self, access: &IndexAccessNode) -> Self::Output;

    /// Visit a binary operation
    fn visit_binary_operation(&mut self, binary: &BinaryOperationNode) -> Self::Output;

    /// Visit a unary operation
    fn visit_unary_operation(&mut self, unary: &UnaryOperationNode) -> Self::Output;

    /// Visit a lambda expression
    fn visit_lambda(&mut self, lambda: &LambdaNode) -> Self::Output;

    /// Visit a collection literal
    fn visit_collection(&mut self, collection: &CollectionNode) -> Self::Output;

    /// Visit a parenthesized expression
    fn visit_parenthesized(&mut self, expr: &ExpressionNode) -> Self::Output;

    /// Visit a type cast
    fn visit_type_cast(&mut self, cast: &TypeCastNode) -> Self::Output;

    /// Visit a filter expression
    fn visit_filter(&mut self, filter: &FilterNode) -> Self::Output;

    /// Visit a union expression
    fn visit_union(&mut self, union: &UnionNode) -> Self::Output;

    /// Visit a type check
    fn visit_type_check(&mut self, check: &TypeCheckNode) -> Self::Output;

    /// Visit a variable reference
    fn visit_variable(&mut self, variable: &VariableNode) -> Self::Output;

    /// Visit a path expression
    fn visit_path(&mut self, path: &PathNode) -> Self::Output;

    /// Visit a type info expression
    fn visit_type_info(&mut self, type_info: &TypeInfoNode) -> Self::Output;
}

/// Default implementation for ExpressionVisitor that does nothing
pub trait DefaultExpressionVisitor: ExpressionVisitor<Output = Result<()>> {
    /// Visit a literal value (default implementation does nothing)
    fn visit_literal(&mut self, _literal: &LiteralNode) -> Result<()> {
        Ok(())
    }

    /// Visit an identifier (default implementation does nothing)
    fn visit_identifier(&mut self, _identifier: &IdentifierNode) -> Result<()> {
        Ok(())
    }

    /// Visit a function call (default implementation visits all arguments)
    fn visit_function_call(&mut self, call: &FunctionCallNode) -> Result<()> {
        for arg in &call.arguments {
            self.visit_expression(arg)?;
        }
        Ok(())
    }

    /// Visit a method call (default implementation visits object and all arguments)
    fn visit_method_call(&mut self, call: &MethodCallNode) -> Result<()> {
        self.visit_expression(&call.object)?;
        for arg in &call.arguments {
            self.visit_expression(arg)?;
        }
        Ok(())
    }

    /// Visit a property access (default implementation visits the object)
    fn visit_property_access(&mut self, access: &PropertyAccessNode) -> Result<()> {
        self.visit_expression(&access.object)
    }

    /// Visit an index access (default implementation visits object and index)
    fn visit_index_access(&mut self, access: &IndexAccessNode) -> Result<()> {
        self.visit_expression(&access.object)?;
        self.visit_expression(&access.index)
    }

    /// Visit a binary operation (default implementation visits left and right operands)
    fn visit_binary_operation(&mut self, binary: &BinaryOperationNode) -> Result<()> {
        self.visit_expression(&binary.left)?;
        self.visit_expression(&binary.right)
    }

    /// Visit a unary operation (default implementation visits the operand)
    fn visit_unary_operation(&mut self, unary: &UnaryOperationNode) -> Result<()> {
        self.visit_expression(&unary.operand)
    }

    /// Visit a lambda expression (default implementation visits the body)
    fn visit_lambda(&mut self, lambda: &LambdaNode) -> Result<()> {
        self.visit_expression(&lambda.body)
    }

    /// Visit a collection literal (default implementation visits all elements)
    fn visit_collection(&mut self, collection: &CollectionNode) -> Result<()> {
        for element in &collection.elements {
            self.visit_expression(element)?;
        }
        Ok(())
    }

    /// Visit a parenthesized expression (default implementation visits the inner expression)
    fn visit_parenthesized(&mut self, expr: &ExpressionNode) -> Result<()> {
        self.visit_expression(expr)
    }

    /// Visit a type cast (default implementation visits the expression)
    fn visit_type_cast(&mut self, cast: &TypeCastNode) -> Result<()> {
        self.visit_expression(&cast.expression)
    }

    /// Visit a filter expression (default implementation visits base and condition)
    fn visit_filter(&mut self, filter: &FilterNode) -> Result<()> {
        self.visit_expression(&filter.base)?;
        self.visit_expression(&filter.condition)
    }

    /// Visit a union expression (default implementation visits left and right expressions)
    fn visit_union(&mut self, union: &UnionNode) -> Result<()> {
        self.visit_expression(&union.left)?;
        self.visit_expression(&union.right)
    }

    /// Visit a type check (default implementation visits the expression)
    fn visit_type_check(&mut self, check: &TypeCheckNode) -> Result<()> {
        self.visit_expression(&check.expression)
    }

    /// Visit a variable reference (default implementation does nothing)
    fn visit_variable(&mut self, _variable: &VariableNode) -> Result<()> {
        Ok(())
    }

    /// Visit a path expression (default implementation visits the base)
    fn visit_path(&mut self, path: &PathNode) -> Result<()> {
        self.visit_expression(&path.base)
    }

    /// Visit a type info expression (default implementation does nothing)
    fn visit_type_info(&mut self, _type_info: &TypeInfoNode) -> Result<()> {
        Ok(())
    }
}

/// A collecting visitor that accumulates results
pub trait CollectingVisitor<T> {
    /// Collect results from a single node (default implementation returns empty vec)
    fn collect_from_node(&mut self, _expr: &ExpressionNode) -> Result<Vec<T>> {
        Ok(vec![])
    }

    /// Visit an expression and collect results from it and all its children
    fn visit_expression(&mut self, expr: &ExpressionNode) -> Result<Vec<T>> {
        let mut results = self.collect_from_node(expr)?;

        match expr {
            ExpressionNode::FunctionCall(call) => {
                for arg in &call.arguments {
                    results.extend(self.visit_expression(arg)?);
                }
            }
            ExpressionNode::MethodCall(call) => {
                results.extend(self.visit_expression(&call.object)?);
                for arg in &call.arguments {
                    results.extend(self.visit_expression(arg)?);
                }
            }
            ExpressionNode::PropertyAccess(access) => {
                results.extend(self.visit_expression(&access.object)?);
            }
            ExpressionNode::IndexAccess(access) => {
                results.extend(self.visit_expression(&access.object)?);
                results.extend(self.visit_expression(&access.index)?);
            }
            ExpressionNode::BinaryOperation(binary) => {
                results.extend(self.visit_expression(&binary.left)?);
                results.extend(self.visit_expression(&binary.right)?);
            }
            ExpressionNode::UnaryOperation(unary) => {
                results.extend(self.visit_expression(&unary.operand)?);
            }
            ExpressionNode::Lambda(lambda) => {
                results.extend(self.visit_expression(&lambda.body)?);
            }
            ExpressionNode::Collection(collection) => {
                for element in &collection.elements {
                    results.extend(self.visit_expression(element)?);
                }
            }
            ExpressionNode::Parenthesized(expr) => {
                results.extend(self.visit_expression(expr)?);
            }
            ExpressionNode::TypeCast(cast) => {
                results.extend(self.visit_expression(&cast.expression)?);
            }
            ExpressionNode::Filter(filter) => {
                results.extend(self.visit_expression(&filter.base)?);
                results.extend(self.visit_expression(&filter.condition)?);
            }
            ExpressionNode::Union(union) => {
                results.extend(self.visit_expression(&union.left)?);
                results.extend(self.visit_expression(&union.right)?);
            }
            ExpressionNode::TypeCheck(check) => {
                results.extend(self.visit_expression(&check.expression)?);
            }
            ExpressionNode::Path(path) => {
                results.extend(self.visit_expression(&path.base)?);
            }
            // Leaf nodes don't have children to visit
            ExpressionNode::Literal(_)
            | ExpressionNode::Identifier(_)
            | ExpressionNode::Variable(_)
            | ExpressionNode::TypeInfo(_) => {}
        }

        Ok(results)
    }
}

/// Helper macro to implement simple visitors
#[macro_export]
macro_rules! impl_default_visitor {
    ($visitor:ident, $output:ty, $default:expr) => {
        impl ExpressionVisitor for $visitor {
            type Output = $output;

            fn visit_literal(&mut self, _literal: &LiteralNode) -> Self::Output {
                $default
            }

            fn visit_identifier(&mut self, _identifier: &IdentifierNode) -> Self::Output {
                $default
            }

            fn visit_function_call(&mut self, call: &FunctionCallNode) -> Self::Output {
                for arg in &call.arguments {
                    let _ = self.visit_expression(arg);
                }
                $default
            }

            fn visit_method_call(&mut self, call: &MethodCallNode) -> Self::Output {
                let _ = self.visit_expression(&call.object);
                for arg in &call.arguments {
                    let _ = self.visit_expression(arg);
                }
                $default
            }

            fn visit_property_access(&mut self, access: &PropertyAccessNode) -> Self::Output {
                let _ = self.visit_expression(&access.object);
                $default
            }

            fn visit_index_access(&mut self, access: &IndexAccessNode) -> Self::Output {
                let _ = self.visit_expression(&access.object);
                let _ = self.visit_expression(&access.index);
                $default
            }

            fn visit_binary_operation(&mut self, binary: &BinaryOperationNode) -> Self::Output {
                let _ = self.visit_expression(&binary.left);
                let _ = self.visit_expression(&binary.right);
                $default
            }

            fn visit_unary_operation(&mut self, unary: &UnaryOperationNode) -> Self::Output {
                let _ = self.visit_expression(&unary.operand);
                $default
            }

            fn visit_lambda(&mut self, lambda: &LambdaNode) -> Self::Output {
                let _ = self.visit_expression(&lambda.body);
                $default
            }

            fn visit_collection(&mut self, collection: &CollectionNode) -> Self::Output {
                for element in &collection.elements {
                    let _ = self.visit_expression(element);
                }
                $default
            }

            fn visit_parenthesized(&mut self, expr: &ExpressionNode) -> Self::Output {
                let _ = self.visit_expression(expr);
                $default
            }

            fn visit_type_cast(&mut self, cast: &TypeCastNode) -> Self::Output {
                let _ = self.visit_expression(&cast.expression);
                $default
            }

            fn visit_filter(&mut self, filter: &FilterNode) -> Self::Output {
                let _ = self.visit_expression(&filter.base);
                let _ = self.visit_expression(&filter.condition);
                $default
            }

            fn visit_union(&mut self, union: &UnionNode) -> Self::Output {
                let _ = self.visit_expression(&union.left);
                let _ = self.visit_expression(&union.right);
                $default
            }

            fn visit_type_check(&mut self, check: &TypeCheckNode) -> Self::Output {
                let _ = self.visit_expression(&check.expression);
                $default
            }

            fn visit_variable(&mut self, _variable: &VariableNode) -> Self::Output {
                $default
            }

            fn visit_path(&mut self, path: &PathNode) -> Self::Output {
                let _ = self.visit_expression(&path.base);
                $default
            }

            fn visit_type_info(&mut self, _type_info: &TypeInfoNode) -> Self::Output {
                $default
            }
        }
    };
}
