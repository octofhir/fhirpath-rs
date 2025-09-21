use std::sync::Arc;

use crate::ast::{ExpressionNode, analysis::AnalysisMetadata};
use crate::core::error_code::{FP0053, FP0054};
use crate::core::{FhirPathError, ModelProvider, SourceLocation};
use crate::diagnostics::{AriadneDiagnostic, Diagnostic, DiagnosticCode, DiagnosticSeverity};
use octofhir_fhir_model::TypeInfo;

/// Result type for function analysis operations
pub type AnalysisResult = Result<AnalysisMetadata, FhirPathError>;

/// Function analyzer for enhanced function validation with context checking
pub struct FunctionAnalyzer {
    #[allow(dead_code)]
    model_provider: Arc<dyn ModelProvider>,
    function_registry: Arc<crate::evaluator::FunctionRegistry>,
}

/// Function signature information for validation
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub required_args: usize,
    pub optional_args: usize,
    pub input_requirements: InputRequirement,
    pub argument_types: Vec<ArgumentType>,
    pub return_type: ReturnType,
}

/// Input requirements for functions
#[derive(Debug, Clone, PartialEq)]
pub enum InputRequirement {
    /// Function can work with any input
    Any,
    /// Function requires a singleton value
    Singleton,
    /// Function requires a collection
    Collection,
    /// Function requires a non-empty collection
    NonEmptyCollection,
    /// Function doesn't take input (root context functions)
    None,
}

/// Argument type specifications
#[derive(Debug, Clone)]
pub enum ArgumentType {
    /// Any type is acceptable
    Any,
    /// Specific type required
    Type(String),
    /// Boolean expression
    Boolean,
    /// Numeric type (integer or decimal)
    Numeric,
    /// String type
    String,
    /// Expression that evaluates in current context
    Expression,
}

/// Return type specifications
#[derive(Debug, Clone)]
pub enum ReturnType {
    /// Same as input type
    SameAsInput,
    /// Specific type
    Type(String),
    /// Boolean
    Boolean,
    /// Numeric
    Numeric,
    /// String
    String,
    /// Collection of input type
    CollectionOf(Box<ReturnType>),
    /// Empty (void)
    Empty,
}

impl FunctionAnalyzer {
    /// Create a new FunctionAnalyzer with the given ModelProvider and FunctionRegistry
    pub fn new(
        model_provider: Arc<dyn ModelProvider>,
        function_registry: Arc<crate::evaluator::FunctionRegistry>,
    ) -> Self {
        Self {
            model_provider,
            function_registry,
        }
    }

    /// Validate function call and return AriadneDiagnostic directly (for static analyzer)
    pub async fn validate_function_call_new(
        &self,
        function_name: &str,
        input_type: &TypeInfo,
        arguments: &[ExpressionNode],
        span: std::ops::Range<usize>,
    ) -> Vec<AriadneDiagnostic> {
        let mut diagnostics = Vec::new();

        // Get function signature
        let signature = match self.get_function_signature(function_name) {
            Some(sig) => sig,
            None => {
                // Unknown function - provide suggestions
                let suggestions = self.suggest_function_names(function_name);
                let message = if !suggestions.is_empty() {
                    format!(
                        "Unknown function '{}', did you mean '{}'?",
                        function_name, suggestions[0]
                    )
                } else {
                    format!("Unknown function '{function_name}'")
                };

                let diagnostic = AriadneDiagnostic {
                    severity: DiagnosticSeverity::Error,
                    error_code: FP0054,
                    message,
                    span,
                    help: None,
                    note: None,
                    related: Vec::new(),
                };
                diagnostics.push(diagnostic);
                return diagnostics;
            }
        };

        // Validate input requirements
        if let Some(input_diagnostic) = self.validate_input_requirements_new(
            function_name,
            input_type,
            &signature.input_requirements,
            span.clone(),
        ) {
            diagnostics.push(input_diagnostic);
        }

        // Validate argument count
        let provided_args = arguments.len();
        let min_args = signature.required_args;
        let max_args = signature.required_args + signature.optional_args;

        if provided_args < min_args {
            let diagnostic = AriadneDiagnostic {
                severity: DiagnosticSeverity::Error,
                error_code: FP0053,
                message: format!(
                    "Function '{function_name}' requires at least {min_args} argument(s), but {provided_args} provided"
                ),
                span,
                help: None,
                note: None,
                related: Vec::new(),
            };
            diagnostics.push(diagnostic);
        } else if provided_args > max_args {
            let diagnostic = AriadneDiagnostic {
                severity: DiagnosticSeverity::Error,
                error_code: FP0053,
                message: format!(
                    "Function '{function_name}' accepts at most {max_args} argument(s), but {provided_args} provided"
                ),
                span,
                help: None,
                note: None,
                related: Vec::new(),
            };
            diagnostics.push(diagnostic);
        }

        diagnostics
    }

    /// Validate input requirements and return AriadneDiagnostic
    fn validate_input_requirements_new(
        &self,
        function_name: &str,
        input_type: &TypeInfo,
        requirement: &InputRequirement,
        span: std::ops::Range<usize>,
    ) -> Option<AriadneDiagnostic> {
        match requirement {
            InputRequirement::Singleton => {
                if !input_type.singleton.unwrap_or(true) {
                    Some(AriadneDiagnostic {
                        severity: DiagnosticSeverity::Error,
                        error_code: FP0053,
                        message: format!("Function '{function_name}()' requires singleton input"),
                        span,
                        help: None,
                        note: None,
                        related: Vec::new(),
                    })
                } else {
                    None
                }
            }
            InputRequirement::Collection => {
                if input_type.singleton.unwrap_or(true) {
                    Some(AriadneDiagnostic {
                        severity: DiagnosticSeverity::Error,
                        error_code: FP0053,
                        message: format!("Function '{function_name}()' requires collection input"),
                        span,
                        help: None,
                        note: None,
                        related: Vec::new(),
                    })
                } else {
                    None
                }
            }
            InputRequirement::NonEmptyCollection => {
                if input_type.singleton.unwrap_or(true) || input_type.is_empty.unwrap_or(false) {
                    Some(AriadneDiagnostic {
                        severity: DiagnosticSeverity::Error,
                        error_code: FP0053,
                        message: format!(
                            "Function '{function_name}()' requires non-empty collection input"
                        ),
                        span,
                        help: None,
                        note: None,
                        related: Vec::new(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Validate function with enhanced context checking
    pub async fn validate_function_call(
        &self,
        function_name: &str,
        input_type: &TypeInfo,
        arguments: &[ExpressionNode],
        location: Option<SourceLocation>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        // Get function signature
        let signature = match self.get_function_signature(function_name) {
            Some(sig) => sig,
            None => {
                // Unknown function - provide suggestions
                let suggestions = self.suggest_function_names(function_name);
                let mut message = format!("Unknown function '{function_name}'");

                if !suggestions.is_empty() {
                    message.push_str(&format!(". Did you mean '{}'?", suggestions[0]));
                }

                let diagnostic = Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "FP0304".to_string(),
                        namespace: None,
                    },
                    message,
                    location,
                    related: vec![],
                };
                metadata.add_diagnostic(diagnostic);
                return Ok(metadata);
            }
        };

        // Validate input requirements
        if let Some(input_diagnostic) = self.validate_input_requirements(
            function_name,
            input_type,
            &signature.input_requirements,
            location.clone(),
        ) {
            metadata.add_diagnostic(input_diagnostic);
        }

        // Validate argument count
        let provided_args = arguments.len();
        let min_args = signature.required_args;
        let max_args = signature.required_args + signature.optional_args;

        if provided_args < min_args {
            let diagnostic = Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode {
                    code: "FP0305".to_string(),
                    namespace: None,
                },
                message: format!(
                    "Function '{function_name}' requires at least {min_args} argument(s), but {provided_args} provided"
                ),
                location,
                related: vec![],
            };
            metadata.add_diagnostic(diagnostic);
        } else if provided_args > max_args {
            let diagnostic = Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode {
                    code: "FP0306".to_string(),
                    namespace: None,
                },
                message: format!(
                    "Function '{function_name}' accepts at most {max_args} argument(s), but {provided_args} provided"
                ),
                location,
                related: vec![],
            };
            metadata.add_diagnostic(diagnostic);
        }

        // Set return type
        metadata.type_info = Some(self.resolve_return_type(&signature.return_type, input_type));

        Ok(metadata)
    }

    /// Suggest function names for typos using edit distance
    fn suggest_function_names(&self, attempted_name: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Get all available function names from registry
        let available_functions = self.get_all_function_names();

        for func_name in available_functions {
            let distance = self.levenshtein_distance(attempted_name, &func_name);
            let max_distance = std::cmp::max(2, attempted_name.len() / 2);

            if distance <= max_distance {
                suggestions.push((func_name, distance));
            }
        }

        suggestions.sort_by_key(|&(_, distance)| distance);
        suggestions
            .into_iter()
            .map(|(name, _)| name)
            .take(3)
            .collect()
    }

    /// Get all function names from the registry
    fn get_all_function_names(&self) -> Vec<String> {
        self.function_registry
            .list_functions()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }

        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

        for i in 0..=a_len {
            matrix[i][0] = i;
        }
        for j in 0..=b_len {
            matrix[0][j] = j;
        }

        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1, // deletion
                        matrix[i][j - 1] + 1, // insertion
                    ),
                    matrix[i - 1][j - 1] + cost, // substitution
                );
            }
        }

        matrix[a_len][b_len]
    }

    /// Validate aggregate functions require collections
    pub fn validate_aggregate_context(
        &self,
        function_name: &str,
        input_type: &TypeInfo,
    ) -> Option<Diagnostic> {
        let aggregate_functions = [
            "count",
            "sum",
            "avg",
            "min",
            "max",
            "distinct",
            "allTrue",
            "anyTrue",
            "allFalse",
            "anyFalse",
            "aggregate",
            "all",
            "any",
        ];

        if aggregate_functions.contains(&function_name) {
            // Check if input is a singleton
            if input_type.singleton.unwrap_or(true) {
                return Some(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "FP0302".to_string(),
                        namespace: None,
                    },
                    message: format!("Function '{function_name}()' requires collection input"),
                    location: None,
                    related: vec![],
                });
            }
        }

        None
    }

    /// Validate argument types more rigorously
    pub async fn validate_argument_types(
        &self,
        function_name: &str,
        expected_signature: &FunctionSignature,
        actual_arguments: &[TypeInfo],
    ) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        for (i, (expected_arg, actual_arg)) in expected_signature
            .argument_types
            .iter()
            .zip(actual_arguments.iter())
            .enumerate()
        {
            if !self
                .is_argument_type_compatible(expected_arg, actual_arg)
                .await
            {
                let diagnostic = Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "FP0307".to_string(),
                        namespace: None,
                    },
                    message: format!(
                        "Function '{function_name}' argument {}: expected {}, got {}",
                        i + 1,
                        self.format_argument_type(expected_arg),
                        actual_arg.type_name
                    ),
                    location: None,
                    related: vec![],
                };
                diagnostics.push(diagnostic);
            }
        }

        diagnostics
    }

    /// Validate input requirements for functions
    fn validate_input_requirements(
        &self,
        function_name: &str,
        input_type: &TypeInfo,
        requirement: &InputRequirement,
        location: Option<SourceLocation>,
    ) -> Option<Diagnostic> {
        match requirement {
            InputRequirement::Singleton => {
                if !input_type.singleton.unwrap_or(true) {
                    Some(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "FP0308".to_string(),
                            namespace: None,
                        },
                        message: format!("Function '{function_name}()' requires singleton input"),
                        location,
                        related: vec![],
                    })
                } else {
                    None
                }
            }
            InputRequirement::Collection => {
                if input_type.singleton.unwrap_or(true) {
                    Some(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "FP0302".to_string(),
                            namespace: None,
                        },
                        message: format!("Function '{function_name}()' requires collection input"),
                        location,
                        related: vec![],
                    })
                } else {
                    None
                }
            }
            InputRequirement::NonEmptyCollection => {
                if input_type.singleton.unwrap_or(true) {
                    Some(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "FP0302".to_string(),
                            namespace: None,
                        },
                        message: format!("Function '{function_name}()' requires collection input"),
                        location,
                        related: vec![],
                    })
                } else if input_type.is_empty.unwrap_or(false) {
                    Some(Diagnostic {
                        severity: DiagnosticSeverity::Warning,
                        code: DiagnosticCode {
                            code: "FP0309".to_string(),
                            namespace: None,
                        },
                        message: format!(
                            "Function '{function_name}()' on empty collection will always return empty"
                        ),
                        location,
                        related: vec![],
                    })
                } else {
                    None
                }
            }
            InputRequirement::None => {
                // Functions that don't take input context (like now(), today(), etc.)
                None
            }
            InputRequirement::Any => None,
        }
    }

    /// Check if argument type is compatible with expected type
    async fn is_argument_type_compatible(
        &self,
        expected: &ArgumentType,
        actual: &TypeInfo,
    ) -> bool {
        match expected {
            ArgumentType::Any => true,
            ArgumentType::Type(expected_type) => {
                actual.type_name == *expected_type
                    || self
                        .is_type_compatible(&actual.type_name, expected_type)
                        .await
            }
            ArgumentType::Boolean => {
                matches!(actual.type_name.as_str(), "Boolean" | "boolean")
            }
            ArgumentType::Numeric => {
                matches!(
                    actual.type_name.as_str(),
                    "Integer"
                        | "integer"
                        | "Decimal"
                        | "decimal"
                        | "unsignedInt"
                        | "positiveInt"
                        | "Number"
                )
            }
            ArgumentType::String => {
                matches!(
                    actual.type_name.as_str(),
                    "String" | "string" | "code" | "id" | "markdown" | "uri" | "url" | "canonical"
                )
            }
            ArgumentType::Expression => true, // Expressions are validated separately
        }
    }

    /// Check if two types are compatible (including inheritance)
    async fn is_type_compatible(&self, actual: &str, expected: &str) -> bool {
        if actual == expected {
            return true;
        }

        // Check inheritance relationships
        // This could be enhanced with ModelProvider inheritance information
        let type_compatibility = self.get_type_compatibility_map();

        if let Some(compatible_types) = type_compatibility.get(actual) {
            compatible_types.contains(&expected.to_string())
        } else {
            false
        }
    }

    /// Get basic type compatibility mapping
    fn get_type_compatibility_map(&self) -> std::collections::HashMap<String, Vec<String>> {
        let mut map = std::collections::HashMap::new();

        // Numeric compatibility
        map.insert(
            "integer".to_string(),
            vec!["decimal".to_string(), "Number".to_string()],
        );
        map.insert("decimal".to_string(), vec!["Number".to_string()]);
        map.insert(
            "unsignedInt".to_string(),
            vec![
                "integer".to_string(),
                "decimal".to_string(),
                "Number".to_string(),
            ],
        );
        map.insert(
            "positiveInt".to_string(),
            vec![
                "integer".to_string(),
                "decimal".to_string(),
                "Number".to_string(),
            ],
        );

        // String compatibility
        map.insert("code".to_string(), vec!["string".to_string()]);
        map.insert("id".to_string(), vec!["string".to_string()]);
        map.insert("markdown".to_string(), vec!["string".to_string()]);
        map.insert("uri".to_string(), vec!["string".to_string()]);
        map.insert(
            "url".to_string(),
            vec!["string".to_string(), "uri".to_string()],
        );
        map.insert(
            "canonical".to_string(),
            vec!["string".to_string(), "uri".to_string()],
        );

        map
    }

    /// Resolve return type based on function signature and input type
    #[allow(clippy::only_used_in_recursion)]
    fn resolve_return_type(&self, return_type: &ReturnType, input_type: &TypeInfo) -> TypeInfo {
        match return_type {
            ReturnType::SameAsInput => input_type.clone(),
            ReturnType::Type(type_name) => TypeInfo {
                type_name: type_name.clone(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("FHIR".to_string()),
                name: Some(type_name.clone()),
            },
            ReturnType::Boolean => TypeInfo {
                type_name: "Boolean".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Boolean".to_string()),
            },
            ReturnType::Numeric => TypeInfo {
                type_name: "Number".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Number".to_string()),
            },
            ReturnType::String => TypeInfo {
                type_name: "String".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("String".to_string()),
            },
            ReturnType::CollectionOf(inner_type) => {
                let mut result = self.resolve_return_type(inner_type, input_type);
                result.singleton = Some(false);
                result
            }
            ReturnType::Empty => TypeInfo {
                type_name: "Empty".to_string(),
                singleton: Some(false),
                is_empty: Some(true),
                namespace: Some("System".to_string()),
                name: Some("Empty".to_string()),
            },
        }
    }

    /// Format argument type for error messages
    fn format_argument_type(&self, arg_type: &ArgumentType) -> String {
        match arg_type {
            ArgumentType::Any => "any type".to_string(),
            ArgumentType::Type(type_name) => type_name.clone(),
            ArgumentType::Boolean => "Boolean".to_string(),
            ArgumentType::Numeric => "Number".to_string(),
            ArgumentType::String => "String".to_string(),
            ArgumentType::Expression => "expression".to_string(),
        }
    }

    /// Get function signature by name
    pub fn get_function_signature(&self, function_name: &str) -> Option<FunctionSignature> {
        // Define signatures for common FHIRPath functions
        match function_name {
            // Aggregate functions
            "count" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 0,
                optional_args: 0,
                input_requirements: InputRequirement::Collection,
                argument_types: vec![],
                return_type: ReturnType::Numeric,
            }),
            "sum" | "avg" | "min" | "max" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 0,
                optional_args: 0,
                input_requirements: InputRequirement::Collection,
                argument_types: vec![],
                return_type: ReturnType::Numeric,
            }),
            "distinct" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 0,
                optional_args: 0,
                input_requirements: InputRequirement::Collection,
                argument_types: vec![],
                return_type: ReturnType::SameAsInput,
            }),

            // Collection functions
            "first" | "last" | "single" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 0,
                optional_args: 0,
                input_requirements: InputRequirement::Collection,
                argument_types: vec![],
                return_type: ReturnType::SameAsInput,
            }),
            "take" | "skip" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 1,
                optional_args: 0,
                input_requirements: InputRequirement::Collection,
                argument_types: vec![ArgumentType::Numeric],
                return_type: ReturnType::SameAsInput,
            }),

            // Boolean functions
            "empty" | "exists" | "hasValue" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 0,
                optional_args: 0,
                input_requirements: InputRequirement::Any,
                argument_types: vec![],
                return_type: ReturnType::Boolean,
            }),
            "all" | "any" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 1,
                optional_args: 0,
                input_requirements: InputRequirement::Collection,
                argument_types: vec![ArgumentType::Expression],
                return_type: ReturnType::Boolean,
            }),
            "allTrue" | "anyTrue" | "allFalse" | "anyFalse" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 0,
                optional_args: 0,
                input_requirements: InputRequirement::Collection,
                argument_types: vec![],
                return_type: ReturnType::Boolean,
            }),

            // String functions
            "length" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 0,
                optional_args: 0,
                input_requirements: InputRequirement::Singleton,
                argument_types: vec![],
                return_type: ReturnType::Numeric,
            }),
            "substring" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 1,
                optional_args: 1,
                input_requirements: InputRequirement::Singleton,
                argument_types: vec![ArgumentType::Numeric, ArgumentType::Numeric],
                return_type: ReturnType::String,
            }),
            "startsWith" | "endsWith" | "contains" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 1,
                optional_args: 0,
                input_requirements: InputRequirement::Singleton,
                argument_types: vec![ArgumentType::String],
                return_type: ReturnType::Boolean,
            }),

            // Context functions (no input required)
            "now" | "today" | "timeOfDay" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 0,
                optional_args: 0,
                input_requirements: InputRequirement::None,
                argument_types: vec![],
                return_type: ReturnType::Type(match function_name {
                    "now" => "DateTime".to_string(),
                    "today" => "Date".to_string(),
                    "timeOfDay" => "Time".to_string(),
                    _ => "DateTime".to_string(),
                }),
            }),

            // Selection functions
            "where" | "select" => Some(FunctionSignature {
                name: function_name.to_string(),
                required_args: 1,
                optional_args: 0,
                input_requirements: InputRequirement::Collection,
                argument_types: vec![ArgumentType::Expression],
                return_type: ReturnType::SameAsInput,
            }),

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::EmptyModelProvider;
    use std::sync::Arc;

    fn create_test_analyzer() -> FunctionAnalyzer {
        let provider = Arc::new(EmptyModelProvider);
        let function_registry = Arc::new(crate::evaluator::FunctionRegistry::new());
        FunctionAnalyzer::new(provider, function_registry)
    }

    fn create_test_type_info(type_name: &str, singleton: bool) -> TypeInfo {
        TypeInfo {
            type_name: type_name.to_string(),
            singleton: Some(singleton),
            is_empty: Some(false),
            namespace: Some("FHIR".to_string()),
            name: Some(type_name.to_string()),
        }
    }

    fn create_test_location(offset: usize, length: usize) -> SourceLocation {
        SourceLocation::new(1, offset + 1, offset, length)
    }

    #[tokio::test]
    async fn test_analyzer_creation() {
        let analyzer = create_test_analyzer();
        assert_eq!(
            std::mem::size_of_val(&analyzer),
            std::mem::size_of::<FunctionAnalyzer>()
        );
    }

    #[tokio::test]
    async fn test_function_signature_lookup() {
        let analyzer = create_test_analyzer();

        // Test known function
        let signature = analyzer.get_function_signature("count");
        assert!(signature.is_some());
        let sig = signature.unwrap();
        assert_eq!(sig.name, "count");
        assert_eq!(sig.input_requirements, InputRequirement::Collection);

        // Test unknown function
        let signature = analyzer.get_function_signature("unknownFunction");
        assert!(signature.is_none());
    }

    #[tokio::test]
    async fn test_aggregate_context_validation() {
        let analyzer = create_test_analyzer();

        // Test with singleton input (should fail)
        let singleton_type = create_test_type_info("Patient", true);
        let diagnostic = analyzer.validate_aggregate_context("count", &singleton_type);
        assert!(diagnostic.is_some());
        let diag = diagnostic.unwrap();
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.code.code, "FP0302");

        // Test with collection input (should pass)
        let collection_type = create_test_type_info("Patient", false);
        let diagnostic = analyzer.validate_aggregate_context("count", &collection_type);
        assert!(diagnostic.is_none());
    }

    #[tokio::test]
    async fn test_function_call_validation_unknown_function() {
        let analyzer = create_test_analyzer();
        let input_type = create_test_type_info("Patient", true);
        let location = Some(create_test_location(0, 15));

        let result = analyzer
            .validate_function_call("unknownFunction", &input_type, &[], location)
            .await;

        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert!(!metadata.diagnostics.is_empty());
        assert!(
            metadata
                .diagnostics
                .iter()
                .any(|d| { d.severity == DiagnosticSeverity::Error && d.code.code == "FP0304" })
        );
    }

    #[tokio::test]
    async fn test_function_call_validation_wrong_input_type() {
        let analyzer = create_test_analyzer();
        let singleton_type = create_test_type_info("Patient", true);
        let location = Some(create_test_location(8, 5));

        let result = analyzer
            .validate_function_call("count", &singleton_type, &[], location)
            .await;

        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert!(!metadata.diagnostics.is_empty());
        assert!(
            metadata
                .diagnostics
                .iter()
                .any(|d| { d.severity == DiagnosticSeverity::Error && d.code.code == "FP0302" })
        );
    }

    #[tokio::test]
    async fn test_function_call_validation_correct_context() {
        let analyzer = create_test_analyzer();
        let collection_type = create_test_type_info("Patient", false);
        let location = Some(create_test_location(8, 5));

        let result = analyzer
            .validate_function_call("count", &collection_type, &[], location)
            .await;

        assert!(result.is_ok());
        let metadata = result.unwrap();

        // Should not have any errors for correct usage
        assert!(
            !metadata
                .diagnostics
                .iter()
                .any(|d| d.severity == DiagnosticSeverity::Error)
        );

        // Should have return type
        assert!(metadata.type_info.is_some());
        let return_type = metadata.type_info.unwrap();
        assert_eq!(return_type.type_name, "Number");
    }

    #[test]
    fn test_return_type_resolution() {
        let analyzer = create_test_analyzer();
        let input_type = create_test_type_info("Patient", true);

        // Test same as input
        let result = analyzer.resolve_return_type(&ReturnType::SameAsInput, &input_type);
        assert_eq!(result.type_name, "Patient");

        // Test boolean return
        let result = analyzer.resolve_return_type(&ReturnType::Boolean, &input_type);
        assert_eq!(result.type_name, "Boolean");
        assert_eq!(result.singleton, Some(true));

        // Test collection return
        let result = analyzer.resolve_return_type(
            &ReturnType::CollectionOf(Box::new(ReturnType::SameAsInput)),
            &input_type,
        );
        assert_eq!(result.type_name, "Patient");
        assert_eq!(result.singleton, Some(false));
    }

    #[tokio::test]
    async fn test_argument_type_compatibility() {
        let analyzer = create_test_analyzer();

        // Test any type compatibility
        let string_type = create_test_type_info("String", true);
        assert!(
            analyzer
                .is_argument_type_compatible(&ArgumentType::Any, &string_type)
                .await
        );

        // Test boolean compatibility
        let boolean_type = create_test_type_info("Boolean", true);
        assert!(
            analyzer
                .is_argument_type_compatible(&ArgumentType::Boolean, &boolean_type)
                .await
        );

        // Test numeric compatibility
        let integer_type = create_test_type_info("integer", true);
        assert!(
            analyzer
                .is_argument_type_compatible(&ArgumentType::Numeric, &integer_type)
                .await
        );

        // Test incompatibility
        assert!(
            !analyzer
                .is_argument_type_compatible(&ArgumentType::Boolean, &string_type)
                .await
        );
    }
}
