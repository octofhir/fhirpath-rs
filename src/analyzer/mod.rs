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


//! Static analysis capabilities for FHIRPath expressions
//!
//! This module provides comprehensive static analysis functionality including:
//! - Type inference and validation
//! - Expression analysis and completion
//! - Symbol resolution and diagnostics
//! - LSP integration support
//!
//! All analyzer functionality is implemented with async-first design to work
//! seamlessly with the async ModelProvider architecture.

pub mod completion_provider;
pub mod diagnostics;
pub mod expression_analyzer;
pub mod symbol_resolver;
pub mod type_analyzer;

use crate::ast::ExpressionNode as Expression;
use crate::model::provider::{ModelProvider, TypeReflectionInfo};
// Removed Span import to avoid lifetime issues
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration options for the analyzer
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Enable detailed type inference
    pub detailed_type_inference: bool,
    /// Enable completion suggestions
    pub enable_completions: bool,
    /// Enable diagnostics generation
    pub enable_diagnostics: bool,
    /// Maximum analysis depth for complex expressions
    pub max_analysis_depth: u32,
    /// Enable symbol tracking for go-to-definition
    pub enable_symbol_tracking: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            detailed_type_inference: true,
            enable_completions: true,
            enable_diagnostics: true,
            max_analysis_depth: 50,
            enable_symbol_tracking: true,
        }
    }
}

/// Result of analyzing a FHIRPath expression
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// The inferred return type of the expression
    pub return_type: Option<TypeReflectionInfo>,
    /// Whether the result is a collection or singleton
    pub is_collection: bool,
    /// All types referenced in the expression
    pub referenced_types: Vec<String>,
    /// Diagnostics found during analysis
    pub diagnostics: Vec<diagnostics::Diagnostic>,
    /// Symbol information for LSP support
    pub symbols: Vec<symbol_resolver::Symbol>,
    /// Completion suggestions at cursor position
    pub completions: Vec<completion_provider::Completion>,
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self {
            return_type: None,
            is_collection: false,
            referenced_types: Vec::new(),
            diagnostics: Vec::new(),
            symbols: Vec::new(),
            completions: Vec::new(),
        }
    }
}

/// Errors that can occur during analysis
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Model provider error: {0}")]
    ModelProvider(#[from] crate::model::error::ModelError),
    
    #[error("Parser error: {0}")]
    Parser(#[from] crate::parser::error::ParseError),
    
    #[error("Analysis depth exceeded maximum of {max_depth}")]
    MaxDepthExceeded { max_depth: u32 },
    
    #[error("Type resolution failed for {type_name}")]
    TypeResolutionFailed { type_name: String },
    
    #[error("Analysis cancelled")]
    Cancelled,
}

/// Main analyzer for FHIRPath expressions
/// 
/// Provides comprehensive static analysis including type inference,
/// diagnostics, completions, and symbol resolution.
pub struct FhirPathAnalyzer<P: ModelProvider> {
    provider: Arc<P>,
    config: AnalyzerConfig,
    type_analyzer: type_analyzer::TypeAnalyzer<P>,
    expression_analyzer: expression_analyzer::ExpressionAnalyzer<P>,
    diagnostic_system: diagnostics::DiagnosticSystem<P>,
    completion_provider: completion_provider::CompletionProvider<P>,
    symbol_resolver: symbol_resolver::SymbolResolver<P>,
}

impl<P: ModelProvider> FhirPathAnalyzer<P> {
    /// Create a new analyzer with the given model provider
    pub fn new(provider: Arc<P>) -> Self {
        let config = AnalyzerConfig::default();
        Self::with_config(provider, config)
    }
    
    /// Create a new analyzer with custom configuration
    pub fn with_config(provider: Arc<P>, config: AnalyzerConfig) -> Self {
        let type_analyzer = type_analyzer::TypeAnalyzer::new(provider.clone());
        let expression_analyzer = expression_analyzer::ExpressionAnalyzer::new(provider.clone());
        let diagnostic_system = diagnostics::DiagnosticSystem::new(provider.clone());
        let completion_provider = completion_provider::CompletionProvider::new(provider.clone());
        let symbol_resolver = symbol_resolver::SymbolResolver::new(provider.clone());
        
        Self {
            provider,
            config,
            type_analyzer,
            expression_analyzer,
            diagnostic_system,
            completion_provider,
            symbol_resolver,
        }
    }
    
    /// Analyze a FHIRPath expression completely
    pub async fn analyze(&self, expression: &Expression, context_type: Option<&str>) -> Result<AnalysisResult, AnalysisError> {
        let mut result = AnalysisResult::default();
        
        // Type inference
        if self.config.detailed_type_inference {
            let type_result = self.type_analyzer.analyze_expression(expression, context_type).await?;
            result.return_type = type_result.return_type;
            result.is_collection = type_result.is_collection;
            result.referenced_types = type_result.referenced_types;
        }
        
        // Expression analysis
        let expression_result = self.expression_analyzer.analyze(expression, context_type).await?;
        result.referenced_types.extend(expression_result.additional_types);
        
        // Diagnostics
        if self.config.enable_diagnostics {
            result.diagnostics = self.diagnostic_system.analyze_expression(expression, context_type).await?;
        }
        
        // Symbol resolution
        if self.config.enable_symbol_tracking {
            result.symbols = self.symbol_resolver.resolve_symbols(expression, context_type).await?;
        }
        
        Ok(result)
    }
    
    /// Analyze expression and provide completions at cursor position
    pub async fn analyze_with_completions(
        &self,
        expression: &Expression,
        context_type: Option<&str>,
        cursor_position: u32,
    ) -> Result<AnalysisResult, AnalysisError> {
        let mut result = self.analyze(expression, context_type).await?;
        
        // Add completions
        if self.config.enable_completions {
            result.completions = self.completion_provider.get_completions(
                expression,
                context_type,
                cursor_position,
            ).await?;
        }
        
        Ok(result)
    }
    
    /// Get the model provider
    pub fn provider(&self) -> &Arc<P> {
        &self.provider
    }
    
    /// Get the analyzer configuration
    pub fn config(&self) -> &AnalyzerConfig {
        &self.config
    }
}

/// Builder for creating analyzers with custom configuration
pub struct AnalyzerBuilder<P: ModelProvider> {
    provider: Arc<P>,
    config: AnalyzerConfig,
}

impl<P: ModelProvider> AnalyzerBuilder<P> {
    /// Create a new builder
    pub fn new(provider: Arc<P>) -> Self {
        Self {
            provider,
            config: AnalyzerConfig::default(),
        }
    }
    
    /// Enable or disable detailed type inference
    pub fn detailed_type_inference(mut self, enable: bool) -> Self {
        self.config.detailed_type_inference = enable;
        self
    }
    
    /// Enable or disable completions
    pub fn completions(mut self, enable: bool) -> Self {
        self.config.enable_completions = enable;
        self
    }
    
    /// Enable or disable diagnostics
    pub fn diagnostics(mut self, enable: bool) -> Self {
        self.config.enable_diagnostics = enable;
        self
    }
    
    /// Set maximum analysis depth
    pub fn max_depth(mut self, depth: u32) -> Self {
        self.config.max_analysis_depth = depth;
        self
    }
    
    /// Enable or disable symbol tracking
    pub fn symbol_tracking(mut self, enable: bool) -> Self {
        self.config.enable_symbol_tracking = enable;
        self
    }
    
    /// Build the analyzer
    pub fn build(self) -> FhirPathAnalyzer<P> {
        FhirPathAnalyzer::with_config(self.provider, self.config)
    }
}

/// Context information for analysis operations
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// The root context type (e.g., "Patient", "Observation")
    pub root_type: Option<String>,
    /// Variable definitions in scope
    pub variables: HashMap<String, TypeReflectionInfo>,
    /// Current analysis depth
    pub depth: u32,
    /// Start byte offset being analyzed
    pub start_offset: Option<usize>,
    /// End byte offset being analyzed  
    pub end_offset: Option<usize>,
}

impl AnalysisContext {
    /// Create a new analysis context
    pub fn new(root_type: Option<String>) -> Self {
        Self {
            root_type,
            variables: HashMap::new(),
            depth: 0,
            start_offset: None,
            end_offset: None,
        }
    }
    
    /// Create a child context with increased depth
    pub fn child(&self) -> Self {
        Self {
            root_type: self.root_type.clone(),
            variables: self.variables.clone(),
            depth: self.depth + 1,
            start_offset: self.start_offset,
            end_offset: self.end_offset,
        }
    }
    
    /// Add a variable to the context
    pub fn with_variable(mut self, name: String, type_info: TypeReflectionInfo) -> Self {
        self.variables.insert(name, type_info);
        self
    }
    
    /// Set the current span offsets
    pub fn with_offsets(mut self, start_offset: usize, end_offset: usize) -> Self {
        self.start_offset = Some(start_offset);
        self.end_offset = Some(end_offset);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::mock_provider::MockModelProvider;
    
    #[tokio::test]
    async fn test_analyzer_creation() {
        let provider = Arc::new(MockModelProvider::empty());
        let analyzer = FhirPathAnalyzer::new(provider.clone());
        
        assert_eq!(analyzer.provider().fhir_version(), provider.fhir_version());
        assert!(analyzer.config().detailed_type_inference);
    }
    
    #[tokio::test]
    async fn test_analyzer_builder() {
        let provider = Arc::new(MockModelProvider::empty());
        
        let analyzer = AnalyzerBuilder::new(provider)
            .detailed_type_inference(false)
            .completions(false)
            .max_depth(10)
            .build();
            
        assert!(!analyzer.config().detailed_type_inference);
        assert!(!analyzer.config().enable_completions);
        assert_eq!(analyzer.config().max_analysis_depth, 10);
    }
    
    #[tokio::test]
    async fn test_analysis_context() {
        let context = AnalysisContext::new(Some("Patient".to_string()));
        assert_eq!(context.root_type.as_ref().unwrap(), "Patient");
        assert_eq!(context.depth, 0);
        
        let child = context.child();
        assert_eq!(child.depth, 1);
        assert_eq!(child.root_type.as_ref().unwrap(), "Patient");
    }
}