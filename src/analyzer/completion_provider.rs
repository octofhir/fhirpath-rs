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


//! Completion provider for FHIRPath expressions
//!
//! This module provides intelligent code completion including property suggestions,
//! function completions, type hints, and context-aware snippets.

use crate::ast::ExpressionNode;
use crate::model::provider::ModelProvider;
use crate::analyzer::{AnalysisContext, AnalysisError};
// Removed Span import to avoid lifetime issues
use crate::registry::{FunctionRegistry};
use crate::registry::function::FunctionImpl;
use std::sync::Arc;
use std::collections::HashMap;

/// A completion suggestion
#[derive(Debug, Clone, PartialEq)]
pub struct Completion {
    /// The completion label (what the user sees)
    pub label: String,
    /// The text to insert when completion is accepted
    pub insert_text: String,
    /// The kind of completion
    pub kind: CompletionKind,
    /// Detailed description
    pub detail: Option<String>,
    /// Documentation for the completion
    pub documentation: Option<String>,
    /// Sort priority (lower = higher priority)
    pub sort_text: String,
    /// Whether this completion is deprecated
    pub deprecated: bool,
    /// Additional data for LSP integration
    pub data: Option<serde_json::Value>,
}

/// Kind of completion item
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionKind {
    /// Property or field
    Property,
    /// Method or function
    Method,
    /// Function
    Function,
    /// Variable
    Variable,
    /// Type or class
    Type,
    /// Keyword
    Keyword,
    /// Snippet template
    Snippet,
    /// Constant value
    Constant,
    /// Operator
    Operator,
}

/// Context for completion generation
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Position in the source where completion is requested
    pub position: u32,
    /// The partial text at cursor position
    pub trigger_text: Option<String>,
    /// Whether completion was triggered by a character (like '.')
    pub trigger_character: Option<char>,
    /// Current expression context
    pub expression_context: Option<ExpressionContext>,
}

/// Context about the expression being completed
#[derive(Debug, Clone)]
pub struct ExpressionContext {
    /// The base type for property completions
    pub base_type: Option<String>,
    /// Whether we're in a collection context
    pub is_collection: bool,
    /// Whether we're inside a function call
    pub in_function_call: Option<String>,
    /// Whether we're inside a method call
    pub in_method_call: Option<String>,
    /// Parameter index if in function/method call
    pub parameter_index: Option<usize>,
}

/// Completion provider for FHIRPath expressions
pub struct CompletionProvider<P: ModelProvider> {
    provider: Arc<P>,
    function_registry: Arc<FunctionRegistry>,
    builtin_functions: HashMap<String, FunctionCompletion>,
    fhir_functions: HashMap<String, FunctionCompletion>,
    keywords: Vec<KeywordCompletion>,
    snippets: Vec<SnippetCompletion>,
}

/// Information about a function for completion
#[derive(Debug, Clone)]
struct FunctionCompletion {
    name: String,
    parameters: Vec<ParameterInfo>,
    return_type: Option<String>,
    description: String,
    category: String,
    snippet: Option<String>,
}

/// Information about a function parameter
#[derive(Debug, Clone)]
struct ParameterInfo {
    name: String,
    type_hint: Option<String>,
    optional: bool,
    description: Option<String>,
}

/// Keyword completion information
#[derive(Debug, Clone)]
struct KeywordCompletion {
    keyword: String,
    description: String,
    snippet: Option<String>,
}

/// Snippet completion information
#[derive(Debug, Clone)]
struct SnippetCompletion {
    label: String,
    snippet: String,
    description: String,
    context: Vec<String>, // Contexts where this snippet applies
}

impl<P: ModelProvider> CompletionProvider<P> {
    /// Create a new completion provider
    pub fn new(provider: Arc<P>) -> Self {
        let (function_registry, _) = crate::registry::create_standard_registries();
        Self::with_registry(provider, Arc::new(function_registry))
    }

    /// Create a new completion provider with a custom function registry
    pub fn with_registry(provider: Arc<P>, function_registry: Arc<FunctionRegistry>) -> Self {
        let mut builtin_functions = HashMap::new();
        let mut fhir_functions = HashMap::new();
        let mut keywords = Vec::new();
        let mut snippets = Vec::new();

        Self::register_builtin_functions(&mut builtin_functions);
        Self::register_fhir_functions(&mut fhir_functions);
        Self::register_keywords(&mut keywords);
        Self::register_snippets(&mut snippets);

        Self {
            provider,
            function_registry,
            builtin_functions,
            fhir_functions,
            keywords,
            snippets,
        }
    }

    /// Get completions for an expression at a cursor position
    pub async fn get_completions(
        &self,
        expression: &ExpressionNode,
        context_type: Option<&str>,
        cursor_position: u32,
    ) -> Result<Vec<Completion>, AnalysisError> {
        let context = AnalysisContext::new(context_type.map(String::from));
        
        let completion_context = CompletionContext {
            position: cursor_position,
            trigger_text: None,
            trigger_character: None,
            expression_context: self.analyze_expression_context(expression, cursor_position, &context).await?,
        };

        self.get_completions_with_context(expression, &context, &completion_context).await
    }

    /// Get completions with full context
    pub async fn get_completions_with_context(
        &self,
        expression: &ExpressionNode,
        context: &AnalysisContext,
        completion_context: &CompletionContext,
    ) -> Result<Vec<Completion>, AnalysisError> {
        let mut completions = Vec::new();

        // Determine what kind of completions are appropriate
        if let Some(expr_context) = &completion_context.expression_context {
            if let Some(base_type) = &expr_context.base_type {
                // Property completions
                completions.extend(self.get_property_completions(base_type).await?);
            }

            if expr_context.in_function_call.is_some() || expr_context.in_method_call.is_some() {
                // Parameter completions
                completions.extend(self.get_parameter_completions(expr_context, context).await?);
            }
        }

        // Function completions (context-aware)
        completions.extend(self.get_function_completions_for_context(completion_context).await?);

        // Method completions (context-dependent)
        completions.extend(self.get_method_completions(completion_context).await?);

        // Variable completions
        completions.extend(self.get_variable_completions(context));

        // Type completions
        completions.extend(self.get_type_completions().await?);

        // Keyword completions
        completions.extend(self.get_keyword_completions());

        // Snippet completions
        completions.extend(self.get_snippet_completions(completion_context));

        // Filter and sort completions
        self.filter_and_sort_completions(completions, completion_context)
    }

    /// Get property completions for a given type
    async fn get_property_completions(&self, type_name: &str) -> Result<Vec<Completion>, AnalysisError> {
        let mut completions = Vec::new();

        // Get properties from the ModelProvider
        let properties = self.provider.get_properties(type_name).await;

        for (property_name, property_info) in properties {
            let completion = Completion {
                label: property_name.clone(),
                insert_text: property_name.clone(),
                kind: CompletionKind::Property,
                detail: Some(format!("{} ({})", property_info.name(), 
                    if property_info.is_collection() { "collection" } else { "single" })),
                documentation: Some("Property documentation".to_string()),
                sort_text: format!("a{}", property_name), // High priority
                deprecated: false,
                data: Some(serde_json::json!({
                    "propertyType": property_info.name(),
                    "isCollection": property_info.is_collection()
                })),
            };
            completions.push(completion);
        }

        Ok(completions)
    }

    /// Get function completions with context awareness and type filtering
    async fn get_function_completions_for_context(&self, completion_context: &CompletionContext) -> Result<Vec<Completion>, AnalysisError> {
        let mut completions = Vec::new();
        
        // Determine current context type for filtering
        let context_type = completion_context.expression_context
            .as_ref()
            .and_then(|ctx| ctx.base_type.as_deref());
            
        let is_collection_context = completion_context.expression_context
            .as_ref()
            .map(|ctx| ctx.is_collection)
            .unwrap_or(false);

        // Get functions suitable for completion from the registry  
        let completion_functions = self.function_registry.get_completion_functions(context_type, is_collection_context);
        
        for (_function_name, function_impl, metadata) in completion_functions {
            let completion = self.create_function_completion_with_metadata(function_impl, metadata);
            completions.push(completion);
        }

        // Note: Using only registry-based functions for best-in-class LSP experience

        Ok(completions)
    }

    /// Check if a function is applicable to the current context based on type information
    async fn is_function_applicable_to_context(
        &self,
        function_impl: &FunctionImpl,
        context_type: Option<&str>,
        is_collection_context: bool,
    ) -> Result<bool, AnalysisError> {
        let signature = function_impl.signature();
        
        // Collection-specific functions
        if is_collection_context {
            match function_impl.name() {
                // Collection functions that require collection input
                "where" | "select" | "first" | "last" | "tail" | "count" | 
                "empty" | "exists" | "distinct" | "union" | "intersect" | "exclude" |
                "all" | "any" | "skip" | "take" | "sort" => return Ok(true),
                _ => {}
            }
        }
        
        // Type-specific function filtering
        if let Some(context_type_name) = context_type {
            match context_type_name {
                "String" => {
                    match function_impl.name() {
                        "substring" | "length" | "upper" | "lower" | "contains" | 
                        "startsWith" | "endsWith" | "matches" | "replace" | "split" |
                        "trim" | "join" | "indexOf" => return Ok(true),
                        _ => {}
                    }
                }
                "Integer" | "Decimal" => {
                    match function_impl.name() {
                        "abs" | "ceiling" | "floor" | "round" | "sqrt" | "exp" | 
                        "ln" | "log" | "power" | "truncate" => return Ok(true),
                        _ => {}
                    }
                }
                "DateTime" | "Date" | "Time" => {
                    match function_impl.name() {
                        "now" | "today" | "timeOfDay" => return Ok(true),
                        _ => {}
                    }
                }
                "Boolean" => {
                    match function_impl.name() {
                        "not" => return Ok(true),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        // Always include universal functions
        match function_impl.name() {
            // Type conversion functions
            "toString" | "toInteger" | "toDecimal" | "toQuantity" | "toBoolean" |
            "convertsToString" | "convertsToInteger" | "convertsToDecimal" | 
            "convertsToQuantity" | "convertsToBoolean" |
            // Utility functions
            "iif" | "trace" | "defineVariable" | "hasValue" |
            // Type checking
            "is" | "as" | "ofType" |
            // Universal math functions
            "sum" | "avg" | "min" | "max" |
            // FHIR-specific
            "extension" | "resolve" | "conformsTo" => Ok(true),
            _ => {
                // Check if function signature accepts the current context type
                Ok(self.check_signature_compatibility(signature, context_type, is_collection_context))
            }
        }
    }

    /// Check if function signature is compatible with context
    fn check_signature_compatibility(
        &self,
        signature: &crate::registry::signature::FunctionSignature,
        context_type: Option<&str>,
        is_collection_context: bool,
    ) -> bool {
        // If no parameters required, function is always applicable
        if signature.min_arity == 0 {
            return true;
        }
        
        // If we have context type information, check compatibility
        if let Some(_context_type) = context_type {
            // For now, accept all functions that can take generic input
            // This can be enhanced with more sophisticated type checking
            true
        } else {
            // Without context type information, allow all functions
            true
        }
    }

    /// Create a completion item from a function implementation with metadata
    fn create_function_completion_with_metadata(&self, function_impl: &FunctionImpl, metadata: &crate::registry::function::FunctionMetadata) -> Completion {
        Completion {
            label: metadata.display_name.clone(),
            insert_text: metadata.lsp_info.snippet.clone(),
            kind: CompletionKind::Function,
            detail: Some(format!("{} → {}", 
                metadata.input_types.join(" | "),
                metadata.output_type
            )),
            documentation: Some(metadata.description.clone()),
            sort_text: format!("{:03}{}", 
                metadata.category.sort_priority() + metadata.lsp_info.sort_priority, 
                metadata.name
            ),
            deprecated: false,
            data: Some(serde_json::json!({
                "category": metadata.category.display_name(),
                "examples": metadata.examples,
                "isPure": metadata.is_pure,
                "inputTypes": metadata.input_types,
                "outputType": metadata.output_type,
                "keywords": metadata.lsp_info.keywords
            })),
        }
    }

    /// Create a completion item from a function implementation (fallback)
    fn create_function_completion(&self, function_impl: &FunctionImpl) -> Completion {
        let signature = function_impl.signature();
        let function_name = function_impl.name();
        
        // Create snippet with parameter placeholders
        let insert_text = if signature.parameters.is_empty() {
            format!("{}()", function_name)
        } else {
            let params = signature.parameters.iter()
                .enumerate()
                .map(|(i, param)| {
                    let placeholder = i + 1;
                    if param.optional {
                        format!("${{{placeholder}:{}}}", param.name)
                    } else {
                        format!("${{{placeholder}:{}}}", param.name)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", function_name, params)
        };

        // Determine category based on function name patterns
        let category = self.categorize_function(function_name);
        
        // Create detailed signature string
        let detail = format!("{} -> {}", 
            signature.parameters.iter()
                .map(|p| format!("{}: {}{}", 
                    p.name, 
                    p.param_type,
                    if p.optional { "?" } else { "" }
                ))
                .collect::<Vec<_>>()
                .join(", "),
            signature.return_type
        );

        Completion {
            label: function_name.to_string(),
            insert_text,
            kind: CompletionKind::Function,
            detail: Some(detail),
            documentation: Some(function_impl.documentation().to_string()),
            sort_text: format!("b{}{}", category, function_name), // Category-based sorting
            deprecated: false,
            data: Some(serde_json::json!({
                "category": category,
                "signature": signature,
                "isPure": function_impl.is_pure()
            })),
        }
    }

    /// Categorize function for sorting purposes
    fn categorize_function(&self, function_name: &str) -> &'static str {
        match function_name {
            name if name.starts_with("to") || name.starts_with("convertsTo") => "1_conversion",
            "where" | "select" | "first" | "last" | "exists" | "all" | "any" => "2_collection",
            "substring" | "length" | "upper" | "lower" | "contains" => "3_string",
            "abs" | "ceiling" | "floor" | "round" | "sum" | "avg" | "min" | "max" => "4_math",
            "now" | "today" | "timeOfDay" => "5_datetime",
            "extension" | "resolve" | "conformsTo" => "6_fhir",
            _ => "7_other"
        }
    }

    /// Get enhanced function completions using the registry's type filtering
    async fn get_enhanced_function_completions(&self, completion_context: &CompletionContext) -> Result<Vec<Completion>, AnalysisError> {
        let mut completions = Vec::new();
        
        // Determine context for filtering
        let context_type = completion_context.expression_context
            .as_ref()
            .and_then(|ctx| ctx.base_type.as_deref());
            
        let is_collection_context = completion_context.expression_context
            .as_ref()
            .map(|ctx| ctx.is_collection)
            .unwrap_or(false);

        // Get functions applicable to current context using the registry's built-in filtering
        let applicable_functions = self.function_registry.get_functions_for_type(context_type, is_collection_context);
        
        for (function_name, function_impl, _metadata) in applicable_functions {
            let completion = self.create_function_completion(function_impl);
            completions.push(completion);
        }
        
        // Sort by category and then by name
        completions.sort_by(|a, b| a.sort_text.cmp(&b.sort_text));
        
        Ok(completions)
    }

    /// Get method completions based on context
    async fn get_method_completions(&self, completion_context: &CompletionContext) -> Result<Vec<Completion>, AnalysisError> {
        let mut completions = Vec::new();

        if let Some(expr_context) = &completion_context.expression_context {
            if expr_context.is_collection {
                // Collection-specific methods
                let collection_methods = [
                    ("where", "Filter collection with condition", "where($1)"),
                    ("select", "Transform collection elements", "select($1)"),
                    ("first", "Get first element", "first()"),
                    ("last", "Get last element", "last()"),
                    ("tail", "Get all except first", "tail()"),
                    ("count", "Count elements", "count()"),
                    ("empty", "Check if collection is empty", "empty()"),
                    ("exists", "Check if collection has elements", "exists($1)"),
                    ("distinct", "Get distinct elements", "distinct()"),
                    ("union", "Union with another collection", "union($1)"),
                ];

                for (method, desc, snippet) in collection_methods {
                    completions.push(Completion {
                        label: method.to_string(),
                        insert_text: snippet.to_string(),
                        kind: CompletionKind::Method,
                        detail: Some("Collection method".to_string()),
                        documentation: Some(desc.to_string()),
                        sort_text: format!("a{}", method), // High priority for methods
                        deprecated: false,
                        data: Some(serde_json::json!({"methodType": "collection"})),
                    });
                }
            }

            // Type-specific methods based on base type
            if let Some(base_type) = &expr_context.base_type {
                completions.extend(self.get_type_specific_methods(base_type).await?);
            }
        }

        Ok(completions)
    }

    /// Get type-specific method completions
    async fn get_type_specific_methods(&self, type_name: &str) -> Result<Vec<Completion>, AnalysisError> {
        let mut completions = Vec::new();

        match type_name {
            "String" => {
                let string_methods = [
                    ("length", "Get string length", "length()"),
                    ("substring", "Get substring", "substring($1)"),
                    ("upper", "Convert to uppercase", "upper()"),
                    ("lower", "Convert to lowercase", "lower()"),
                    ("contains", "Check if contains substring", "contains($1)"),
                    ("startsWith", "Check if starts with", "startsWith($1)"),
                    ("endsWith", "Check if ends with", "endsWith($1)"),
                    ("matches", "Match against regex", "matches($1)"),
                    ("replace", "Replace substring", "replace($1, $2)"),
                    ("split", "Split string", "split($1)"),
                ];

                for (method, desc, snippet) in string_methods {
                    completions.push(Completion {
                        label: method.to_string(),
                        insert_text: snippet.to_string(),
                        kind: CompletionKind::Method,
                        detail: Some("String method".to_string()),
                        documentation: Some(desc.to_string()),
                        sort_text: format!("a{}", method),
                        deprecated: false,
                        data: Some(serde_json::json!({"methodType": "string"})),
                    });
                }
            }
            "Integer" | "Decimal" => {
                let numeric_methods = [
                    ("toString", "Convert to string", "toString()"),
                    ("abs", "Absolute value", "abs()"),
                    ("ceiling", "Round up", "ceiling()"),
                    ("floor", "Round down", "floor()"),
                    ("round", "Round to nearest", "round()"),
                    ("truncate", "Truncate decimals", "truncate()"),
                ];

                for (method, desc, snippet) in numeric_methods {
                    completions.push(Completion {
                        label: method.to_string(),
                        insert_text: snippet.to_string(),
                        kind: CompletionKind::Method,
                        detail: Some("Numeric method".to_string()),
                        documentation: Some(desc.to_string()),
                        sort_text: format!("a{}", method),
                        deprecated: false,
                        data: Some(serde_json::json!({"methodType": "numeric"})),
                    });
                }
            }
            _ => {
                // Generic methods available on all types
                let generic_methods = [
                    ("toString", "Convert to string representation", "toString()"),
                    ("hasValue", "Check if has value", "hasValue()"),
                    ("is", "Type check", "is($1)"),
                    ("as", "Type cast", "as($1)"),
                ];

                for (method, desc, snippet) in generic_methods {
                    completions.push(Completion {
                        label: method.to_string(),
                        insert_text: snippet.to_string(),
                        kind: CompletionKind::Method,
                        detail: Some("Generic method".to_string()),
                        documentation: Some(desc.to_string()),
                        sort_text: format!("b{}", method),
                        deprecated: false,
                        data: Some(serde_json::json!({"methodType": "generic"})),
                    });
                }
            }
        }

        Ok(completions)
    }

    /// Get parameter completions for function/method calls
    async fn get_parameter_completions(
        &self,
        expr_context: &ExpressionContext,
        context: &AnalysisContext,
    ) -> Result<Vec<Completion>, AnalysisError> {
        let mut completions = Vec::new();

        // Variable completions
        for (var_name, var_type) in &context.variables {
            completions.push(Completion {
                label: var_name.clone(),
                insert_text: var_name.clone(),
                kind: CompletionKind::Variable,
                detail: Some(format!("Variable of type {}", var_type.name())),
                documentation: None,
                sort_text: format!("a{}", var_name),
                deprecated: false,
                data: Some(serde_json::json!({"variableType": var_type.name()})),
            });
        }

        // System variables
        let system_vars = [
            ("$this", "Current context item"),
            ("$index", "Current index in iteration"),
            ("$total", "Total count in iteration"),
        ];

        for (var, desc) in system_vars {
            completions.push(Completion {
                label: var.to_string(),
                insert_text: var.to_string(),
                kind: CompletionKind::Variable,
                detail: Some("System variable".to_string()),
                documentation: Some(desc.to_string()),
                sort_text: format!("a{}", var),
                deprecated: false,
                data: Some(serde_json::json!({"systemVariable": true})),
            });
        }

        Ok(completions)
    }

    /// Get variable completions
    fn get_variable_completions(&self, context: &AnalysisContext) -> Vec<Completion> {
        let mut completions = Vec::new();

        // Context variables
        for (var_name, var_type) in &context.variables {
            completions.push(Completion {
                label: var_name.clone(),
                insert_text: var_name.clone(),
                kind: CompletionKind::Variable,
                detail: Some(format!("Variable ({})", var_type.name())),
                documentation: None,
                sort_text: format!("c{}", var_name), // Lower priority
                deprecated: false,
                data: None,
            });
        }

        // System variables
        if context.root_type.is_some() {
            completions.push(Completion {
                label: "$this".to_string(),
                insert_text: "$this".to_string(),
                kind: CompletionKind::Variable,
                detail: Some("Current context".to_string()),
                documentation: Some("Reference to the current context item".to_string()),
                sort_text: "c$this".to_string(),
                deprecated: false,
                data: None,
            });
        }

        completions
    }

    /// Get type completions
    async fn get_type_completions(&self) -> Result<Vec<Completion>, AnalysisError> {
        let mut completions = Vec::new();

        // Common FHIR types
        let common_types = [
            "Patient", "Observation", "Condition", "Medication", "Procedure",
            "Encounter", "Practitioner", "Organization", "Location", "Device",
            "String", "Integer", "Decimal", "Boolean", "Date", "DateTime", "Time",
            "Quantity", "Code", "Coding", "CodeableConcept", "Identifier", "Reference"
        ];

        for type_name in common_types {
            if self.provider.get_type_reflection(type_name).await.is_some() {
                completions.push(Completion {
                    label: type_name.to_string(),
                    insert_text: type_name.to_string(),
                    kind: CompletionKind::Type,
                    detail: Some("FHIR type".to_string()),
                    documentation: None,
                    sort_text: format!("d{}", type_name), // Lower priority
                    deprecated: false,
                    data: Some(serde_json::json!({"fhirType": true})),
                });
            }
        }

        Ok(completions)
    }

    /// Get keyword completions
    fn get_keyword_completions(&self) -> Vec<Completion> {
        self.keywords.iter().map(|keyword| Completion {
            label: keyword.keyword.clone(),
            insert_text: keyword.snippet.as_ref().unwrap_or(&keyword.keyword).clone(),
            kind: CompletionKind::Keyword,
            detail: Some("Keyword".to_string()),
            documentation: Some(keyword.description.clone()),
            sort_text: format!("e{}", keyword.keyword), // Lowest priority
            deprecated: false,
            data: None,
        }).collect()
    }

    /// Get snippet completions
    fn get_snippet_completions(&self, completion_context: &CompletionContext) -> Vec<Completion> {
        self.snippets.iter()
            .filter(|snippet| {
                // Filter snippets based on context
                snippet.context.is_empty() || 
                completion_context.expression_context.as_ref()
                    .map(|ctx| snippet.context.iter().any(|c| self.matches_context(c, ctx)))
                    .unwrap_or(true)
            })
            .map(|snippet| Completion {
                label: snippet.label.clone(),
                insert_text: snippet.snippet.clone(),
                kind: CompletionKind::Snippet,
                detail: Some("Snippet".to_string()),
                documentation: Some(snippet.description.clone()),
                sort_text: format!("f{}", snippet.label), // Very low priority
                deprecated: false,
                data: Some(serde_json::json!({"isSnippet": true})),
            })
            .collect()
    }

    /// Check if a snippet context matches the current context
    fn matches_context(&self, snippet_context: &str, expr_context: &ExpressionContext) -> bool {
        match snippet_context {
            "collection" => expr_context.is_collection,
            "function" => expr_context.in_function_call.is_some(),
            "method" => expr_context.in_method_call.is_some(),
            _ => true,
        }
    }

    /// Filter and sort completions based on context
    fn filter_and_sort_completions(
        &self,
        mut completions: Vec<Completion>,
        completion_context: &CompletionContext,
    ) -> Result<Vec<Completion>, AnalysisError> {
        // Remove duplicates
        completions.sort_by(|a, b| a.label.cmp(&b.label));
        completions.dedup_by(|a, b| a.label == b.label);

        // Filter by trigger text if available
        if let Some(trigger_text) = &completion_context.trigger_text {
            completions.retain(|c| c.label.starts_with(trigger_text));
        }

        // Sort by priority (sort_text)
        completions.sort_by(|a, b| a.sort_text.cmp(&b.sort_text));

        // Limit results to avoid overwhelming the user
        completions.truncate(50);

        Ok(completions)
    }

    /// Analyze expression context for completion
    async fn analyze_expression_context(
        &self,
        expression: &ExpressionNode,
        cursor_position: u32,
        context: &AnalysisContext,
    ) -> Result<Option<ExpressionContext>, AnalysisError> {
        // This is a simplified implementation
        // Real implementation would traverse the AST and find the context at cursor position
        
        match expression {
            ExpressionNode::Path { base, .. } => {
                // Determine base type for property completions
                if let ExpressionNode::Identifier(name) = base.as_ref() {
                    let base_type = if name == "$this" || Some(name) == context.root_type.as_ref() {
                        context.root_type.clone()
                    } else {
                        None
                    };

                    Ok(Some(ExpressionContext {
                        base_type,
                        is_collection: false, // Would need type inference to determine this
                        in_function_call: None,
                        in_method_call: None,
                        parameter_index: None,
                    }))
                } else {
                    Ok(Some(ExpressionContext {
                        base_type: None,
                        is_collection: false,
                        in_function_call: None,
                        in_method_call: None,
                        parameter_index: None,
                    }))
                }
            }
            _ => Ok(None),
        }
    }

    /// Register built-in functions
    fn register_builtin_functions(functions: &mut HashMap<String, FunctionCompletion>) {
        let builtin_funcs = [
            ("empty", FunctionCompletion {
                name: "empty".to_string(),
                parameters: vec![],
                return_type: Some("Boolean".to_string()),
                description: "Returns true if the collection is empty".to_string(),
                category: "Collection".to_string(),
                snippet: Some("empty()".to_string()),
            }),
            ("exists", FunctionCompletion {
                name: "exists".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "condition".to_string(),
                        type_hint: Some("Boolean".to_string()),
                        optional: true,
                        description: Some("Optional condition to check".to_string()),
                    }
                ],
                return_type: Some("Boolean".to_string()),
                description: "Returns true if any element matches the condition".to_string(),
                category: "Collection".to_string(),
                snippet: Some("exists($1)".to_string()),
            }),
            ("count", FunctionCompletion {
                name: "count".to_string(),
                parameters: vec![],
                return_type: Some("Integer".to_string()),
                description: "Returns the number of elements in the collection".to_string(),
                category: "Collection".to_string(),
                snippet: Some("count()".to_string()),
            }),
            ("first", FunctionCompletion {
                name: "first".to_string(),
                parameters: vec![],
                return_type: None,
                description: "Returns the first element of the collection".to_string(),
                category: "Collection".to_string(),
                snippet: Some("first()".to_string()),
            }),
            ("where", FunctionCompletion {
                name: "where".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "condition".to_string(),
                        type_hint: Some("Boolean".to_string()),
                        optional: false,
                        description: Some("Condition to filter by".to_string()),
                    }
                ],
                return_type: None,
                description: "Filters the collection by the given condition".to_string(),
                category: "Collection".to_string(),
                snippet: Some("where($1)".to_string()),
            }),
            ("select", FunctionCompletion {
                name: "select".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "expression".to_string(),
                        type_hint: None,
                        optional: false,
                        description: Some("Expression to transform each element".to_string()),
                    }
                ],
                return_type: None,
                description: "Transforms each element using the given expression".to_string(),
                category: "Collection".to_string(),
                snippet: Some("select($1)".to_string()),
            }),
        ];

        for (name, func) in builtin_funcs {
            functions.insert(name.to_string(), func);
        }
    }

    /// Register FHIR-specific functions
    fn register_fhir_functions(functions: &mut HashMap<String, FunctionCompletion>) {
        let fhir_funcs = [
            ("resolve", FunctionCompletion {
                name: "resolve".to_string(),
                parameters: vec![],
                return_type: None,
                description: "Resolves a reference to the referenced resource".to_string(),
                category: "FHIR".to_string(),
                snippet: Some("resolve()".to_string()),
            }),
            ("extension", FunctionCompletion {
                name: "extension".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "url".to_string(),
                        type_hint: Some("String".to_string()),
                        optional: false,
                        description: Some("Extension URL to find".to_string()),
                    }
                ],
                return_type: Some("Extension".to_string()),
                description: "Gets extensions with the specified URL".to_string(),
                category: "FHIR".to_string(),
                snippet: Some("extension('$1')".to_string()),
            }),
            ("conformsTo", FunctionCompletion {
                name: "conformsTo".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "profile".to_string(),
                        type_hint: Some("String".to_string()),
                        optional: false,
                        description: Some("Profile URL to check conformance against".to_string()),
                    }
                ],
                return_type: Some("Boolean".to_string()),
                description: "Checks if the resource conforms to the given profile".to_string(),
                category: "FHIR".to_string(),
                snippet: Some("conformsTo('$1')".to_string()),
            }),
        ];

        for (name, func) in fhir_funcs {
            functions.insert(name.to_string(), func);
        }
    }

    /// Register keywords
    fn register_keywords(keywords: &mut Vec<KeywordCompletion>) {
        let keyword_list = [
            KeywordCompletion {
                keyword: "and".to_string(),
                description: "Logical AND operator".to_string(),
                snippet: Some(" and ".to_string()),
            },
            KeywordCompletion {
                keyword: "or".to_string(),
                description: "Logical OR operator".to_string(),
                snippet: Some(" or ".to_string()),
            },
            KeywordCompletion {
                keyword: "is".to_string(),
                description: "Type checking operator".to_string(),
                snippet: Some(" is $1".to_string()),
            },
            KeywordCompletion {
                keyword: "as".to_string(),
                description: "Type casting operator".to_string(),
                snippet: Some(" as $1".to_string()),
            },
            KeywordCompletion {
                keyword: "in".to_string(),
                description: "Collection membership operator".to_string(),
                snippet: Some(" in $1".to_string()),
            },
            KeywordCompletion {
                keyword: "contains".to_string(),
                description: "Collection containment operator".to_string(),
                snippet: Some(" contains $1".to_string()),
            },
        ];

        keywords.extend(keyword_list);
    }

    /// Register code snippets
    fn register_snippets(snippets: &mut Vec<SnippetCompletion>) {
        let snippet_list = [
            SnippetCompletion {
                label: "where exists".to_string(),
                snippet: "where($1.exists())".to_string(),
                description: "Filter where property exists".to_string(),
                context: vec!["collection".to_string()],
            },
            SnippetCompletion {
                label: "where empty".to_string(),
                snippet: "where($1.empty())".to_string(),
                description: "Filter where property is empty".to_string(),
                context: vec!["collection".to_string()],
            },
            SnippetCompletion {
                label: "select value".to_string(),
                snippet: "select($1)".to_string(),
                description: "Select a value from each element".to_string(),
                context: vec!["collection".to_string()],
            },
            SnippetCompletion {
                label: "first or empty".to_string(),
                snippet: "first() | {}".to_string(),
                description: "Get first element or empty collection".to_string(),
                context: vec!["collection".to_string()],
            },
            SnippetCompletion {
                label: "extension by url".to_string(),
                snippet: "extension('$1').value".to_string(),
                description: "Get extension value by URL".to_string(),
                context: vec![],
            },
        ];

        snippets.extend(snippet_list);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::mock_provider::MockModelProvider;
    use crate::ast::ExpressionNode;

    #[tokio::test]
    async fn test_completion_provider_creation() {
        let provider = Arc::new(MockModelProvider::empty());
        let completion_provider = CompletionProvider::new(provider);

        assert!(!completion_provider.builtin_functions.is_empty());
        assert!(!completion_provider.keywords.is_empty());
    }

    #[tokio::test]
    async fn test_function_completions() {
        let provider = Arc::new(MockModelProvider::empty());
        let completion_provider = CompletionProvider::new(provider);

        let completions = completion_provider.get_function_completions();
        assert!(completions.iter().any(|c| c.label == "count"));
        assert!(completions.iter().any(|c| c.label == "where"));
        assert!(completions.iter().any(|c| c.kind == CompletionKind::Function));
    }

    #[tokio::test]
    async fn test_basic_completions() {
        let provider = Arc::new(MockModelProvider::empty());
        let completion_provider = CompletionProvider::new(provider);

        let expr = ExpressionNode::identifier("Patient");
        let completions = completion_provider.get_completions(&expr, Some("Patient"), 0).await.unwrap();

        assert!(!completions.is_empty());
        // With MockModelProvider, we won't get real property completions
        // but we should get function and other completions
    }
}