// Error types for FHIRPath evaluation

use crate::diagnostics::Diagnostic;
use thiserror::Error;

/// Result type for evaluation operations
pub type EvaluationResult<T> = Result<T, EvaluationError>;

/// Errors that can occur during FHIRPath evaluation
#[derive(Error, Debug, Clone, PartialEq)]
pub enum EvaluationError {
    /// Function evaluation error
    #[error("Function error: {0}")]
    Function(#[from] crate::registry::function::FunctionError),

    /// Operator evaluation error
    #[error("Operator error: {0}")]
    Operator(String),

    /// Type error during evaluation
    #[error("Type error: expected {expected}, got {actual}")]
    TypeError {
        /// Expected type
        expected: String,
        /// Actual type found
        actual: String,
    },

    /// Property not found
    #[error("Property {property} not found on {resource_type}")]
    PropertyNotFound {
        /// Property name
        property: String,
        /// Resource type
        resource_type: String,
    },

    /// Index out of bounds
    #[error("Index {index} out of bounds for collection of size {size}")]
    IndexOutOfBounds {
        /// Requested index
        index: i64,
        /// Collection size
        size: usize,
    },

    /// Variable not found
    #[error("Variable {name} not found")]
    VariableNotFound {
        /// Variable name
        name: String,
    },

    /// Invalid operation
    #[error("Invalid operation: {message}")]
    InvalidOperation {
        /// Error message
        message: String,
    },
}

impl EvaluationError {
    /// Convert to a diagnostic
    pub fn to_diagnostic(&self) -> Diagnostic {
        use crate::diagnostics::*;

        match self {
            EvaluationError::Function(err) => {
                DiagnosticBuilder::error(DiagnosticCode::UnknownFunction)
                    .with_message(err.to_string())
                    .build()
            }
            EvaluationError::TypeError { expected, actual } => {
                DiagnosticBuilder::error(DiagnosticCode::TypeMismatch {
                    expected: expected.clone(),
                    actual: actual.clone(),
                })
                .with_message(self.to_string())
                .build()
            }
            EvaluationError::PropertyNotFound { property, .. } => {
                DiagnosticBuilder::error(DiagnosticCode::PropertyNotFound)
                    .with_message(format!("Property {property} not found"))
                    .build()
            }
            EvaluationError::VariableNotFound { name } => {
                DiagnosticBuilder::error(DiagnosticCode::UndefinedVariable)
                    .with_message(format!("Variable {name} not found"))
                    .build()
            }
            _ => DiagnosticBuilder::error(DiagnosticCode::Custom("evaluation_error".to_string()))
                .with_message(self.to_string())
                .build(),
        }
    }
}
