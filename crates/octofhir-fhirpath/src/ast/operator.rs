//! Operator definitions for FHIRPath expressions
//!
//! This module defines all binary and unary operators used in FHIRPath expressions,
//! with proper precedence rules and semantic validation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Binary operators in FHIRPath expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic operators
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Subtract,
    /// Multiplication (*)
    Multiply,
    /// Division (/)
    Divide,
    /// Modulo (mod)
    Modulo,
    /// Integer division (div)
    IntegerDivide,

    // Comparison operators
    /// Equality (=)
    Equal,
    /// Inequality (!=)
    NotEqual,
    /// Equivalence (~)
    Equivalent,
    /// Non-equivalence (!~)
    NotEquivalent,
    /// Less than (<)
    LessThan,
    /// Less than or equal (<=)
    LessThanOrEqual,
    /// Greater than (>)
    GreaterThan,
    /// Greater than or equal (>=)
    GreaterThanOrEqual,

    // Logical operators
    /// Logical AND (and)
    And,
    /// Logical OR (or)
    Or,
    /// Logical XOR (xor)
    Xor,
    /// Implication (implies)
    Implies,

    // String operators
    /// String concatenation (&)
    Concatenate,

    // Collection operators
    /// Collection union (|)
    Union,
    /// Collection membership (in)
    In,
    /// Collection containment (contains)
    Contains,

    // Type operators
    /// Type checking (is)
    Is,
    /// Type casting (as)
    As,
}

/// Unary operators in FHIRPath expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// Arithmetic negation (-)
    Negate,
    /// Logical negation (not)
    Not,
    /// Positive sign (+)
    Positive,
}

impl BinaryOperator {
    /// Get the precedence level of this operator (higher = binds tighter)
    /// Based on FHIRPath specification: http://hl7.org/fhirpath/#grammar
    /// Spec precedence levels converted to higher=tighter internal representation
    pub fn precedence(self) -> u8 {
        match self {
            // Multiplicative operators: *, /, div, mod (spec level 4 -> internal 10)
            Self::Multiply | Self::Divide | Self::IntegerDivide | Self::Modulo => 10,

            // Additive operators: +, -, & (spec level 5 -> internal 9)
            Self::Add | Self::Subtract | Self::Concatenate => 9,

            // Type operators: is, as (spec level 6 -> internal 8)
            Self::Is | Self::As => 8,

            // Collection union: | (spec level 7 -> internal 7)
            Self::Union => 7,

            // Relational operators: >, <, >=, <= (spec level 8 -> internal 6)
            Self::LessThan
            | Self::LessThanOrEqual
            | Self::GreaterThan
            | Self::GreaterThanOrEqual => 6,

            // Equality operators: =, ~, !=, !~ (spec level 9 -> internal 5)
            Self::Equal | Self::NotEqual | Self::Equivalent | Self::NotEquivalent => 5,

            // Membership operators: in, contains (spec level 10 -> internal 4)
            Self::In | Self::Contains => 4,

            // Logical AND (spec level 11 -> internal 3)
            Self::And => 3,

            // Logical XOR, OR (spec level 12 -> internal 2)
            Self::Xor | Self::Or => 2,

            // Lowest precedence: Implication (spec level 13 -> internal 1)
            Self::Implies => 1,
        }
    }

    /// Check if this operator is left-associative
    pub fn is_left_associative(self) -> bool {
        match self {
            // Right-associative operators
            Self::Implies => false,

            // All others are left-associative
            _ => true,
        }
    }

    /// Check if this operator is arithmetic
    pub fn is_arithmetic(self) -> bool {
        matches!(
            self,
            Self::Add
                | Self::Subtract
                | Self::Multiply
                | Self::Divide
                | Self::IntegerDivide
                | Self::Modulo
        )
    }

    /// Check if this operator is comparison
    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Equal
                | Self::NotEqual
                | Self::Equivalent
                | Self::NotEquivalent
                | Self::LessThan
                | Self::LessThanOrEqual
                | Self::GreaterThan
                | Self::GreaterThanOrEqual
        )
    }

    /// Check if this operator is logical
    pub fn is_logical(self) -> bool {
        matches!(self, Self::And | Self::Or | Self::Xor | Self::Implies)
    }

    /// Check if this operator works on collections
    pub fn is_collection_operator(self) -> bool {
        matches!(self, Self::Union | Self::In | Self::Contains)
    }

    /// Check if this operator is a type operator
    pub fn is_type_operator(self) -> bool {
        matches!(self, Self::Is | Self::As)
    }

    /// Get the symbol representation of this operator
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "mod",
            Self::IntegerDivide => "div",
            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::Equivalent => "~",
            Self::NotEquivalent => "!~",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Implies => "implies",
            Self::Concatenate => "&",
            Self::Union => "|",
            Self::In => "in",
            Self::Contains => "contains",
            Self::Is => "is",
            Self::As => "as",
        }
    }

    /// Parse an operator from its string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "+" => Some(Self::Add),
            "-" => Some(Self::Subtract),
            "*" => Some(Self::Multiply),
            "/" => Some(Self::Divide),
            "mod" => Some(Self::Modulo),
            "div" => Some(Self::IntegerDivide),
            "=" => Some(Self::Equal),
            "!=" => Some(Self::NotEqual),
            "~" => Some(Self::Equivalent),
            "!~" => Some(Self::NotEquivalent),
            "<" => Some(Self::LessThan),
            "<=" => Some(Self::LessThanOrEqual),
            ">" => Some(Self::GreaterThan),
            ">=" => Some(Self::GreaterThanOrEqual),
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
            "xor" => Some(Self::Xor),
            "implies" => Some(Self::Implies),
            "&" => Some(Self::Concatenate),
            "|" => Some(Self::Union),
            "in" => Some(Self::In),
            "contains" => Some(Self::Contains),
            "is" => Some(Self::Is),
            "as" => Some(Self::As),
            _ => None,
        }
    }

    /// Get a human-readable description of what this operator does
    pub fn description(&self) -> &'static str {
        match self {
            Self::Add => "Addition of numeric values or quantities",
            Self::Subtract => "Subtraction of numeric values or quantities",
            Self::Multiply => "Multiplication of numeric values or quantities",
            Self::Divide => "Division of numeric values or quantities",
            Self::Modulo => "Modulo operation (remainder after division)",
            Self::IntegerDivide => "Integer division (truncated result)",
            Self::Equal => "Equality comparison (type-aware)",
            Self::NotEqual => "Inequality comparison",
            Self::Equivalent => "Equivalence comparison (type-coercing)",
            Self::NotEquivalent => "Non-equivalence comparison",
            Self::LessThan => "Less than comparison for ordered types",
            Self::LessThanOrEqual => "Less than or equal comparison",
            Self::GreaterThan => "Greater than comparison for ordered types",
            Self::GreaterThanOrEqual => "Greater than or equal comparison",
            Self::And => "Logical AND operation",
            Self::Or => "Logical OR operation",
            Self::Xor => "Logical exclusive OR operation",
            Self::Implies => "Logical implication (if A then B)",
            Self::Concatenate => "String concatenation",
            Self::Union => "Collection union (combines collections)",
            Self::In => "Membership test (item in collection)",
            Self::Contains => "Containment test (collection contains item)",
            Self::Is => "Type checking (runtime type test)",
            Self::As => "Type casting (convert to specified type)",
        }
    }
}

impl UnaryOperator {
    /// Get the precedence level of unary operators (always high)
    pub fn precedence() -> u8 {
        14 // Higher than all binary operators
    }

    /// Get the symbol representation of this operator
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Negate => "-",
            Self::Not => "not",
            Self::Positive => "+",
        }
    }

    /// Parse a unary operator from its string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "-" => Some(Self::Negate),
            "not" => Some(Self::Not),
            "+" => Some(Self::Positive),
            _ => None,
        }
    }

    /// Check if this operator is arithmetic
    pub fn is_arithmetic(self) -> bool {
        matches!(self, Self::Negate | Self::Positive)
    }

    /// Check if this operator is logical
    pub fn is_logical(self) -> bool {
        matches!(self, Self::Not)
    }

    /// Get a human-readable description of what this operator does
    pub fn description(&self) -> &'static str {
        match self {
            Self::Negate => "Arithmetic negation (changes sign of numeric values)",
            Self::Not => "Logical negation (boolean NOT operation)",
            Self::Positive => "Positive sign (no-op for numeric values)",
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// Operator associativity for parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    /// Left-associative operators
    Left,
    /// Right-associative operators  
    Right,
}

impl BinaryOperator {
    /// Get the associativity of this operator
    pub fn associativity(self) -> Associativity {
        if self.is_left_associative() {
            Associativity::Left
        } else {
            Associativity::Right
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_ordering() {
        // Higher precedence operators should bind tighter
        assert!(BinaryOperator::Multiply.precedence() > BinaryOperator::Add.precedence());
        assert!(BinaryOperator::Add.precedence() > BinaryOperator::Equal.precedence());
        assert!(BinaryOperator::Equal.precedence() > BinaryOperator::And.precedence());
        assert!(BinaryOperator::And.precedence() > BinaryOperator::Or.precedence());
        assert!(BinaryOperator::Or.precedence() > BinaryOperator::Implies.precedence());
    }

    #[test]
    fn test_operator_parsing() {
        assert_eq!(BinaryOperator::from_str("+"), Some(BinaryOperator::Add));
        assert_eq!(BinaryOperator::from_str("and"), Some(BinaryOperator::And));
        assert_eq!(BinaryOperator::from_str("invalid"), None);

        assert_eq!(UnaryOperator::from_str("-"), Some(UnaryOperator::Negate));
        assert_eq!(UnaryOperator::from_str("not"), Some(UnaryOperator::Not));
        assert_eq!(UnaryOperator::from_str("invalid"), None);
    }

    #[test]
    fn test_operator_categorization() {
        assert!(BinaryOperator::Add.is_arithmetic());
        assert!(BinaryOperator::Equal.is_comparison());
        assert!(BinaryOperator::And.is_logical());
        assert!(BinaryOperator::Union.is_collection_operator());
        assert!(BinaryOperator::Is.is_type_operator());

        assert!(UnaryOperator::Negate.is_arithmetic());
        assert!(UnaryOperator::Not.is_logical());
    }

    #[test]
    fn test_associativity() {
        assert!(BinaryOperator::Add.is_left_associative());
        assert!(!BinaryOperator::Implies.is_left_associative());

        assert_eq!(BinaryOperator::Add.associativity(), Associativity::Left);
        assert_eq!(
            BinaryOperator::Implies.associativity(),
            Associativity::Right
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(BinaryOperator::Add.to_string(), "+");
        assert_eq!(BinaryOperator::And.to_string(), "and");
        assert_eq!(UnaryOperator::Negate.to_string(), "-");
        assert_eq!(UnaryOperator::Not.to_string(), "not");
    }

    #[test]
    fn test_unary_precedence() {
        // Unary operators should have higher precedence than binary operators
        assert!(UnaryOperator::precedence() > BinaryOperator::Multiply.precedence());
    }
}
