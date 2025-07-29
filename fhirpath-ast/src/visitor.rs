//! Visitor pattern for AST traversal

use crate::expression::ExpressionNode;

/// Trait for visiting AST nodes
pub trait Visitor: Sized {
    /// The result type of visiting a node
    type Result;

    /// Visit an expression node
    fn visit_expression(&mut self, expr: &ExpressionNode) -> Self::Result {
        walk_expression(self, expr)
    }

    /// Visit a literal expression
    fn visit_literal(&mut self, _literal: &crate::expression::LiteralValue) -> Self::Result;

    /// Visit an identifier
    fn visit_identifier(&mut self, _name: &str) -> Self::Result;

    /// Visit a function call
    fn visit_function_call(&mut self, _name: &str, _args: &[ExpressionNode]) -> Self::Result;

    /// Visit a method call
    fn visit_method_call(
        &mut self,
        _base: &ExpressionNode,
        _method: &str,
        _args: &[ExpressionNode],
    ) -> Self::Result;

    /// Visit a binary operation
    fn visit_binary_op(
        &mut self,
        _op: &crate::operator::BinaryOperator,
        _left: &ExpressionNode,
        _right: &ExpressionNode,
    ) -> Self::Result;

    /// Visit a unary operation
    fn visit_unary_op(
        &mut self,
        _op: &crate::operator::UnaryOperator,
        _operand: &ExpressionNode,
    ) -> Self::Result;

    /// Visit a path navigation
    fn visit_path(&mut self, _base: &ExpressionNode, _path: &str) -> Self::Result;

    /// Visit an index access
    fn visit_index(&mut self, _base: &ExpressionNode, _index: &ExpressionNode) -> Self::Result;

    /// Visit a filter expression
    fn visit_filter(&mut self, _base: &ExpressionNode, _condition: &ExpressionNode)
    -> Self::Result;

    /// Visit a union expression
    fn visit_union(&mut self, _left: &ExpressionNode, _right: &ExpressionNode) -> Self::Result;

    /// Visit a type check
    fn visit_type_check(&mut self, _expr: &ExpressionNode, _type_name: &str) -> Self::Result;

    /// Visit a type cast
    fn visit_type_cast(&mut self, _expr: &ExpressionNode, _type_name: &str) -> Self::Result;

    /// Visit a lambda expression
    fn visit_lambda(&mut self, _param: &str, _body: &ExpressionNode) -> Self::Result;

    /// Visit a conditional expression
    fn visit_conditional(
        &mut self,
        _condition: &ExpressionNode,
        _then_expr: &ExpressionNode,
        _else_expr: Option<&ExpressionNode>,
    ) -> Self::Result;

    /// Visit a variable reference
    fn visit_variable(&mut self, _name: &str) -> Self::Result;
}

/// Default implementation of walking an expression tree
pub fn walk_expression<V: Visitor>(visitor: &mut V, expr: &ExpressionNode) -> V::Result {
    match expr {
        ExpressionNode::Literal(lit) => visitor.visit_literal(lit),
        ExpressionNode::Identifier(name) => visitor.visit_identifier(name),
        ExpressionNode::FunctionCall { name, args } => visitor.visit_function_call(name, args),
        ExpressionNode::MethodCall { base, method, args } => {
            visitor.visit_method_call(base, method, args)
        }
        ExpressionNode::BinaryOp { op, left, right } => visitor.visit_binary_op(op, left, right),
        ExpressionNode::UnaryOp { op, operand } => visitor.visit_unary_op(op, operand),
        ExpressionNode::Path { base, path } => visitor.visit_path(base, path),
        ExpressionNode::Index { base, index } => visitor.visit_index(base, index),
        ExpressionNode::Filter { base, condition } => visitor.visit_filter(base, condition),
        ExpressionNode::Union { left, right } => visitor.visit_union(left, right),
        ExpressionNode::TypeCheck {
            expression,
            type_name,
        } => visitor.visit_type_check(expression, type_name),
        ExpressionNode::TypeCast {
            expression,
            type_name,
        } => visitor.visit_type_cast(expression, type_name),
        ExpressionNode::Lambda { param, body } => visitor.visit_lambda(param, body),
        ExpressionNode::Conditional {
            condition,
            then_expr,
            else_expr,
        } => visitor.visit_conditional(condition, then_expr, else_expr.as_deref()),
        ExpressionNode::Variable(name) => visitor.visit_variable(name),
    }
}

/// Mutable visitor trait for modifying AST nodes
pub trait MutVisitor: Sized {
    /// Visit and potentially modify an expression node
    fn visit_expression_mut(&mut self, expr: &mut ExpressionNode) {
        walk_expression_mut(self, expr)
    }

    /// Visit a literal expression
    fn visit_literal_mut(&mut self, _literal: &mut crate::expression::LiteralValue) {}

    /// Visit an identifier
    fn visit_identifier_mut(&mut self, _name: &mut String) {}

    /// Visit a function call
    fn visit_function_call_mut(&mut self, _name: &mut String, args: &mut Vec<ExpressionNode>) {
        for arg in args {
            self.visit_expression_mut(arg);
        }
    }

    /// Visit a method call
    fn visit_method_call_mut(
        &mut self,
        base: &mut ExpressionNode,
        _method: &mut String,
        args: &mut Vec<ExpressionNode>,
    ) {
        self.visit_expression_mut(base);
        for arg in args {
            self.visit_expression_mut(arg);
        }
    }

    /// Visit a binary operation
    fn visit_binary_op_mut(
        &mut self,
        _op: &mut crate::operator::BinaryOperator,
        left: &mut ExpressionNode,
        right: &mut ExpressionNode,
    ) {
        self.visit_expression_mut(left);
        self.visit_expression_mut(right);
    }

    /// Visit a unary operation
    fn visit_unary_op_mut(
        &mut self,
        _op: &mut crate::operator::UnaryOperator,
        operand: &mut ExpressionNode,
    ) {
        self.visit_expression_mut(operand);
    }

    /// Visit a path navigation
    fn visit_path_mut(&mut self, base: &mut ExpressionNode, _path: &mut String) {
        self.visit_expression_mut(base);
    }

    /// Visit an index access
    fn visit_index_mut(&mut self, base: &mut ExpressionNode, index: &mut ExpressionNode) {
        self.visit_expression_mut(base);
        self.visit_expression_mut(index);
    }

    /// Visit a filter expression
    fn visit_filter_mut(&mut self, base: &mut ExpressionNode, condition: &mut ExpressionNode) {
        self.visit_expression_mut(base);
        self.visit_expression_mut(condition);
    }

    /// Visit a union expression
    fn visit_union_mut(&mut self, left: &mut ExpressionNode, right: &mut ExpressionNode) {
        self.visit_expression_mut(left);
        self.visit_expression_mut(right);
    }

    /// Visit a type check
    fn visit_type_check_mut(&mut self, expr: &mut ExpressionNode, _type_name: &mut String) {
        self.visit_expression_mut(expr);
    }

    /// Visit a type cast
    fn visit_type_cast_mut(&mut self, expr: &mut ExpressionNode, _type_name: &mut String) {
        self.visit_expression_mut(expr);
    }

    /// Visit a lambda expression
    fn visit_lambda_mut(&mut self, _param: &mut String, body: &mut ExpressionNode) {
        self.visit_expression_mut(body);
    }

    /// Visit a conditional expression
    fn visit_conditional_mut(
        &mut self,
        condition: &mut ExpressionNode,
        then_expr: &mut ExpressionNode,
        else_expr: &mut Option<Box<ExpressionNode>>,
    ) {
        self.visit_expression_mut(condition);
        self.visit_expression_mut(then_expr);
        if let Some(else_expr) = else_expr {
            self.visit_expression_mut(else_expr);
        }
    }

    /// Visit a variable reference
    fn visit_variable_mut(&mut self, _name: &mut String) {}
}

/// Default implementation of walking and modifying an expression tree
pub fn walk_expression_mut<V: MutVisitor>(visitor: &mut V, expr: &mut ExpressionNode) {
    match expr {
        ExpressionNode::Literal(lit) => visitor.visit_literal_mut(lit),
        ExpressionNode::Identifier(name) => visitor.visit_identifier_mut(name),
        ExpressionNode::FunctionCall { name, args } => visitor.visit_function_call_mut(name, args),
        ExpressionNode::MethodCall { base, method, args } => {
            visitor.visit_method_call_mut(base, method, args)
        }
        ExpressionNode::BinaryOp { op, left, right } => {
            visitor.visit_binary_op_mut(op, left, right)
        }
        ExpressionNode::UnaryOp { op, operand } => visitor.visit_unary_op_mut(op, operand),
        ExpressionNode::Path { base, path } => visitor.visit_path_mut(base, path),
        ExpressionNode::Index { base, index } => visitor.visit_index_mut(base, index),
        ExpressionNode::Filter { base, condition } => visitor.visit_filter_mut(base, condition),
        ExpressionNode::Union { left, right } => visitor.visit_union_mut(left, right),
        ExpressionNode::TypeCheck {
            expression,
            type_name,
        } => visitor.visit_type_check_mut(expression, type_name),
        ExpressionNode::TypeCast {
            expression,
            type_name,
        } => visitor.visit_type_cast_mut(expression, type_name),
        ExpressionNode::Lambda { param, body } => visitor.visit_lambda_mut(param, body),
        ExpressionNode::Conditional {
            condition,
            then_expr,
            else_expr,
        } => visitor.visit_conditional_mut(condition, then_expr, else_expr),
        ExpressionNode::Variable(name) => visitor.visit_variable_mut(name),
    }
}
