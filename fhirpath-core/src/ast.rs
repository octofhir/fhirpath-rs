//! Abstract Syntax Tree (AST) definitions for FHIRPath expressions
//!
//! This module defines the AST nodes that represent parsed FHIRPath expressions.

use crate::model::FhirPathValue;

/// AST representation of FHIRPath expressions
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionNode {
    /// Literal value (string, number, boolean, etc.)
    Literal(FhirPathValue),

    /// Identifier (variable name, property name)
    Identifier(String),

    /// Function call with name and arguments
    FunctionCall {
        name: String,
        args: Vec<ExpressionNode>
    },

    /// Binary operation (arithmetic, comparison, logical)
    BinaryOp {
        op: BinaryOperator,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>
    },

    /// Unary operation (negation, not)
    UnaryOp {
        op: UnaryOperator,
        operand: Box<ExpressionNode>
    },

    /// Path navigation (object.property)
    Path {
        base: Box<ExpressionNode>,
        path: String
    },

    /// Index access (collection[index])
    Index {
        base: Box<ExpressionNode>,
        index: Box<ExpressionNode>
    },

    /// Filter expression (collection.where(condition))
    Filter {
        base: Box<ExpressionNode>,
        condition: Box<ExpressionNode>
    },

    /// Union of collections (collection1 | collection2)
    Union {
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>
    },

    /// Type check (value is Type)
    TypeCheck {
        expression: Box<ExpressionNode>,
        type_name: String
    },

    /// Type cast (value as Type)
    TypeCast {
        expression: Box<ExpressionNode>,
        type_name: String
    },
}

/// Binary operators in FHIRPath
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic operators
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison operators
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    // Equivalence operators
    Equivalent,
    NotEquivalent,

    // Logical operators
    And,
    Or,
    Xor,
    Implies,

    // String operators
    Contains,
    In,

    // Collection operators
    Union,

    // String concatenation
    Concatenate,

    // Type checking
    Is,
}

/// Unary operators in FHIRPath
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    /// Logical negation (not)
    Not,
    /// Arithmetic negation (-)
    Minus,
    /// Arithmetic positive (+)
    Plus,
}

impl ExpressionNode {
    /// Create a literal expression
    pub fn literal(value: FhirPathValue) -> Self {
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

    /// Check if this expression is a literal
    pub fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    /// Check if this expression is an identifier
    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::Identifier(_))
    }

    /// Get the literal value if this is a literal expression
    pub fn as_literal(&self) -> Option<&FhirPathValue> {
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

impl BinaryOperator {
    /// Get the precedence of this operator (higher number = higher precedence)
    pub fn precedence(&self) -> u8 {
        match self {
            // Highest precedence
            Self::Multiply | Self::Divide | Self::Modulo => 7,
            Self::Add | Self::Subtract => 6,
            Self::LessThan | Self::LessThanOrEqual | Self::GreaterThan | Self::GreaterThanOrEqual => 5,
            Self::Equal | Self::NotEqual | Self::Equivalent | Self::NotEquivalent => 4,
            Self::Contains | Self::In | Self::Is => 4,
            Self::Concatenate => 3,
            Self::And => 3,
            Self::Xor => 2,
            Self::Or => 1,
            Self::Implies => 0,
            Self::Union => 0,
        }
    }

    /// Check if this operator is left-associative
    pub fn is_left_associative(&self) -> bool {
        // Most operators are left-associative, implies is right-associative
        !matches!(self, Self::Implies)
    }

    /// Get the string representation of this operator
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "mod",
            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::Equivalent => "~",
            Self::NotEquivalent => "!~",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Implies => "implies",
            Self::Contains => "contains",
            Self::In => "in",
            Self::Union => "|",
            Self::Concatenate => "&",
            Self::Is => "is",
        }
    }
}

impl UnaryOperator {
    /// Get the string representation of this operator
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Not => "not",
            Self::Minus => "-",
            Self::Plus => "+",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_creation() {
        let literal = ExpressionNode::literal(FhirPathValue::Integer(42));
        assert!(literal.is_literal());
        assert_eq!(literal.as_literal(), Some(&FhirPathValue::Integer(42)));

        let identifier = ExpressionNode::identifier("name");
        assert!(identifier.is_identifier());
        assert_eq!(identifier.as_identifier(), Some("name"));
    }

    #[test]
    fn test_binary_operator_precedence() {
        assert!(BinaryOperator::Multiply.precedence() > BinaryOperator::Add.precedence());
        assert!(BinaryOperator::Add.precedence() > BinaryOperator::Equal.precedence());
        assert!(BinaryOperator::Equal.precedence() > BinaryOperator::And.precedence());
    }

    #[test]
    fn test_complex_expression() {
        // Create expression: name.first() + " " + name.last()
        let name_first = ExpressionNode::function_call(
            "first",
            vec![ExpressionNode::path(
                ExpressionNode::identifier("name"),
                "given"
            )]
        );

        let space = ExpressionNode::literal(FhirPathValue::String(" ".to_string()));

        let name_last = ExpressionNode::path(
            ExpressionNode::identifier("name"),
            "family"
        );

        let full_name = ExpressionNode::binary_op(
            BinaryOperator::Add,
            ExpressionNode::binary_op(BinaryOperator::Add, name_first, space),
            name_last
        );

        // Just verify it compiles and has the right structure
        match full_name {
            ExpressionNode::BinaryOp { op: BinaryOperator::Add, .. } => {},
            _ => panic!("Expected binary operation"),
        }
    }
}
