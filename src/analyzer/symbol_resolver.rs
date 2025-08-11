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


//! Symbol resolution for FHIRPath expressions
//!
//! This module provides symbol tracking, go-to-definition, hover information,
//! and reference finding capabilities for LSP integration.

use crate::ast::ExpressionNode;
use crate::model::provider::{ModelProvider, TypeReflectionInfo, ElementDefinition, StructureDefinition};
use crate::analyzer::{AnalysisContext, AnalysisError};
// Removed Span import to avoid lifetime issues
use std::sync::Arc;
use std::collections::HashMap;

/// A symbol in a FHIRPath expression
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Start byte offset where symbol is defined/used
    pub start_offset: Option<usize>,
    /// End byte offset where symbol is defined/used
    pub end_offset: Option<usize>,
    /// Type information for the symbol
    pub type_info: Option<TypeReflectionInfo>,
    /// Definition location (for go-to-definition)
    pub definition_location: Option<DefinitionLocation>,
    /// Documentation for hover
    pub documentation: Option<String>,
    /// Whether this symbol is deprecated
    pub deprecated: bool,
}

/// Kind of symbol
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// Property or field access
    Property,
    /// Function call
    Function,
    /// Method call
    Method,
    /// Variable reference
    Variable,
    /// Type reference
    Type,
    /// Parameter in function/method
    Parameter,
    /// System variable ($this, $index, etc.)
    SystemVariable,
}

/// Location of a symbol definition
#[derive(Debug, Clone, PartialEq)]
pub struct DefinitionLocation {
    /// URI of the definition (could be FHIR spec, schema file, etc.)
    pub uri: String,
    /// Range within the definition
    pub range: Option<Range>,
    /// Description of the definition source
    pub source: String,
}

/// Range in a document
#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    /// Start position
    pub start: Position,
    /// End position
    pub end: Position,
}

/// Position in a document
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// Line number (0-based)
    pub line: u32,
    /// Character offset (0-based)
    pub character: u32,
}

/// Hover information for a symbol
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// Content to display
    pub contents: String,
    /// Start byte offset where hover applies
    pub start_offset: Option<usize>,
    /// End byte offset where hover applies
    pub end_offset: Option<usize>,
}

/// Reference to a symbol
#[derive(Debug, Clone)]
pub struct SymbolReference {
    /// Start byte offset of the reference
    pub start_offset: usize,
    /// End byte offset of the reference
    pub end_offset: usize,
    /// Kind of reference (read, write, etc.)
    pub reference_kind: ReferenceKind,
    /// Context information
    pub context: String,
}

/// Kind of reference
#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceKind {
    /// Reading/accessing the symbol
    Read,
    /// Defining the symbol
    Definition,
    /// Type annotation
    Type,
}

/// Symbol resolver for FHIRPath expressions
pub struct SymbolResolver<P: ModelProvider> {
    provider: Arc<P>,
    symbol_cache: tokio::sync::RwLock<HashMap<String, Symbol>>,
    definition_cache: tokio::sync::RwLock<HashMap<String, DefinitionLocation>>,
}

impl<P: ModelProvider> SymbolResolver<P> {
    /// Create a new symbol resolver
    pub fn new(provider: Arc<P>) -> Self {
        Self {
            provider,
            symbol_cache: tokio::sync::RwLock::new(HashMap::new()),
            definition_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Resolve symbols in an expression
    pub async fn resolve_symbols(
        &self,
        expression: &ExpressionNode,
        context_type: Option<&str>,
    ) -> Result<Vec<Symbol>, AnalysisError> {
        let context = AnalysisContext::new(context_type.map(String::from));
        self.resolve_symbols_with_context(expression, &context).await
    }

    /// Resolve symbols with full context
    pub async fn resolve_symbols_with_context(
        &self,
        expression: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Result<Vec<Symbol>, AnalysisError> {
        if context.depth > 50 {
            return Err(AnalysisError::MaxDepthExceeded { max_depth: 50 });
        }

        let mut symbols = Vec::new();
        let child_context = context.child();

        match expression {
            ExpressionNode::Literal(_) => {
                // Literals don't create symbols
            }
            ExpressionNode::Identifier(name) => {
                if let Some(symbol) = self.resolve_identifier_symbol(name, &child_context).await? {
                    symbols.push(symbol);
                }
            }
            ExpressionNode::Path { base, path } => {
                // Resolve base symbols first
                let base_symbols = self.resolve_symbols_with_context(base, &child_context).await?;
                symbols.extend(base_symbols);

                // Then resolve the path property
                if let Some(symbol) = self.resolve_property_symbol(base, path, &child_context).await? {
                    symbols.push(symbol);
                }
            }
            ExpressionNode::BinaryOp(data) => {
                let left_symbols = self.resolve_symbols_with_context(&data.left, &child_context).await?;
                let right_symbols = self.resolve_symbols_with_context(&data.right, &child_context).await?;
                symbols.extend(left_symbols);
                symbols.extend(right_symbols);
            }
            ExpressionNode::UnaryOp { operand, .. } => {
                let operand_symbols = self.resolve_symbols_with_context(operand, &child_context).await?;
                symbols.extend(operand_symbols);
            }
            ExpressionNode::FunctionCall(data) => {
                // Function symbol
                if let Some(function_symbol) = self.resolve_function_symbol(&data.name, &child_context).await? {
                    symbols.push(function_symbol);
                }

                // Argument symbols
                for arg in &data.args {
                    let arg_symbols = self.resolve_symbols_with_context(arg, &child_context).await?;
                    symbols.extend(arg_symbols);
                }
            }
            ExpressionNode::MethodCall(data) => {
                // Base symbols
                let base_symbols = self.resolve_symbols_with_context(&data.base, &child_context).await?;
                symbols.extend(base_symbols);

                // Method symbol
                if let Some(method_symbol) = self.resolve_method_symbol(&data.method, &child_context).await? {
                    symbols.push(method_symbol);
                }

                // Argument symbols
                for arg in &data.args {
                    let arg_symbols = self.resolve_symbols_with_context(arg, &child_context).await?;
                    symbols.extend(arg_symbols);
                }
            }
            ExpressionNode::Index { base, index } => {
                let base_symbols = self.resolve_symbols_with_context(base, &child_context).await?;
                let index_symbols = self.resolve_symbols_with_context(index, &child_context).await?;
                symbols.extend(base_symbols);
                symbols.extend(index_symbols);
            }
            ExpressionNode::Filter { base, condition } => {
                let base_symbols = self.resolve_symbols_with_context(base, &child_context).await?;
                let condition_symbols = self.resolve_symbols_with_context(condition, &child_context).await?;
                symbols.extend(base_symbols);
                symbols.extend(condition_symbols);
            }
            ExpressionNode::Union { left, right } => {
                let left_symbols = self.resolve_symbols_with_context(left, &child_context).await?;
                let right_symbols = self.resolve_symbols_with_context(right, &child_context).await?;
                symbols.extend(left_symbols);
                symbols.extend(right_symbols);
            }
            ExpressionNode::TypeCheck { expression, type_name } |
            ExpressionNode::TypeCast { expression, type_name } => {
                let expr_symbols = self.resolve_symbols_with_context(expression, &child_context).await?;
                symbols.extend(expr_symbols);

                if let Some(type_symbol) = self.resolve_type_symbol(type_name, &child_context).await? {
                    symbols.push(type_symbol);
                }
            }
            ExpressionNode::Lambda(data) => {
                // Parameters create symbols in the lambda scope
                let mut lambda_context = child_context.clone();
                for param in &data.params {
                    symbols.push(Symbol {
                        name: param.clone(),
                        kind: SymbolKind::Parameter,
                        start_offset: context.start_offset,
                        end_offset: context.end_offset,
                        type_info: None, // Would need type inference
                        definition_location: None,
                        documentation: Some(format!("Lambda parameter: {}", param)),
                        deprecated: false,
                    });

                    // Add parameter to lambda context (simplified type)
                    if let Some(type_info) = self.infer_parameter_type(param).await {
                        lambda_context = lambda_context.with_variable(param.clone(), type_info);
                    }
                }

                // Body symbols
                let body_symbols = self.resolve_symbols_with_context(&data.body, &lambda_context).await?;
                symbols.extend(body_symbols);
            }
            ExpressionNode::Conditional(data) => {
                let condition_symbols = self.resolve_symbols_with_context(&data.condition, &child_context).await?;
                let then_symbols = self.resolve_symbols_with_context(&data.then_expr, &child_context).await?;
                symbols.extend(condition_symbols);
                symbols.extend(then_symbols);

                if let Some(else_expr) = &data.else_expr {
                    let else_symbols = self.resolve_symbols_with_context(else_expr, &child_context).await?;
                    symbols.extend(else_symbols);
                }
            }
            ExpressionNode::Variable(name) => {
                if let Some(symbol) = self.resolve_variable_symbol(name, &child_context).await? {
                    symbols.push(symbol);
                }
            }
        }

        Ok(symbols)
    }

    /// Resolve identifier symbol
    async fn resolve_identifier_symbol(
        &self,
        name: &str,
        context: &AnalysisContext,
    ) -> Result<Option<Symbol>, AnalysisError> {
        // Check if it's a context variable
        if let Some(var_type) = context.variables.get(name) {
            return Ok(Some(Symbol {
                name: name.to_string(),
                kind: SymbolKind::Variable,
                start_offset: context.start_offset,
                end_offset: context.end_offset,
                type_info: Some(var_type.clone()),
                definition_location: None,
                documentation: Some(format!("Variable of type {}", var_type.name())),
                deprecated: false,
            }));
        }

        // Check if it's the root context
        if let Some(root_type) = &context.root_type {
            if name == root_type || name == "$this" {
                let type_info = self.provider.get_type_reflection(root_type).await;
                return Ok(Some(Symbol {
                    name: name.to_string(),
                    kind: if name == "$this" { SymbolKind::SystemVariable } else { SymbolKind::Type },
                    start_offset: context.start_offset,
                end_offset: context.end_offset,
                    type_info,
                    definition_location: self.get_type_definition_location(root_type).await,
                    documentation: Some(format!("Context type: {}", root_type)),
                    deprecated: false,
                }));
            }
        }

        // Check if it's a FHIR type
        if let Some(type_info) = self.provider.get_type_reflection(name).await {
            return Ok(Some(Symbol {
                name: name.to_string(),
                kind: SymbolKind::Type,
                start_offset: context.start_offset,
                end_offset: context.end_offset,
                type_info: Some(type_info),
                definition_location: self.get_type_definition_location(name).await,
                documentation: Some(format!("FHIR type: {}", name)),
                deprecated: false,
            }));
        }

        Ok(None)
    }

    /// Resolve property symbol
    async fn resolve_property_symbol(
        &self,
        base: &ExpressionNode,
        path: &str,
        context: &AnalysisContext,
    ) -> Result<Option<Symbol>, AnalysisError> {
        // Determine base type
        let base_type = match base {
            ExpressionNode::Identifier(name) => {
                if name == "$this" || Some(name) == context.root_type.as_ref() {
                    context.root_type.clone()
                } else {
                    Some(name.clone())
                }
            }
            _ => None, // Complex expressions would need full type inference
        };

        if let Some(base_type) = base_type {
            // Get property information from ModelProvider
            if let Some(property_type) = self.provider.get_property_type(&base_type, path).await {
                return Ok(Some(Symbol {
                    name: path.to_string(),
                    kind: SymbolKind::Property,
                    start_offset: context.start_offset,
                end_offset: context.end_offset,
                    type_info: Some(property_type),
                    definition_location: self.get_property_definition_location(&base_type, path).await,
                    documentation: Some(format!("Property {} of {}", path, base_type)),
                    deprecated: false,
                }));
            }
        }

        Ok(None)
    }

    /// Resolve function symbol
    async fn resolve_function_symbol(
        &self,
        name: &str,
        context: &AnalysisContext,
    ) -> Result<Option<Symbol>, AnalysisError> {
        // Check cache first
        {
            let cache = self.symbol_cache.read().await;
            if let Some(cached) = cache.get(&format!("function:{}", name)) {
                return Ok(Some(cached.clone()));
            }
        }

        let (documentation, definition_location) = self.get_function_info(name).await;

        let symbol = Symbol {
            name: name.to_string(),
            kind: SymbolKind::Function,
            start_offset: context.start_offset,
            end_offset: context.end_offset,
            type_info: None, // Functions don't have simple type info
            definition_location,
            documentation,
            deprecated: self.is_function_deprecated(name),
        };

        // Cache the result
        {
            let mut cache = self.symbol_cache.write().await;
            cache.insert(format!("function:{}", name), symbol.clone());
        }

        Ok(Some(symbol))
    }

    /// Resolve method symbol
    async fn resolve_method_symbol(
        &self,
        name: &str,
        context: &AnalysisContext,
    ) -> Result<Option<Symbol>, AnalysisError> {
        // Methods are treated similarly to functions for now
        // In a more sophisticated implementation, we'd consider the base type
        self.resolve_function_symbol(name, context).await.map(|opt_symbol| 
            opt_symbol.map(|mut symbol| {
                symbol.kind = SymbolKind::Method;
                symbol
            })
        )
    }

    /// Resolve type symbol
    async fn resolve_type_symbol(
        &self,
        type_name: &str,
        context: &AnalysisContext,
    ) -> Result<Option<Symbol>, AnalysisError> {
        if let Some(type_info) = self.provider.get_type_reflection(type_name).await {
            return Ok(Some(Symbol {
                name: type_name.to_string(),
                kind: SymbolKind::Type,
                start_offset: context.start_offset,
                end_offset: context.end_offset,
                type_info: Some(type_info),
                definition_location: self.get_type_definition_location(type_name).await,
                documentation: Some(format!("FHIR type: {}", type_name)),
                deprecated: false,
            }));
        }

        Ok(None)
    }

    /// Resolve variable symbol
    async fn resolve_variable_symbol(
        &self,
        name: &str,
        context: &AnalysisContext,
    ) -> Result<Option<Symbol>, AnalysisError> {
        // System variables
        match name {
            "$this" => {
                if let Some(root_type) = &context.root_type {
                    let type_info = self.provider.get_type_reflection(root_type).await;
                    return Ok(Some(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::SystemVariable,
                        start_offset: context.start_offset,
                        end_offset: context.end_offset,
                        type_info,
                        definition_location: None,
                        documentation: Some("Current context item".to_string()),
                        deprecated: false,
                    }));
                }
            }
            "$index" => {
                return Ok(Some(Symbol {
                    name: name.to_string(),
                    kind: SymbolKind::SystemVariable,
                    start_offset: context.start_offset,
                end_offset: context.end_offset,
                    type_info: self.provider.get_type_reflection("Integer").await,
                    definition_location: None,
                    documentation: Some("Current index in collection iteration".to_string()),
                    deprecated: false,
                }));
            }
            "$total" => {
                return Ok(Some(Symbol {
                    name: name.to_string(),
                    kind: SymbolKind::SystemVariable,
                    start_offset: context.start_offset,
                end_offset: context.end_offset,
                    type_info: self.provider.get_type_reflection("Integer").await,
                    definition_location: None,
                    documentation: Some("Total count in collection iteration".to_string()),
                    deprecated: false,
                }));
            }
            _ => {
                // Check context variables
                if let Some(var_type) = context.variables.get(name) {
                    return Ok(Some(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Variable,
                        start_offset: context.start_offset,
                        end_offset: context.end_offset,
                        type_info: Some(var_type.clone()),
                        definition_location: None,
                        documentation: Some(format!("Variable of type {}", var_type.name())),
                        deprecated: false,
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Get hover information for a position
    pub async fn get_hover_info(
        &self,
        expression: &ExpressionNode,
        context_type: Option<&str>,
        position: u32,
    ) -> Result<Option<HoverInfo>, AnalysisError> {
        let symbols = self.resolve_symbols(expression, context_type).await?;
        
        // Find symbol at position (simplified - would need proper position mapping)
        if let Some(symbol) = symbols.first() {
            let contents = self.format_hover_contents(symbol).await;
            return Ok(Some(HoverInfo {
                contents,
                start_offset: symbol.start_offset,
                end_offset: symbol.end_offset,
            }));
        }

        Ok(None)
    }

    /// Find references to a symbol
    pub async fn find_references(
        &self,
        expression: &ExpressionNode,
        context_type: Option<&str>,
        symbol_name: &str,
    ) -> Result<Vec<SymbolReference>, AnalysisError> {
        let symbols = self.resolve_symbols(expression, context_type).await?;
        
        let references = symbols.iter()
            .filter(|symbol| symbol.name == symbol_name)
            .filter_map(|symbol| {
                if let (Some(start), Some(end)) = (symbol.start_offset, symbol.end_offset) {
                    Some(SymbolReference {
                        start_offset: start,
                        end_offset: end,
                reference_kind: match symbol.kind {
                    SymbolKind::Parameter => ReferenceKind::Definition,
                    SymbolKind::Type => ReferenceKind::Type,
                    _ => ReferenceKind::Read,
                },
                context: format!("{} reference", symbol.kind_name()),
                    })
                } else {
                    None
                }
            })            .collect();

        Ok(references)
    }

    /// Get definition location for a type
    async fn get_type_definition_location(&self, type_name: &str) -> Option<DefinitionLocation> {
        // Check cache first
        {
            let cache = self.definition_cache.read().await;
            if let Some(cached) = cache.get(&format!("type:{}", type_name)) {
                return Some(cached.clone());
            }
        }

        // Try to get structure definition
        if let Some(structure_def) = self.provider.get_structure_definition(type_name).await {
            let definition = DefinitionLocation {
                uri: if structure_def.url.is_empty() {
                    format!("fhir://StructureDefinition/{}", type_name)
                } else {
                    structure_def.url
                },
                range: None, // Would need to parse from structure definition
                source: "FHIR Specification".to_string(),
            };

            // Cache the result
            {
                let mut cache = self.definition_cache.write().await;
                cache.insert(format!("type:{}", type_name), definition.clone());
            }

            return Some(definition);
        }

        // Fallback to FHIR specification URL
        if self.is_fhir_type(type_name) {
            let definition = DefinitionLocation {
                uri: format!("https://hl7.org/fhir/R4/{}.html", type_name.to_lowercase()),
                range: None,
                source: "FHIR R4 Specification".to_string(),
            };

            // Cache the result
            {
                let mut cache = self.definition_cache.write().await;
                cache.insert(format!("type:{}", type_name), definition.clone());
            }

            return Some(definition);
        }

        None
    }

    /// Get definition location for a property
    async fn get_property_definition_location(&self, base_type: &str, property: &str) -> Option<DefinitionLocation> {
        // Try to get element definition
        if let Some(_type_info) = self.provider.get_element_reflection(base_type, property).await {
            // Element found - create definition location
            {
                return Some(DefinitionLocation {
                    uri: format!("fhir://StructureDefinition/{}#{}", base_type, property),
                    range: None,
                    source: "FHIR Element Definition".to_string(),
                });
            }
        }

        // Fallback to FHIR specification
        if self.is_fhir_type(base_type) {
            return Some(DefinitionLocation {
                uri: format!("https://hl7.org/fhir/R4/{}.html#{}", base_type.to_lowercase(), property),
                range: None,
                source: "FHIR R4 Specification".to_string(),
            });
        }

        None
    }

    /// Get function information
    async fn get_function_info(&self, function_name: &str) -> (Option<String>, Option<DefinitionLocation>) {
        let documentation = match function_name {
            "empty" => Some("Returns true if the collection is empty, false otherwise".to_string()),
            "exists" => Some("Returns true if the collection contains any elements, optionally matching a condition".to_string()),
            "count" => Some("Returns the number of elements in the collection".to_string()),
            "first" => Some("Returns the first element of the collection".to_string()),
            "last" => Some("Returns the last element of the collection".to_string()),
            "where" => Some("Filters the collection to elements that match the given condition".to_string()),
            "select" => Some("Transforms each element of the collection using the given expression".to_string()),
            "resolve" => Some("Resolves a reference to the referenced resource".to_string()),
            "extension" => Some("Gets extensions with the specified URL".to_string()),
            "conformsTo" => Some("Checks if the resource conforms to the given profile".to_string()),
            _ => None,
        };

        let definition_location = Some(DefinitionLocation {
            uri: format!("https://hl7.org/fhir/fhirpath.html#{}", function_name),
            range: None,
            source: "FHIRPath Specification".to_string(),
        });

        (documentation, definition_location)
    }

    /// Check if a function is deprecated
    fn is_function_deprecated(&self, _function_name: &str) -> bool {
        // No deprecated functions currently
        false
    }

    /// Check if a type is a FHIR type
    fn is_fhir_type(&self, type_name: &str) -> bool {
        // Common FHIR types - in a real implementation this would be more comprehensive
        matches!(type_name, 
            "Patient" | "Observation" | "Condition" | "Medication" | "Procedure" |
            "Encounter" | "Practitioner" | "Organization" | "Location" | "Device" |
            "String" | "Integer" | "Decimal" | "Boolean" | "Date" | "DateTime" | "Time" |
            "Quantity" | "Code" | "Coding" | "CodeableConcept" | "Identifier" | "Reference"
        )
    }

    /// Infer parameter type (simplified)
    async fn infer_parameter_type(&self, _param_name: &str) -> Option<TypeReflectionInfo> {
        // Simplified - would need more sophisticated inference
        None
    }

    /// Format hover contents for a symbol
    async fn format_hover_contents(&self, symbol: &Symbol) -> String {
        let mut contents = String::new();

        // Symbol name and kind
        contents.push_str(&format!("**{}** *({})*\n\n", symbol.name, symbol.kind_name()));

        // Type information
        if let Some(type_info) = &symbol.type_info {
            contents.push_str(&format!("Type: `{}`", type_info.name()));
            if type_info.is_collection() {
                contents.push_str(" (collection)");
            }
            contents.push_str("\n\n");
        }

        // Documentation
        if let Some(doc) = &symbol.documentation {
            contents.push_str(doc);
            contents.push_str("\n\n");
        }

        // Definition source
        if let Some(def_loc) = &symbol.definition_location {
            contents.push_str(&format!("*Source: {}*", def_loc.source));
        }

        contents
    }
}

impl Symbol {
    /// Get the human-readable kind name
    pub fn kind_name(&self) -> &'static str {
        match self.kind {
            SymbolKind::Property => "property",
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Variable => "variable",
            SymbolKind::Type => "type",
            SymbolKind::Parameter => "parameter",
            SymbolKind::SystemVariable => "system variable",
        }
    }

    /// Convert to LSP symbol information
    pub fn to_lsp_symbol_info(&self) -> serde_json::Value {
        let location_value = if let (Some(start), Some(end)) = (self.start_offset, self.end_offset) {
            serde_json::json!({
                "range": {
                    "start": {"line": start as u32, "character": 0},
                    "end": {"line": end as u32, "character": 0}
                }
            })
        } else {
            serde_json::Value::Null
        };
        
        serde_json::json!({
            "name": self.name,
            "kind": match self.kind {
                SymbolKind::Property => 7,     // Field
                SymbolKind::Function => 12,    // Function
                SymbolKind::Method => 6,       // Method
                SymbolKind::Variable => 13,    // Variable
                SymbolKind::Type => 5,         // Class
                SymbolKind::Parameter => 17,   // TypeParameter
                SymbolKind::SystemVariable => 13, // Variable
            },
            "location": location_value,
            "deprecated": self.deprecated
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::mock_provider::MockModelProvider;
    use crate::ast::ExpressionNode;

    #[tokio::test]
    async fn test_symbol_resolver_creation() {
        let provider = Arc::new(MockModelProvider::empty());
        let resolver = SymbolResolver::new(provider);

        // Test that resolver is created successfully
        assert!(resolver.symbol_cache.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_identifier_symbol_resolution() {
        let provider = Arc::new(MockModelProvider::empty());
        let resolver = SymbolResolver::new(provider);

        let expr = ExpressionNode::identifier("Patient");
        let symbols = resolver.resolve_symbols(&expr, Some("Patient")).await.unwrap();

        // With MockModelProvider, we won't get real type info but should get a symbol
        assert!(!symbols.is_empty());
    }

    #[tokio::test]
    async fn test_function_symbol_resolution() {
        let provider = Arc::new(MockModelProvider::empty());
        let resolver = SymbolResolver::new(provider);

        let expr = ExpressionNode::function_call("count", vec![]);
        let symbols = resolver.resolve_symbols(&expr, None).await.unwrap();

        assert!(symbols.iter().any(|s| s.name == "count" && s.kind == SymbolKind::Function));
    }

    #[tokio::test]
    async fn test_variable_symbol_resolution() {
        let provider = Arc::new(MockModelProvider::empty());
        let resolver = SymbolResolver::new(provider);

        let expr = ExpressionNode::variable("$this");
        let symbols = resolver.resolve_symbols(&expr, Some("Patient")).await.unwrap();

        assert!(symbols.iter().any(|s| s.name == "$this" && s.kind == SymbolKind::SystemVariable));
    }

    #[tokio::test]
    async fn test_hover_info_generation() {
        let provider = Arc::new(MockModelProvider::empty());
        let resolver = SymbolResolver::new(provider);

        let expr = ExpressionNode::function_call("count", vec![]);
        let hover = resolver.get_hover_info(&expr, None, 0).await.unwrap();

        assert!(hover.is_some());
        if let Some(hover_info) = hover {
            assert!(hover_info.contents.contains("count"));
        }
    }
}