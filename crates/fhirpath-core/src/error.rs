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

//! Error types for FHIRPath evaluation
//!
//! This module defines the error types used throughout the FHIRPath engine.

use thiserror::Error;

/// Result type alias for FHIRPath operations
pub type Result<T> = std::result::Result<T, FhirPathError>;

/// Comprehensive error type for FHIRPath operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum FhirPathError {
    /// Parsing errors
    #[error("Parse error at position {position}: {message}")]
    ParseError {
        /// Position in the input where the parse error occurred
        position: usize,
        /// Human-readable error message
        message: String,
    },

    /// Type errors during evaluation
    #[error("Type error: {message}")]
    TypeError {
        /// Human-readable type error message
        message: String,
    },

    /// Runtime evaluation errors
    #[error("Evaluation error: {message}")]
    EvaluationError {
        /// Human-readable evaluation error message
        message: String,
    },

    /// Function call errors
    #[error("Function '{function_name}' error: {message}")]
    FunctionError {
        /// Name of the function that caused the error
        function_name: String,
        /// Human-readable error message
        message: String,
    },

    /// Invalid expression structure
    #[error("Invalid expression: {message}")]
    InvalidExpression {
        /// Human-readable invalid expression error message
        message: String,
    },

    /// Division by zero or other arithmetic errors
    #[error("Arithmetic error: {message}")]
    ArithmeticError {
        /// Human-readable arithmetic error message
        message: String,
    },

    /// Index out of bounds
    #[error("Index out of bounds: {index} for collection of size {size}")]
    IndexOutOfBounds {
        /// The index that was out of bounds
        index: i64,
        /// The size of the collection
        size: usize,
    },

    /// Unknown function
    #[error("Unknown function: {function_name}")]
    UnknownFunction {
        /// Name of the unknown function
        function_name: String,
    },

    /// Invalid argument count
    #[error("Function '{function_name}' expects {expected} arguments, got {actual}")]
    InvalidArgumentCount {
        /// Name of the function with invalid argument count
        function_name: String,
        /// Expected number of arguments
        expected: usize,
        /// Actual number of arguments received
        actual: usize,
    },

    /// Conversion errors
    #[error("Conversion error: cannot convert {from} to {to}")]
    ConversionError {
        /// Source type that could not be converted
        from: String,
        /// Target type for the conversion
        to: String,
    },

    /// Generic error for compatibility
    #[error("FHIRPath error: {message}")]
    Generic {
        /// Generic error message
        message: String,
    },

    /// Invalid function arguments
    #[error("Invalid arguments: {message}")]
    InvalidArguments {
        /// Human-readable error message
        message: String,
    },

    /// Unknown operator
    #[error("Unknown operator: '{operator}'")]
    UnknownOperator {
        /// The unknown operator
        operator: String,
    },

    /// Invalid operand types for operator
    #[error("Invalid operand types for operator '{operator}': {left_type} and {right_type}")]
    InvalidOperandTypes {
        /// The operator with invalid operand types
        operator: String,
        /// Type of the left operand
        left_type: String,
        /// Type of the right operand
        right_type: String,
    },

    /// Incompatible units
    #[error("Incompatible units: '{left_unit}' and '{right_unit}'")]
    IncompatibleUnits {
        /// Unit of the left operand
        left_unit: String,
        /// Unit of the right operand
        right_unit: String,
    },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Arithmetic overflow
    #[error("Arithmetic overflow in {operation}")]
    ArithmeticOverflow {
        /// The operation that caused the overflow
        operation: String,
    },

    /// Invalid type specifier
    #[error("Invalid type specifier")]
    InvalidTypeSpecifier,

    /// Invalid function arity
    #[error("Function '{name}' expects {min_arity}{} arguments, got {actual}",
            max_arity.map(|m| format!("-{m}")).unwrap_or_else(|| String::from(" or more")))]
    InvalidArity {
        /// Name of the function with invalid arity
        name: String,
        /// Minimum number of arguments required
        min_arity: usize,
        /// Maximum number of arguments allowed (None for unlimited)
        max_arity: Option<usize>,
        /// Actual number of arguments provided
        actual: usize,
    },
}

impl FhirPathError {
    /// Create a parse error
    pub fn parse_error(position: usize, message: impl Into<String>) -> Self {
        Self::ParseError {
            position,
            message: message.into(),
        }
    }

    /// Create a type error
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
        }
    }

    /// Create an evaluation error
    pub fn evaluation_error(message: impl Into<String>) -> Self {
        Self::EvaluationError {
            message: message.into(),
        }
    }

    /// Create a function error
    pub fn function_error(function_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FunctionError {
            function_name: function_name.into(),
            message: message.into(),
        }
    }

    /// Create an invalid expression error
    pub fn invalid_expression(message: impl Into<String>) -> Self {
        Self::InvalidExpression {
            message: message.into(),
        }
    }

    /// Create an arithmetic error
    pub fn arithmetic_error(message: impl Into<String>) -> Self {
        Self::ArithmeticError {
            message: message.into(),
        }
    }

    /// Create an index out of bounds error
    pub fn index_out_of_bounds(index: i64, size: usize) -> Self {
        Self::IndexOutOfBounds { index, size }
    }

    /// Create an unknown function error
    pub fn unknown_function(function_name: impl Into<String>) -> Self {
        Self::UnknownFunction {
            function_name: function_name.into(),
        }
    }

    /// Create an invalid argument count error
    pub fn invalid_argument_count(
        function_name: impl Into<String>,
        expected: usize,
        actual: usize,
    ) -> Self {
        Self::InvalidArgumentCount {
            function_name: function_name.into(),
            expected,
            actual,
        }
    }

    /// Create a conversion error
    pub fn conversion_error(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::ConversionError {
            from: from.into(),
            to: to.into(),
        }
    }

    /// Create a generic error
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Create an unknown operator error
    pub fn unknown_operator(operator: impl Into<String>) -> Self {
        Self::UnknownOperator {
            operator: operator.into(),
        }
    }

    /// Create an invalid operand types error
    pub fn invalid_operand_types(
        operator: impl Into<String>,
        left_type: impl Into<String>,
        right_type: impl Into<String>,
    ) -> Self {
        Self::InvalidOperandTypes {
            operator: operator.into(),
            left_type: left_type.into(),
            right_type: right_type.into(),
        }
    }

    /// Create an incompatible units error
    pub fn incompatible_units(left_unit: impl Into<String>, right_unit: impl Into<String>) -> Self {
        Self::IncompatibleUnits {
            left_unit: left_unit.into(),
            right_unit: right_unit.into(),
        }
    }

    /// Create a division by zero error
    pub fn division_by_zero() -> Self {
        Self::DivisionByZero
    }

    /// Create an arithmetic overflow error
    pub fn arithmetic_overflow(operation: impl Into<String>) -> Self {
        Self::ArithmeticOverflow {
            operation: operation.into(),
        }
    }

    /// Create an invalid type specifier error
    pub fn invalid_type_specifier() -> Self {
        Self::InvalidTypeSpecifier
    }

    /// Create an invalid arity error
    pub fn invalid_arity(
        name: impl Into<String>,
        min_arity: usize,
        max_arity: Option<usize>,
        actual: usize,
    ) -> Self {
        Self::InvalidArity {
            name: name.into(),
            min_arity,
            max_arity,
            actual,
        }
    }
}

/// Convert from `Box<dyn std::error::Error>` for compatibility with tests
impl From<Box<dyn std::error::Error>> for FhirPathError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Self::Generic {
            message: err.to_string(),
        }
    }
}

// Note: From<FhirPathError> for Box<dyn std::error::Error> is automatically provided by Rust
// since FhirPathError implements std::error::Error via thiserror
