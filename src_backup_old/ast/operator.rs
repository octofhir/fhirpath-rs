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

//! Operator definitions for FHIRPath

/// Binary operators in FHIRPath
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Integer division (div)
    IntegerDivide,
    /// Modulo (mod)
    Modulo,

    // Comparison operators
    /// Equality (=)
    Equal,
    /// Inequality (!=)
    NotEqual,
    /// Less than (<)
    LessThan,
    /// Less than or equal (<=)
    LessThanOrEqual,
    /// Greater than (>)
    GreaterThan,
    /// Greater than or equal (>=)
    GreaterThanOrEqual,

    // Equivalence operators
    /// Equivalence (~)
    Equivalent,
    /// Non-equivalence (!~)
    NotEquivalent,

    // Logical operators
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Logical XOR
    Xor,
    /// Implication
    Implies,

    // String operators
    /// String/collection contains
    Contains,
    /// Element in collection
    In,

    // Collection operators
    /// Union (|)
    Union,

    // String concatenation
    /// Concatenation (&)
    Concatenate,

    // Type checking
    /// Type check (is)
    Is,
}

/// Unary operators in FHIRPath
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnaryOperator {
    /// Logical negation (not)
    Not,
    /// Arithmetic negation (-)
    Minus,
    /// Arithmetic positive (+)
    Plus,
}

/// Operator associativity
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Associativity {
    /// Left-to-right associativity
    Left,
    /// Right-to-left associativity
    Right,
}

impl BinaryOperator {
    /// Get the precedence of this operator (higher number = higher precedence)
    pub fn precedence(&self) -> u8 {
        match self {
            // Highest precedence
            Self::Multiply | Self::Divide | Self::IntegerDivide | Self::Modulo => 7,
            Self::Add | Self::Subtract => 6,
            Self::LessThan
            | Self::LessThanOrEqual
            | Self::GreaterThan
            | Self::GreaterThanOrEqual => 5,
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

    /// Get the associativity of this operator
    pub fn associativity(&self) -> Associativity {
        if self.is_left_associative() {
            Associativity::Left
        } else {
            Associativity::Right
        }
    }

    /// Get the string representation of this operator
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::IntegerDivide => "div",
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

    /// Get the precedence of this operator
    pub fn precedence(&self) -> u8 {
        10 // Unary operators have highest precedence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_operator_precedence() {
        assert!(BinaryOperator::Multiply.precedence() > BinaryOperator::Add.precedence());
        assert!(BinaryOperator::Add.precedence() > BinaryOperator::Equal.precedence());
        assert!(BinaryOperator::Equal.precedence() > BinaryOperator::And.precedence());
        assert!(BinaryOperator::And.precedence() > BinaryOperator::Or.precedence());
    }

    #[test]
    fn test_operator_associativity() {
        assert_eq!(BinaryOperator::Add.associativity(), Associativity::Left);
        assert_eq!(
            BinaryOperator::Implies.associativity(),
            Associativity::Right
        );
    }

    #[test]
    fn test_operator_string_representation() {
        assert_eq!(BinaryOperator::Add.as_str(), "+");
        assert_eq!(BinaryOperator::IntegerDivide.as_str(), "div");
        assert_eq!(BinaryOperator::Equivalent.as_str(), "~");
        assert_eq!(UnaryOperator::Not.as_str(), "not");
    }
}
