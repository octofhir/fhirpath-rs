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

//! Core evaluation types and error handling
//!
//! This module provides shared types for evaluation results and errors
//! that can be used across different crates without circular dependencies.

use thiserror::Error;

/// Result type for evaluation operations
pub type EvaluationResult<T> = Result<T, EvaluationError>;

/// Core errors that can occur during FHIRPath evaluation
#[derive(Error, Debug, Clone, PartialEq)]
pub enum EvaluationError {
    /// Function evaluation error
    #[error("Function error: {0}")]
    Function(String),

    /// Operator evaluation error
    #[error("Operator error: {0}")]
    Operator(String),

    /// Invalid operation
    #[error("Invalid operation: {message}")]
    InvalidOperation {
        /// Error message
        message: String,
    },

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
    },

    /// Index out of bounds
    #[error("Index {index} out of bounds for collection of size {size}")]
    IndexOutOfBounds {
        /// Index that was accessed
        index: usize,
        /// Size of the collection
        size: usize,
    },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Invalid argument
    #[error("Invalid argument: {message}")]
    InvalidArgument {
        /// Error message
        message: String,
    },

    /// Generic runtime error
    #[error("Runtime error: {message}")]
    RuntimeError {
        /// Error message
        message: String,
    },
}

impl From<Box<dyn std::error::Error + Send + Sync>> for EvaluationError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        EvaluationError::RuntimeError {
            message: err.to_string(),
        }
    }
}

impl From<String> for EvaluationError {
    fn from(message: String) -> Self {
        EvaluationError::RuntimeError { message }
    }
}

impl From<&str> for EvaluationError {
    fn from(message: &str) -> Self {
        EvaluationError::RuntimeError {
            message: message.to_string(),
        }
    }
}

// Helper conversion to avoid circular dependency with registry types
impl EvaluationError {
    /// Create a Function error from a generic error
    pub fn from_function_error(err: impl std::fmt::Display) -> Self {
        EvaluationError::Function(err.to_string())
    }
}
