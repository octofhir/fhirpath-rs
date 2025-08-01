//! Operator registry and built-in operators

use crate::model::FhirPathValue;
use crate::registry::signature::OperatorSignature;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use thiserror::Error;

use crate::registry::operators;

/// Result type for operator operations
pub type OperatorResult<T> = Result<T, OperatorError>;

/// Operator evaluation errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum OperatorError {
    /// Invalid operand types for binary operation
    #[error("Operator '{operator}' cannot be applied to types {left_type} and {right_type}")]
    InvalidOperandTypes {
        /// The operator symbol that failed
        operator: String,
        /// Type of the left operand
        left_type: String,
        /// Type of the right operand
        right_type: String,
    },
    /// Invalid operand type for unary operation
    #[error("Operator '{operator}' cannot be applied to type {operand_type}")]
    InvalidUnaryOperandType {
        /// The operator symbol that failed
        operator: String,
        /// Type of the operand
        operand_type: String,
    },
    /// General evaluation error
    #[error("Error evaluating operator '{operator}': {message}")]
    EvaluationError {
        /// The operator that caused the error
        operator: String,
        /// Error message describing what went wrong
        message: String,
    },
    /// Incompatible units for quantity operations
    #[error("Cannot perform operation with incompatible units: {left_unit} and {right_unit}")]
    IncompatibleUnits {
        /// Unit of the left operand
        left_unit: String,
        /// Unit of the right operand
        right_unit: String,
    },
}

/// Operator associativity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    /// Left-associative operator (evaluated left to right)
    Left,
    /// Right-associative operator (evaluated right to left)
    Right,
}

/// Trait for implementing FHIRPath operators
pub trait FhirPathOperator: Send + Sync {
    /// Get the operator symbol (e.g., "+", "-", "=")
    fn symbol(&self) -> &str;

    /// Get a human-friendly name for the operator
    fn human_friendly_name(&self) -> &str;

    /// Get the operator precedence (higher values bind tighter)
    fn precedence(&self) -> u8;

    /// Get the operator associativity
    fn associativity(&self) -> Associativity;

    /// Get the type signatures supported by this operator
    fn signatures(&self) -> &[OperatorSignature];

    /// Evaluate the operator with two operands
    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue>;

    /// Evaluate the operator with one operand (for unary operators)
    fn evaluate_unary(&self, _operand: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        Err(OperatorError::EvaluationError {
            operator: self.symbol().to_string(),
            message: "This operator does not support unary operations".to_string(),
        })
    }
}

/// Registry for FHIRPath operators
#[derive(Clone)]
pub struct OperatorRegistry {
    binary_operators: FxHashMap<String, Arc<dyn FhirPathOperator>>,
    unary_operators: FxHashMap<String, Arc<dyn FhirPathOperator>>,
    precedence: FxHashMap<String, u8>,
    associativity: FxHashMap<String, Associativity>,
}

impl Default for OperatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OperatorRegistry {
    /// Create a new operator registry
    pub fn new() -> Self {
        OperatorRegistry {
            binary_operators: FxHashMap::default(),
            unary_operators: FxHashMap::default(),
            precedence: FxHashMap::default(),
            associativity: FxHashMap::default(),
        }
    }

    /// Register an operator in the registry
    pub fn register<O: FhirPathOperator + 'static>(&mut self, operator: O) {
        let arc_op = Arc::new(operator);
        let symbol = arc_op.symbol().to_string();

        // Store precedence and associativity
        self.precedence.insert(symbol.clone(), arc_op.precedence());
        self.associativity
            .insert(symbol.clone(), arc_op.associativity());

        // Check if it supports binary operations
        if arc_op
            .signatures()
            .iter()
            .any(|sig| sig.right_type.is_some())
        {
            self.binary_operators.insert(symbol.clone(), arc_op.clone());
        }

        // Check if it supports unary operations
        if arc_op
            .signatures()
            .iter()
            .any(|sig| sig.right_type.is_none())
        {
            self.unary_operators.insert(symbol.clone(), arc_op);
        }
    }

    /// Get a binary operator by symbol
    pub fn get_binary(&self, symbol: &str) -> Option<Arc<dyn FhirPathOperator>> {
        self.binary_operators.get(symbol).cloned()
    }

    /// Get a unary operator by symbol
    pub fn get_unary(&self, symbol: &str) -> Option<Arc<dyn FhirPathOperator>> {
        self.unary_operators.get(symbol).cloned()
    }

    /// Get operator precedence
    pub fn get_precedence(&self, symbol: &str) -> Option<u8> {
        self.precedence.get(symbol).copied()
    }

    /// Get operator associativity
    pub fn get_associativity(&self, symbol: &str) -> Option<Associativity> {
        self.associativity.get(symbol).copied()
    }

    /// Check if a binary operator exists
    pub fn contains_binary(&self, symbol: &str) -> bool {
        self.binary_operators.contains_key(symbol)
    }

    /// Check if a unary operator exists
    pub fn contains_unary(&self, symbol: &str) -> bool {
        self.unary_operators.contains_key(symbol)
    }

    /// Get all binary operator symbols
    pub fn binary_operator_symbols(&self) -> Vec<&str> {
        self.binary_operators.keys().map(|s| s.as_str()).collect()
    }

    /// Get all unary operator symbols
    pub fn unary_operator_symbols(&self) -> Vec<&str> {
        self.unary_operators.keys().map(|s| s.as_str()).collect()
    }
}

/// Register all built-in FHIRPath operators
pub fn register_builtin_operators(registry: &mut OperatorRegistry) {
    operators::register_builtin_operators(registry);
}
