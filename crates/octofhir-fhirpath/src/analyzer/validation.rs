//! FHIRPath expression validation
//!
//! This module provides validation capabilities for FHIRPath expressions,
//! ensuring they conform to the specification and best practices.

use crate::ast::ExpressionNode;
use crate::core::Result;

/// Validator for FHIRPath expressions
#[derive(Debug, Default)]
pub struct Validator {
    /// Whether to enforce strict validation rules
    pub strict: bool,
}

impl Validator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self { strict: false }
    }

    /// Create a new strict validator
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Validate a FHIRPath expression
    pub fn validate(&self, expression: &ExpressionNode) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Perform basic AST validation
        if let Err(err) = expression.validate() {
            result
                .errors
                .push(format!("AST validation failed: {}", err));
            result.is_valid = false;
        }

        // TODO: Add more comprehensive validation rules

        Ok(result)
    }

    /// Validate expression syntax and semantics
    pub fn validate_syntax(&self, _expression: &ExpressionNode) -> Result<()> {
        // TODO: Implement syntax validation
        Ok(())
    }

    /// Validate expression semantics
    pub fn validate_semantics(&self, _expression: &ExpressionNode) -> Result<()> {
        // TODO: Implement semantic validation
        Ok(())
    }
}

/// Result of expression validation
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Whether the expression is valid
    pub is_valid: bool,
    /// Validation errors found
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Check if the validation passed
    pub fn is_valid(&self) -> bool {
        self.is_valid && self.errors.is_empty()
    }
}
