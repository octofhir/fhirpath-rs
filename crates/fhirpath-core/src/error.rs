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

/// Source location information for error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Character position in the source (0-based)
    pub position: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: usize, column: usize, position: usize) -> Self {
        Self {
            line,
            column,
            position,
        }
    }
}

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
    #[error("Evaluation error: {message}{}{}", 
        expression.as_ref().map(|e| format!(" in expression: {e}")).unwrap_or_default(),
        location.as_ref().map(|l| format!(" at line {}, column {}", l.line, l.column)).unwrap_or_default()
    )]
    EvaluationError {
        /// Human-readable evaluation error message
        message: String,
        /// Expression being evaluated when error occurred
        expression: Option<String>,
        /// Source location where error occurred
        location: Option<SourceLocation>,
    },

    /// Function call errors
    #[error("Function '{function_name}' error: {message}{}", arguments.as_ref().map(|args| format!(" with arguments: {}", args.join(", "))).unwrap_or_default())]
    FunctionError {
        /// Name of the function that caused the error
        function_name: String,
        /// Human-readable error message
        message: String,
        /// Arguments passed to the function
        arguments: Option<Vec<String>>,
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

    /// Invalid operation for given types or context
    #[error("Invalid operation '{operation}': {message}{}", 
        match (left_type.as_ref(), right_type.as_ref()) {
            (Some(left), Some(right)) => format!(" (left: {left}, right: {right})"),
            _ => String::new()
        }
    )]
    InvalidOperation {
        /// The operation that was invalid
        operation: String,
        /// Type of the left operand (if applicable)
        left_type: Option<String>,
        /// Type of the right operand (if applicable)  
        right_type: Option<String>,
        /// Human-readable error message
        message: String,
    },

    /// Type mismatch error with context
    #[error("Type mismatch: expected {expected}, got {actual}{}", context.as_ref().map(|c| format!(" in {c}")).unwrap_or_default())]
    TypeMismatch {
        /// Expected type
        expected: String,
        /// Actual type received
        actual: String,
        /// Additional context about where the mismatch occurred
        context: Option<String>,
    },

    /// Timeout error during evaluation
    #[error("Evaluation timeout after {timeout_ms}ms{}", expression.as_ref().map(|e| format!(" in expression: {e}")).unwrap_or_default())]
    TimeoutError {
        /// Timeout duration in milliseconds
        timeout_ms: u64,
        /// Expression being evaluated when timeout occurred
        expression: Option<String>,
    },

    /// Recursion limit exceeded
    #[error("Recursion limit of {limit} exceeded{}", expression.as_ref().map(|e| format!(" in expression: {e}")).unwrap_or_default())]
    RecursionLimitExceeded {
        /// The recursion limit that was exceeded
        limit: usize,
        /// Expression being evaluated when limit was exceeded
        expression: Option<String>,
    },

    /// Memory limit exceeded
    #[error("Memory limit of {limit_mb}MB exceeded (current: {current_mb}MB)")]
    MemoryLimitExceeded {
        /// Memory limit in megabytes
        limit_mb: usize,
        /// Current memory usage in megabytes
        current_mb: usize,
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
            expression: None,
            location: None,
        }
    }

    /// Create an evaluation error with expression context
    pub fn evaluation_error_with_context(
        message: impl Into<String>,
        expression: Option<String>,
        location: Option<SourceLocation>,
    ) -> Self {
        Self::EvaluationError {
            message: message.into(),
            expression,
            location,
        }
    }

    /// Create a function error
    pub fn function_error(function_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FunctionError {
            function_name: function_name.into(),
            message: message.into(),
            arguments: None,
        }
    }

    /// Create a function error with arguments context
    pub fn function_error_with_args(
        function_name: impl Into<String>,
        message: impl Into<String>,
        arguments: Option<Vec<String>>,
    ) -> Self {
        Self::FunctionError {
            function_name: function_name.into(),
            message: message.into(),
            arguments,
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

    /// Create an invalid operation error
    pub fn invalid_operation(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidOperation {
            operation: operation.into(),
            left_type: None,
            right_type: None,
            message: message.into(),
        }
    }

    /// Create an invalid operation error with type context
    pub fn invalid_operation_with_types(
        operation: impl Into<String>,
        left_type: Option<String>,
        right_type: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidOperation {
            operation: operation.into(),
            left_type,
            right_type,
            message: message.into(),
        }
    }

    /// Create a type mismatch error
    pub fn type_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            actual: actual.into(),
            context: None,
        }
    }

    /// Create a type mismatch error with context
    pub fn type_mismatch_with_context(
        expected: impl Into<String>,
        actual: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            actual: actual.into(),
            context: Some(context.into()),
        }
    }

    /// Create a timeout error
    pub fn timeout_error(timeout_ms: u64) -> Self {
        Self::TimeoutError {
            timeout_ms,
            expression: None,
        }
    }

    /// Create a timeout error with expression context
    pub fn timeout_error_with_expression(timeout_ms: u64, expression: impl Into<String>) -> Self {
        Self::TimeoutError {
            timeout_ms,
            expression: Some(expression.into()),
        }
    }

    /// Create a recursion limit exceeded error
    pub fn recursion_limit_exceeded(limit: usize) -> Self {
        Self::RecursionLimitExceeded {
            limit,
            expression: None,
        }
    }

    /// Create a recursion limit exceeded error with expression context
    pub fn recursion_limit_exceeded_with_expression(
        limit: usize,
        expression: impl Into<String>,
    ) -> Self {
        Self::RecursionLimitExceeded {
            limit,
            expression: Some(expression.into()),
        }
    }

    /// Create a memory limit exceeded error
    pub fn memory_limit_exceeded(limit_mb: usize, current_mb: usize) -> Self {
        Self::MemoryLimitExceeded {
            limit_mb,
            current_mb,
        }
    }

    /// Add context to an existing error
    pub fn with_context(mut self, context: &str) -> Self {
        match &mut self {
            Self::EvaluationError { message, .. } => {
                *message = format!("{message} (context: {context})");
            }
            Self::TypeMismatch { context: ctx, .. } => {
                *ctx = Some(context.to_string());
            }
            Self::FunctionError { message, .. } => {
                *message = format!("{message} (context: {context})");
            }
            Self::InvalidOperation { message, .. } => {
                *message = format!("{message} (context: {context})");
            }
            Self::TypeError { message } => {
                *message = format!("{message} (context: {context})");
            }
            Self::ArithmeticError { message } => {
                *message = format!("{message} (context: {context})");
            }
            _ => {}
        }
        self
    }

    /// Add expression context to an error
    pub fn with_expression(mut self, expression: &str) -> Self {
        match &mut self {
            Self::EvaluationError {
                expression: expr, ..
            } => {
                *expr = Some(expression.to_string());
            }
            Self::TimeoutError {
                expression: expr, ..
            } => {
                *expr = Some(expression.to_string());
            }
            Self::RecursionLimitExceeded {
                expression: expr, ..
            } => {
                *expr = Some(expression.to_string());
            }
            _ => {}
        }
        self
    }

    /// Add location context to an error
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        if let Self::EvaluationError { location: loc, .. } = &mut self {
            *loc = Some(location);
        }
        self
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

/// Helper trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add function context to an error
    fn with_function_context(self, function_name: &str) -> Result<T>;
    /// Add operation context to an error
    fn with_operation_context(self, operation: &str) -> Result<T>;
    /// Add expression context to an error
    fn with_expression_context(self, expression: &str) -> Result<T>;
    /// Add location context to an error
    fn with_location_context(self, location: SourceLocation) -> Result<T>;
}

impl<T> ErrorContext<T> for Result<T> {
    fn with_function_context(self, function_name: &str) -> Result<T> {
        self.map_err(|e| e.with_context(&format!("function {function_name}")))
    }

    fn with_operation_context(self, operation: &str) -> Result<T> {
        self.map_err(|e| e.with_context(&format!("operation {operation}")))
    }

    fn with_expression_context(self, expression: &str) -> Result<T> {
        self.map_err(|e| e.with_expression(expression))
    }

    fn with_location_context(self, location: SourceLocation) -> Result<T> {
        self.map_err(|e| e.with_location(location))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_creation() {
        let loc = SourceLocation::new(10, 5, 42);
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.position, 42);
    }

    #[test]
    fn test_basic_error_constructors() {
        // Test parse error
        let parse_err = FhirPathError::parse_error(5, "Unexpected token");
        assert!(matches!(
            parse_err,
            FhirPathError::ParseError { position: 5, .. }
        ));

        // Test type error
        let type_err = FhirPathError::type_error("Type mismatch");
        assert!(matches!(type_err, FhirPathError::TypeError { .. }));

        // Test evaluation error
        let eval_err = FhirPathError::evaluation_error("Evaluation failed");
        assert!(matches!(eval_err, FhirPathError::EvaluationError { .. }));

        // Test function error
        let func_err = FhirPathError::function_error("count", "Invalid call");
        assert!(matches!(func_err, FhirPathError::FunctionError { .. }));
    }

    #[test]
    fn test_enhanced_error_constructors() {
        // Test invalid operation
        let invalid_op = FhirPathError::invalid_operation("division", "Division by zero");
        assert!(matches!(invalid_op, FhirPathError::InvalidOperation { .. }));

        // Test invalid operation with types
        let invalid_op_types = FhirPathError::invalid_operation_with_types(
            "addition",
            Some("String".to_string()),
            Some("Integer".to_string()),
            "Cannot add string and integer",
        );
        assert!(matches!(
            invalid_op_types,
            FhirPathError::InvalidOperation {
                left_type: Some(_),
                right_type: Some(_),
                ..
            }
        ));

        // Test type mismatch
        let type_mismatch = FhirPathError::type_mismatch("Integer", "String");
        assert!(matches!(type_mismatch, FhirPathError::TypeMismatch { .. }));

        // Test type mismatch with context
        let type_mismatch_ctx =
            FhirPathError::type_mismatch_with_context("Integer", "String", "arithmetic operation");
        assert!(matches!(
            type_mismatch_ctx,
            FhirPathError::TypeMismatch {
                context: Some(_),
                ..
            }
        ));

        // Test timeout error
        let timeout_err = FhirPathError::timeout_error(5000);
        assert!(matches!(
            timeout_err,
            FhirPathError::TimeoutError {
                timeout_ms: 5000,
                ..
            }
        ));

        // Test recursion limit exceeded
        let recursion_err = FhirPathError::recursion_limit_exceeded(100);
        assert!(matches!(
            recursion_err,
            FhirPathError::RecursionLimitExceeded { limit: 100, .. }
        ));

        // Test memory limit exceeded
        let memory_err = FhirPathError::memory_limit_exceeded(512, 600);
        assert!(matches!(
            memory_err,
            FhirPathError::MemoryLimitExceeded {
                limit_mb: 512,
                current_mb: 600
            }
        ));
    }

    #[test]
    fn test_error_context_helpers() {
        let err = FhirPathError::evaluation_error("Test error");

        // Test with_context
        let err_with_ctx = err.clone().with_context("test context");
        if let FhirPathError::EvaluationError { message, .. } = err_with_ctx {
            assert!(message.contains("(context: test context)"));
        } else {
            panic!("Expected EvaluationError");
        }

        // Test with_expression
        let err_with_expr = err.clone().with_expression("Patient.name");
        if let FhirPathError::EvaluationError { expression, .. } = err_with_expr {
            assert_eq!(expression, Some("Patient.name".to_string()));
        } else {
            panic!("Expected EvaluationError");
        }

        // Test with_location
        let location = SourceLocation::new(5, 10, 42);
        let err_with_loc = err.with_location(location.clone());
        if let FhirPathError::EvaluationError { location: loc, .. } = err_with_loc {
            assert_eq!(loc, Some(location));
        } else {
            panic!("Expected EvaluationError");
        }
    }

    #[test]
    fn test_error_context_trait() {
        let ok_result: Result<i32> = Ok(42);
        let err_result: Result<i32> = Err(FhirPathError::evaluation_error("Test error"));

        // Test function context
        let with_func_ctx = err_result.clone().with_function_context("count");
        assert!(with_func_ctx.is_err());
        if let Err(FhirPathError::EvaluationError { message, .. }) = with_func_ctx {
            assert!(message.contains("(context: function count)"));
        }

        // Test operation context
        let with_op_ctx = err_result.clone().with_operation_context("division");
        assert!(with_op_ctx.is_err());
        if let Err(FhirPathError::EvaluationError { message, .. }) = with_op_ctx {
            assert!(message.contains("(context: operation division)"));
        }

        // Test expression context
        let with_expr_ctx = err_result.clone().with_expression_context("Patient.name");
        assert!(with_expr_ctx.is_err());
        if let Err(FhirPathError::EvaluationError { expression, .. }) = with_expr_ctx {
            assert_eq!(expression, Some("Patient.name".to_string()));
        }

        // Test location context
        let location = SourceLocation::new(1, 1, 0);
        let with_loc_ctx = err_result.with_location_context(location.clone());
        assert!(with_loc_ctx.is_err());
        if let Err(FhirPathError::EvaluationError { location: loc, .. }) = with_loc_ctx {
            assert_eq!(loc, Some(location));
        }

        // Test that Ok results pass through unchanged
        let unchanged = ok_result.with_function_context("test");
        assert_eq!(unchanged.unwrap(), 42);
    }

    #[test]
    fn test_error_display() {
        // Test InvalidOperation display
        let invalid_op = FhirPathError::invalid_operation_with_types(
            "division",
            Some("String".to_string()),
            Some("Integer".to_string()),
            "Cannot divide string by integer",
        );
        let display = format!("{invalid_op}");
        assert!(display.contains("Invalid operation 'division'"));
        assert!(display.contains("Cannot divide string by integer"));

        // Test TypeMismatch display
        let type_mismatch =
            FhirPathError::type_mismatch_with_context("Integer", "String", "arithmetic operation");
        let display = format!("{type_mismatch}");
        assert!(display.contains("Type mismatch: expected Integer, got String"));
        assert!(display.contains("in arithmetic operation"));

        // Test TimeoutError display
        let timeout = FhirPathError::timeout_error_with_expression(5000, "Patient.name.where(...)");
        let display = format!("{timeout}");
        assert!(display.contains("Evaluation timeout after 5000ms"));
        assert!(display.contains("in expression: Patient.name.where(...)"));

        // Test RecursionLimitExceeded display
        let recursion = FhirPathError::recursion_limit_exceeded_with_expression(100, "recursive()");
        let display = format!("{recursion}");
        assert!(display.contains("Recursion limit of 100 exceeded"));
        assert!(display.contains("in expression: recursive()"));

        // Test MemoryLimitExceeded display
        let memory = FhirPathError::memory_limit_exceeded(512, 600);
        let display = format!("{memory}");
        assert!(display.contains("Memory limit of 512MB exceeded (current: 600MB)"));
    }

    #[test]
    fn test_function_error_with_arguments() {
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let func_err = FhirPathError::function_error_with_args(
            "count",
            "Invalid arguments",
            Some(args.clone()),
        );

        if let FhirPathError::FunctionError { arguments, .. } = func_err {
            assert_eq!(arguments, Some(args));
        } else {
            panic!("Expected FunctionError");
        }
    }

    #[test]
    fn test_evaluation_error_with_context() {
        let location = SourceLocation::new(10, 5, 42);
        let eval_err = FhirPathError::evaluation_error_with_context(
            "Evaluation failed",
            Some("Patient.name".to_string()),
            Some(location.clone()),
        );

        if let FhirPathError::EvaluationError {
            expression,
            location: loc,
            ..
        } = eval_err
        {
            assert_eq!(expression, Some("Patient.name".to_string()));
            assert_eq!(loc, Some(location));
        } else {
            panic!("Expected EvaluationError");
        }
    }

    #[test]
    fn test_legacy_error_constructors_still_work() {
        // Ensure backward compatibility
        let division_by_zero = FhirPathError::division_by_zero();
        assert!(matches!(division_by_zero, FhirPathError::DivisionByZero));

        let overflow = FhirPathError::arithmetic_overflow("multiplication");
        assert!(matches!(overflow, FhirPathError::ArithmeticOverflow { .. }));

        let unknown_func = FhirPathError::unknown_function("unknownFunction");
        assert!(matches!(
            unknown_func,
            FhirPathError::UnknownFunction { .. }
        ));

        let invalid_args = FhirPathError::invalid_argument_count("count", 1, 2);
        assert!(matches!(
            invalid_args,
            FhirPathError::InvalidArgumentCount { .. }
        ));
    }
}
