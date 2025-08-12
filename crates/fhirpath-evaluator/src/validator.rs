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

//! Runtime type validation for FHIRPath expressions
//!
//! This module provides async-compatible runtime validation of function parameters
//! and operator type compatibility using the ModelProvider.

use super::context::EvaluationContext;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::{provider::ModelProvider, FhirPathValue};
use std::sync::Arc;

/// Runtime validator for function parameters and operator compatibility
pub struct RuntimeValidator {
    /// Reference to the async ModelProvider
    provider: Arc<dyn ModelProvider>,
    /// Validation mode configuration
    mode: ValidationMode,
}

/// Validation modes for different use cases
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationMode {
    /// Strict validation - all type mismatches are errors
    Strict,
    /// Lenient validation - allows common type coercions
    Lenient,
    /// Disabled - no runtime validation (performance mode)
    Disabled,
}

/// Validation result with detailed information
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Error messages if validation failed
    pub errors: Vec<String>,
    /// Warning messages for lenient validation
    pub warnings: Vec<String>,
    /// Suggested fixes or alternative approaches
    pub suggestions: Vec<String>,
}

impl RuntimeValidator {
    /// Create a new runtime validator
    pub fn new(provider: Arc<dyn ModelProvider>, mode: ValidationMode) -> Self {
        Self { provider, mode }
    }

    /// Validate function parameters against expected signature
    pub async fn validate_function_parameters(
        &self,
        _context: &EvaluationContext,
        function_name: &str,
        parameters: &[FhirPathValue],
    ) -> EvaluationResult<ValidationResult> {
        if self.mode == ValidationMode::Disabled {
            return Ok(ValidationResult::valid());
        }

        let mut result = ValidationResult::new();

        match function_name {
            // String functions
            "substring" => {
                if parameters.is_empty() || parameters.len() > 2 {
                    result.add_error(format!(
                        "substring() expects 1 or 2 parameters, got {}",
                        parameters.len()
                    ));
                } else {
                    // First parameter should be integer (start index)
                    if !self.is_integer_compatible(&parameters[0]) {
                        result.add_error(
                            "substring() first parameter must be an integer".to_string(),
                        );
                    }
                    // Second parameter (if present) should be integer (length)
                    if parameters.len() == 2 && !self.is_integer_compatible(&parameters[1]) {
                        result.add_error(
                            "substring() second parameter must be an integer".to_string(),
                        );
                    }
                }
            }
            "length" => {
                if !parameters.is_empty() {
                    result.add_error("length() expects no parameters".to_string());
                }
            }
            // Math functions
            "abs" => {
                if !parameters.is_empty() {
                    result.add_error("abs() expects no parameters (operates on input)".to_string());
                }
            }
            // Collection functions
            "count" => {
                if !parameters.is_empty() {
                    result.add_error("count() expects no parameters".to_string());
                }
            }
            "where" | "select" => {
                if parameters.len() != 1 {
                    result.add_error(format!("{function_name}() expects exactly 1 parameter"));
                }
            }
            // Type checking functions
            "is" | "as" => {
                if parameters.len() != 1 {
                    result.add_error(format!("{function_name}() expects exactly 1 parameter"));
                } else if !self.is_string_compatible(&parameters[0]) {
                    result.add_error(format!(
                        "{function_name}() parameter must be a type name (string)"
                    ));
                }
            }
            _ => {
                // Unknown function - add warning in lenient mode
                if self.mode == ValidationMode::Lenient {
                    result.add_warning(format!(
                        "Unknown function '{function_name}' - validation skipped"
                    ));
                }
            }
        }

        Ok(result)
    }

    /// Validate operator type compatibility
    pub async fn validate_operator_compatibility(
        &self,
        _context: &EvaluationContext,
        operator: &str,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<ValidationResult> {
        if self.mode == ValidationMode::Disabled {
            return Ok(ValidationResult::valid());
        }

        let mut result = ValidationResult::new();

        match operator {
            // Arithmetic operators
            "+" | "-" | "*" | "/" => {
                if !self.is_numeric_compatible(left) {
                    result.add_error(format!("Left operand of '{operator}' must be numeric"));
                }
                if !self.is_numeric_compatible(right) {
                    result.add_error(format!("Right operand of '{operator}' must be numeric"));
                }
            }
            // String concatenation
            "&" => {
                if !self.is_string_compatible(left) && !self.can_convert_to_string(left) {
                    result
                        .add_error("Left operand of '&' must be convertible to string".to_string());
                }
                if !self.is_string_compatible(right) && !self.can_convert_to_string(right) {
                    result.add_error(
                        "Right operand of '&' must be convertible to string".to_string(),
                    );
                }
            }
            // Comparison operators
            "=" | "!=" | "<" | "<=" | ">" | ">=" => {
                // Comparison requires compatible types
                if !self.are_comparable_types(left, right) {
                    if self.mode == ValidationMode::Strict {
                        result.add_error(format!(
                            "Cannot compare {} with {}",
                            self.get_type_name(left),
                            self.get_type_name(right)
                        ));
                    } else {
                        result.add_warning(format!(
                            "Comparing {} with {} may produce unexpected results",
                            self.get_type_name(left),
                            self.get_type_name(right)
                        ));
                    }
                }
            }
            // Logical operators
            "and" | "or" | "xor" => {
                if !self.is_boolean_compatible(left) {
                    result.add_error(format!("Left operand of '{operator}' must be boolean"));
                }
                if !self.is_boolean_compatible(right) {
                    result.add_error(format!("Right operand of '{operator}' must be boolean"));
                }
            }
            // Membership operators
            "in" | "contains" => {
                // Right operand should be a collection
                if !self.is_collection_compatible(right) {
                    result.add_error(format!(
                        "Right operand of '{operator}' must be a collection"
                    ));
                }
            }
            _ => {
                if self.mode == ValidationMode::Lenient {
                    result.add_warning(format!(
                        "Unknown operator '{operator}' - validation skipped"
                    ));
                }
            }
        }

        Ok(result)
    }

    /// Validate collection operations
    pub async fn validate_collection_operation(
        &self,
        _context: &EvaluationContext,
        input: &FhirPathValue,
        operation: &str,
    ) -> EvaluationResult<ValidationResult> {
        if self.mode == ValidationMode::Disabled {
            return Ok(ValidationResult::valid());
        }

        let mut result = ValidationResult::new();

        match operation {
            "first" | "last" | "tail" | "skip" | "take" => {
                // These operations work on collections but also single values
                if matches!(input, FhirPathValue::Empty) {
                    result.add_warning(format!("{operation}() on empty value will return empty"));
                }
            }
            "distinct" | "union" | "intersect" => {
                // These require collections
                if !self.is_collection_compatible(input) && !matches!(input, FhirPathValue::Empty) {
                    result.add_error(format!("{operation}() requires a collection input"));
                }
            }
            _ => {}
        }

        Ok(result)
    }

    /// Check if value is integer compatible
    fn is_integer_compatible(&self, value: &FhirPathValue) -> bool {
        matches!(value, FhirPathValue::Integer(_))
    }

    /// Check if value is numeric compatible
    fn is_numeric_compatible(&self, value: &FhirPathValue) -> bool {
        matches!(
            value,
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) | FhirPathValue::Quantity(_)
        )
    }

    /// Check if value is string compatible
    fn is_string_compatible(&self, value: &FhirPathValue) -> bool {
        matches!(value, FhirPathValue::String(_))
    }

    /// Check if value is boolean compatible
    fn is_boolean_compatible(&self, value: &FhirPathValue) -> bool {
        matches!(value, FhirPathValue::Boolean(_))
    }

    /// Check if value is collection compatible
    fn is_collection_compatible(&self, value: &FhirPathValue) -> bool {
        matches!(value, FhirPathValue::Collection(_))
    }

    /// Check if value can be converted to string
    fn can_convert_to_string(&self, value: &FhirPathValue) -> bool {
        !matches!(value, FhirPathValue::Empty)
    }

    /// Check if two types are comparable
    fn are_comparable_types(&self, left: &FhirPathValue, right: &FhirPathValue) -> bool {
        use std::mem::discriminant;

        // Same type is always comparable
        if discriminant(left) == discriminant(right) {
            return true;
        }

        // Numeric types are comparable with each other
        if self.is_numeric_compatible(left) && self.is_numeric_compatible(right) {
            return true;
        }

        // In lenient mode, allow more comparisons
        if self.mode == ValidationMode::Lenient {
            // Allow comparing with empty
            if matches!(left, FhirPathValue::Empty) || matches!(right, FhirPathValue::Empty) {
                return true;
            }
        }

        false
    }

    /// Get human-readable type name for error messages
    fn get_type_name(&self, value: &FhirPathValue) -> &'static str {
        match value {
            FhirPathValue::Boolean(_) => "boolean",
            FhirPathValue::Integer(_) => "integer",
            FhirPathValue::Decimal(_) => "decimal",
            FhirPathValue::String(_) => "string",
            FhirPathValue::Date(_) => "date",
            FhirPathValue::DateTime(_) => "dateTime",
            FhirPathValue::Time(_) => "time",
            FhirPathValue::Quantity(_) => "quantity",
            FhirPathValue::Collection(_) => "collection",
            FhirPathValue::Resource(_) => "resource",
            FhirPathValue::JsonValue(_) => "object",
            FhirPathValue::TypeInfoObject { .. } => "type",
            FhirPathValue::Empty => "empty",
        }
    }
}

impl ValidationResult {
    /// Create a new empty validation result
    fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Create a valid result
    fn valid() -> Self {
        Self::new()
    }

    /// Add an error message
    fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    /// Add a warning message
    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add a suggestion
    fn _add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::MockModelProvider;
    use octofhir_fhirpath_registry::{FunctionRegistry, OperatorRegistry};
    use tokio;

    #[tokio::test]
    async fn test_function_parameter_validation() {
        let provider = Arc::new(MockModelProvider::empty());
        let validator = RuntimeValidator::new(provider, ValidationMode::Strict);

        let mock_provider = Arc::new(MockModelProvider::empty());
        let context = EvaluationContext::new(
            FhirPathValue::Empty,
            Arc::new(FunctionRegistry::new()),
            Arc::new(OperatorRegistry::new()),
            mock_provider,
        );

        // Test valid substring parameters
        let params = vec![FhirPathValue::Integer(1), FhirPathValue::Integer(3)];
        let result = validator
            .validate_function_parameters(&context, "substring", &params)
            .await
            .unwrap();
        assert!(result.is_valid);

        // Test invalid substring parameters
        let params = vec![FhirPathValue::String("not_an_int".into())];
        let result = validator
            .validate_function_parameters(&context, "substring", &params)
            .await
            .unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_operator_compatibility() {
        let provider = Arc::new(MockModelProvider::empty());
        let validator = RuntimeValidator::new(provider, ValidationMode::Strict);

        let mock_provider = Arc::new(MockModelProvider::empty());
        let context = EvaluationContext::new(
            FhirPathValue::Empty,
            Arc::new(FunctionRegistry::new()),
            Arc::new(OperatorRegistry::new()),
            mock_provider,
        );

        // Test valid arithmetic operation
        let result = validator
            .validate_operator_compatibility(
                &context,
                "+",
                &FhirPathValue::Integer(1),
                &FhirPathValue::Integer(2),
            )
            .await
            .unwrap();
        assert!(result.is_valid);

        // Test invalid arithmetic operation
        let result = validator
            .validate_operator_compatibility(
                &context,
                "+",
                &FhirPathValue::String("hello".into()),
                &FhirPathValue::Integer(2),
            )
            .await
            .unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validation_modes() {
        let provider = Arc::new(MockModelProvider::empty());

        // Strict mode
        let strict_validator = RuntimeValidator::new(provider.clone(), ValidationMode::Strict);
        // Lenient mode
        let lenient_validator = RuntimeValidator::new(provider.clone(), ValidationMode::Lenient);
        // Disabled mode
        let disabled_validator = RuntimeValidator::new(provider, ValidationMode::Disabled);

        let mock_provider = Arc::new(MockModelProvider::empty());
        let context = EvaluationContext::new(
            FhirPathValue::Empty,
            Arc::new(FunctionRegistry::new()),
            Arc::new(OperatorRegistry::new()),
            mock_provider,
        );

        // Test with incompatible types
        let left = FhirPathValue::String("hello".into());
        let right = FhirPathValue::Integer(42);

        // Strict mode should error
        let result = strict_validator
            .validate_operator_compatibility(&context, "=", &left, &right)
            .await
            .unwrap();
        assert!(!result.is_valid);

        // Lenient mode should warn
        let result = lenient_validator
            .validate_operator_compatibility(&context, "=", &left, &right)
            .await
            .unwrap();
        assert!(result.is_valid); // Still valid but with warnings
        assert!(!result.warnings.is_empty());

        // Disabled mode should always pass
        let result = disabled_validator
            .validate_operator_compatibility(&context, "=", &left, &right)
            .await
            .unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }
}
