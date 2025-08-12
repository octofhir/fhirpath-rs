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

//! Expression AST node definitions

use crate::operator::{BinaryOperator, UnaryOperator};
use smallvec::SmallVec;

/// AST representation of FHIRPath expressions
///
/// Memory layout optimized: frequently used variants are placed first,
/// and large variants are boxed to reduce overall enum size.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExpressionNode {
    /// Literal value (string, number, boolean, etc.)
    Literal(LiteralValue),

    /// Identifier (variable name, property name)
    Identifier(String),

    /// Path navigation (object.property) - Very common in FHIRPath
    Path {
        /// Base expression
        base: Box<ExpressionNode>,
        /// Property path
        path: String,
    },

    /// Binary operation (arithmetic, comparison, logical) (boxed for size optimization)
    BinaryOp(Box<BinaryOpData>),

    /// Unary operation (negation, not)
    UnaryOp {
        /// The operator
        op: UnaryOperator,
        /// The operand
        operand: Box<ExpressionNode>,
    },

    /// Function call with name and arguments (boxed for size optimization)
    FunctionCall(Box<FunctionCallData>),

    /// Method call on an expression (expression.method(args)) (boxed for size optimization)
    MethodCall(Box<MethodCallData>),

    /// Index access (collection\[index\])
    Index {
        /// Base expression
        base: Box<ExpressionNode>,
        /// Index expression
        index: Box<ExpressionNode>,
    },

    /// Filter expression (collection.where(condition))
    Filter {
        /// Base expression
        base: Box<ExpressionNode>,
        /// Filter condition
        condition: Box<ExpressionNode>,
    },

    /// Union of collections (collection1 | collection2)
    Union {
        /// Left collection
        left: Box<ExpressionNode>,
        /// Right collection
        right: Box<ExpressionNode>,
    },

    /// Type check (value is Type)
    TypeCheck {
        /// Expression to check
        expression: Box<ExpressionNode>,
        /// Type name
        type_name: String,
    },

    /// Type cast (value as Type)
    TypeCast {
        /// Expression to cast
        expression: Box<ExpressionNode>,
        /// Target type name
        type_name: String,
    },

    /// Lambda expression for functions like where, select (boxed for size optimization)
    Lambda(Box<LambdaData>),

    /// Conditional expression (if-then-else) (boxed for size optimization)
    Conditional(Box<ConditionalData>),

    /// Variable reference ($this, $index, etc.)
    Variable(String),
}

/// Fast expression type enumeration for performance-critical code paths
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ExpressionType {
    /// Literal value
    Literal = 0,
    /// Identifier
    Identifier = 1,
    /// Path navigation
    Path = 2,
    /// Binary operation
    BinaryOp = 3,
    /// Unary operation
    UnaryOp = 4,
    /// Function call
    FunctionCall = 5,
    /// Method call
    MethodCall = 6,
    /// Index access
    Index = 7,
    /// Filter expression
    Filter = 8,
    /// Union of collections
    Union = 9,
    /// Type check
    TypeCheck = 10,
    /// Type cast
    TypeCast = 11,
    /// Lambda expression
    Lambda = 12,
    /// Conditional expression
    Conditional = 13,
    /// Variable reference
    Variable = 14,
}

/// Binary operation data (separate struct to optimize enum size)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinaryOpData {
    /// The operator
    pub op: BinaryOperator,
    /// Left operand
    pub left: ExpressionNode,
    /// Right operand  
    pub right: ExpressionNode,
}

/// Function call data (separate struct to optimize enum size)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionCallData {
    /// Function name
    pub name: String,
    /// Function arguments (SmallVec for common case of 2-4 args)
    pub args: SmallVec<[ExpressionNode; 4]>,
}

/// Method call data (separate struct to optimize enum size)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MethodCallData {
    /// Base expression to call method on
    pub base: ExpressionNode,
    /// Method name
    pub method: String,
    /// Method arguments (SmallVec for common case of 2-4 args)
    pub args: SmallVec<[ExpressionNode; 4]>,
}

/// Lambda expression data (separate struct to optimize enum size)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LambdaData {
    /// Parameter names (SmallVec for common case of 0-2 params)
    pub params: SmallVec<[String; 2]>,
    /// Lambda body
    pub body: ExpressionNode,
}

/// Conditional expression data (separate struct to optimize enum size)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConditionalData {
    /// Condition
    pub condition: ExpressionNode,
    /// Then branch
    pub then_expr: ExpressionNode,
    /// Else branch (optional)
    pub else_expr: Option<Box<ExpressionNode>>,
}

/// Literal values in FHIRPath
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LiteralValue {
    /// Boolean literal
    Boolean(bool),
    /// Integer literal
    Integer(i64),
    /// Decimal literal (stored as string to preserve precision)
    Decimal(String),
    /// String literal
    String(String),
    /// Date literal (YYYY-MM-DD)
    Date(String),
    /// DateTime literal (ISO 8601)
    DateTime(String),
    /// Time literal (HH:MM:SS)
    Time(String),
    /// Quantity literal
    Quantity {
        /// Numeric value
        value: String,
        /// Unit
        unit: String,
    },
    /// Null/empty literal
    Null,
}

impl ExpressionNode {
    /// Create a literal expression
    pub fn literal(value: LiteralValue) -> Self {
        Self::Literal(value)
    }

    /// Create an identifier expression
    pub fn identifier(name: impl Into<String>) -> Self {
        Self::Identifier(name.into())
    }

    /// Create a function call expression
    pub fn function_call(
        name: impl Into<String>,
        args: impl Into<SmallVec<[ExpressionNode; 4]>>,
    ) -> Self {
        Self::FunctionCall(Box::new(FunctionCallData {
            name: name.into(),
            args: args.into(),
        }))
    }

    /// Create a method call expression
    pub fn method_call(
        base: ExpressionNode,
        method: impl Into<String>,
        args: impl Into<SmallVec<[ExpressionNode; 4]>>,
    ) -> Self {
        Self::MethodCall(Box::new(MethodCallData {
            base,
            method: method.into(),
            args: args.into(),
        }))
    }

    /// Create a binary operation expression
    pub fn binary_op(op: BinaryOperator, left: ExpressionNode, right: ExpressionNode) -> Self {
        Self::BinaryOp(Box::new(BinaryOpData { op, left, right }))
    }

    /// Create a unary operation expression
    pub fn unary_op(op: UnaryOperator, operand: ExpressionNode) -> Self {
        Self::UnaryOp {
            op,
            operand: Box::new(operand),
        }
    }

    /// Create a path navigation expression
    pub fn path(base: ExpressionNode, path: impl Into<String>) -> Self {
        Self::Path {
            base: Box::new(base),
            path: path.into(),
        }
    }

    /// Create an index access expression
    pub fn index(base: ExpressionNode, index: ExpressionNode) -> Self {
        Self::Index {
            base: Box::new(base),
            index: Box::new(index),
        }
    }

    /// Create a filter expression
    pub fn filter(base: ExpressionNode, condition: ExpressionNode) -> Self {
        Self::Filter {
            base: Box::new(base),
            condition: Box::new(condition),
        }
    }

    /// Create a union expression
    pub fn union(left: ExpressionNode, right: ExpressionNode) -> Self {
        Self::Union {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Create a type check expression
    pub fn type_check(expression: ExpressionNode, type_name: impl Into<String>) -> Self {
        Self::TypeCheck {
            expression: Box::new(expression),
            type_name: type_name.into(),
        }
    }

    /// Create a type cast expression
    pub fn type_cast(expression: ExpressionNode, type_name: impl Into<String>) -> Self {
        Self::TypeCast {
            expression: Box::new(expression),
            type_name: type_name.into(),
        }
    }

    /// Create a lambda expression with multiple parameters
    pub fn lambda(params: impl Into<SmallVec<[String; 2]>>, body: ExpressionNode) -> Self {
        Self::Lambda(Box::new(LambdaData {
            params: params.into(),
            body,
        }))
    }

    /// Create a lambda expression with a single parameter
    pub fn lambda_single(param: impl Into<String>, body: ExpressionNode) -> Self {
        Self::Lambda(Box::new(LambdaData {
            params: vec![param.into()].into(),
            body,
        }))
    }

    /// Create an anonymous lambda expression (no parameters)
    pub fn lambda_anonymous(body: ExpressionNode) -> Self {
        Self::Lambda(Box::new(LambdaData {
            params: SmallVec::new(),
            body,
        }))
    }

    /// Create a conditional expression
    pub fn conditional(
        condition: ExpressionNode,
        then_expr: ExpressionNode,
        else_expr: Option<ExpressionNode>,
    ) -> Self {
        Self::Conditional(Box::new(ConditionalData {
            condition,
            then_expr,
            else_expr: else_expr.map(Box::new),
        }))
    }

    /// Create a variable reference
    pub fn variable(name: impl Into<String>) -> Self {
        Self::Variable(name.into())
    }

    /// Check if this expression is a literal
    pub fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    /// Check if this expression is an identifier
    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::Identifier(_))
    }

    /// Get the literal value if this is a literal expression
    pub fn as_literal(&self) -> Option<&LiteralValue> {
        match self {
            Self::Literal(value) => Some(value),
            _ => None,
        }
    }

    /// Get the identifier name if this is an identifier expression
    pub fn as_identifier(&self) -> Option<&str> {
        match self {
            Self::Identifier(name) => Some(name),
            _ => None,
        }
    }

    /// Fast check for common expression types (optimized for hot paths)
    #[inline(always)]
    pub fn expression_type(&self) -> ExpressionType {
        match self {
            Self::Literal(_) => ExpressionType::Literal,
            Self::Identifier(_) => ExpressionType::Identifier,
            Self::Path { .. } => ExpressionType::Path,
            Self::BinaryOp(..) => ExpressionType::BinaryOp,
            Self::UnaryOp { .. } => ExpressionType::UnaryOp,
            Self::FunctionCall(_) => ExpressionType::FunctionCall,
            Self::MethodCall(_) => ExpressionType::MethodCall,
            Self::Index { .. } => ExpressionType::Index,
            Self::Filter { .. } => ExpressionType::Filter,
            Self::Union { .. } => ExpressionType::Union,
            Self::TypeCheck { .. } => ExpressionType::TypeCheck,
            Self::TypeCast { .. } => ExpressionType::TypeCast,
            Self::Lambda(_) => ExpressionType::Lambda,
            Self::Conditional(_) => ExpressionType::Conditional,
            Self::Variable(_) => ExpressionType::Variable,
        }
    }

    /// Check if this is a path expression (optimized for hot path)
    #[inline(always)]
    pub fn is_path(&self) -> bool {
        matches!(self, Self::Path { .. })
    }

    /// Check if this is a binary operation (optimized for hot path)
    #[inline(always)]
    pub fn is_binary_op(&self) -> bool {
        matches!(self, Self::BinaryOp(..))
    }

    /// Get path components if this is a path expression
    pub fn as_path(&self) -> Option<(&ExpressionNode, &str)> {
        match self {
            Self::Path { base, path } => Some((base, path)),
            _ => None,
        }
    }

    /// Get binary operation components if this is a binary operation
    pub fn as_binary_op(&self) -> Option<(BinaryOperator, &ExpressionNode, &ExpressionNode)> {
        match self {
            Self::BinaryOp(data) => Some((data.op, &data.left, &data.right)),
            _ => None,
        }
    }

    /// Get variable name if this is a variable expression
    pub fn as_variable(&self) -> Option<&str> {
        match self {
            Self::Variable(name) => Some(name),
            _ => None,
        }
    }

    /// Check if this expression is "simple" (contains no nested expressions)
    /// Useful for optimization decisions
    #[inline]
    pub fn is_simple(&self) -> bool {
        matches!(
            self,
            Self::Literal(_) | Self::Identifier(_) | Self::Variable(_)
        )
    }

    /// Estimate the complexity of this expression (for optimization hints)
    pub fn complexity(&self) -> usize {
        match self {
            Self::Literal(_) | Self::Identifier(_) | Self::Variable(_) => 1,
            Self::Path { base, .. } => 1 + base.complexity(),
            Self::BinaryOp(data) => 1 + data.left.complexity() + data.right.complexity(),
            Self::UnaryOp { operand, .. } => 1 + operand.complexity(),
            Self::FunctionCall(data) => {
                1 + data.args.iter().map(|arg| arg.complexity()).sum::<usize>()
            }
            Self::MethodCall(data) => {
                1 + data.base.complexity()
                    + data.args.iter().map(|arg| arg.complexity()).sum::<usize>()
            }
            Self::Index { base, index } => 1 + base.complexity() + index.complexity(),
            Self::Filter { base, condition } => 1 + base.complexity() + condition.complexity(),
            Self::Union { left, right } => 1 + left.complexity() + right.complexity(),
            Self::TypeCheck { expression, .. } => 1 + expression.complexity(),
            Self::TypeCast { expression, .. } => 1 + expression.complexity(),
            Self::Lambda(data) => 1 + data.body.complexity(),
            Self::Conditional(data) => {
                1 + data.condition.complexity()
                    + data.then_expr.complexity()
                    + data.else_expr.as_ref().map_or(0, |e| e.complexity())
            }
        }
    }

    /// Fast clone for simple expressions (avoids deep cloning when possible)
    ///
    /// For simple expressions (literals, identifiers, variables), this performs
    /// a regular clone. For complex expressions, it may use reference counting
    /// or other optimizations to avoid expensive deep clones.
    #[inline]
    pub fn clone_optimized(&self) -> Self {
        if self.is_simple() {
            // Simple expressions are cheap to clone
            self.clone()
        } else {
            // For complex expressions, just clone normally
            // In the future, this could use Rc/Arc for expensive operations
            self.clone()
        }
    }

    /// Try to clone without deep copying by checking if this is a simple expression
    /// Returns None if the expression is too complex and should be cloned normally
    #[inline]
    pub fn try_clone_shallow(&self) -> Option<Self> {
        match self {
            Self::Literal(lit) => Some(Self::Literal(lit.clone())),
            Self::Identifier(name) => Some(Self::Identifier(name.clone())),
            Self::Variable(name) => Some(Self::Variable(name.clone())),
            _ => None, // Complex expressions require full clone
        }
    }

    /// Check if cloning this expression would be expensive
    /// Useful for making optimization decisions in hot paths
    #[inline]
    pub fn is_expensive_to_clone(&self) -> bool {
        match self {
            Self::Literal(_) | Self::Identifier(_) | Self::Variable(_) => false,
            Self::Path { .. } => false, // Single level of boxing
            Self::BinaryOp(..) | Self::UnaryOp { .. } => true, // Multiple boxed children
            Self::FunctionCall(data) => !data.args.is_empty(),
            Self::MethodCall(_) => true, // Always has base + potentially args
            _ => true,                   // Conservative: assume complex expressions are expensive
        }
    }
}

impl LiteralValue {
    /// Check if this is a null literal
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Check if this is a boolean literal
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    /// Check if this is an integer literal
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    /// Check if this is a decimal literal
    pub fn is_decimal(&self) -> bool {
        matches!(self, Self::Decimal(_))
    }

    /// Check if this is a string literal
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Get the boolean value if this is a boolean literal
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get the integer value if this is an integer literal
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get the string value if this is a string literal
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Check if this literal is expensive to clone (contains heap-allocated data)
    #[inline]
    pub fn is_expensive_to_clone(&self) -> bool {
        matches!(
            self,
            Self::Decimal(_)
                | Self::String(_)
                | Self::Date(_)
                | Self::DateTime(_)
                | Self::Time(_)
                | Self::Quantity { .. }
        )
    }

    /// Get the estimated size in bytes for this literal value
    pub fn estimated_size(&self) -> usize {
        match self {
            Self::Boolean(_) => 1,
            Self::Integer(_) => 8,
            Self::Decimal(s) => std::mem::size_of::<String>() + s.len(),
            Self::String(s) => std::mem::size_of::<String>() + s.len(),
            Self::Date(s) => std::mem::size_of::<String>() + s.len(),
            Self::DateTime(s) => std::mem::size_of::<String>() + s.len(),
            Self::Time(s) => std::mem::size_of::<String>() + s.len(),
            Self::Quantity { value, unit } => {
                std::mem::size_of::<String>() * 2 + value.len() + unit.len()
            }
            Self::Null => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda_constructors() {
        // Test anonymous lambda
        let anon_lambda = ExpressionNode::lambda_anonymous(ExpressionNode::identifier("value"));
        if let ExpressionNode::Lambda(lambda_data) = anon_lambda {
            assert_eq!(lambda_data.params.len(), 0);
        } else {
            panic!("Expected Lambda");
        }

        // Test single parameter lambda
        let single_lambda = ExpressionNode::lambda_single("x", ExpressionNode::identifier("x"));
        if let ExpressionNode::Lambda(lambda_data) = single_lambda {
            assert_eq!(lambda_data.params.len(), 1);
            assert_eq!(lambda_data.params[0], "x");
        } else {
            panic!("Expected Lambda");
        }

        // Test multiple parameter lambda
        let multi_lambda = ExpressionNode::lambda(
            vec!["x".to_string(), "y".to_string()],
            ExpressionNode::identifier("result"),
        );
        if let ExpressionNode::Lambda(lambda_data) = multi_lambda {
            assert_eq!(lambda_data.params.len(), 2);
            assert_eq!(lambda_data.params[0], "x");
            assert_eq!(lambda_data.params[1], "y");
        } else {
            panic!("Expected Lambda");
        }
    }

    #[test]
    fn test_performance_optimizations() {
        // Test expression type identification
        let literal = ExpressionNode::literal(LiteralValue::Integer(42));
        assert_eq!(literal.expression_type(), ExpressionType::Literal);
        assert!(literal.is_simple());
        assert!(!literal.is_expensive_to_clone());
        assert_eq!(literal.complexity(), 1);

        let path = ExpressionNode::path(ExpressionNode::identifier("Patient"), "name");
        assert_eq!(path.expression_type(), ExpressionType::Path);
        assert!(path.is_path());
        assert!(!path.is_simple());
        assert!(!path.is_expensive_to_clone());
        assert_eq!(path.complexity(), 2);

        let binary_op = ExpressionNode::binary_op(
            BinaryOperator::Equal,
            ExpressionNode::identifier("active"),
            ExpressionNode::literal(LiteralValue::Boolean(true)),
        );
        assert_eq!(binary_op.expression_type(), ExpressionType::BinaryOp);
        assert!(binary_op.is_binary_op());
        assert!(!binary_op.is_simple());
        assert!(binary_op.is_expensive_to_clone());
        assert_eq!(binary_op.complexity(), 3);
    }

    #[test]
    fn test_clone_optimizations() {
        // Test shallow cloning for simple expressions
        let literal = ExpressionNode::literal(LiteralValue::Integer(42));
        let shallow_clone = literal.try_clone_shallow();
        assert!(shallow_clone.is_some());
        assert_eq!(shallow_clone.unwrap(), literal);

        // Test that complex expressions return None for shallow clone
        let complex = ExpressionNode::path(ExpressionNode::identifier("Patient"), "name");
        assert!(complex.try_clone_shallow().is_none());

        // Test optimized clone
        let optimized_clone = literal.clone_optimized();
        assert_eq!(optimized_clone, literal);
    }

    #[test]
    fn test_literal_value_optimizations() {
        let bool_lit = LiteralValue::Boolean(true);
        assert!(!bool_lit.is_expensive_to_clone());
        assert_eq!(bool_lit.estimated_size(), 1);

        let string_lit = LiteralValue::String("test123".to_string());
        assert!(string_lit.is_expensive_to_clone());
        assert!(string_lit.estimated_size() > 20); // String + content

        let quantity_lit = LiteralValue::Quantity {
            value: "42".to_string(),
            unit: "kg".to_string(),
        };
        assert!(quantity_lit.is_expensive_to_clone());
        assert!(quantity_lit.estimated_size() > 40); // Two strings + content
    }
}
