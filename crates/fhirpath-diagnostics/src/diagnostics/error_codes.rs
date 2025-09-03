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

//! Structured error codes system for FHIRPath diagnostics

use std::fmt;

/// Enhanced diagnostic code with category and help information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructuredDiagnosticCode {
    /// Error code identifier
    pub code: String,
    /// Category of the error
    pub category: ErrorCategory,
    /// Optional help URL
    pub help_url: Option<String>,
    /// Human-readable description
    pub description: String,
}

/// Categories of diagnostic codes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ErrorCategory {
    /// Syntax and parsing errors
    Syntax,
    /// Type-related errors
    Type,
    /// Property access errors
    Property,
    /// Function call errors
    Function,
    /// Lambda and variable errors
    Lambda,
    /// Performance suggestions
    Performance,
    /// Deprecation warnings
    Deprecation,
    /// FHIR-specific errors
    Fhir,
    /// General errors
    General,
}

impl StructuredDiagnosticCode {
    /// Create a new structured diagnostic code
    pub fn new(
        code: &str,
        category: ErrorCategory,
        description: &str,
    ) -> Self {
        Self {
            code: code.to_string(),
            category,
            help_url: None,
            description: description.to_string(),
        }
    }
    
    /// Create a new structured diagnostic code with help URL
    pub fn with_help_url(
        code: &str,
        category: ErrorCategory,
        description: &str,
        help_url: &str,
    ) -> Self {
        Self {
            code: code.to_string(),
            category,
            help_url: Some(help_url.to_string()),
            description: description.to_string(),
        }
    }
    
    /// Get the full help URL
    pub fn full_help_url(&self) -> Option<String> {
        self.help_url.clone().or_else(|| {
            Some(format!(
                "https://docs.octofhir.com/fhirpath/errors/{}",
                self.code
            ))
        })
    }
}

impl fmt::Display for StructuredDiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Syntax => write!(f, "syntax"),
            ErrorCategory::Type => write!(f, "type"),
            ErrorCategory::Property => write!(f, "property"),
            ErrorCategory::Function => write!(f, "function"),
            ErrorCategory::Lambda => write!(f, "lambda"),
            ErrorCategory::Performance => write!(f, "performance"),
            ErrorCategory::Deprecation => write!(f, "deprecation"),
            ErrorCategory::Fhir => write!(f, "fhir"),
            ErrorCategory::General => write!(f, "general"),
        }
    }
}

/// Factory functions for creating common error codes
pub mod codes {
    use super::*;
    
    // Syntax errors (E001-E099)
    pub fn unexpected_token() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E001",
            ErrorCategory::Syntax,
            "Unexpected token encountered while parsing expression"
        )
    }
    
    pub fn expected_token() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E002",
            ErrorCategory::Syntax,
            "Expected a specific token but found something else"
        )
    }
    
    pub fn unclosed_string() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E003",
            ErrorCategory::Syntax,
            "String literal was not properly closed"
        )
    }
    
    pub fn invalid_number() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E004",
            ErrorCategory::Syntax,
            "Number format is invalid"
        )
    }
    
    pub fn invalid_datetime() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E005",
            ErrorCategory::Syntax,
            "Date/time format is invalid"
        )
    }
    
    pub fn unknown_operator() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E006",
            ErrorCategory::Syntax,
            "Operator is not recognized"
        )
    }
    
    pub fn invalid_escape() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E008",
            ErrorCategory::Syntax,
            "Invalid escape sequence in string"
        )
    }
    
    // Type errors (E100-E199)
    pub fn type_mismatch() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E100",
            ErrorCategory::Type,
            "Expected type does not match actual type"
        )
    }
    
    pub fn invalid_operand_types() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E101",
            ErrorCategory::Type,
            "Operand types are not valid for this operator"
        )
    }
    
    pub fn invalid_argument_types() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E102",
            ErrorCategory::Type,
            "Argument types are not valid for this function"
        )
    }
    
    pub fn conversion_error() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E103",
            ErrorCategory::Type,
            "Cannot convert between these types"
        )
    }
    
    pub fn invalid_cardinality() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E104",
            ErrorCategory::Type,
            "Collection cardinality is invalid for this operation"
        )
    }
    
    // Property errors (E200-E299)
    pub fn property_not_found() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E200",
            ErrorCategory::Property,
            "Property does not exist on this type"
        )
    }
    
    pub fn property_access_denied() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E201",
            ErrorCategory::Property,
            "Property access is not allowed in this context"
        )
    }
    
    pub fn invalid_type_specifier() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E202",
            ErrorCategory::Property,
            "Type specifier is invalid"
        )
    }
    
    // Function errors (E300-E399)
    pub fn function_not_found() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E300",
            ErrorCategory::Function,
            "Function is not defined"
        )
    }
    
    pub fn invalid_arity() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E301",
            ErrorCategory::Function,
            "Incorrect number of arguments for function"
        )
    }
    
    pub fn function_not_applicable() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E302",
            ErrorCategory::Function,
            "Function cannot be applied to this type"
        )
    }
    
    // Lambda and variable errors (E400-E499)
    pub fn undefined_variable() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E400",
            ErrorCategory::Lambda,
            "Variable is not defined in current scope"
        )
    }
    
    pub fn invalid_lambda_expression() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E401",
            ErrorCategory::Lambda,
            "Lambda expression is invalid"
        )
    }
    
    // Runtime errors (E500-E599)
    pub fn division_by_zero() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E500",
            ErrorCategory::General,
            "Division by zero is not allowed"
        )
    }
    
    pub fn index_out_of_bounds() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E501",
            ErrorCategory::General,
            "Index is outside the bounds of the collection"
        )
    }
    
    pub fn arithmetic_overflow() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E502",
            ErrorCategory::General,
            "Arithmetic operation resulted in overflow"
        )
    }
    
    pub fn invalid_regex() -> StructuredDiagnosticCode {
        StructuredDiagnosticCode::new(
            "E503",
            ErrorCategory::General,
            "Regular expression is invalid"
        )
    }
}

/// Registry for looking up error code information
pub struct ErrorCodeRegistry {
    codes: std::collections::HashMap<String, StructuredDiagnosticCode>,
}

impl ErrorCodeRegistry {
    /// Create a new error code registry with predefined codes
    pub fn new() -> Self {
        let mut registry = Self {
            codes: std::collections::HashMap::new(),
        };
        
        // Register all predefined codes
        registry.register(codes::unexpected_token());
        registry.register(codes::expected_token());
        registry.register(codes::unclosed_string());
        registry.register(codes::invalid_number());
        registry.register(codes::invalid_datetime());
        registry.register(codes::unknown_operator());
        registry.register(codes::invalid_escape());
        registry.register(codes::type_mismatch());
        registry.register(codes::invalid_operand_types());
        registry.register(codes::invalid_argument_types());
        registry.register(codes::conversion_error());
        registry.register(codes::invalid_cardinality());
        registry.register(codes::property_not_found());
        registry.register(codes::property_access_denied());
        registry.register(codes::invalid_type_specifier());
        registry.register(codes::function_not_found());
        registry.register(codes::invalid_arity());
        registry.register(codes::function_not_applicable());
        registry.register(codes::undefined_variable());
        registry.register(codes::invalid_lambda_expression());
        registry.register(codes::division_by_zero());
        registry.register(codes::index_out_of_bounds());
        registry.register(codes::arithmetic_overflow());
        registry.register(codes::invalid_regex());
        
        registry
    }
    
    /// Register a new error code
    pub fn register(&mut self, code: StructuredDiagnosticCode) {
        self.codes.insert(code.code.clone(), code);
    }
    
    /// Look up an error code by its identifier
    pub fn get(&self, code: &str) -> Option<&StructuredDiagnosticCode> {
        self.codes.get(code)
    }
    
    /// Get all error codes in a specific category
    pub fn get_by_category(&self, category: &ErrorCategory) -> Vec<&StructuredDiagnosticCode> {
        self.codes
            .values()
            .filter(|code| &code.category == category)
            .collect()
    }
    
    /// Get all available error codes
    pub fn all_codes(&self) -> Vec<&StructuredDiagnosticCode> {
        self.codes.values().collect()
    }
    
    /// Check if a code exists
    pub fn exists(&self, code: &str) -> bool {
        self.codes.contains_key(code)
    }
}

impl Default for ErrorCodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_structured_diagnostic_code() {
        let code = StructuredDiagnosticCode::new(
            "E001",
            ErrorCategory::Syntax,
            "Test error"
        );
        
        assert_eq!(code.code, "E001");
        assert_eq!(code.category, ErrorCategory::Syntax);
        assert_eq!(code.description, "Test error");
        assert!(code.help_url.is_none());
        assert!(code.full_help_url().is_some());
    }
    
    #[test]
    fn test_error_code_registry() {
        let registry = ErrorCodeRegistry::new();
        
        assert!(registry.exists("E001"));
        assert!(registry.exists("E100"));
        assert!(!registry.exists("X999"));
        
        let syntax_codes = registry.get_by_category(&ErrorCategory::Syntax);
        assert!(!syntax_codes.is_empty());
        
        let code = registry.get("E001").unwrap();
        assert_eq!(code.category, ErrorCategory::Syntax);
    }
    
    #[test]
    fn test_error_categories() {
        assert_eq!(ErrorCategory::Syntax.to_string(), "syntax");
        assert_eq!(ErrorCategory::Performance.to_string(), "performance");
    }
    
    #[test]
    fn test_code_functions() {
        let code = codes::unexpected_token();
        assert_eq!(code.code, "E001");
        assert_eq!(code.category, ErrorCategory::Syntax);
        
        let type_code = codes::type_mismatch();
        assert_eq!(type_code.code, "E100");
        assert_eq!(type_code.category, ErrorCategory::Type);
    }
}