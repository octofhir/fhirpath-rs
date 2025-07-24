//! Expression AST node definitions

use crate::operator::{BinaryOperator, UnaryOperator};

/// AST representation of FHIRPath expressions
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExpressionNode {
    /// Literal value (string, number, boolean, etc.)
    Literal(LiteralValue),

    /// Identifier (variable name, property name)
    Identifier(String),

    /// Function call with name and arguments
    FunctionCall {
        /// Function name
        name: String,
        /// Function arguments
        args: Vec<ExpressionNode>,
    },

    /// Binary operation (arithmetic, comparison, logical)
    BinaryOp {
        /// The operator
        op: BinaryOperator,
        /// Left operand
        left: Box<ExpressionNode>,
        /// Right operand
        right: Box<ExpressionNode>,
    },

    /// Unary operation (negation, not)
    UnaryOp {
        /// The operator
        op: UnaryOperator,
        /// The operand
        operand: Box<ExpressionNode>,
    },

    /// Path navigation (object.property)
    Path {
        /// Base expression
        base: Box<ExpressionNode>,
        /// Property path
        path: String,
    },

    /// Index access (collection[index])
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

    /// Lambda expression for functions like where, select
    Lambda {
        /// Parameter name (usually $this)
        param: String,
        /// Lambda body
        body: Box<ExpressionNode>,
    },

    /// Conditional expression (if-then-else)
    Conditional {
        /// Condition
        condition: Box<ExpressionNode>,
        /// Then branch
        then_expr: Box<ExpressionNode>,
        /// Else branch (optional)
        else_expr: Option<Box<ExpressionNode>>,
    },

    /// Variable reference ($this, $index, etc.)
    Variable(String),
}

/// Literal values in FHIRPath
#[derive(Debug, Clone, PartialEq)]
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
    pub fn function_call(name: impl Into<String>, args: Vec<ExpressionNode>) -> Self {
        Self::FunctionCall {
            name: name.into(),
            args,
        }
    }

    /// Create a binary operation expression
    pub fn binary_op(op: BinaryOperator, left: ExpressionNode, right: ExpressionNode) -> Self {
        Self::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
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

    /// Create a lambda expression
    pub fn lambda(param: impl Into<String>, body: ExpressionNode) -> Self {
        Self::Lambda {
            param: param.into(),
            body: Box::new(body),
        }
    }

    /// Create a conditional expression
    pub fn conditional(
        condition: ExpressionNode,
        then_expr: ExpressionNode,
        else_expr: Option<ExpressionNode>,
    ) -> Self {
        Self::Conditional {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: else_expr.map(Box::new),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::BinaryOperator;

    #[test]
    fn test_expression_creation() {
        let literal = ExpressionNode::literal(LiteralValue::Integer(42));
        assert!(literal.is_literal());
        assert_eq!(
            literal.as_literal(),
            Some(&LiteralValue::Integer(42))
        );

        let identifier = ExpressionNode::identifier("name");
        assert!(identifier.is_identifier());
        assert_eq!(identifier.as_identifier(), Some("name"));
    }

    #[test]
    fn test_complex_expression() {
        // Create expression: name.first() + " " + name.last()
        let name_first = ExpressionNode::function_call(
            "first",
            vec![ExpressionNode::path(
                ExpressionNode::identifier("name"),
                "given",
            )],
        );

        let space = ExpressionNode::literal(LiteralValue::String(" ".to_string()));

        let name_last = ExpressionNode::path(ExpressionNode::identifier("name"), "family");

        let full_name = ExpressionNode::binary_op(
            BinaryOperator::Add,
            ExpressionNode::binary_op(BinaryOperator::Add, name_first, space),
            name_last,
        );

        // Just verify it compiles and has the right structure
        match full_name {
            ExpressionNode::BinaryOp {
                op: BinaryOperator::Add,
                ..
            } => {}
            _ => panic!("Expected binary operation"),
        }
    }
}