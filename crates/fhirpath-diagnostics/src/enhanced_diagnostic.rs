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

//! Enhanced diagnostic system with contextual help and intelligent suggestions

use super::{
    diagnostic::{Diagnostic, DiagnosticCode},
    location::SourceLocation,
};

#[cfg(test)]
use super::diagnostic::Severity;
use std::collections::HashMap;
use std::fmt;
/// Enhanced diagnostic with contextual help and intelligent suggestions
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnhancedDiagnostic {
    /// Core diagnostic information
    pub diagnostic: Diagnostic,
    /// Contextual help and explanations
    pub context: Vec<String>,
    /// Intelligent suggestions generated based on context
    pub smart_suggestions: Vec<SmartSuggestion>,
    /// Documentation links for further reading
    pub documentation_links: Vec<DocumentationLink>,
    /// Fix-it suggestions that can be automatically applied
    pub quick_fixes: Vec<QuickFix>,
}

/// Smart suggestion with confidence and rationale
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmartSuggestion {
    /// The suggestion message
    pub message: String,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Rationale for why this suggestion is provided
    pub rationale: String,
    /// Optional code example demonstrating the suggestion
    pub example: Option<String>,
    /// Category of the suggestion
    pub category: SuggestionCategory,
}

/// Categories of suggestions
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SuggestionCategory {
    /// Syntax corrections
    Syntax,
    /// Type-related suggestions
    Type,
    /// Function usage suggestions
    Function,
    /// Performance improvements
    Performance,
    /// Best practices
    BestPractice,
    /// Common mistakes
    CommonMistake,
}

/// Documentation link for additional help
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentationLink {
    /// Title of the documentation
    pub title: String,
    /// URL or reference to the documentation
    pub url: String,
    /// Brief description of what the link contains
    pub description: String,
}

/// Automatic fix that can be applied
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuickFix {
    /// Title of the fix
    pub title: String,
    /// Description of what the fix does
    pub description: String,
    /// The replacement text
    pub replacement: String,
    /// Location where the fix should be applied
    pub location: SourceLocation,
    /// Whether this fix is safe to apply automatically
    pub is_safe: bool,
}

/// Suggestion generator that creates contextual help
pub struct SuggestionGenerator {
    /// Common typos and their corrections
    typo_corrections: HashMap<String, Vec<String>>,
    /// Function signature database for suggestions
    function_signatures: HashMap<String, FunctionSignature>,
}

/// Function signature information for suggestions
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Function parameters
    pub parameters: Vec<ParameterInfo>,
    /// Return type description
    pub return_type: String,
    /// Function description
    pub description: String,
    /// Usage examples
    pub examples: Vec<String>,
}

/// Information about a function parameter
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    /// Parameter name
    pub name: String,
    /// Parameter type name
    pub type_name: String,
    /// Whether the parameter is optional
    pub optional: bool,
    /// Parameter description
    pub description: String,
}

impl EnhancedDiagnostic {
    /// Create an enhanced diagnostic from a basic diagnostic
    pub fn from_diagnostic(diagnostic: Diagnostic) -> Self {
        let mut enhanced = Self {
            diagnostic,
            context: Vec::new(),
            smart_suggestions: Vec::new(),
            documentation_links: Vec::new(),
            quick_fixes: Vec::new(),
        };

        enhanced.generate_enhancements();
        enhanced
    }

    /// Generate contextual enhancements based on the diagnostic
    fn generate_enhancements(&mut self) {
        let generator = SuggestionGenerator::new();

        let code = self.diagnostic.code.clone();
        match code {
            DiagnosticCode::UnknownFunction => {
                self.generate_unknown_function_help(&generator);
            }
            DiagnosticCode::ExpectedToken(expected) => {
                self.generate_expected_token_help(&generator, &expected);
            }
            DiagnosticCode::TypeMismatch { expected, actual } => {
                self.generate_type_mismatch_help(&generator, &expected, &actual);
            }
            DiagnosticCode::InvalidArity => {
                self.generate_arity_help(&generator);
            }
            DiagnosticCode::UndefinedVariable => {
                self.generate_undefined_variable_help(&generator);
            }
            DiagnosticCode::UnexpectedToken => {
                self.generate_unexpected_token_help(&generator);
            }
            _ => {
                self.generate_generic_help(&generator);
            }
        }

        // Add standard documentation links
        self.add_standard_documentation();
    }

    fn generate_unknown_function_help(&mut self, generator: &SuggestionGenerator) {
        self.context
            .push("FHIRPath functions must be defined in the function registry.".to_string());
        self.context.push(
            "Check for typos in the function name or verify the function is available.".to_string(),
        );

        // Extract function name from diagnostic message
        if let Some(func_name) = self.extract_function_name_from_message() {
            // Find similar function names
            let suggestions = generator.find_similar_functions(&func_name);
            for suggestion in suggestions {
                self.smart_suggestions.push(SmartSuggestion {
                    message: format!("Did you mean '{}'?", suggestion.name),
                    confidence: 0.8,
                    rationale: "Function name similarity based on edit distance".to_string(),
                    example: Some(format!("{}()", suggestion.name)),
                    category: SuggestionCategory::CommonMistake,
                });
            }

            // Add quick fix for common typos
            if let Some(correction) = generator.get_typo_correction(&func_name) {
                self.quick_fixes.push(QuickFix {
                    title: format!("Replace '{func_name}' with '{correction}'"),
                    description: "Fix common typo".to_string(),
                    replacement: correction.clone(),
                    location: self.diagnostic.location.clone(),
                    is_safe: true,
                });
            }
        }
    }

    fn generate_expected_token_help(&mut self, _generator: &SuggestionGenerator, expected: &str) {
        self.context.push(format!(
            "The parser expected to find '{expected}' at this position."
        ));

        match expected {
            ")" => {
                self.context
                    .push("Check for unmatched parentheses in your expression.".to_string());
                self.smart_suggestions.push(SmartSuggestion {
                    message: "Add closing parenthesis".to_string(),
                    confidence: 0.9,
                    rationale: "Missing closing parenthesis is a common syntax error".to_string(),
                    example: Some("Patient.name.where(use = 'official')".to_string()),
                    category: SuggestionCategory::Syntax,
                });
            }
            "]" => {
                self.context
                    .push("Check for unmatched brackets in array indexing.".to_string());
                self.smart_suggestions.push(SmartSuggestion {
                    message: "Add closing bracket".to_string(),
                    confidence: 0.9,
                    rationale: "Missing closing bracket in array indexing".to_string(),
                    example: Some("Patient.name[0]".to_string()),
                    category: SuggestionCategory::Syntax,
                });
            }
            "." => {
                self.context.push(
                    "Property access requires a dot (.) before the property name.".to_string(),
                );
                self.smart_suggestions.push(SmartSuggestion {
                    message: "Add dot for property access".to_string(),
                    confidence: 0.8,
                    rationale: "Property access syntax in FHIRPath".to_string(),
                    example: Some("Patient.name.family".to_string()),
                    category: SuggestionCategory::Syntax,
                });
            }
            _ => {
                self.smart_suggestions.push(SmartSuggestion {
                    message: format!("Add the missing '{expected}'"),
                    confidence: 0.7,
                    rationale: "Required by FHIRPath syntax".to_string(),
                    example: None,
                    category: SuggestionCategory::Syntax,
                });
            }
        }
    }

    fn generate_type_mismatch_help(
        &mut self,
        _generator: &SuggestionGenerator,
        expected: &str,
        actual: &str,
    ) {
        self.context
            .push(format!("Expected type '{expected}' but found '{actual}'."));
        self.context.push(
            "Type mismatches often occur when using incorrect operators or function arguments."
                .to_string(),
        );

        // Suggest type conversions
        match (expected, actual) {
            ("string", "integer") => {
                self.smart_suggestions.push(SmartSuggestion {
                    message: "Convert integer to string using toString()".to_string(),
                    confidence: 0.8,
                    rationale: "toString() converts numeric values to strings".to_string(),
                    example: Some("Patient.id.toString()".to_string()),
                    category: SuggestionCategory::Type,
                });
            }
            ("integer", "string") => {
                self.smart_suggestions.push(SmartSuggestion {
                    message: "Convert string to integer using toInteger()".to_string(),
                    confidence: 0.8,
                    rationale: "toInteger() parses numeric strings to integers".to_string(),
                    example: Some("Patient.age.toInteger()".to_string()),
                    category: SuggestionCategory::Type,
                });
            }
            ("boolean", _) => {
                self.smart_suggestions.push(SmartSuggestion {
                    message: "Use comparison operators to create boolean values".to_string(),
                    confidence: 0.7,
                    rationale: "Boolean expressions use comparison or logical operators"
                        .to_string(),
                    example: Some("Patient.active = true".to_string()),
                    category: SuggestionCategory::Type,
                });
            }
            _ => {
                self.smart_suggestions.push(SmartSuggestion {
                    message: format!(
                        "Check if type conversion from {actual} to {expected} is needed"
                    ),
                    confidence: 0.6,
                    rationale: "Type conversion might resolve the mismatch".to_string(),
                    example: None,
                    category: SuggestionCategory::Type,
                });
            }
        }
    }

    fn generate_arity_help(&mut self, generator: &SuggestionGenerator) {
        self.context
            .push("The function was called with the wrong number of arguments.".to_string());
        self.context
            .push("Check the function signature to see the required parameters.".to_string());

        // Extract function name and suggest correct signature
        if let Some(func_name) = self.extract_function_name_from_message() {
            if let Some(signature) = generator.get_function_signature(&func_name) {
                let param_list: Vec<String> = signature
                    .parameters
                    .iter()
                    .map(|p| {
                        if p.optional {
                            format!("[{}]", p.name)
                        } else {
                            p.name.clone()
                        }
                    })
                    .collect();

                self.smart_suggestions.push(SmartSuggestion {
                    message: format!(
                        "{}({}) - {}",
                        signature.name,
                        param_list.join(", "),
                        signature.description
                    ),
                    confidence: 0.9,
                    rationale: "Correct function signature".to_string(),
                    example: signature.examples.first().cloned(),
                    category: SuggestionCategory::Function,
                });
            }
        }
    }

    fn generate_undefined_variable_help(&mut self, generator: &SuggestionGenerator) {
        self.context.push(
            "Variables in FHIRPath start with '$' and must be defined in the current scope."
                .to_string(),
        );
        self.context
            .push("Common variables include $this, $context, and $resource.".to_string());

        if let Some(var_name) = self.extract_variable_name_from_message() {
            // Suggest similar variable names
            let similar_vars = vec!["$this", "$context", "$resource", "$total", "$index"];
            for similar in similar_vars {
                if generator.is_similar_string(&var_name, similar) {
                    self.smart_suggestions.push(SmartSuggestion {
                        message: format!("Did you mean '{similar}'?"),
                        confidence: 0.7,
                        rationale: "Similar variable name".to_string(),
                        example: Some(format!("{similar}.value")),
                        category: SuggestionCategory::CommonMistake,
                    });
                }
            }

            // Suggest defining the variable
            self.smart_suggestions.push(SmartSuggestion {
                message: "Ensure the variable is defined in the current scope".to_string(),
                confidence: 0.8,
                rationale: "Variables must be defined before use".to_string(),
                example: Some("where($this.active = true)".to_string()),
                category: SuggestionCategory::BestPractice,
            });
        }
    }

    fn generate_unexpected_token_help(&mut self, _generator: &SuggestionGenerator) {
        self.context
            .push("The parser encountered a token it didn't expect at this position.".to_string());
        self.context
            .push("This often indicates a syntax error or typo.".to_string());

        self.smart_suggestions.push(SmartSuggestion {
            message: "Check for syntax errors, missing operators, or typos".to_string(),
            confidence: 0.6,
            rationale: "Unexpected tokens usually indicate syntax issues".to_string(),
            example: Some("Patient.name.where(use = 'official')".to_string()),
            category: SuggestionCategory::Syntax,
        });
    }

    fn generate_generic_help(&mut self, _generator: &SuggestionGenerator) {
        self.context
            .push("Refer to the FHIRPath specification for detailed syntax rules.".to_string());

        self.smart_suggestions.push(SmartSuggestion {
            message: "Check the FHIRPath specification for correct syntax".to_string(),
            confidence: 0.5,
            rationale: "Generic guidance for unrecognized error patterns".to_string(),
            example: None,
            category: SuggestionCategory::BestPractice,
        });
    }

    fn add_standard_documentation(&mut self) {
        self.documentation_links.push(DocumentationLink {
            title: "FHIRPath Specification".to_string(),
            url: "http://hl7.org/fhirpath/".to_string(),
            description: "Official FHIRPath language specification".to_string(),
        });

        self.documentation_links.push(DocumentationLink {
            title: "FHIRPath Functions Reference".to_string(),
            url: "http://hl7.org/fhirpath/#functions".to_string(),
            description: "Complete reference of built-in FHIRPath functions".to_string(),
        });

        let code = self.diagnostic.code.clone();
        match code {
            DiagnosticCode::TypeMismatch { .. } => {
                self.documentation_links.push(DocumentationLink {
                    title: "FHIRPath Type System".to_string(),
                    url: "http://hl7.org/fhirpath/#types".to_string(),
                    description: "Understanding FHIRPath types and conversions".to_string(),
                });
            }
            DiagnosticCode::UnknownFunction => {
                self.documentation_links.push(DocumentationLink {
                    title: "Function Registry".to_string(),
                    url: "docs/functions.md".to_string(),
                    description: "Available functions in this implementation".to_string(),
                });
            }
            _ => {}
        }
    }

    /// Extract function name from diagnostic message
    fn extract_function_name_from_message(&self) -> Option<String> {
        // Simple extraction - in a real implementation, this would be more sophisticated
        let message = &self.diagnostic.message;
        if message.contains("function") {
            // Extract quoted function name
            if let Some(start) = message.find('\'') {
                if let Some(end) = message[start + 1..].find('\'') {
                    return Some(message[start + 1..start + 1 + end].to_string());
                }
            }
        }
        None
    }

    /// Extract variable name from diagnostic message
    fn extract_variable_name_from_message(&self) -> Option<String> {
        let message = &self.diagnostic.message;
        if message.contains("variable") {
            // Extract quoted variable name
            if let Some(start) = message.find('\'') {
                if let Some(end) = message[start + 1..].find('\'') {
                    return Some(message[start + 1..start + 1 + end].to_string());
                }
            }
        }
        None
    }

    /// Check if this diagnostic has any suggestions
    pub fn has_suggestions(&self) -> bool {
        !self.smart_suggestions.is_empty() || !self.quick_fixes.is_empty()
    }

    /// Get the highest confidence suggestion
    pub fn best_suggestion(&self) -> Option<&SmartSuggestion> {
        self.smart_suggestions.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get all safe quick fixes
    pub fn safe_quick_fixes(&self) -> Vec<&QuickFix> {
        self.quick_fixes.iter().filter(|fix| fix.is_safe).collect()
    }
}

impl Default for SuggestionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SuggestionGenerator {
    /// Creates a new suggestion generator with default corrections and signatures
    pub fn new() -> Self {
        let mut generator = Self {
            typo_corrections: HashMap::new(),
            function_signatures: HashMap::new(),
        };

        generator.initialize_common_corrections();
        generator.initialize_function_signatures();
        generator
    }

    fn initialize_common_corrections(&mut self) {
        // Common typos and corrections
        self.typo_corrections
            .insert("lenght".to_string(), vec!["length".to_string()]);
        self.typo_corrections
            .insert("exsits".to_string(), vec!["exists".to_string()]);
        self.typo_corrections
            .insert("whre".to_string(), vec!["where".to_string()]);
        self.typo_corrections
            .insert("slect".to_string(), vec!["select".to_string()]);
        self.typo_corrections
            .insert("frist".to_string(), vec!["first".to_string()]);
        self.typo_corrections
            .insert("lsat".to_string(), vec!["last".to_string()]);
        self.typo_corrections
            .insert("singel".to_string(), vec!["single".to_string()]);
        self.typo_corrections
            .insert("conatin".to_string(), vec!["contains".to_string()]);
        self.typo_corrections
            .insert("startsWtih".to_string(), vec!["startsWith".to_string()]);
        self.typo_corrections
            .insert("endsWtih".to_string(), vec!["endsWith".to_string()]);
    }

    fn initialize_function_signatures(&mut self) {
        // Common FHIRPath functions
        self.function_signatures.insert(
            "where".to_string(),
            FunctionSignature {
                name: "where".to_string(),
                parameters: vec![ParameterInfo {
                    name: "criteria".to_string(),
                    type_name: "boolean".to_string(),
                    optional: false,
                    description: "Boolean expression to filter by".to_string(),
                }],
                return_type: "collection".to_string(),
                description: "Returns collection items that match the criteria".to_string(),
                examples: vec![
                    "Patient.name.where(use = 'official')".to_string(),
                    "Bundle.entry.where(resource is Patient)".to_string(),
                ],
            },
        );

        self.function_signatures.insert(
            "select".to_string(),
            FunctionSignature {
                name: "select".to_string(),
                parameters: vec![ParameterInfo {
                    name: "expression".to_string(),
                    type_name: "any".to_string(),
                    optional: false,
                    description: "Expression to evaluate for each item".to_string(),
                }],
                return_type: "collection".to_string(),
                description: "Transforms each item in the collection".to_string(),
                examples: vec![
                    "Patient.name.select(family)".to_string(),
                    "Bundle.entry.select(resource.id)".to_string(),
                ],
            },
        );

        self.function_signatures.insert(
            "exists".to_string(),
            FunctionSignature {
                name: "exists".to_string(),
                parameters: vec![ParameterInfo {
                    name: "criteria".to_string(),
                    type_name: "boolean".to_string(),
                    optional: true,
                    description: "Optional criteria expression".to_string(),
                }],
                return_type: "boolean".to_string(),
                description: "Returns true if any items exist (optionally matching criteria)"
                    .to_string(),
                examples: vec![
                    "Patient.name.exists()".to_string(),
                    "Patient.telecom.exists(system = 'email')".to_string(),
                ],
            },
        );
    }

    /// Finds function signatures similar to the given name
    pub fn find_similar_functions(&self, name: &str) -> Vec<&FunctionSignature> {
        self.function_signatures
            .values()
            .filter(|sig| self.is_similar_string(name, &sig.name))
            .collect()
    }

    /// Gets a typo correction for the given name
    pub fn get_typo_correction(&self, name: &str) -> Option<&String> {
        self.typo_corrections
            .get(name)
            .and_then(|corrections| corrections.first())
    }

    /// Gets the function signature for the given name
    pub fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature> {
        self.function_signatures.get(name)
    }

    /// Checks if two strings are similar using simple heuristics
    pub fn is_similar_string(&self, a: &str, b: &str) -> bool {
        // Simple similarity check - in practice, use edit distance
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        // Check for common prefixes/suffixes
        if a_lower.len() >= 3 && b_lower.len() >= 3 {
            a_lower[..3] == b_lower[..3]
                || a_lower.ends_with(&b_lower[b_lower.len() - 3..])
                || b_lower.ends_with(&a_lower[a_lower.len() - 3..])
        } else {
            false
        }
    }
}

impl fmt::Display for EnhancedDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display the core diagnostic
        writeln!(
            f,
            "{}: {} [{}]",
            self.diagnostic.severity,
            self.diagnostic.message,
            self.diagnostic.code_string()
        )?;

        // Display context
        if !self.context.is_empty() {
            writeln!(f, "\nContext:")?;
            for ctx in &self.context {
                writeln!(f, "  • {ctx}")?;
            }
        }

        // Display suggestions
        if !self.smart_suggestions.is_empty() {
            writeln!(f, "\nSuggestions:")?;
            for suggestion in &self.smart_suggestions {
                writeln!(
                    f,
                    "  • {} (confidence: {:.0}%)",
                    suggestion.message,
                    suggestion.confidence * 100.0
                )?;
                if let Some(example) = &suggestion.example {
                    writeln!(f, "    Example: {example}")?;
                }
            }
        }

        // Display quick fixes
        if !self.quick_fixes.is_empty() {
            writeln!(f, "\nQuick fixes:")?;
            for fix in &self.quick_fixes {
                writeln!(f, "  • {}: {}", fix.title, fix.description)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::{Position, Span};

    fn create_test_location() -> SourceLocation {
        SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 5)),
            source_text: Some("test".to_string()),
            file_path: None,
        }
    }

    #[test]
    fn test_enhanced_diagnostic_from_unknown_function() {
        let diagnostic = Diagnostic::new(
            Severity::Error,
            DiagnosticCode::UnknownFunction,
            "Unknown function 'lenght'".to_string(),
            create_test_location(),
        );

        let enhanced = EnhancedDiagnostic::from_diagnostic(diagnostic);

        assert!(!enhanced.context.is_empty());
        assert!(enhanced.has_suggestions());
        assert!(!enhanced.documentation_links.is_empty());
    }

    #[test]
    fn test_enhanced_diagnostic_from_type_mismatch() {
        let diagnostic = Diagnostic::new(
            Severity::Error,
            DiagnosticCode::TypeMismatch {
                expected: "string".to_string(),
                actual: "integer".to_string(),
            },
            "Type mismatch: expected string, found integer".to_string(),
            create_test_location(),
        );

        let enhanced = EnhancedDiagnostic::from_diagnostic(diagnostic);

        assert!(enhanced.context.len() >= 2);
        assert!(!enhanced.smart_suggestions.is_empty());

        // Should suggest toString() conversion
        let has_tostring_suggestion = enhanced
            .smart_suggestions
            .iter()
            .any(|s| s.message.contains("toString()"));
        assert!(has_tostring_suggestion);
    }

    #[test]
    fn test_suggestion_generator_typo_correction() {
        let generator = SuggestionGenerator::new();

        let correction = generator.get_typo_correction("lenght");
        assert_eq!(correction, Some(&"length".to_string()));

        let no_correction = generator.get_typo_correction("correct_name");
        assert_eq!(no_correction, None);
    }

    #[test]
    fn test_suggestion_generator_function_signature() {
        let generator = SuggestionGenerator::new();

        let signature = generator.get_function_signature("where");
        assert!(signature.is_some());

        let signature = signature.unwrap();
        assert_eq!(signature.name, "where");
        assert_eq!(signature.parameters.len(), 1);
        assert_eq!(signature.parameters[0].type_name, "boolean");
    }

    #[test]
    fn test_enhanced_diagnostic_display() {
        let diagnostic = Diagnostic::new(
            Severity::Error,
            DiagnosticCode::UnknownFunction,
            "Unknown function 'test'".to_string(),
            create_test_location(),
        );

        let enhanced = EnhancedDiagnostic::from_diagnostic(diagnostic);
        let display = format!("{enhanced}");

        assert!(display.contains("error:"));
        assert!(display.contains("Context:"));
        // Note: This test doesn't generate suggestions because "test" doesn't match any known functions
        assert!(!display.contains("Suggestions:"));
    }
}
