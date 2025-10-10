//! Semantic analyzer for FHIRPath expressions
//!
//! This module provides semantic analysis capabilities that can enhance AST nodes
//! with type information, path tracking, and validation. It's designed to be used
//! optionally during analysis parsing mode without affecting fast parsing performance.

use std::collections::HashSet;
use std::sync::Arc;

use crate::ast::{
    AnalysisMetadata, BinaryOperationNode, BinaryOperator, ExpressionAnalysis, ExpressionNode,
    FunctionCallNode, IdentifierNode, LiteralNode, LiteralValue, MethodCallNode,
    PropertyAccessNode,
};
use crate::core::{FhirPathError, SourceLocation};
use crate::diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
use octofhir_fhir_model::{ModelProvider, TypeInfo};

/// Semantic analyzer for FHIRPath expressions
pub struct SemanticAnalyzer {
    /// Model provider for type resolution
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
    /// Current input type context
    input_type: Option<TypeInfo>,
    /// Whether we're at the head of a navigation chain
    is_chain_head: bool,
    /// Expression text for span calculation
    expression_text: Option<String>,
    /// Stack of variable scopes for defineVariable() analysis
    var_scopes: Vec<HashSet<String>>,
    /// Reserved/system variable names that cannot be user-defined
    reserved_vars: HashSet<String>,
}

impl SemanticAnalyzer {
    /// Create new semantic analyzer
    pub fn new(model_provider: Arc<dyn ModelProvider + Send + Sync>) -> Self {
        // Initialize reserved/system variables that cannot be user-defined
        let reserved: HashSet<String> = [
            "this",
            "$this",
            "index",
            "$index",
            "total",
            "$total",
            "context",
            "%context",
            "resource",
            "%resource",
            "terminologies",
            "sct",
            "loinc",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        // Patterns like vs-*, ext-* are handled separately
        let scopes = vec![HashSet::new()];

        Self {
            model_provider,
            input_type: None,
            is_chain_head: true,
            expression_text: None,
            var_scopes: scopes,
            reserved_vars: reserved,
        }
    }

    /// Analyze an expression and return analysis metadata
    pub async fn analyze_expression(
        &mut self,
        expr: &ExpressionNode,
        context_type: Option<TypeInfo>,
    ) -> Result<ExpressionAnalysis, FhirPathError> {
        // Initialize context if provided
        self.input_type = context_type;
        self.is_chain_head = true;
        // Reset variable scopes for a new analysis
        self.var_scopes.clear();
        self.var_scopes.push(HashSet::new());

        let mut analysis = ExpressionAnalysis::success(None);

        // Perform semantic analysis on the expression
        if let Err(err) = self.analyze_node(expr, &mut analysis).await {
            analysis.success = false;
            analysis.add_diagnostic(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode {
                    code: "ANALYSIS_ERROR".to_string(),
                    namespace: Some("fhirpath".to_string()),
                },
                message: err.to_string(),
                location: expr.location().cloned(),
                related: vec![],
            });
        }

        Ok(analysis)
    }

    /// Analyze an expression with text for span calculation
    pub async fn analyze_expression_with_text(
        &mut self,
        expr: &ExpressionNode,
        context_type: Option<TypeInfo>,
        expression_text: &str,
    ) -> Result<ExpressionAnalysis, FhirPathError> {
        // Store the expression text for span calculation
        self.expression_text = Some(expression_text.to_string());

        // Analyze the expression normally
        let mut result = self.analyze_expression(expr, context_type).await?;

        // Post-process diagnostics to calculate accurate spans
        for diagnostic in &mut result.diagnostics {
            if diagnostic.location.is_none() {
                // Try to calculate span from the diagnostic message
                diagnostic.location =
                    self.calculate_location_from_message(&diagnostic.message, expression_text);
            }
        }

        Ok(result)
    }

    /// Calculate source location from diagnostic message
    fn calculate_location_from_message(
        &self,
        message: &str,
        expression_text: &str,
    ) -> Option<SourceLocation> {
        // Try to extract identifiers or function names from the diagnostic message
        // and find their position in the expression

        // Look for property names in quotes
        if let Some(property_start) = message.find("'")
            && let Some(property_end) = message[property_start + 1..].find("'")
        {
            let property_name = &message[property_start + 1..property_start + 1 + property_end];
            if let Some(pos) = expression_text.find(property_name) {
                return Some(SourceLocation {
                    offset: pos,
                    length: property_name.len(),
                    line: 1,
                    column: pos + 1,
                });
            }
        }

        // Look for function names in the message (not hardcoded resource types)
        let function_names = [
            "resourceType",
            "ofType",
            "is",
            "as",
            "where",
            "first",
            "last",
        ];
        for word in &function_names {
            if message.contains(word)
                && let Some(pos) = expression_text.find(word)
            {
                return Some(SourceLocation {
                    offset: pos,
                    length: word.len(),
                    line: 1,
                    column: pos + 1,
                });
            }
        }

        None
    }

    /// Check if a type is a Reference type
    fn is_reference_type(&self, type_info: &TypeInfo) -> bool {
        type_info.type_name == "Reference"
            || type_info.type_name.ends_with("Reference")
            || type_info.type_name.contains("Reference[")
    }

    /// Analyze a single AST node
    fn analyze_node<'a>(
        &'a mut self,
        node: &'a ExpressionNode,
        analysis: &'a mut ExpressionAnalysis,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<AnalysisMetadata, FhirPathError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let result = match node {
                ExpressionNode::Union(union_node) => {
                    // Isolate variable scopes across union branches
                    let snapshot_before = self.var_scopes.clone();
                    // Left
                    self.var_scopes = snapshot_before.clone();
                    let _ = self.analyze_node(&union_node.left, analysis).await?;
                    // Right
                    self.var_scopes = snapshot_before.clone();
                    let _ = self.analyze_node(&union_node.right, analysis).await?;
                    // Restore
                    self.var_scopes = snapshot_before;
                    Ok(AnalysisMetadata::new())
                }
                ExpressionNode::Variable(var) => {
                    // Analyze variable reference: treat non-system names as user or environment variables
                    let var_name = var.name.as_str();
                    // Normalize: strip leading '%' if present
                    let base = if let Some(stripped) = var_name.strip_prefix('%') {
                        stripped
                    } else {
                        var_name
                    };

                    // Recognize system variables that are always valid
                    let is_system = matches!(base, "this" | "index" | "total");
                    if is_system {
                        return Ok(AnalysisMetadata::new());
                    }

                    // Known environment variables and patterns
                    let is_env = base == "sct"
                        || base == "loinc"
                        || base == "terminologies"
                        || base == "context"
                        || base == "resource"
                        || base.starts_with("vs-")
                        || base.starts_with("ext-");

                    if !is_env {
                        // Check user-defined variables in current scopes
                        let found = self
                            .var_scopes
                            .iter()
                            .rev()
                            .any(|scope| scope.contains(base));
                        if !found {
                            analysis.success = false;
                            analysis.add_diagnostic(Diagnostic {
                                severity: DiagnosticSeverity::Error,
                                code: DiagnosticCode {
                                    code: "UNDEFINED_VARIABLE".to_string(),
                                    namespace: Some("fhirpath".to_string()),
                                },
                                message: format!("Undefined variable: %{}", base),
                                location: var.location.clone(),
                                related: vec![],
                            });
                        }
                    }

                    Ok(AnalysisMetadata::new())
                }
                ExpressionNode::Identifier(identifier) => {
                    self.analyze_identifier(identifier, analysis).await
                }
                ExpressionNode::PropertyAccess(prop) => {
                    self.analyze_property_access(prop, analysis).await
                }
                ExpressionNode::Literal(literal) => Ok(self.analyze_literal(literal)),
                ExpressionNode::BinaryOperation(binary_op) => {
                    self.analyze_binary_operation(binary_op, analysis).await
                }
                ExpressionNode::FunctionCall(func_call) => {
                    self.analyze_function_call(func_call, analysis).await
                }
                ExpressionNode::MethodCall(method_call) => {
                    self.analyze_method_call(method_call, analysis).await
                }
                _ => {
                    // For now, return empty metadata for other node types
                    Ok(AnalysisMetadata::new())
                }
            };

            // After analyzing a node, we're no longer at the chain head
            self.is_chain_head = false;
            result
        })
    }

    /// Analyze identifier node
    async fn analyze_identifier(
        &mut self,
        identifier: &IdentifierNode,
        analysis: &mut ExpressionAnalysis,
    ) -> Result<AnalysisMetadata, FhirPathError> {
        let mut metadata = AnalysisMetadata::new();
        let name = &identifier.name;

        // Try to use model provider for accurate type information
        if let Some(ref input_type) = self.input_type {
            // Special case: resourceType is always valid on Reference types
            if name == "resourceType" && self.is_reference_type(input_type) {
                metadata.type_info = Some(TypeInfo {
                    type_name: "String".to_string(),
                    singleton: Some(true),
                    is_empty: Some(false),
                    namespace: Some("System".to_string()),
                    name: Some("String".to_string()),
                });
                self.input_type = metadata.type_info.clone();
                self.is_chain_head = false;
                return Ok(metadata);
            }

            // First try to navigate from input type (property access)
            if let Ok(Some(element_type)) =
                self.model_provider.get_element_type(input_type, name).await
            {
                metadata.type_info = Some(element_type);
                self.input_type = metadata.type_info.clone();
                self.is_chain_head = false;
                return Ok(metadata);
            }

            // If property not found and we have a concrete type, this is a semantic error
            // Use PropertyAnalyzer to get suggestions
            let property_provider = self.model_provider.clone();
            let property_analyzer = crate::analyzer::PropertyAnalyzer::new(property_provider);
            let suggestions = property_analyzer.suggest_properties(input_type, name).await;

            let type_display = if input_type.singleton.unwrap_or(true) {
                input_type.type_name.clone()
            } else {
                format!("{}[]", input_type.type_name)
            };

            let message = format!("prop '{name}' not found on {type_display}");
            let _help_text = if !suggestions.is_empty() {
                Some(format!("Did you mean '{}'?", suggestions[0].property_name))
            } else {
                None
            };

            let diagnostic = Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode {
                    code: "PROPERTY_NOT_FOUND".to_string(),
                    namespace: Some("fhirpath".to_string()),
                },
                message,
                location: identifier.location.clone(),
                related: vec![],
            };
            metadata.add_diagnostic(diagnostic.clone());
            analysis.add_diagnostic(diagnostic);

            // Return empty collection type but mark analysis as failed
            metadata.type_info = Some(TypeInfo {
                type_name: "Any".to_string(),
                singleton: Some(false),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Any".to_string()),
            });
            return Ok(metadata);
        }

        // Chain-head rule: at the head of a navigation chain, allow treating the
        // identifier as a known type to seed the chain (e.g., Patient.name)
        if self.is_chain_head
            && let Ok(Some(type_info)) = self.model_provider.get_type(name).await
        {
            metadata.type_info = Some(type_info.clone());
            self.input_type = Some(type_info);
            self.is_chain_head = false;
            return Ok(metadata);
        }

        // Without a model provider or context, we can't know the type
        // Return Any type - don't make assumptions
        metadata.type_info = Some(TypeInfo {
            type_name: "Any".to_string(),
            singleton: Some(false),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Any".to_string()),
        });
        Ok(metadata)
    }

    /// Analyze property access node
    async fn analyze_property_access(
        &mut self,
        prop: &PropertyAccessNode,
        analysis: &mut ExpressionAnalysis,
    ) -> Result<AnalysisMetadata, FhirPathError> {
        // First analyze the object to establish context
        let _object_metadata = self.analyze_node(&prop.object, analysis).await?;

        // Mark that we're no longer at chain head
        self.is_chain_head = false;

        // Then analyze the property access
        let identifier = IdentifierNode {
            name: prop.property.clone(),
            location: prop.location.clone(),
        };

        self.analyze_identifier(&identifier, analysis).await
    }

    /// Analyze literal node (always valid)
    fn analyze_literal(&self, _literal: &crate::ast::LiteralNode) -> AnalysisMetadata {
        let mut metadata = AnalysisMetadata::new();

        // TODO: Infer type from literal value
        metadata.type_info = Some(TypeInfo {
            type_name: "Any".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Any".to_string()),
        });

        metadata
    }

    /// Analyze binary operation node for semantic errors
    async fn analyze_binary_operation(
        &mut self,
        binary_op: &BinaryOperationNode,
        analysis: &mut ExpressionAnalysis,
    ) -> Result<AnalysisMetadata, FhirPathError> {
        // Special scoping rules for union (|): variables defined on one branch must not leak
        // into the other branch, nor to the outer scope past the union.
        if binary_op.operator == BinaryOperator::Union {
            // Snapshot current variable scopes
            let snapshot_before = self.var_scopes.clone();

            // Analyze left with isolated scopes based on snapshot_before
            self.var_scopes = snapshot_before.clone();
            let _left_metadata = self.analyze_node(&binary_op.left, analysis).await?;

            // Analyze right with isolated scopes based on snapshot_before (not left's)
            self.var_scopes = snapshot_before.clone();
            let _right_metadata = self.analyze_node(&binary_op.right, analysis).await?;

            // Restore original scopes so nothing from either branch escapes
            self.var_scopes = snapshot_before;
        } else {
            // Default behavior: analyze both operands in current scope
            let _left_metadata = self.analyze_node(&binary_op.left, analysis).await?;
            let _right_metadata = self.analyze_node(&binary_op.right, analysis).await?;
        }

        // Check for semantic errors in addition operations
        if binary_op.operator == BinaryOperator::Add {
            // Check if left is a date/datetime/time literal and right is a plain number
            if let (ExpressionNode::Literal(left_literal), ExpressionNode::Literal(right_literal)) =
                (&*binary_op.left, &*binary_op.right)
            {
                // Check if left is a temporal literal and right is a number
                if self.is_temporal_literal(left_literal) && self.is_number_literal(right_literal) {
                    // Extract the actual number value for a better error message
                    let number_str = self.get_number_string(right_literal);
                    let suggestion = format!("+ {number_str} 'days'");

                    // This is a semantic error: date + plain number is not allowed
                    analysis.add_diagnostic(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "FP0082".to_string(),
                            namespace: None,
                        },
                        message: format!("Cannot add a plain number to a date/time value. Use a quantity with units instead (e.g., {suggestion})"),
                        location: binary_op.location.clone(),
                        related: Vec::new(),
                    });
                    analysis.success = false;
                }
            }
        }

        Ok(AnalysisMetadata::new())
    }

    /// Check if a literal is a temporal type (date, datetime, time)
    fn is_temporal_literal(&self, literal: &LiteralNode) -> bool {
        use crate::ast::LiteralValue;
        matches!(
            literal.value,
            LiteralValue::Date(_) | LiteralValue::DateTime(_) | LiteralValue::Time(_)
        )
    }

    /// Check if a literal is a number (integer or decimal)
    fn is_number_literal(&self, literal: &LiteralNode) -> bool {
        use crate::ast::LiteralValue;
        matches!(
            literal.value,
            LiteralValue::Integer(_) | LiteralValue::Decimal(_)
        )
    }

    /// Get the string representation of a number literal
    fn get_number_string(&self, literal: &LiteralNode) -> String {
        use crate::ast::LiteralValue;
        match &literal.value {
            LiteralValue::Integer(i) => i.to_string(),
            LiteralValue::Decimal(d) => d.to_string(),
            _ => "number".to_string(), // fallback
        }
    }

    /// Suggest property name fixes for typos
    #[allow(dead_code)]
    async fn suggest_property_fixes(
        &self,
        property_name: &str,
        context_type: &TypeInfo,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Get available properties for the current type
        let properties = self.model_provider.get_element_names(context_type);
        // Simple distance-based suggestions (could be enhanced with better algorithms)
        for prop in properties.iter().take(5) {
            if self.string_distance(property_name, prop) <= 2 {
                suggestions.push(prop.clone());
            }
        }

        suggestions
    }

    /// Calculate simple edit distance between strings
    #[allow(dead_code)]
    fn string_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();

        if a_chars.is_empty() {
            return b_chars.len();
        }
        if b_chars.is_empty() {
            return a_chars.len();
        }

        let mut matrix = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];

        #[allow(clippy::needless_range_loop)]
        for i in 0..=a_chars.len() {
            matrix[i][0] = i;
        }
        for j in 0..=b_chars.len() {
            matrix[0][j] = j;
        }

        for i in 1..=a_chars.len() {
            for j in 1..=b_chars.len() {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }

        matrix[a_chars.len()][b_chars.len()]
    }

    /// Analyze function call node
    async fn analyze_function_call(
        &mut self,
        func_call: &FunctionCallNode,
        analysis: &mut ExpressionAnalysis,
    ) -> Result<AnalysisMetadata, FhirPathError> {
        // Check for functions that require an input context but are called without one
        let functions_requiring_context = [
            "length",
            "empty",
            "count",
            "first",
            "last",
            "tail",
            "skip",
            "take",
            "exists",
            "all",
            "any",
            "allTrue",
            "anyTrue",
            "distinct",
            "children",
            "descendants",
            "where",
            "select",
            "single",
            "hasValue",
        ];

        if functions_requiring_context.contains(&func_call.name.as_str()) && self.is_chain_head {
            // This function requires an input context but is being called at the start of a chain
            analysis.success = false;
            analysis.add_diagnostic(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode {
                    code: "CONTEXT_REQUIRED".to_string(),
                    namespace: Some("fhirpath".to_string()),
                },
                message: format!("{} function requires an input context", func_call.name),
                location: func_call.location.clone(),
                related: vec![],
            });
        }

        // Check for specific function type validation
        if func_call.name == "iif" && !func_call.arguments.is_empty() {
            // Analyze the first argument (condition) - should be boolean
            let condition_expr = &func_call.arguments[0];
            let _metadata = self.analyze_node(condition_expr, analysis).await?;

            // Check if it's a literal non-boolean value
            if let ExpressionNode::Literal(literal) = condition_expr
                && !matches!(literal.value, LiteralValue::Boolean(_))
            {
                analysis.success = false;
                analysis.add_diagnostic(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "TYPE_MISMATCH".to_string(),
                        namespace: Some("fhirpath".to_string()),
                    },
                    message: "iif function condition must be boolean".to_string(),
                    location: condition_expr.location().cloned(),
                    related: vec![],
                });
            }

            // Check if condition involves union operations (creates collections)
            if self.contains_union_operation(condition_expr) {
                analysis.success = false;
                analysis.add_diagnostic(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "COLLECTION_IN_BOOLEAN_CONTEXT".to_string(),
                        namespace: Some("fhirpath".to_string()),
                    },
                    message: "iif function condition cannot be a collection, must be a single boolean value".to_string(),
                    location: condition_expr.location().cloned(),
                    related: vec![],
                });
            }
        }

        // Analyze function arguments recursively with isolated parameter scopes
        for arg in func_call.arguments.iter() {
            let prev_chain_head = self.is_chain_head;
            self.is_chain_head = true; // Arguments start their own chains
            // Snapshot current scopes and analyze argument
            let snapshot = self.var_scopes.clone();
            let _arg_metadata = self.analyze_node(arg, analysis).await?;
            // Restore scopes so parameters don't collide
            self.var_scopes = snapshot;
            self.is_chain_head = prev_chain_head;
        }

        // defineVariable-specific semantic checks: add variable to current scope when name is literal
        if func_call.name == "defineVariable"
            && let Some(first_arg) = func_call.arguments.first()
            && let ExpressionNode::Literal(lit) = first_arg
            && let LiteralValue::String(var_name) = &lit.value
        {
            let base = var_name.as_str();
            // Reserved names check
            let is_reserved = self.reserved_vars.contains(base)
                || base.starts_with("vs-")
                || base.starts_with("ext-");
            if is_reserved {
                analysis.success = false;
                analysis.add_diagnostic(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "RESERVED_VARIABLE".to_string(),
                        namespace: Some("fhirpath".to_string()),
                    },
                    message: format!("Variable name '{base}' is reserved and cannot be redefined"),
                    location: first_arg.location().cloned(),
                    related: vec![],
                });
            } else {
                // Duplicate in current scope?
                if let Some(current) = self.var_scopes.last()
                    && current.contains(base)
                {
                    analysis.success = false;
                    analysis.add_diagnostic(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "VARIABLE_REDEFINITION".to_string(),
                            namespace: Some("fhirpath".to_string()),
                        },
                        message: format!("Variable '{base}' is already defined in this scope"),
                        location: first_arg.location().cloned(),
                        related: vec![],
                    });
                }
                // Insert into current scope for subsequent chain
                if let Some(current) = self.var_scopes.last_mut() {
                    current.insert(base.to_string());
                }
            }
        }

        Ok(AnalysisMetadata::new())
    }

    /// Analyze method call node
    async fn analyze_method_call(
        &mut self,
        method_call: &MethodCallNode,
        analysis: &mut ExpressionAnalysis,
    ) -> Result<AnalysisMetadata, FhirPathError> {
        // Analyze the object first - this provides the context for the method
        let prev_chain_head = self.is_chain_head;
        let _object_metadata = self.analyze_node(&method_call.object, analysis).await?;

        // The method call is not at the chain head since it has an object
        self.is_chain_head = false;

        // Check for specific method type validation
        if method_call.method == "iif" && !method_call.arguments.is_empty() {
            // Analyze the first argument (condition) - should be boolean
            let condition_expr = &method_call.arguments[0];

            // Arguments start their own chains
            self.is_chain_head = true;
            let _metadata = self.analyze_node(condition_expr, analysis).await?;

            // Check if it's a literal non-boolean value
            if let ExpressionNode::Literal(literal) = condition_expr
                && !matches!(literal.value, LiteralValue::Boolean(_))
            {
                analysis.success = false;
                analysis.add_diagnostic(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "TYPE_MISMATCH".to_string(),
                        namespace: Some("fhirpath".to_string()),
                    },
                    message: "iif function condition must be boolean".to_string(),
                    location: condition_expr.location().cloned(),
                    related: vec![],
                });
            }

            // Check if condition involves union operations (creates collections)
            if self.contains_union_operation(condition_expr) {
                analysis.success = false;
                analysis.add_diagnostic(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "COLLECTION_IN_BOOLEAN_CONTEXT".to_string(),
                        namespace: Some("fhirpath".to_string()),
                    },
                    message: "iif function condition cannot be a collection, must be a single boolean value".to_string(),
                    location: condition_expr.location().cloned(),
                    related: vec![],
                });
            }
        }

        // Analyze any other method arguments with isolated scopes
        for arg in &method_call.arguments {
            self.is_chain_head = true; // Arguments start their own chains
            let snapshot = self.var_scopes.clone();
            let _arg_metadata = self.analyze_node(arg, analysis).await?;
            self.var_scopes = snapshot; // Restore to prevent leakage out of argument bodies
        }

        // defineVariable-specific handling for method calls
        if method_call.method == "defineVariable"
            && let Some(first_arg) = method_call.arguments.first()
            && let ExpressionNode::Literal(lit) = first_arg
            && let LiteralValue::String(var_name) = &lit.value
        {
            let base = var_name.as_str();
            let is_reserved = self.reserved_vars.contains(base)
                || base.starts_with("vs-")
                || base.starts_with("ext-");
            if is_reserved {
                analysis.success = false;
                analysis.add_diagnostic(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "RESERVED_VARIABLE".to_string(),
                        namespace: Some("fhirpath".to_string()),
                    },
                    message: format!("Variable name '{base}' is reserved and cannot be redefined"),
                    location: first_arg.location().cloned(),
                    related: vec![],
                });
            } else {
                if let Some(current) = self.var_scopes.last()
                    && current.contains(base)
                {
                    analysis.success = false;
                    analysis.add_diagnostic(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "VARIABLE_REDEFINITION".to_string(),
                            namespace: Some("fhirpath".to_string()),
                        },
                        message: format!("Variable '{base}' is already defined in this scope"),
                        location: first_arg.location().cloned(),
                        related: vec![],
                    });
                }
                if let Some(current) = self.var_scopes.last_mut() {
                    current.insert(base.to_string());
                }
            }
        }

        self.is_chain_head = prev_chain_head;
        Ok(AnalysisMetadata::new())
    }

    /// Reset analyzer state
    pub fn reset(&mut self) {
        self.input_type = None;
        self.is_chain_head = true;
    }

    /// Check if an expression contains union operations (creates collections)
    #[allow(clippy::only_used_in_recursion)]
    fn contains_union_operation(&self, expr: &ExpressionNode) -> bool {
        match expr {
            ExpressionNode::Union(_) => {
                // This is a union operation - direct detection
                true
            }
            ExpressionNode::BinaryOperation(binary_op) => {
                // Recursively check both operands
                self.contains_union_operation(&binary_op.left)
                    || self.contains_union_operation(&binary_op.right)
            }
            ExpressionNode::FunctionCall(func_call) => {
                // Check function arguments
                func_call
                    .arguments
                    .iter()
                    .any(|arg| self.contains_union_operation(arg))
            }
            ExpressionNode::MethodCall(method_call) => {
                // Check object and method arguments
                self.contains_union_operation(&method_call.object)
                    || method_call
                        .arguments
                        .iter()
                        .any(|arg| self.contains_union_operation(arg))
            }
            ExpressionNode::PropertyAccess(prop_access) => {
                // Check the object being accessed
                self.contains_union_operation(&prop_access.object)
            }
            // Literals and identifiers don't contain unions
            ExpressionNode::Literal(_) | ExpressionNode::Identifier(_) => false,
            // For other node types, assume no union for now
            _ => false,
        }
    }
}

/// Enhanced parsing result that includes semantic analysis
#[derive(Debug, Clone)]
pub struct AnalyzedParseResult {
    /// The parsed AST
    pub ast: Option<ExpressionNode>,
    /// Semantic analysis results
    pub analysis: ExpressionAnalysis,
    /// Whether parsing succeeded
    pub success: bool,
}

impl AnalyzedParseResult {
    /// Create successful result
    pub fn success(ast: ExpressionNode, analysis: ExpressionAnalysis) -> Self {
        Self {
            ast: Some(ast),
            analysis,
            success: true,
        }
    }

    /// Create failed result
    pub fn failure(analysis: ExpressionAnalysis) -> Self {
        Self {
            ast: None,
            analysis,
            success: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ExpressionNode;
    use octofhir_fhir_model::EmptyModelProvider;

    #[tokio::test]
    async fn test_semantic_analyzer_creation() {
        let model_provider = Arc::new(EmptyModelProvider);
        let analyzer = SemanticAnalyzer::new(model_provider);
        assert!(analyzer.input_type.is_none());
        assert!(analyzer.is_chain_head);
    }

    #[tokio::test]
    async fn test_resource_type_analysis() {
        let model_provider = Arc::new(EmptyModelProvider);
        let mut analyzer = SemanticAnalyzer::new(model_provider);

        let identifier = ExpressionNode::identifier("Patient");
        let result = analyzer
            .analyze_expression(&identifier, None)
            .await
            .unwrap();

        // Should succeed for known resource type
        assert!(result.success);
    }

    #[test]
    fn test_string_distance() {
        let model_provider = Arc::new(EmptyModelProvider);
        let analyzer = SemanticAnalyzer::new(model_provider);

        assert_eq!(analyzer.string_distance("test", "test"), 0);
        assert_eq!(analyzer.string_distance("test", "best"), 1);
        assert_eq!(analyzer.string_distance("kitten", "sitting"), 3);
    }
}
