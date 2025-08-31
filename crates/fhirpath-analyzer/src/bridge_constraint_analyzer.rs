//! Bridge API-enabled constraint analysis with violation detection
//!
//! This module provides FHIRPath constraint validation that leverages the Bridge Support Architecture
//! for comprehensive constraint analysis and violation reporting.

use octofhir_fhirpath_model::provider::ModelProvider;
use octofhir_fhirschema::FhirSchemaPackageManager;
use std::sync::Arc;

use crate::bridge_field_validator::AnalyzerError;

/// Result of constraint validation
#[derive(Debug, Clone)]
pub struct ConstraintValidationResult {
    /// Whether the constraint is valid
    pub is_valid: bool,
    /// The constraint expression that was validated
    pub constraint_expression: String,
    /// Resource type the constraint applies to
    pub resource_type: String,
    /// List of violations found
    pub violations: Vec<ConstraintViolation>,
    /// Suggestions for fixing violations
    pub suggestions: Vec<ConstraintSuggestion>,
    /// Overall confidence in the validation result
    pub confidence: f64,
    /// Performance metrics for the constraint
    pub performance_metrics: ConstraintPerformanceMetrics,
}

/// Types of constraint violations
#[derive(Debug, Clone)]
pub enum ConstraintViolation {
    /// Property referenced in constraint doesn't exist
    PropertyNotFound {
        resource_type: String,
        property: String,
        location: Option<String>,
    },
    /// Function used in constraint is invalid
    InvalidFunction {
        function_name: String,
        reason: String,
        location: Option<String>,
    },
    /// Type mismatch in constraint logic
    TypeMismatch {
        expected: String,
        actual: String,
        location: Option<String>,
    },
    /// Syntax error in constraint expression
    SyntaxError {
        message: String,
        location: Option<String>,
    },
    /// Logical inconsistency in constraint
    LogicalInconsistency {
        description: String,
        location: Option<String>,
    },
    /// Performance warning for complex constraint
    PerformanceWarning {
        description: String,
        estimated_complexity: f64,
    },
}

/// Suggestion for fixing constraint violations
#[derive(Debug, Clone)]
pub struct ConstraintSuggestion {
    /// Description of the suggestion
    pub description: String,
    /// Confidence level of the suggestion
    pub confidence: f64,
    /// Category of the suggestion
    pub category: ConstraintSuggestionCategory,
    /// Original problematic text
    pub original: Option<String>,
    /// Suggested replacement
    pub replacement: Option<String>,
    /// Example of correct usage
    pub example: Option<String>,
}

/// Categories of constraint suggestions
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintSuggestionCategory {
    /// Fix property name
    PropertyFix,
    /// Fix function usage
    FunctionFix,
    /// Fix type conversion
    TypeFix,
    /// Fix syntax error
    SyntaxFix,
    /// Optimize performance
    PerformanceOptimization,
    /// General best practice
    BestPractice,
}

/// Performance metrics for constraints
#[derive(Debug, Clone)]
pub struct ConstraintPerformanceMetrics {
    /// Estimated complexity score (0.0 to 1.0)
    pub complexity_score: f64,
    /// Expected execution time category
    pub execution_time_category: ExecutionTimeCategory,
    /// Memory usage estimate
    pub memory_usage_estimate: MemoryUsageCategory,
    /// Performance recommendations
    pub recommendations: Vec<String>,
}

/// Execution time categories
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionTimeCategory {
    Fast,     // < 1ms
    Medium,   // 1-10ms
    Slow,     // 10-100ms
    VerySlow, // > 100ms
}

/// Memory usage categories
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryUsageCategory {
    Low,      // < 1MB
    Medium,   // 1-10MB
    High,     // 10-100MB
    VeryHigh, // > 100MB
}

/// Abstract constraint node for parsing
#[derive(Debug, Clone)]
pub enum ConstraintNode {
    /// Property access in constraint
    PropertyAccess { resource: String, property: String },
    /// Function call in constraint
    FunctionCall {
        name: String,
        args: Vec<ConstraintNode>,
    },
    /// Binary operation
    BinaryOp {
        left: Box<ConstraintNode>,
        operator: String,
        right: Box<ConstraintNode>,
    },
    /// Literal value
    Literal { value: String, literal_type: String },
}

/// FHIRPath constraint analyzer with bridge support
pub struct ConstraintAnalyzer {
    /// Schema manager for bridge API operations
    schema_manager: Arc<FhirSchemaPackageManager>,
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,
}

impl ConstraintAnalyzer {
    /// Create new constraint analyzer with bridge support
    pub async fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Result<Self, AnalyzerError> {
        let model_provider: Arc<dyn ModelProvider> = Arc::new(
            octofhir_fhirpath_model::FhirSchemaModelProvider::new()
                .await
                .map_err(|e| AnalyzerError::InitializationError {
                    message: format!("Failed to create model provider: {}", e),
                })?,
        );

        Ok(Self {
            schema_manager,
            model_provider,
        })
    }

    /// Validate a FHIRPath constraint
    pub async fn validate_constraint(
        &self,
        constraint_expression: &str,
        resource_type: &str,
    ) -> Result<ConstraintValidationResult, AnalyzerError> {
        // Parse the constraint expression
        let constraint_ast = self.parse_constraint(constraint_expression)?;

        // Analyze the constraint for violations
        let violations = self
            .analyze_constraint_violations(&constraint_ast, resource_type)
            .await?;

        // Generate suggestions for fixing violations
        let suggestions = self
            .generate_constraint_fixes(constraint_expression, &violations)
            .await?;

        // Calculate performance metrics
        let performance_metrics = self
            .calculate_performance_metrics(&constraint_ast, constraint_expression)
            .await?;

        // Determine overall confidence
        let confidence = if violations.is_empty() {
            0.95 - (performance_metrics.complexity_score * 0.1) // Lower confidence for complex constraints
        } else {
            0.5 - (violations.len() as f64 * 0.1) // Lower confidence with more violations
        }
        .max(0.0)
        .min(1.0);

        Ok(ConstraintValidationResult {
            is_valid: violations.is_empty(),
            constraint_expression: constraint_expression.to_string(),
            resource_type: resource_type.to_string(),
            violations,
            suggestions,
            confidence,
            performance_metrics,
        })
    }

    /// Parse constraint expression into AST
    fn parse_constraint(&self, constraint: &str) -> Result<Vec<ConstraintNode>, AnalyzerError> {
        // This is a simplified parser - in a real implementation,
        // we would use the full FHIRPath parser
        let mut nodes = Vec::new();

        // Simple parsing logic for demonstration
        if constraint.contains('.') {
            let parts: Vec<&str> = constraint.split('.').collect();
            if parts.len() >= 2 {
                nodes.push(ConstraintNode::PropertyAccess {
                    resource: parts[0].to_string(),
                    property: parts[1..].join("."),
                });
            }
        }

        // Check for function calls
        if constraint.contains('(') && constraint.contains(')') {
            if let Some(start) = constraint.find('(') {
                let function_name = constraint[..start]
                    .split('.')
                    .last()
                    .unwrap_or(constraint)
                    .to_string();
                nodes.push(ConstraintNode::FunctionCall {
                    name: function_name,
                    args: Vec::new(), // Simplified - would parse arguments
                });
            }
        }

        if nodes.is_empty() {
            // Fallback for simple literals
            nodes.push(ConstraintNode::Literal {
                value: constraint.to_string(),
                literal_type: "string".to_string(),
            });
        }

        Ok(nodes)
    }

    /// Analyze constraint violations
    async fn analyze_constraint_violations(
        &self,
        constraint_ast: &[ConstraintNode],
        _resource_type: &str,
    ) -> Result<Vec<ConstraintViolation>, AnalyzerError> {
        let mut violations = Vec::new();

        for node in constraint_ast {
            match node {
                ConstraintNode::PropertyAccess { resource, property } => {
                    // Check if property exists in the resource type
                    let property_exists = self.check_property_exists(resource, property).await?;

                    if !property_exists {
                        violations.push(ConstraintViolation::PropertyNotFound {
                            resource_type: resource.clone(),
                            property: property.clone(),
                            location: Some(format!("{}.{}", resource, property)),
                        });
                    }
                }
                ConstraintNode::FunctionCall { name, .. } => {
                    // Check if function is valid for constraints
                    if !self.is_valid_constraint_function(name).await? {
                        violations.push(ConstraintViolation::InvalidFunction {
                            function_name: name.clone(),
                            reason: "Function not available in constraint context".to_string(),
                            location: Some(name.clone()),
                        });
                    }
                }
                ConstraintNode::BinaryOp {
                    left,
                    operator,
                    right,
                } => {
                    // Analyze binary operations for type consistency
                    // This would be more sophisticated in a real implementation
                    if operator == "=" && self.detect_type_mismatch(left, right).await? {
                        violations.push(ConstraintViolation::TypeMismatch {
                            expected: "compatible types".to_string(),
                            actual: "incompatible types".to_string(),
                            location: Some(format!(
                                "{} {} {}",
                                self.node_to_string(left),
                                operator,
                                self.node_to_string(right)
                            )),
                        });
                    }
                }
                ConstraintNode::Literal { .. } => {
                    // Literals are generally valid
                }
            }
        }

        Ok(violations)
    }

    /// Check if property exists in resource type
    async fn check_property_exists(
        &self,
        resource_type: &str,
        property: &str,
    ) -> Result<bool, AnalyzerError> {
        if let Some(type_info) = self.model_provider.get_type_reflection(resource_type).await {
            if let octofhir_fhirpath_model::provider::TypeReflectionInfo::ClassInfo {
                elements,
                ..
            } = type_info
            {
                return Ok(elements.iter().any(|element| element.name == property));
            }
        }
        Ok(false)
    }

    /// Check if function is valid in constraint context
    async fn is_valid_constraint_function(
        &self,
        function_name: &str,
    ) -> Result<bool, AnalyzerError> {
        // List of functions commonly allowed in constraints
        let valid_functions = [
            "exists",
            "empty",
            "count",
            "length",
            "first",
            "last",
            "single",
            "where",
            "select",
            "all",
            "any",
            "matches",
            "contains",
            "startsWith",
            "endsWith",
            "toString",
            "toInteger",
        ];

        Ok(valid_functions.contains(&function_name))
    }

    /// Detect type mismatches in binary operations
    async fn detect_type_mismatch(
        &self,
        _left: &ConstraintNode,
        _right: &ConstraintNode,
    ) -> Result<bool, AnalyzerError> {
        // Simplified implementation - would analyze actual types
        Ok(false)
    }

    /// Convert constraint node to string representation
    fn node_to_string(&self, node: &ConstraintNode) -> String {
        match node {
            ConstraintNode::PropertyAccess { resource, property } => {
                format!("{}.{}", resource, property)
            }
            ConstraintNode::FunctionCall { name, .. } => {
                format!("{}()", name)
            }
            ConstraintNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                format!(
                    "{} {} {}",
                    self.node_to_string(left),
                    operator,
                    self.node_to_string(right)
                )
            }
            ConstraintNode::Literal { value, .. } => value.clone(),
        }
    }

    /// Generate suggestions for fixing constraint violations
    async fn generate_constraint_fixes(
        &self,
        _constraint_expression: &str,
        violations: &[ConstraintViolation],
    ) -> Result<Vec<ConstraintSuggestion>, AnalyzerError> {
        let mut suggestions = Vec::new();

        for violation in violations {
            match violation {
                ConstraintViolation::PropertyNotFound {
                    resource_type,
                    property,
                    ..
                } => {
                    // Generate property suggestions
                    if let Some(type_info) =
                        self.model_provider.get_type_reflection(resource_type).await
                    {
                        if let octofhir_fhirpath_model::provider::TypeReflectionInfo::ClassInfo {
                            elements,
                            ..
                        } = type_info
                        {
                            // Find similar property names
                            for element in elements.iter().take(3) {
                                // Top 3 alternatives
                                if self.calculate_similarity(property, &element.name) > 0.6 {
                                    suggestions.push(ConstraintSuggestion {
                                        description: format!("Did you mean '{}'?", element.name),
                                        confidence: 0.8,
                                        category: ConstraintSuggestionCategory::PropertyFix,
                                        original: Some(property.clone()),
                                        replacement: Some(element.name.clone()),
                                        example: Some(format!(
                                            "{}.{}",
                                            resource_type, element.name
                                        )),
                                    });
                                }
                            }
                        }
                    }
                }
                ConstraintViolation::InvalidFunction { function_name, .. } => {
                    // Suggest valid alternatives
                    let valid_alternatives = ["exists", "empty", "count", "where", "all", "any"];
                    for alt in &valid_alternatives {
                        if self.calculate_similarity(function_name, alt) > 0.6 {
                            suggestions.push(ConstraintSuggestion {
                                description: format!("Consider using '{}()' instead", alt),
                                confidence: 0.7,
                                category: ConstraintSuggestionCategory::FunctionFix,
                                original: Some(function_name.clone()),
                                replacement: Some(alt.to_string()),
                                example: Some(format!("field.{}()", alt)),
                            });
                        }
                    }
                }
                ConstraintViolation::TypeMismatch { .. } => {
                    suggestions.push(ConstraintSuggestion {
                        description: "Check that compared values have compatible types".to_string(),
                        confidence: 0.6,
                        category: ConstraintSuggestionCategory::TypeFix,
                        original: None,
                        replacement: None,
                        example: Some("field.toString() = 'value' or field.exists()".to_string()),
                    });
                }
                ConstraintViolation::SyntaxError { message, .. } => {
                    suggestions.push(ConstraintSuggestion {
                        description: format!("Fix syntax error: {}", message),
                        confidence: 0.9,
                        category: ConstraintSuggestionCategory::SyntaxFix,
                        original: None,
                        replacement: None,
                        example: Some("Check parentheses, quotes, and operators".to_string()),
                    });
                }
                ConstraintViolation::PerformanceWarning { description, .. } => {
                    suggestions.push(ConstraintSuggestion {
                        description: format!("Performance optimization: {}", description),
                        confidence: 0.7,
                        category: ConstraintSuggestionCategory::PerformanceOptimization,
                        original: None,
                        replacement: None,
                        example: Some(
                            "Consider adding .exists() checks before complex operations"
                                .to_string(),
                        ),
                    });
                }
                _ => {} // Other violation types
            }
        }

        Ok(suggestions)
    }

    /// Calculate performance metrics for a constraint
    async fn calculate_performance_metrics(
        &self,
        constraint_ast: &[ConstraintNode],
        constraint_expression: &str,
    ) -> Result<ConstraintPerformanceMetrics, AnalyzerError> {
        let mut complexity_score = 0.0;
        let mut recommendations = Vec::new();

        // Analyze AST for complexity
        for node in constraint_ast {
            complexity_score += self.calculate_node_complexity(node);
        }

        // Factor in expression length and structure
        complexity_score += (constraint_expression.len() as f64 / 100.0).min(0.3);

        if constraint_expression.matches('.').count() > 3 {
            complexity_score += 0.2;
            recommendations.push("Consider breaking complex navigation paths".to_string());
        }

        if constraint_expression.contains("where(") {
            complexity_score += 0.3;
            recommendations
                .push("where() clauses can impact performance on large datasets".to_string());
        }

        // Normalize complexity score
        complexity_score = complexity_score.min(1.0);

        // Determine execution time category
        let execution_time_category = match complexity_score {
            s if s < 0.3 => ExecutionTimeCategory::Fast,
            s if s < 0.6 => ExecutionTimeCategory::Medium,
            s if s < 0.8 => ExecutionTimeCategory::Slow,
            _ => ExecutionTimeCategory::VerySlow,
        };

        // Determine memory usage category
        let memory_usage_category = match complexity_score {
            s if s < 0.4 => MemoryUsageCategory::Low,
            s if s < 0.7 => MemoryUsageCategory::Medium,
            s if s < 0.9 => MemoryUsageCategory::High,
            _ => MemoryUsageCategory::VeryHigh,
        };

        Ok(ConstraintPerformanceMetrics {
            complexity_score,
            execution_time_category,
            memory_usage_estimate: memory_usage_category,
            recommendations,
        })
    }

    /// Calculate complexity for individual nodes
    fn calculate_node_complexity(&self, node: &ConstraintNode) -> f64 {
        match node {
            ConstraintNode::PropertyAccess { .. } => 0.1,
            ConstraintNode::FunctionCall { name, args } => {
                let base_complexity = match name.as_str() {
                    "exists" | "empty" => 0.1,
                    "count" | "length" => 0.2,
                    "where" | "select" => 0.4,
                    _ => 0.2,
                };
                base_complexity + (args.len() as f64 * 0.1)
            }
            ConstraintNode::BinaryOp { left, right, .. } => {
                0.1 + self.calculate_node_complexity(left) + self.calculate_node_complexity(right)
            }
            ConstraintNode::Literal { .. } => 0.05,
        }
    }

    /// Calculate similarity between two strings
    fn calculate_similarity(&self, a: &str, b: &str) -> f64 {
        // Simple Levenshtein-based similarity
        if a == b {
            return 1.0;
        }

        let max_len = a.len().max(b.len());
        if max_len == 0 {
            return 1.0;
        }

        let distance = self.levenshtein_distance(a, b);
        1.0 - (distance as f64 / max_len as f64)
    }

    /// Calculate Levenshtein distance
    fn levenshtein_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let len_a = a_chars.len();
        let len_b = b_chars.len();

        if len_a == 0 {
            return len_b;
        }
        if len_b == 0 {
            return len_a;
        }

        let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

        for i in 0..=len_a {
            matrix[i][0] = i;
        }
        for j in 0..=len_b {
            matrix[0][j] = j;
        }

        for i in 1..=len_a {
            for j in 1..=len_b {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len_a][len_b]
    }

    /// Get schema manager reference
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_canonical_manager::FcmConfig;
    use octofhir_fhirschema::PackageManagerConfig;

    async fn create_test_analyzer() -> Result<ConstraintAnalyzer, AnalyzerError> {
        let fcm_config = FcmConfig::default();
        let config = PackageManagerConfig::default();
        let schema_manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .map_err(|e| AnalyzerError::InitializationError {
                    message: format!("Failed to create schema manager: {}", e),
                })?,
        );

        ConstraintAnalyzer::new(schema_manager).await
    }

    #[tokio::test]
    async fn test_valid_constraint() -> Result<(), Box<dyn std::error::Error>> {
        let analyzer = create_test_analyzer().await?;

        let result = analyzer
            .validate_constraint("Patient.name.exists()", "Patient")
            .await?;

        assert!(result.is_valid);
        assert!(result.violations.is_empty());
        assert!(result.confidence > 0.5);

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_property() -> Result<(), Box<dyn std::error::Error>> {
        let analyzer = create_test_analyzer().await?;

        let result = analyzer
            .validate_constraint("Patient.invalidProperty.exists()", "Patient")
            .await?;

        assert!(!result.is_valid);
        assert!(!result.violations.is_empty());
        assert!(!result.suggestions.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_performance_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let analyzer = create_test_analyzer().await?;

        // Simple constraint should have low complexity
        let result1 = analyzer
            .validate_constraint("Patient.id.exists()", "Patient")
            .await?;

        assert!(result1.performance_metrics.complexity_score < 0.5);

        // Complex constraint should have higher complexity
        let result2 = analyzer
            .validate_constraint(
                "Patient.name.where(use='official').family.exists()",
                "Patient",
            )
            .await?;

        assert!(
            result2.performance_metrics.complexity_score
                > result1.performance_metrics.complexity_score
        );

        Ok(())
    }
}
