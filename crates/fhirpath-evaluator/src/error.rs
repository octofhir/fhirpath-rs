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

// Error types for FHIRPath evaluation

use fhirpath_diagnostics::Diagnostic;
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
    #[error("Property {property} not found on {type_name}")]
    PropertyNotFound {
        /// Property name
        property: String,
        /// Type name
        type_name: String,
        /// Suggested property names
        suggestions: Vec<String>,
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

    // VM execution error (handled at integration layer to avoid circular deps)
    // Vm(#[from] fhirpath_core::VmError),
}

impl EvaluationError {
    /// Convert to a diagnostic
    pub fn to_diagnostic(&self) -> Diagnostic {
        use fhirpath_diagnostics::*;

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
